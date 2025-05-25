use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::result::Result;
use std::time::{Duration, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::signal;
use tokio::sync::mpsc;
use toml;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
  root_dir: String,
  command: Cmd,
  exclude: Exclude,
}

#[derive(Serialize, Deserialize, Debug)]
struct Cmd {
  dev: Vec<String>,
  prod: Vec<String>,
  test: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Exclude {
  dir: Vec<String>,
  file: Vec<String>,
  ext: Vec<String>,
}

struct Process {
  child: Child,
  cmd: String,
}

struct ProcessManager {
  processes: Vec<Process>,
}

impl ProcessManager {
  fn new() -> Self {
    ProcessManager {
      processes: Vec::new(),
    }
  }

  async fn kill_all(&mut self) {
    // println!("processes len: {}", self.processes.len());
    if self.processes.len() > 0 {
      println!("{}", "Shuting down commands".blue().bold());
      let mut processes = std::mem::take(&mut self.processes);
      for mut process in processes.drain(..) {
        match process.child.kill().await {
          Ok(_) => println!(
            "{} {} {}",
            "✓".green(),
            "shutdown:".green(),
            &process.cmd.cyan()
          ),
          Err(_) => println!(
            "{} {} {}",
            "✗".red(),
            "Failed to shutdown:".red(),
            &process.cmd.cyan()
          ),
        }
      }
    }
  }

  async fn spawn_cmds(&mut self, cmds: &Vec<String>) {
    // Clear any existing process
    self.kill_all().await;
    // println!(" spawn: processes len: {}", self.processes.len());
    println!("{}", "Starting commands".blue().bold());
    for cmd in cmds {
      let process = self.spawn_cmd(cmd.to_owned()).await;
      self.processes.push(process)
    }
  }

  async fn spawn_cmd(&mut self, cmd: String) -> Process {
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
          println!("{}: {}", label.cyan(), line);
        }
      });
    }

    // Capture stderr
    if let Some(stderr) = child.stderr.take() {
      let label = cmd.clone();
      tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
          eprintln!("{}: {}", label.cyan(), line);
        }
      });
    }

    Process { child, cmd }
  }
}

fn print_usage() {
  let usage = format!(
    r#"
    {}
      To create a configuration file -> {} init
      To run a command in the commands table -> {} run [command]
      For live reload -> {} run [command] --watch
      To exit -> {} 
    "#,
    "Usage:".cyan().bold(),
    "./tide".green(),
    "./tide".green(),
    "./tide".green(),
    "CTRL + C".yellow()
  );
  println!("{}", usage);
}

#[derive(Debug)]
enum ConfigError {
  IOError(std::io::Error),
  TomlError(toml::ser::Error),
}

impl std::fmt::Display for ConfigError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      ConfigError::IOError(err) => write!(f, "IOError: {}", err),
      ConfigError::TomlError(err) => write!(f, "TomlError: {}", err),
    }
  }
}

// Initialize tide by creating a tide.toml file in the projects root dir
fn init() -> Result<(), ConfigError> {
  match fs::read_to_string("tide.toml") {
    Ok(_) => {
      println!("tide.toml file exists");
      return Ok(());
    }
    Err(_) => {
      // Create tide config file
      let config = Config {
        root_dir: ".".to_string(),
        command: Cmd {
          dev: vec![],
          prod: vec![],
          test: vec![],
        },
        exclude: Exclude {
          dir: vec![String::from("./.git")],
          file: vec![],
          ext: vec![],
        },
      };

      let toml_str = match toml::to_string_pretty(&config) {
        Ok(value) => value,
        Err(e) => return Err(ConfigError::TomlError(e)),
      };

      match fs::write("tide.toml", toml_str) {
        Ok(_) => return Ok(()),
        Err(e) => return Err(ConfigError::IOError(e)),
      };
    }
  }
}

// Watcher function
fn watcher(
  root_path: &Path,
  ignore_dirs: &Vec<String>,
  ignore_files: &Vec<String>,
  ignore_exts: &Vec<String>,
  files: &mut HashMap<PathBuf, u64>,
) -> bool {
  let mut init_run = false;
  match fs::read_dir(root_path) {
    Ok(entries) => {
      for e in entries {
        let path = e.expect("Invalid entry").path();

        if path.is_dir() && ignore_dirs.contains(&path.display().to_string()) == false {
          if watcher(&path, ignore_dirs, ignore_files, ignore_exts, files) {
            init_run = true
          }
        } else if path.is_file() {
          let path_ext = match path.extension() {
            Some(ext) => match ext.to_owned().into_string() {
              Ok(value) => value,
              Err(_) => "".to_string(),
            },
            None => "".to_string(),
          };

          if ignore_exts.contains(&path_ext) == false {
            if ignore_files.contains(&path.display().to_string()) == false {
              let metadata = fs::metadata(&path);

              if let Ok(time) = metadata.expect("Error getting file metadata").modified() {
                // The last time the file was modified
                let time_secs = time
                  .duration_since(UNIX_EPOCH)
                  .expect("Error getting system time")
                  .as_secs();

                match files.get(&path) {
                  Some(value) => {
                    if *value != time_secs {
                      files.insert(path.clone(), time_secs);
                      println!("{:#?} as been modified", &path);
                      init_run = true;
                    }
                  }
                  None => {
                    files.insert(path.clone(), time_secs);
                    // println!("{:#?} as been modified at {:#?}", path, time);
                    init_run = true
                  }
                }
              }
            }
          }
        }
      }
    }
    Err(_) => {
      eprintln!("Error reading directory entries");
      std::process::exit(1)
    }
  }
  init_run
}

async fn start(cmd: &String, watch: bool) {
  // Open config file
  let toml_str = match fs::read_to_string("tide.toml") {
    Ok(value) => value,
    Err(_) => {
      eprintln!("Error reading config file");
      return;
    }
  };

  // Parse toml file to config
  let toml_config: Config = match toml::from_str(&toml_str) {
    Ok(value) => value,
    Err(_) => {
      eprintln!("Error parsing config file");
      return;
    }
  };

  // Check if cmd is a valid command
  let cmds: Vec<String>;
  if cmd == "dev" {
    cmds = toml_config.command.dev;
  } else if cmd == "prod" {
    cmds = toml_config.command.prod;
  } else if cmd == "test" {
    cmds = toml_config.command.test
  } else {
    eprintln!("Run value not in commands");
    std::process::exit(1)
  }

  let styled_name = format!(
    r#"{}
     __   _      __
    / /_ (_)____/ /___
   / __// // __  // _ \
  / /_ / // /_/ //  __/
  \__//_/ \__,_/ \___/  version: {}"#,
    "".blue().bold(),
    "0.1.0".blue().bold()
  )
  .blue()
  .bold();

  println!("{}\n", styled_name);

  let mut processes = ProcessManager::new();

  // Setup shutdown signal
  let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

  // Ctrl+C handle
  tokio::spawn(async move {
    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    shutdown_tx
      .send(())
      .await
      .expect("Failed to send shutdown signal");
  });

  if watch {
    // Hashmap to store file edit time
    let mut files: HashMap<PathBuf, u64> = HashMap::new();
    println!("{}", "Watching...\n".green().bold());
    let path = Path::new(&toml_config.root_dir);
    loop {
      tokio::select! {
        // Handle file watching
        _ = async {
          let should_run = watcher(
            path,
            &toml_config.exclude.dir,
            &toml_config.exclude.file,
            &toml_config.exclude.ext,
            &mut files,
          );

          if should_run {
            processes.spawn_cmds(&cmds).await;
          }

          // Sleep for 100ms
          tokio::time::sleep(Duration::from_millis(100)).await;
        } => {}

        // Handle shutdown signal
        Some(_) = shutdown_rx.recv() => {
          println!();
          println!("{}", "Received shutdown signal!!! Shutting down gracefully...".green());
          processes.kill_all().await;
          println!("{}", "Shutdown complete!!!".green());
          std::process::exit(0);
        }
      }
    }
  } else {
    // Run command without watching for file changes
    processes.spawn_cmds(&cmds).await;

    // Shutdown handler
    if shutdown_rx.recv().await.is_some() {
      println!(
        "{}",
        "Received shutdown signal! Shutting down gracefully...".green()
      );
      processes.kill_all().await;
      std::process::exit(0);
    }
  }
}

#[tokio::main]
async fn main() {
  // Get command line arguments
  let args: Vec<String> = env::args().collect();

  if args.len() == 2 && args[1] == "init" {
    match init() {
      Ok(_) => return,
      Err(e) => {
        eprintln!("Error creating a toml configuration file: {:#?}", e);
        return;
      }
    };
  } else if args.len() == 3 {
    if args[1] == "run" {
      start(&args[2], false).await;
    } else {
      print_usage();
      return ();
    }
  } else if args.len() == 4 {
    if args[1] == "run" && (args[3] == "--watch" || args[3] == "-w") {
      start(&args[2], true).await;
    } else {
      print_usage();
      return ();
    }
  } else {
    print_usage();
    return ();
  }
}
