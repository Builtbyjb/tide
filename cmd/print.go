package main

import "fmt"

// Simple ANSI color helper. We avoid external dependencies and use basic ANSI escape codes.
// color codes: 30 black, 31 red, 32 green, 33 yellow, 34 blue, 35 magenta, 36 cyan, 37 white
func colorize(s string, colorCode int, bold bool) string {
	reset := "\033[0m"
	boldCode := ""
	if bold {
		boldCode = "\033[1m"
	}
	color := fmt.Sprintf("\033[%dm", colorCode)
	return fmt.Sprintf("%s%s%s%s", boldCode, color, s, reset)
}

// Usage prints a help/usage message to stdout.
func Usage() {
	header := colorize("Usage:", 36, true) // cyan bold
	usage := fmt.Sprintf(`
%s tide [options] [command]

Options:
    init               Create a tide configuration file
    run [command]      Run commands
        -w, --watch    Watch for changes and re-run commands
    -h, --help         Display this help message
    -v, --version      Display the version number
`, header)
	fmt.Println(usage)
}

// Title prints the ASCII art title with the given version string.
// The styling mirrors the original utilities: a blue bold banner with the version highlighted.
func Title(version string) {
	blueBold := func(s string) string { return colorize(s, 34, true) }
	versionStyled := colorize(version, 34, true) // blue bold version

	title := fmt.Sprintf(`%s
     __   _      __
    / /_ (_)____/ /___
   / __// // __  // _ \
  / /_ / // /_/ //  __/
  \__//_/ \__,_/ \___/  v%s

 Press Ctrl + C to exit`, "", versionStyled)

	// Surround the ASCII art with blue bold coloring
	fmt.Println(blueBold(title))
}
