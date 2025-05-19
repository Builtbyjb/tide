use std::{ fs, path::{Path, PathBuf}, io::Result, env };

//serde: parsing .toml
// toml: creating .toml
// tokio: async

/* ##### Sample tide.toml config #####
root_dir = "."

[command]
dev = []
prod = []
test = []

[exclude]
dir = []
file = []
ext = []
*/

fn print_usage() {
  let usage = r#" 
    Usage: 
      To create a configuration file -> ./tide init
      To run a command in the commands table -> ./tide run [command]
      To exit -> CTRL + C
  "#;

  println!("{}", usage)
}

// Initialize tide by creating a tide.toml file in the projects root dir
fn init() {
  println!("tide.toml file created")
}

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

fn run(cmd:&String) {
  let styled_name = r#"
       __   _      __    
      / /_ (_)____/ /___ 
     / __// // __  // _ \
    / /_ / // /_/ //  __/
    \__//_/ \__,_/ \___/  version: 0.1.0.
  "#;

  println!("{}", styled_name);
  println!("{}", cmd);
}

fn main() {
  // Get command line arguments
  let args: Vec<String> = env::args().collect();

  if args.len() == 2 && args[1] == "init" {
    init()
  } else if args.len() == 3 && args[1] == "run" {
    run(&args[2]);
  } else {
    print_usage();
    return
  }

  // TODO
  // handle keyboard interrupt gracefully
  // The watcher runs on a different thread
  // Function calls doesn't require a while loop

  let path = Path::new(".");
  let mut files = Vec::new();
  visit(path, &mut |e| files.push(e)).unwrap();
  for file in files { println!("{:?}", file); }
}
