use colored::*;

pub fn usage() {
    let usage = format!(
        r#"
    {}: tide [options] [command]

    Options:
        init               Create a tide configuration file
        run [command]      Run commands
            -w, --watch    Watch for changes and re-run commands
        -h, --help         Display this help message
        -v, --version      Display the version number
    "#,
        "Usage:".cyan().bold(),
    );
    println!("{}", usage);
}

pub fn title(version: &str) {
    let styled_title = format!(
        r#"{}
     __   _      __
    / /_ (_)____/ /___
   / __// // __  // _ \
  / /_ / // /_/ //  __/
  \__//_/ \__,_/ \___/  v{}

 Press Crtl + C to exit"#,
        "".blue().bold(),
        version.blue().bold(),
    )
    .blue()
    .bold();

    println!("{}\n", styled_title)
}
