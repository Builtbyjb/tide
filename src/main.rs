use std::env ;
use std::fs;
use std::io::Result;
use std:: path::{Path, PathBuf}; 
use std::thread;
use std::time::{UNIX_EPOCH, Duration};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use toml;
use tokio::process::{Command, Child};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::signal;
use tokio::sync::mpsc;

// TODO: proper error handling
// TODO: Character case should not matter
// TODO: Allow users to add commands and edit default command names
// TODO: Add support for windows machines

#[derive(Serialize, Deserialize, Debug)]
struct Config {
  root_dir:String,
  command:Cmd,
  exclude:Exclude,
}

#[derive(Serialize, Deserialize, Debug)]
struct Cmd {
  dev:Vec<String>,
  prod:Vec<String>,
  test:Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
struct Exclude {
  dir:Vec<String>,
  file:Vec<String>,
  ext:Vec<String>,
}

struct Process {
  child: Child,
  cmd: String,
}

struct ProcessManager {
  processes: Vec<Process>
}

impl ProcessManager {
  fn new() -> Self {
    ProcessManager { processes: Vec::new(), }
  }

  async fn kill_all(&mut self) {
    if self.processes.len() > 0 {
      println!("Shuting down commands");
      let mut processes = std::mem::take(&mut self.processes);
      for mut process in processes.drain(..) {
        let cmd = process.cmd.clone();
        match process.child.kill().await{
          Ok(_) => println!("shutdown: {}", cmd),
          Err(_) => println!("Failed to shutdown {}", cmd)
        }
      }
    }
  }

  async fn spawn_cmds(&mut self, cmds:&Vec<String>) {
    // Clear any existing process
    self.kill_all().await;

    for cmd in cmds {
      let process = self.spawn_cmd(cmd.to_owned()).await;
      self.processes.push(process)
    }
  }

  async fn spawn_cmd(&mut self, cmd:String) -> Process {

    // #[cfg(unix)]
    let (shell, shell_arg) = ("sh", "-c");
    // #[cfg(windows)]
    // let (shell, shell_arg) = ("cmd", "/C");

    let mut child = Command::new(shell)
      .arg(shell_arg)
      .arg(&cmd)
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped())
      .spawn()
      .expect(&format!("Failed to start command: {}", &cmd));

    // Capture stdout
    if let Some(stdout) = child.stdout.take() {
      let label = cmd.clone();
      tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
          println!("[{}] STDOUT: {}", label, line);
        }
      });
    }

    // Capture stderr
    if let Some(stderr) = child.stderr.take() {
      let label = cmd.clone();
      tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
          eprintln!("[{}] STDERR: {}", label, line);
        }
      });
    }

    Process{child, cmd}
  }
}

fn print_usage() {
  let usage = r#" 
    Usage: 
      To create a configuration file -> ./tide init
      To run a command in the commands table -> ./tide run [command]
      For live reload -> ./tide run [command] --watch
      To exit -> CTRL + C
  "#;
  println!("{}", usage)
}

// Initialize tide by creating a tide.toml file in the projects root dir
fn init() -> Result<()> {
  match fs::read_to_string("tide.toml") {
    Ok(_) => {
      println!("tide.toml file exists");
      return Ok(())
    },
    Err(_) => {
      let config = Config {
      root_dir: ".".to_string(),
      command: Cmd {
        dev: vec![],
        prod: vec![],
        test: vec![]
      },
      exclude: Exclude { 
        dir: vec![String::from(".git") ], 
        file: vec![String::from("README.md"), String::from("LICENSE.md")], 
        ext:vec![] 
      }
    };
    let toml_str = toml::to_string_pretty(&config).unwrap();
    fs::write("tide.toml", toml_str).unwrap();

    return Ok(())
    }
  }
}

// Run commands
async fn run(cmds:&Vec<String>) {
  let mut processes = ProcessManager::new();
  // Run new processes
  processes.spawn_cmds(cmds).await;


  // Setup shutdown signal
  let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

  // Ctrl+C handle
  tokio::spawn(async move {
    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    shutdown_tx.send(()).await.expect("Failed to send shutdown signal");
  });

  // Wait for shutdown signal
  shutdown_rx.recv().await;

  println!("Received shutdown signal! Shutting down gracefully...");
   
  // Clear all processes
  processes.kill_all().await;
   
  println!("Cleanup complete!!!");
  std::process::exit(1)
}

// Watcher function
// Is this a sign of bad software design
fn watcher(
  r_path:&Path, 
  ignore_dirs:&Vec<String>, 
  ignore_files:&Vec<String>, 
  ignore_exts:&Vec<String>, 
  files:&mut HashMap<PathBuf, u64>
) -> bool {
  let mut init_run = false;
  for e in fs::read_dir(r_path).unwrap() {
    let e = e.unwrap();
    let path = e.path();

    if path.is_dir() && ignore_dirs.contains(&path.display().to_string()) == false {
      if watcher(&path, ignore_dirs, ignore_files, ignore_exts, files) { init_run = true }
    } else if path.is_file() {
      let path_ext = match path.extension() {
        Some(ext) => {
          match ext.to_owned().into_string() {
            Ok(value) => value,
            Err(_) => "".to_string()
          }
        },
        None => "".to_string(),
      };

      if ignore_exts.contains(&path_ext) == false {
        if ignore_files.contains(&path.display().to_string()) == false {
          let metadata = fs::metadata(&path);

          if let Ok(time) = metadata.unwrap().modified() { // The last time the file was modified
            let time_secs = time.duration_since(UNIX_EPOCH).unwrap().as_secs();
            match files.get(&path) {
              Some(value) => {
                if *value != time_secs {
                  files.insert(path.clone(), time_secs);
                  println!("{:#?} as been modified", &path);
                  init_run = true;
                }
              },
              None => {
                files.insert(path.clone(), time_secs);
                // println!("{:#?} as been modified at {:#?}", path, time);
                init_run = true
              },
            }
          }
        }
      }
    }
  }
  init_run
}

async fn start(cmd:&String, watch:bool) {
  // Open config file
  let toml_str = fs::read_to_string("tide.toml").unwrap();
  
  // Parse toml file to config
  let toml_config:Config = toml::from_str(&toml_str).unwrap();

  // Check if cmd is a valid command
  let cmds:Vec<String>;
  if cmd == "dev" {
    cmds = toml_config.command.dev;
  } else if cmd == "prod" {
    cmds = toml_config.command.prod;
  } else if cmd == "test" {
    cmds = toml_config.command.test
  } else {
    println!("Run value not in commands");
    std::process::exit(1)
  }

  let styled_name = r#"
       __   _      __    
      / /_ (_)____/ /___ 
     / __// // __  // _ \
    / /_ / // /_/ //  __/
    \__//_/ \__,_/ \___/  version: 0.1.0.
  "#;

  println!("{}", styled_name);

  if watch { 
    // Hashmap to store file edit time
    let mut files:HashMap<PathBuf, u64> = HashMap::new();
    println!("Watching...");
    let path = Path::new(&toml_config.root_dir);
    loop {
      let should_run = watcher(
        path, 
        &toml_config.exclude.dir, 
        &toml_config.exclude.file, 
        &toml_config.exclude.ext, 
        &mut files
      );

      if should_run { 
        run(&cmds).await;
      }

      // Sleep for 100ms
      thread::sleep(Duration::from_millis(100))
    }
  } else {
    // Run command without watching for file changes
    run(&cmds).await;
  }
}

#[tokio::main]
async fn main() {
  // Get command line arguments
  let args: Vec<String> = env::args().collect();

  // Is there a better way to implement this?
  if args.len() == 2 && args[1] == "init" {
    init().unwrap();
  } else if args.len() == 3 {
    if args[1] == "run" {
      start(&args[2], false).await;
    } else {
      print_usage();
      return ()
    }
  } else if args.len() == 4 {
    if args[1] == "run" &&  ( args[3] == "--watch" || args[3] == "-w" ) {
      start(&args[2], true).await;
    } else {
      print_usage();
      return ()
    }
  } else {
    print_usage();
    return ();
  }
}
