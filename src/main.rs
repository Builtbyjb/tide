use std::{ fs, path::{Path, PathBuf}, io::Result, env };
use serde::{Serialize, Deserialize};
use toml;

// tokio: async

// TODO: proper error handling
// TODO: Character case should not matter

#[derive(Serialize, Deserialize, Debug)]
struct Config {
  root_dir:String,
  command:Command,
  exclude:Exclude,
}

#[derive(Serialize, Deserialize, Debug)]
struct Command {
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
  // TODO: Check if tide.toml file exists and create a new one if not
  let config = Config {
    root_dir: ".".to_string(),
    command: Command {
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

  Ok(())
}

// Watcher function
fn visit(path: &Path, cb: &mut dyn FnMut(PathBuf)) -> Result<()> {
  for e in fs::read_dir(path)? {
    let e = e?;
    let path = e.path();
    if path.is_dir() {
      visit(&path, cb)?;
    } else if path.is_file() {
      cb(path);
    }
  }
  Ok(())
}

fn run(cmd:&String, watch:bool) {
  // Open config file
  let toml_str = fs::read_to_string("tide.toml").unwrap();
  
  // Parse toml file to config
  let toml_config:Config = toml::from_str(&toml_str).unwrap();

  // check if cmd is a valid command
  let styled_name = r#"
       __   _      __    
      / /_ (_)____/ /___ 
     / __// // __  // _ \
    / /_ / // /_/ //  __/
    \__//_/ \__,_/ \___/  version: 0.1.0.
  "#;

  println!("{}", styled_name);
  println!("{}, {}", cmd, watch);
  println!("{:#?}", toml_config)
  // TODO
  // handle keyboard interrupt gracefully
  // The watcher runs on a different thread
  // Function calls doesn't require a while loop

  // let path = Path::new(".");
  // let mut files = Vec::new();
  // visit(path, &mut |e| files.push(e)).unwrap();
  // for file in files { println!("{:?}", file); }
}

fn main() {
  // Get command line arguments
  let args: Vec<String> = env::args().collect();

  // Is there a better way to implement this?
  if args.len() == 2 && args[1] == "init" {
    init().unwrap();
  } else if args.len() == 3 {
    if args[1] == "run" {
      run(&args[2], false)
    } else {
      print_usage();
      return
    }
  } else if args.len() == 4 {
    if args[1] == "run" &&  ( args[3] == "--watch" || args[3] == "-w" ) {
      run(&args[2], true)
    } else {
      print_usage();
      return
    }
  } else {
    print_usage();
    return;
  }
}
