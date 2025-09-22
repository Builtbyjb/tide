use colored::*;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

struct Process {
    child: Child,
    cmd: String,
}

pub struct ProcessManager {
    processes: Vec<Process>,
}

impl ProcessManager {
    pub fn new() -> Self {
        ProcessManager {
            processes: Vec::new(),
        }
    }

    pub async fn kill_all(&mut self) {
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

    pub async fn spawn_cmds(&mut self, cmds: &Vec<String>) {
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
                if let Ok(Some(first_line)) = reader.next_line().await {
                    println!("{}:", label.cyan());
                    println!("\t{}", first_line);

                    while let Ok(Some(line)) = reader.next_line().await {
                        println!("\t{}", line);
                    }
                }
            });
        }

        // Capture stderr
        if let Some(stderr) = child.stderr.take() {
            let label = cmd.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                if let Ok(Some(first_line)) = reader.next_line().await {
                    println!("{}:", label.cyan());
                    println!("\t{}", first_line);

                    while let Ok(Some(line)) = reader.next_line().await {
                        println!("\t{}", line);
                    }
                }
            });
        }

        Process { child, cmd }
    }
}
