# Tide

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![GitHub release](https://img.shields.io/github/release/builtbyjb/tide.svg)](https://github.com/builtbyjb/tide/releases)

**Tide** is a powerful CLI tool that runs multiple commands concurrently with built-in live reloading capabilities. Perfect for development workflows where you need to run multiple processes simultaneously and automatically restart them when files change.

## Features

- **Concurrent Execution**: Run multiple commands simultaneously
- **Live Reloading**: Automatically restart commands when files change
- **Cross-Platform**: Works on macOS, Linux, and Windows
- **Flexible Configuration**: Customize behavior with TOML configuration
- **Intelligent File Watching**: Exclude specific directories, files, or extensions
- **OS-Specific Commands**: Define different commands for different operating systems

## Installation

### macOS (ARM64) & Linux (x86_64)
```bash
curl -LsSf https://raw.githubusercontent.com/builtbyjb/tide/main/install.sh | sh
```

### Windows
```powershell
powershell -ExecutionPolicy ByPass -c "irm https://raw.githubusercontent.com/builtbyjb/tide/main/install.ps1 | iex"
```

## Commands

### `tide init`
Creates a new `tide.toml` configuration file in the current directory with sensible defaults.

### `tide run <variable>`
Runs the commands defined for the specified variable name in your configuration. The variable name can be anything you choose (e.g., `dev`, `prod`, `test`, `build`, `start`, `server`, etc.).

**Examples:**
```bash
# Run commands defined under 'dev' variable
tide run dev

# Run commands defined under 'prod' variable
tide run prod

# Run commands defined under 'test' variable
tide run test
```

### `tide run <variable> --watch`
Runs commands with live reloading enabled. Commands will automatically restart when files are modified.

```bash
# Run with file watching
tide run dev --watch
```

### `tide --version`
Display the current version of Tide.

## Configuration

Tide uses a `tide.toml` file for configuration. This file defines which commands to run for different environments and how file watching should behave.

### Configuration Structure

| Setting | Description |
|---------|-------------|
| `root_dir` | The root directory for file watching (default: `"."`) |
| `[os.unix]` | Commands for Unix-based systems (macOS, Linux) |
| `[os.windows]` | Commands for Windows systems |
| `[exclude]` | Files, directories, and extensions to exclude from watching |

### Variable Commands

Each OS section can define commands under any variable name you choose. The variable names are completely flexible and can be anything that makes sense for your workflow.

**Variable Format**: Each variable should contain a list of commands. This can be multiple commands or a single command (still in list format).

**Examples:**
```toml
# Multiple commands
dev = [
  "npm run dev",
  "python3 -m http.server 8080",
  "npx tailwindcss --watch"
]

# Single command (still in list format)
build = ["npm run build"]
```

### Exclusion Options

The `[exclude]` table supports:
- **`dir`**: Directories to exclude from watching
- **`file`**: Specific files to exclude
- **`ext`**: File extensions to exclude

## Example Configuration

```toml
root_dir = "."

[os.unix]
dev = [
  "npm run dev",
  "python3 -m http.server 8080",
  "npx tailwindcss -i ./src/input.css -o ./dist/output.css --watch"
]


[os.windows]
dev = [
  "npm run dev",
  "python -m http.server 8080",
  "npx tailwindcss -i .\\src\\input.css -o .\\dist\\output.css --watch"
]

[exclude]
dir = [".git", "node_modules", ".mypy_cache", "__pycache__", "dist"]
file = ["README.md", "LICENSE"]
ext = ["toml", "log", "tmp"]
```

## Uninstall

### macOS & Linux
```bash
rm -rf ~/.local/bin/tide
```

### Windows
```powershell
powershell -ExecutionPolicy ByPass -c "irm https://raw.githubusercontent.com/builtbyjb/tide/main/uninstall.ps1 | iex"
```
