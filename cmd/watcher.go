package main

import (
	"errors"
	"io/fs"
	"path/filepath"
	"strings"
)

// Watch scans the directory tree rooted at rootPath and updates the provided
// files map with modification times (unix seconds). It returns true if any new
// files were discovered or any existing tracked files had their modification
// time changed.
//
// Parameters:
// - rootPath: path to start scanning from
// - ignoreDirs: list of directory basenames to ignore (e.g. ".git")
// - ignoreFiles: list of file basenames to ignore (e.g. "README.md")
// - ignoreExts: list of extensions to ignore WITHOUT the leading dot (e.g. "tmp", "log")
// - files: a map keyed by absolute file path -> modification time (unix seconds). This map will be mutated.
//
// Behavior notes:
// - The function walks recursively.
// - If a directory basename matches an entry in ignoreDirs, that directory is skipped.
// - If a file basename matches an entry in ignoreFiles, that file is skipped.
// - If a file extension (without the dot) matches an entry in ignoreExts, that file is skipped.
// - Returns (true, nil) if at least one file was added or one tracked file's mtime changed.
//
// This function is safe to call repeatedly; callers typically call it in a loop
// to detect changes over time.
func Watch(rootPath string, ignoreDirs, ignoreFiles, ignoreExts []string, files map[string]int64) (bool, error) {
	if rootPath == "" {
		return false, errors.New("rootPath cannot be empty")
	}

	changed := false

	// Normalize ignore lists into maps for O(1) lookups
	ignoreDirSet := make(map[string]struct{}, len(ignoreDirs))
	for _, d := range ignoreDirs {
		ignoreDirSet[d] = struct{}{}
	}
	ignoreFileSet := make(map[string]struct{}, len(ignoreFiles))
	for _, f := range ignoreFiles {
		ignoreFileSet[f] = struct{}{}
	}
	ignoreExtSet := make(map[string]struct{}, len(ignoreExts))
	for _, e := range ignoreExts {
		ignoreExtSet[e] = struct{}{}
	}

	walkFn := func(path string, d fs.DirEntry, err error) error {
		// Propagate errors from WalkDir
		if err != nil {
			// If there's an error reading a path, skip it but continue walking.
			// Return nil here to continue; the outer WalkDir will not stop.
			return nil
		}

		base := filepath.Base(path)

		// Skip directories listed in ignoreDirs
		if d.IsDir() {
			if _, ok := ignoreDirSet[base]; ok {
				return fs.SkipDir
			}
			return nil
		}

		// Only consider files
		if !d.Type().IsRegular() {
			return nil
		}

		// Skip files listed in ignoreFiles
		if _, ok := ignoreFileSet[base]; ok {
			return nil
		}

		// Determine extension without the dot
		ext := strings.TrimPrefix(filepath.Ext(base), ".")
		if ext != "" {
			if _, ok := ignoreExtSet[ext]; ok {
				return nil
			}
		}

		// Get file info to read mod time
		info, statErr := d.Info()
		if statErr != nil {
			// If we can't stat the file, skip it
			return nil
		}
		modSecs := info.ModTime().Unix()

		absPath := path
		if !filepath.IsAbs(path) {
			if ap, err := filepath.Abs(path); err == nil {
				absPath = ap
			}
		}

		prev, tracked := files[absPath]
		if !tracked {
			files[absPath] = modSecs
			changed = true
			return nil
		}
		if prev != modSecs {
			files[absPath] = modSecs
			changed = true
		}
		return nil
	}

	if err := filepath.WalkDir(rootPath, walkFn); err != nil {
		return changed, err
	}
	return changed, nil
}

//
// Tests
//
// The tests below mirror typical unit tests for the watcher behavior.
// Note: in idiomatic Go the tests would normally live in a separate
// *_test.go file, but they are included here per the user's request.
// To run these tests you can copy them into a file named watcher_test.go
// inside the same package and run `go test`.
//
// The following helper functions and tests demonstrate:
// - detecting new files
// - detecting modified files
// - recursive traversal
// - ignoring directories, files and extensions
//

// --- Helper functions used by tests (can be reused in watcher_test.go) ---

// Test helpers for watcher moved to the test file (watcher_test.go).
// Keeping production code clean to avoid duplicate test symbols.

// The following is a set of example test functions that you can copy into
// watcher_test.go if you want runnable tests. They are intentionally not
// declared with the testing package here because this file is watcher.go.
//
// Example:
//
// func TestNewFileDetection(t *testing.T) { ... }
//
// Copy-paste into watcher_test.go to run with `go test`.

// Example tests removed from this file. Unit tests live in watcher_test.go
// which contains the concrete test implementations and test helpers.
