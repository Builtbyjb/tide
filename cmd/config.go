package main

import (
	"fmt"
	"os"

	"github.com/BurntSushi/toml"
)

// Config represents the structure of the tide.toml configuration file.
type Config struct {
	RootDir string        `toml:"root_dir"`
	OS      OSConfig      `toml:"os"`
	Exclude ExcludeConfig `toml:"exclude"`
}

// OSConfig holds platform-specific command maps.
type OSConfig struct {
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
	// If the file already exists, do nothing.
	if _, err := os.Stat(path); err == nil {
		fmt.Println("tide.toml file exists")
		return nil
	} else if !os.IsNotExist(err) {
		// Some other error while trying to stat the file.
		return fmt.Errorf("checking config file: %w", err)
	}

	// Default TOML string (human-friendly)
	defaultToml := `root_dir = "."

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
	var cfg Config
	if _, err := os.Stat(path); err != nil {
		return nil, fmt.Errorf("reading config file: %w", err)
	}

	if _, err := toml.DecodeFile(path, &cfg); err != nil {
		return nil, fmt.Errorf("parsing config file: %w", err)
	}

	// Ensure non-nil maps to avoid nil map panics downstream
	if cfg.OS.Unix == nil {
		cfg.OS.Unix = map[string][]string{}
	}
	if cfg.OS.Windows == nil {
		cfg.OS.Windows = map[string][]string{}
	}

	return &cfg, nil
}
