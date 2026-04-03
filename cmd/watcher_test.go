package main

import (
	"os"
	"path/filepath"
	"testing"
	"time"
)

// helper to create a file with given content and set its mod time (seconds since epoch)
func createFile(t *testing.T, path, content string, modSec int64) {
	t.Helper()
	if err := os.WriteFile(path, []byte(content), 0644); err != nil {
		t.Fatalf("failed to write file %s: %v", path, err)
	}
	modTime := time.Unix(modSec, 0)
	if err := os.Chtimes(path, modTime, modTime); err != nil {
		t.Fatalf("failed to chtimes file %s: %v", path, err)
	}
}

func TestNewFileDetection(t *testing.T) {
	tmpDir := t.TempDir()
	filePath := filepath.Join(tmpDir, "test.txt")
	createFile(t, filePath, "content", 1000)

	files := make(map[string]int64)
	ignoreDirs := []string{}
	ignoreFiles := []string{}
	ignoreExts := []string{}

	changed, err := Watch(tmpDir, ignoreDirs, ignoreFiles, ignoreExts, files)
	if err != nil {
		t.Fatalf("watch returned error: %v", err)
	}
	if !changed {
		t.Fatalf("expected watcher to detect new file")
	}
	if len(files) != 1 {
		t.Fatalf("expected files map to contain 1 entry, got %d", len(files))
	}
	if v, ok := files[filePath]; !ok || v != 1000 {
		t.Fatalf("expected files[%s] == 1000, got (%d, %v)", filePath, v, ok)
	}
}

func TestFileModificationDetection(t *testing.T) {
	tmpDir := t.TempDir()
	filePath := filepath.Join(tmpDir, "test.txt")
	createFile(t, filePath, "initial", 1000)

	files := map[string]int64{
		filePath: 1000,
	}

	// modify file
	createFile(t, filePath, "modified", 2000)

	ignoreDirs := []string{}
	ignoreFiles := []string{}
	ignoreExts := []string{}

	changed, err := Watch(tmpDir, ignoreDirs, ignoreFiles, ignoreExts, files)
	if err != nil {
		t.Fatalf("watch returned error: %v", err)
	}
	if !changed {
		t.Fatalf("expected watcher to detect modification")
	}
	if len(files) != 1 {
		t.Fatalf("expected files map to contain 1 entry, got %d", len(files))
	}
	if v := files[filePath]; v != 2000 {
		t.Fatalf("expected files[%s] == 2000, got %d", filePath, v)
	}
}

func TestDirectoryRecursion(t *testing.T) {
	tmpDir := t.TempDir()
	subDir := filepath.Join(tmpDir, "subdir")
	if err := os.Mkdir(subDir, 0755); err != nil {
		t.Fatalf("failed to create subdir: %v", err)
	}
	filePath := filepath.Join(subDir, "nested.txt")
	createFile(t, filePath, "content", 1000)

	files := make(map[string]int64)
	ignoreDirs := []string{}
	ignoreFiles := []string{}
	ignoreExts := []string{}

	changed, err := Watch(tmpDir, ignoreDirs, ignoreFiles, ignoreExts, files)
	if err != nil {
		t.Fatalf("watch returned error: %v", err)
	}
	if !changed {
		t.Fatalf("expected watcher to detect nested file")
	}
	if len(files) != 1 {
		t.Fatalf("expected files map to contain 1 entry, got %d", len(files))
	}
	if _, ok := files[filePath]; !ok {
		t.Fatalf("expected files to contain nested file %s", filePath)
	}
}

func TestIgnoreDirectories(t *testing.T) {
	tmpDir := t.TempDir()
	ignoreDir := filepath.Join(tmpDir, "ignore_me")
	if err := os.Mkdir(ignoreDir, 0755); err != nil {
		t.Fatalf("failed to create ignore_dir: %v", err)
	}
	filePath := filepath.Join(ignoreDir, "test.txt")
	createFile(t, filePath, "content", 1000)

	files := make(map[string]int64)
	ignoreDirs := []string{"ignore_me"}
	ignoreFiles := []string{}
	ignoreExts := []string{}

	changed, err := Watch(tmpDir, ignoreDirs, ignoreFiles, ignoreExts, files)
	if err != nil {
		t.Fatalf("watch returned error: %v", err)
	}
	if changed {
		t.Fatalf("expected watcher NOT to detect files in ignored directory")
	}
	if len(files) != 0 {
		t.Fatalf("expected files map to be empty, got %d entries", len(files))
	}
}

func TestIgnoreFiles(t *testing.T) {
	tmpDir := t.TempDir()
	filePath := filepath.Join(tmpDir, "ignore.txt")
	createFile(t, filePath, "content", 1000)

	files := make(map[string]int64)
	ignoreDirs := []string{}
	ignoreFiles := []string{"ignore.txt"}
	ignoreExts := []string{}

	changed, err := Watch(tmpDir, ignoreDirs, ignoreFiles, ignoreExts, files)
	if err != nil {
		t.Fatalf("watch returned error: %v", err)
	}
	if changed {
		t.Fatalf("expected watcher NOT to detect ignored file")
	}
	if len(files) != 0 {
		t.Fatalf("expected files map to be empty, got %d entries", len(files))
	}
}

func TestIgnoreExtensions(t *testing.T) {
	tmpDir := t.TempDir()
	filePath := filepath.Join(tmpDir, "test.ignore")
	createFile(t, filePath, "content", 1000)

	files := make(map[string]int64)
	ignoreDirs := []string{}
	ignoreFiles := []string{}
	ignoreExts := []string{"ignore"}

	changed, err := Watch(tmpDir, ignoreDirs, ignoreFiles, ignoreExts, files)
	if err != nil {
		t.Fatalf("watch returned error: %v", err)
	}
	if changed {
		t.Fatalf("expected watcher NOT to detect file with ignored extension")
	}
	if len(files) != 0 {
		t.Fatalf("expected files map to be empty, got %d entries", len(files))
	}
}

func TestEmptyDirectory(t *testing.T) {
	tmpDir := t.TempDir()

	files := make(map[string]int64)
	ignoreDirs := []string{}
	ignoreFiles := []string{}
	ignoreExts := []string{}

	changed, err := Watch(tmpDir, ignoreDirs, ignoreFiles, ignoreExts, files)
	if err != nil {
		t.Fatalf("watch returned error: %v", err)
	}
	if changed {
		t.Fatalf("expected watcher NOT to detect changes in empty directory")
	}
	if len(files) != 0 {
		t.Fatalf("expected files map to be empty, got %d entries", len(files))
	}
}
