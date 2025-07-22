#[cfg(test)]
mod test;

use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::result::Result;
use std::time::{Duration, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::signal;
use tokio::sync::mpsc;

const VERSION: &str = "v0.1.2";

#[derive(Serialize, Deserialize, Debug)]
struct Config {
  root_dir: String,
  os: Os,
  exclude: Exclude,
}

#[derive(Serialize, Deserialize, Debug)]
struct Os {
  unix: Cmd,
  windows: Cmd,
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
    if !self.processes.is_empty() {
      println!("{}", "Shutting down commands".blue().bold());
      let mut processes = std::mem::take(&mut self.processes);
      for mut process in processes.drain(..) {
        match process.child.kill().await {
          Ok(_) => println!("[{}]: {}", "shutting down".green(), &process.cmd.cyan()),
          Err(_) => println!("[{}]: {}", "Failed to shutdown".red(), &process.cmd.cyan()),
        }
      }
      // Wait for all processes to shutdown completely
      tokio::time::sleep(Duration::from_millis(1000)).await;
      println!("{}", "Shutdown complete!!!".green());
    }
  }

  async fn spawn_cmds(&mut self, cmds: &Vec<String>) {
    // Clear any existing process
    self.kill_all().await;
    // println!(" spawn: processes len: {}", self.processes.len());
    println!("{}", "Starting up commands".blue().bold());
    for cmd in cmds {
      let process = self.spawn_cmd(cmd.to_owned()).await;
      self.processes.push(process)
    }
  }

  async fn spawn_cmd(&mut self, cmd: String) -> Process {
    // Check device
    #[cfg(unix)]
    let (shell, shell_arg) = ("sh", "-c");
    #[cfg(windows)]
    let (shell, shell_arg) = ("cmd", "/C");

    let mut child = Command::new(shell)
      .arg(shell_arg)
      .arg(&cmd)
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped())
      .spawn()
      .unwrap_or_else(|_| panic!("Failed to start command: {}", &cmd));

    // Capture stdout
    if let Some(stdout) = child.stdout.take() {
      let label = cmd.clone();
      tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
          println!("[{}]: {}", label.cyan(), line);
        }
      });
    }

    // Capture stderr
    if let Some(stderr) = child.stderr.take() {
      let label = cmd.clone();
      tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
          eprintln!("[{}]: {}", label.cyan(), line);
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
    "tide".cyan(),
    "tide".cyan(),
    "tide".cyan(),
    "CTRL + C".cyan()
  );
  println!("{}", usage);
}

fn print_title() {
  let styled_title = format!(
    r#"{}
     __   _      __
    / /_ (_)____/ /___
   / __// // __  // _ \
  / /_ / // /_/ //  __/
  \__//_/ \__,_/ \___/  version: {}
  
 Press Crtl + C to exit"#,
    "".blue().bold(),
    &VERSION.blue().bold(),
  )
  .blue()
  .bold();

  println!("{}\n", styled_title)
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
      Ok(())
    }
    Err(_) => {
      // Create tide config file
      let config = Config {
        root_dir: ".".to_string(),
        os: Os {
          unix: Cmd {
            dev: vec![],
            prod: vec![],
            test: vec![],
          },
          windows: Cmd {
            dev: vec![],
            prod: vec![],
            test: vec![],
          },
        },
        exclude: Exclude {
          dir: vec![".git".to_string()],
          file: vec![],
          ext: vec!["toml".to_string()],
        },
      };

      let toml_str = match toml::to_string_pretty(&config) {
        Ok(value) => value,
        Err(e) => return Err(ConfigError::TomlError(e)),
      };

      match fs::write("tide.toml", toml_str) {
        Ok(_) => {
          println!("tide.toml file created");
          Ok(())
        }
        Err(e) => Err(ConfigError::IOError(e)),
      }
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
) -> io::Result<bool> {
  let mut init_run = false;
  match fs::read_dir(root_path) {
    Ok(entries) => {
      for e in entries {
        let path = e.expect("Invalid entry").path();
        #[cfg(unix)]
        let path_name = path.display().to_string().split("/").last().unwrap().to_string();
        #[cfg(windows)]
        let path_name = path.display().to_string().split("\\").last().unwrap().to_string();

        if path.is_dir() && !ignore_dirs.contains(&path_name) {
          if watcher(&path, ignore_dirs, ignore_files, ignore_exts, files)? {
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

          if !ignore_exts.contains(&path_ext) && !ignore_files.contains(&path_name) {
            let metadata = fs::metadata(&path);

            if let Ok(time) = metadata.expect("Error getting file metadata").modified() {
              // The last time the file was modified
              let time_secs =
                time.duration_since(UNIX_EPOCH).expect("Error getting system time").as_secs();

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
    Err(err) => {
      eprintln!("Error reading directory entries");
      return Err(err);
    }
  }
  Ok(init_run)
}

async fn start(cmd: &str, watch: bool) {
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

  // Check device
  #[cfg(unix)]
  let cmd_config: Cmd = toml_config.os.unix;
  #[cfg(windows)]
  let cmd_config: Cmd = toml_config.os.windows;

  let cmds: Vec<String> = match cmd {
    "dev" => cmd_config.dev,
    "prod" => cmd_config.prod,
    "test" => cmd_config.test,
    _ => {
      eprintln!("Run value not in os table");
      std::process::exit(1)
    }
  };

  if cmds.is_empty() {
    println!("No command found in the '{cmd}' variable");
    std::process::exit(1)
  }

  print_title();

  let mut processes = ProcessManager::new();

  // Setup shutdown signal
  let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

  // Ctrl+C handle
  tokio::spawn(async move {
    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    shutdown_tx.send(()).await.expect("Failed to send shutdown signal");
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

          if should_run.unwrap() {
            processes.spawn_cmds(&cmds).await;
          }

          // Sleep for 100ms
          tokio::time::sleep(Duration::from_millis(100)).await;
        } => {}

        // Handle shutdown signal
        Some(_) = shutdown_rx.recv() => {
          println!("\n{}", "Received shutdown signal!!! Shutting down gracefully...".green());
          processes.kill_all().await;
          std::process::exit(0);
        }
      }
    }
  } else {
    // Run command without watching for file changes
    processes.spawn_cmds(&cmds).await;

    // Shutdown handler
    if shutdown_rx.recv().await.is_some() {
      println!("\n{}", "Received shutdown signal!!! Shutting down gracefully...".green());
      processes.kill_all().await;
      std::process::exit(0);
    }
  }
}

#[tokio::main]
async fn main() {
  let args: Vec<String> = env::args().collect();

  match (args.len(), args[1].as_str()) {
    (2, "init") => match init() {
      Ok(_) => return,
      Err(e) => eprintln!("Error creating a toml configuration file: {:#?}", e),
    },
    (2, "--version") | (2, "-v") => println!("tide {}", &VERSION),
    (3, "run") => start(&args[2], false).await,
    (4, "run") if args[3] == "--watch" || args[3] == "-w" => start(&args[2], true).await,
    _ => print_usage(),
  }
}
