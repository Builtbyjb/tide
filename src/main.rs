mod print;
mod process_manager;
mod watcher;

use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::result::Result;
use std::time::Duration;
use tokio::signal;
use tokio::sync::mpsc;

const VERSION: &str = "0.1.2";

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    root_dir: String,
    os: Os,
    exclude: Exclude,
}

#[derive(Serialize, Deserialize, Debug)]
struct Os {
    unix: HashMap<String, Vec<String>>,
    windows: HashMap<String, Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Exclude {
    dir: Vec<String>,
    file: Vec<String>,
    ext: Vec<String>,
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
            let mut cmd = HashMap::new();
            cmd.insert("dev".to_string(), vec![]);

            let config = Config {
                root_dir: ".".to_string(),
                os: Os {
                    unix: cmd.clone(),
                    windows: cmd.clone(),
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
    let cmd_config = toml_config.os.unix;
    #[cfg(windows)]
    let cmd_config = toml_config.os.windows;

    let cmds: Vec<String> = cmd_config.get(cmd).cloned().unwrap_or_default();

    if cmds.is_empty() {
        println!("No command found in the '{cmd}' variable");
        std::process::exit(1)
    }

    print::title(&VERSION);

    let mut processes = process_manager::ProcessManager::new();

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
                let should_run = watcher::watch(
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
        (2, "--version") | (2, "-v") => println!("tide v{}", &VERSION),
        (3, "run") => start(&args[2], false).await,
        (4, "run") if args[3] == "--watch" || args[3] == "-w" => start(&args[2], true).await,
        _ => print::usage(),
    }
}
