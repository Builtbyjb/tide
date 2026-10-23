package main

import (
	"fmt"
	"os"
	"runtime"

	"github.com/BurntSushi/toml"
)

// Config represents the structure of the tide.toml configuration file.
type Config struct {
	RootDir string        `toml:"root_dir"`
	Cmds    OSCommands    `toml:"os"`
	Exclude ExcludeConfig `toml:"exclude"`
}

// OSCommands holds platform-specific command maps.
type OSCommands struct {
	Unix    map[string][]string `toml:"unix"`
	Windows map[string][]string `toml:"windows"`
}

// ExcludeConfig lists directories, files and extensions to ignore.
type ExcludeConfig struct {
	Dir  []string `toml:"dir"`
	File []string `toml:"file"`
	Ext  []string `toml:"ext"`
}

// InitConfig creates a tide.toml configuration file at the given path if one
// does not already exist. The function will return nil if the file already
// exists. On successful creation it writes a sensible default configuration.
//
// Example usage:
//
//	if err := InitConfig("tide.toml"); err != nil {
//	    // handle error
//	}
func InitConfig(path string) error {
	if _, err := os.Stat(path); err == nil {
		fmt.Println("tide.toml file exists")
		return nil
	} else if !os.IsNotExist(err) {
		return fmt.Errorf("checking config file: %w", err)
	}

	defaultToml := `
	root_dir = "."

	[os.unix]
	dev = []

	[os.windows]
	dev = []

	[exclude]
	dir = [".git"]
	file = []
	ext = ["toml"]
	`

	// Write file with 0644 permissions
	if err := os.WriteFile(path, []byte(defaultToml), 0o644); err != nil {
		return fmt.Errorf("writing config file: %w", err)
	}

	fmt.Println("tide.toml file created")
	return nil
}

// LoadConfig reads and parses the tide.toml file at the given path. It returns
// a Config struct or an error.
func LoadConfig(path string) (*Config, error) {
	var config Config
	if _, err := os.Stat(path); err != nil {
		return nil, fmt.Errorf("reading config file: %w", err)
	}

	if _, err := toml.DecodeFile(path, &config); err != nil {
		return nil, fmt.Errorf("parsing config file: %w", err)
	}

	// Ensure non-nil maps to avoid nil map panics downstream
	if config.Cmds.Unix == nil {
		config.Cmds.Unix = map[string][]string{}
	}
	if config.Cmds.Windows == nil {
		config.Cmds.Windows = map[string][]string{}
	}

	return &config, nil
}

func ChooseCmds(config *Config, key string) []string {
	if runtime.GOOS == "windows" {
		if config.Cmds.Windows != nil {
			if v, ok := config.Cmds.Windows[key]; ok {
				return v
			}
		}
	} else {
		if config.Cmds.Unix != nil {
			if v, ok := config.Cmds.Unix[key]; ok {
				return v
			}
		}
	}
	return nil
}
