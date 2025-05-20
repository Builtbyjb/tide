use std::env ;
use std::fs;
// use std::io::{BufRead, BufReader, Result};
use std::io::Result;
// use std::panic::PanicHookInfo;
use std:: path::{Path, PathBuf}; 
// use std::process::{exit, Command, Stdio};
use std::process::exit;
// use std::sync::mpsc;
use std::thread;
use std::time::{UNIX_EPOCH, Duration};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use toml;

// tokio: async

// TODO: proper error handling
// TODO: Character case should not matter
// TODO: Allow uses to add commands and edit default command names
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

// Parse command string into list
// fn parse_cmd(cmd:&String) -> Vec<String> {
//   println!("{}", cmd);

//   return [].to_vec()
// }

// TODO: run commands
fn run(cmds:&Vec<String>) -> Result<()> {
  for c in cmds {
    println!("{}", c)
  }

  // Exit previous running processes if any

  // Run new processes
  Ok(())
}

// Watcher function
// Is this a sign of bad software design?
fn watcher(path:&Path, dir:&Vec<String>, file:&Vec<String>, ext:&Vec<String>, files:&mut HashMap<PathBuf, u64>) -> bool {
  for e in fs::read_dir(path).unwrap() {
    let e = e.unwrap();
    let path = e.path();

    if path.is_dir() && dir.contains(&path.display().to_string()) == false {
      watcher(&path, dir, file, ext, files);
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

      if ext.contains(&path_ext) == false {
        if file.contains(&path.display().to_string()) == false {
          let metadata = fs::metadata(&path);

          if let Ok(time) = metadata.unwrap().modified() { // The last time the file was modified
            let time_secs = time.duration_since(UNIX_EPOCH).unwrap().as_secs();
            match files.get(&path) {
              Some(value) => {
                if value.to_owned() != time_secs {
                  files.insert(path.clone(), time_secs);
                  println!("{:#?} as been modified at {:#?}", path, time);
                  return true;
                }
              },
              None => {
                files.insert(path.clone(), time_secs);
                println!("{:#?} as been modified at {:#?}", path, time);
                return true;
              }
            }
          }
        }
      }
    }
  }
  false
}

fn start(cmd:&String, watch:bool) {
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
    exit(1)
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

      if should_run { run(&cmds).unwrap() }
      // Sleep for 100ms
      thread::sleep(Duration::from_millis(100))
    }
  } else {
    // Run command without watching for file changes
    run(&cmds).unwrap()
  }
}

fn main() {
  // Get command line arguments
  let args: Vec<String> = env::args().collect();

  // Is there a better way to implement this?
  if args.len() == 2 && args[1] == "init" {
    init().unwrap();
  } else if args.len() == 3 {
    if args[1] == "run" {
      start(&args[2], false)
    } else {
      print_usage();
      return
    }
  } else if args.len() == 4 {
    if args[1] == "run" &&  ( args[3] == "--watch" || args[3] == "-w" ) {
      start(&args[2], true)
    } else {
      print_usage();
      return
    }
  } else {
    print_usage();
    return;
  }
}
