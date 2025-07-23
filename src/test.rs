// #![cfg(test)]
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, UNIX_EPOCH};
use tempfile::TempDir;

use super::*;

// Helper to create a file with content and set modification time
fn create_file(path: &Path, content: &str, modify_time_offset: u64) {
  File::create(path).unwrap().write_all(content.as_bytes()).unwrap();
  let new_time = UNIX_EPOCH + Duration::from_secs(modify_time_offset);
  filetime::set_file_mtime(path, filetime::FileTime::from_system_time(new_time)).unwrap();
}

#[test]
fn test_input_args() {}

#[test]
fn test_new_file_detection() {
  let temp_dir = TempDir::new().unwrap();
  let file_path = temp_dir.path().join("test.txt");
  create_file(&file_path, "content", 1000);

  let mut files = HashMap::new();
  let ignore_dirs = vec![];
  let ignore_files = vec![];
  let ignore_exts = vec![];

  let result = watcher(temp_dir.path(), &ignore_dirs, &ignore_files, &ignore_exts, &mut files);

  assert!(result.unwrap(), "Should return true for new file");
  assert_eq!(files.len(), 1, "Should track one file");
  assert!(files.contains_key(&file_path), "Should track test.txt");
  assert_eq!(files.get(&file_path), Some(&1000), "Should store correct modification time");
}

#[test]
fn test_file_modification_detection() {
  let temp_dir = TempDir::new().unwrap();
  let file_path = temp_dir.path().join("test.txt");
  create_file(&file_path, "initial", 1000);

  let mut files = HashMap::new();
  files.insert(file_path.clone(), 1000);

  // Modify the file
  sleep(Duration::from_millis(100)); // Ensure time difference
  create_file(&file_path, "modified", 2000);

  let ignore_dirs = vec![];
  let ignore_files = vec![];
  let ignore_exts = vec![];

  let result = watcher(temp_dir.path(), &ignore_dirs, &ignore_files, &ignore_exts, &mut files);

  assert!(result.unwrap(), "Should return true for modified file");
  assert_eq!(files.len(), 1, "Should track one file");
  assert_eq!(files.get(&file_path), Some(&2000), "Should update modification time");
}

#[test]
fn test_directory_recursion() {
  let temp_dir = TempDir::new().unwrap();
  let sub_dir = temp_dir.path().join("subdir");
  fs::create_dir(&sub_dir).unwrap();
  let file_path = sub_dir.join("nested.txt");
  create_file(&file_path, "content", 1000);

  let mut files = HashMap::new();
  let ignore_dirs = vec![];
  let ignore_files = vec![];
  let ignore_exts = vec![];

  let result = watcher(temp_dir.path(), &ignore_dirs, &ignore_files, &ignore_exts, &mut files);

  assert!(result.unwrap(), "Should return true for nested file");
  assert_eq!(files.len(), 1, "Should track one file");
  assert!(files.contains_key(&file_path), "Should track nested.txt");
}

#[test]
fn test_ignore_directories() {
  let temp_dir = TempDir::new().unwrap();
  let ignore_dir = temp_dir.path().join("ignore_me");
  fs::create_dir(&ignore_dir).unwrap();
  let file_path = ignore_dir.join("test.txt");
  create_file(&file_path, "content", 1000);

  let mut files = HashMap::new();
  let ignore_dirs = vec!["ignore_me".to_string()];
  let ignore_files = vec![];
  let ignore_exts = vec![];

  let result = watcher(temp_dir.path(), &ignore_dirs, &ignore_files, &ignore_exts, &mut files);

  assert!(!result.unwrap(), "Should return false when ignoring directory");
  assert_eq!(files.len(), 0, "Should not track files in ignored directory");
}

#[test]
fn test_ignore_files() {
  let temp_dir = TempDir::new().unwrap();
  let file_path = temp_dir.path().join("ignore.txt");
  create_file(&file_path, "content", 1000);
  println!("{:#?}", temp_dir);

  let mut files = HashMap::new();
  let ignore_dirs = vec![];
  let ignore_files = vec!["ignore.txt".to_string()];
  let ignore_exts = vec![];

  let result = watcher(temp_dir.path(), &ignore_dirs, &ignore_files, &ignore_exts, &mut files);

  assert!(!result.unwrap(), "Should return false when ignoring file");
  assert_eq!(files.len(), 0, "Should not track ignored file");
}

#[test]
fn test_ignore_extensions() {
  let temp_dir = TempDir::new().unwrap();
  let file_path = temp_dir.path().join("test.ignore");
  create_file(&file_path, "content", 1000);

  let mut files = HashMap::new();
  let ignore_dirs = vec![];
  let ignore_files = vec![];
  let ignore_exts = vec!["ignore".to_string()];

  let result = watcher(temp_dir.path(), &ignore_dirs, &ignore_files, &ignore_exts, &mut files);

  assert!(!result.unwrap(), "Should return false when ignoring extension");
  assert_eq!(files.len(), 0, "Should not track file with ignored extension");
}

#[test]
fn test_empty_directory() {
  let temp_dir = TempDir::new().unwrap();
  let mut files = HashMap::new();
  let ignore_dirs = vec![];
  let ignore_files = vec![];
  let ignore_exts = vec![];

  let result = watcher(temp_dir.path(), &ignore_dirs, &ignore_files, &ignore_exts, &mut files);

  assert!(!result.unwrap(), "Should return false for empty directory");
  assert_eq!(files.len(), 0, "Should not track any files");
}
