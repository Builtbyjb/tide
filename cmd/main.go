package main

import (
	"context"
	"fmt"
	"os"
	"os/signal"
	"syscall"

	"github.com/spf13/cobra"
)

const VERSION string = "0.2.0"

var (
	watch bool
	split bool
)

// Represents the application is called without any commands
var rootCmd = &cobra.Command{
	Use:   "tide",
	Short: "An application runner",
	Long:  "Runs multiple commands in parallel and hot reloads when a file changes",
	Run: func(cobraCmd *cobra.Command, args []string) {
		PrintUsage()
		os.Exit(0)
	},
}

// Represents when the application is called with the version command
var versionCmd = &cobra.Command{
	Use:   "version",
	Short: "Prints the current version of the application",
	Long:  "Prints the current version of the applications",
	Run: func(cobraCmd *cobra.Command, args []string) {
		fmt.Printf("tide version: %s\n", VERSION)
		os.Exit(0)
	},
}

// Represents when the application is called with the help command
var helpCmd = &cobra.Command{
	Use:   "help",
	Short: "Prints the help manual of the application",
	Long:  "Prints the help manual of the application",
	Run: func(cobraCmd *cobra.Command, args []string) {
		PrintUsage()
		os.Exit(0)
	},
}

// Represents when the application is called with the init command
var initCmd = &cobra.Command{
	Use:   "init",
	Short: "Creates a new tide.toml file",
	Long:  "Creates a new tide.toml file if one does not exist",
	Run: func(cobraCmd *cobra.Command, args []string) {
		if err := InitConfig("tide.toml"); err != nil {
			fmt.Fprintln(os.Stderr, "Error creating tide.toml file: ", err)
			os.Exit(1)
		}
	},
}

// Represents when the application is called with the run command
var runCmd = &cobra.Command{
	Use:   "run",
	Short: "Runs a specified command",
	Long:  "Runs one of the commands specified in the tide.toml config",
	Args:  cobra.ExactArgs(1),
	Run: func(cobraCmd *cobra.Command, args []string) {

		// Load config
		config, err := LoadConfig("tide.toml")
		if err != nil {
			fmt.Fprintln(os.Stderr, "Error reading tide config file: ", err)
			os.Exit(1)
		}

		cmds := ChooseCmds(config, args[0])
		if len(cmds) == 0 {
			fmt.Printf("No command found in the '%s' variable \n", args[0])
			os.Exit(1)
		}

		PrintTitle(VERSION)

		processManager := NewProcessManager()

		// Signal-aware context to control shutdown
		ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
		defer stop()

		if watch {
			RunWatch(ctx, processManager, config, cmds)
		} else {
			RunNonWatch(ctx, processManager, cmds)
		}
	},
}

func init() {
	// Add run command flags
	runCmdFlagSet := runCmd.Flags()
	runCmdFlagSet.BoolVarP(&watch, "watch", "w", false, "tide run [command] --watch")
	runCmdFlagSet.BoolVarP(&split, "split", "s", false, "tide run [command] --split")

	// Add sub commands
	rootCmd.AddCommand(runCmd, initCmd, helpCmd, versionCmd)
}

func main() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, "Failed to launch application: ", err)
		os.Exit(1)
	}
}
