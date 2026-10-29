package main

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"time"
)

func RunNonWatch(ctx context.Context, pm *ProcessManager, cmds []string) {
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

func RunWatch(ctx context.Context, pm *ProcessManager, config *Config, cmds []string) {
	fmt.Println("Watching...")
	state := make(map[string]int64)

	// Prime the state with an initial scan (so initial run is considered a change
	// only if files are discovered): Why?
	if _, err := Watch(config.RootDir, config.Exclude.Dir, config.Exclude.File, config.Exclude.Ext, state); err != nil {
		fmt.Fprintln(os.Stderr, "watch initial scan error:", err)
	}

	// Start commands initially
	if err := pm.SpawnCmds(ctx, cmds); err != nil {
		fmt.Fprintln(os.Stderr, "failed to spawn commands:", err)
		// Print out the commands that fail to spawn, exit if all commands fail
	}

	// Goroutine to log when any managed process exits (but we continue watching): Why?
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

	// Sets the watch function to run every 100 millisecond
	ticker := time.NewTicker(100 * time.Millisecond)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			fmt.Println("\nReceived shutdown signal!!! Shutting down gracefully...")
			_ = pm.KillAll()
			os.Exit(0)
		case <-ticker.C:
			changed, err := Watch(config.RootDir, config.Exclude.Dir, config.Exclude.File, config.Exclude.Ext, state)
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
