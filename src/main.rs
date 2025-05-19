use std::{ fs, path::{Path, PathBuf}, io::Result };
//serde: parsing .toml
// toml: creating .toml
// tokio: async

/*
  TODO:
  * create a tide.toml file in programs root dir
  * parse the tide.toml file
  * the commands concurrently
*/

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

// fn print_usage() {}

// fn init() {

// }

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

fn run(cmd:String) {
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
  run(String::from("dev"));

  let path = Path::new(".");
  let mut files = Vec::new();
  visit(path, &mut |e| files.push(e)).unwrap();
  for file in files {
    println!("{:?}", file);
  }
}
