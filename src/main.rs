
/*
  NOTE:
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

fn init() {

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
  run(String::from("dev"))
}
