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
