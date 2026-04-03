package main

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"os/signal"
	"runtime"
	"syscall"
	"time"
)

// VERSION for this binary
const VERSION = "0.1.2"

func chooseCmds(cfg *Config, key string) []string {
	if runtime.GOOS == "windows" {
		if cfg.OS.Windows != nil {
			if v, ok := cfg.OS.Windows[key]; ok {
				return v
			}
		}
	} else {
		if cfg.OS.Unix != nil {
			if v, ok := cfg.OS.Unix[key]; ok {
				return v
			}
		}
	}
	return nil
}

func runNonWatch(ctx context.Context, pm *ProcessManager, cmds []string) {
	// Start commands
	if err := pm.SpawnCmds(ctx, cmds); err != nil {
		fmt.Fprintln(os.Stderr, "failed to spawn commands:", err)
		os.Exit(1)
	}

	// Wait for any process to exit
	label, err := pm.WaitAny()
	// Ensure we attempt to kill all managed processes before exiting
	_ = pm.KillAll()

	exitCode := 0
	if err != nil {
		// Try to determine exit code from exec.ExitError if available
		if exitErr, ok := err.(*exec.ExitError); ok {
			exitCode = exitErr.ExitCode()
		} else {
			// unknown error, use 1
			exitCode = 1
		}
		fmt.Fprintf(os.Stderr, "Command '%s' exited with error: %v (code %d)\n", label, err, exitCode)
	} else {
		fmt.Printf("Command '%s' exited successfully\n", label)
		exitCode = 0
	}
	os.Exit(exitCode)
}

func runWatch(ctx context.Context, pm *ProcessManager, cfg *Config, cmds []string) {
	fmt.Println("Watching...")
	state := make(map[string]int64)

	// Prime the state with an initial scan (so initial run is considered a change
	// only if files are discovered)
	if _, err := Watch(cfg.RootDir, cfg.Exclude.Dir, cfg.Exclude.File, cfg.Exclude.Ext, state); err != nil {
		fmt.Fprintln(os.Stderr, "watch initial scan error:", err)
	}

	// Start commands initially
	if err := pm.SpawnCmds(ctx, cmds); err != nil {
		fmt.Fprintln(os.Stderr, "failed to spawn commands:", err)
		// continue: watcher will try to respawn on changes
	}

	// Goroutine to log when any managed process exits (but we continue watching)
	go func() {
		for {
			label, err := pm.WaitAny()
			if err != nil {
				if exitErr, ok := err.(*exec.ExitError); ok {
					fmt.Fprintf(os.Stderr, "Command '%s' exited with code %d\n", label, exitErr.ExitCode())
				} else {
					fmt.Fprintf(os.Stderr, "Command '%s' exited with error: %v\n", label, err)
				}
			} else {
				fmt.Printf("Command '%s' exited successfully\n", label)
			}
		}
	}()

	ticker := time.NewTicker(100 * time.Millisecond)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			fmt.Println("\nReceived shutdown signal!!! Shutting down gracefully...")
			_ = pm.KillAll()
			os.Exit(0)
		case <-ticker.C:
			changed, err := Watch(cfg.RootDir, cfg.Exclude.Dir, cfg.Exclude.File, cfg.Exclude.Ext, state)
			if err != nil {
				// Log and continue watching
				fmt.Fprintln(os.Stderr, "watch error:", err)
				continue
			}
			if changed {
				// restart commands
				if err := pm.SpawnCmds(ctx, cmds); err != nil {
					fmt.Fprintln(os.Stderr, "failed to respawn commands:", err)
				}
			}
		}
	}
}

func main() {
	if len(os.Args) < 2 {
		Usage()
		return
	}

	switch os.Args[1] {
	case "init":
		if err := InitConfig("tide.toml"); err != nil {
			fmt.Fprintln(os.Stderr, "Error creating a toml configuration file:", err)
			os.Exit(1)
		}
		return
	case "--version", "-v":
		fmt.Printf("tide v%s\n", VERSION)
		return
	case "-h", "--help":
		Usage()
		return
	case "run":
		if len(os.Args) < 3 {
			fmt.Fprintln(os.Stderr, "run requires a command key (e.g. dev)")
			os.Exit(1)
		}
		key := os.Args[2]
		watchMode := false
		if len(os.Args) >= 4 {
			if os.Args[3] == "--watch" || os.Args[3] == "-w" {
				watchMode = true
			}
		}

		// Load config
		cfg, err := LoadConfig("tide.toml")
		if err != nil {
			fmt.Fprintln(os.Stderr, "Error reading config file:", err)
			os.Exit(1)
		}

		cmds := chooseCmds(cfg, key)
		if len(cmds) == 0 {
			fmt.Printf("No command found in the '%s' variable\n", key)
			os.Exit(1)
		}

		Title(VERSION)

		pm := NewProcessManager()

		// Signal-aware context to control shutdown
		ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
		defer stop()

		if watchMode {
			runWatch(ctx, pm, cfg, cmds)
		} else {
			runNonWatch(ctx, pm, cmds)
		}
	default:
		Usage()
	}
}
