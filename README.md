# Tide

Tide is a cli program that runs multiple commands concurrently. Tide also offers live reloading.

### Commands

Create a tide configuration file.
```bash
tide init 
```

Runs the list of commands assigned to the **dev** variable under the command table. The command table contains three variables **dev**, **prod**, and **test**
```bash
tide run dev 
```

Re runs the commands in the **dev** variable every time a file is modified (live reloading)
```bash
tide run dev --watch 
```

Check current version
```bash
tide --version
```

### Tide Configuration
You can configure how tide works by editing the tide.toml configuration file.

The variable **root_dir** sets the starting point of the directories **tide** will watch.

The table **[command]** contains three variables:
+ **dev**: A list of all the commands to run in a development environment.
+ **prod**: A list of all the commands to run in a production environment.
+ **test**: A list of all the commands to run tests.

The table **[exclude]** contains three variables:
+ **dir**: A list of directories *tide* should not watch.
+ **file**: A list of files **tide** should not watch.
+ **ext**: A list of file extensions **tide** should not watch.

#### Example configuration
```toml
root_dir = "."

[command]
dev = [
  "python3 main.py", 
  "npx @tailwindcss/cli -i ./src/input.css -o ./src/output.css --minify", 
]
prod = []
test = []

[exclude]
dir = ["./.git", "./node_modules", "./.mypy_cache", "./.vscode"]
file = ["README.md"]
ext = []
```

### Installation
mac os (arm64) and linux (x86_64)
```sh 
curl -LsSf https://raw.githubusercontent.com/builtbyjb/tide/main/install.sh | sh
```

### Uninstall
mac os and linux
```bash
rm -rf ~/.local/bin/tide
```
