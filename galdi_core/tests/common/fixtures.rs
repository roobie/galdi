#![allow(dead_code)]
// Fixture creation helpers for tests
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Create a temporary test directory that will be automatically cleaned up
pub fn create_test_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Create a file with specific content in the given directory
pub fn create_file_with_content(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
    let file_path = dir.join(name);
    let mut file = fs::File::create(&file_path).expect("Failed to create file");
    file.write_all(content).expect("Failed to write content");
    file_path
}

/// Create a file with a specific size filled with zeros
pub fn create_file_with_size(dir: &Path, name: &str, size: u64) -> PathBuf {
    let file_path = dir.join(name);
    let file = fs::File::create(&file_path).expect("Failed to create file");
    file.set_len(size).expect("Failed to set file size");
    file_path
}

/// Create an empty directory
pub fn create_dir(parent: &Path, name: &str) -> PathBuf {
    let dir_path = parent.join(name);
    fs::create_dir(&dir_path).expect("Failed to create directory");
    dir_path
}

/// Create a deep directory hierarchy (nested directories)
pub fn create_deep_hierarchy(dir: &Path, depth: usize) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut current = dir.to_path_buf();

    for i in 0..depth {
        current = current.join(format!("level_{}", i));
        fs::create_dir(&current).expect("Failed to create directory");
        paths.push(current.clone());
    }

    paths
}

/// Create a wide directory hierarchy (many files in one directory)
pub fn create_wide_hierarchy(dir: &Path, count: usize) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    for i in 0..count {
        let file_path = dir.join(format!("file_{}.txt", i));
        fs::File::create(&file_path).expect("Failed to create file");
        paths.push(file_path);
    }

    paths
}

/// Create a symbolic link (Unix only for now)
#[cfg(unix)]
pub fn create_symlink(dir: &Path, name: &str, target: &Path) -> PathBuf {
    use std::os::unix::fs as unix_fs;
    let link_path = dir.join(name);
    unix_fs::symlink(target, &link_path).expect("Failed to create symlink");
    link_path
}

/// Create a symbolic link (Windows)
#[cfg(windows)]
pub fn create_symlink(dir: &Path, name: &str, target: &Path) -> PathBuf {
    use std::os::windows::fs as windows_fs;
    let link_path = dir.join(name);

    // Try to create symlink, but skip if no permission
    if target.is_dir() {
        windows_fs::symlink_dir(target, &link_path).ok();
    } else {
        windows_fs::symlink_file(target, &link_path).ok();
    }

    link_path
}

/// Set Unix file permissions
#[cfg(unix)]
pub fn set_permissions(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    let permissions = fs::Permissions::from_mode(mode);
    fs::set_permissions(path, permissions).expect("Failed to set permissions");
}

/// Set Windows file to read-only
#[cfg(windows)]
pub fn set_readonly(path: &Path, readonly: bool) {
    let mut permissions = fs::metadata(path)
        .expect("Failed to get metadata")
        .permissions();
    permissions.set_readonly(readonly);
    fs::set_permissions(path, permissions).expect("Failed to set permissions");
}
