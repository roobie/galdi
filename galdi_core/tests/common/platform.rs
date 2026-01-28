#![allow(dead_code)]
// Platform-specific helper functions
use std::path::PathBuf;

/// Check if running on Unix
pub fn is_unix() -> bool {
    cfg!(unix)
}

/// Check if running on Windows
pub fn is_windows() -> bool {
    cfg!(windows)
}

/// Skip test if on Windows without symlink permissions
/// Windows requires admin privileges or developer mode for symlinks
#[cfg(windows)]
pub fn skip_if_no_symlink_permission() {
    use std::fs;
    use std::os::windows::fs as windows_fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let target_file = temp_dir.path().join("target.txt");
    let link_file = temp_dir.path().join("link.txt");

    // Create target file
    fs::File::create(&target_file).expect("Failed to create target");

    // Try to create symlink
    if windows_fs::symlink_file(&target_file, &link_file).is_err() {
        eprintln!("Skipping test: No symlink permission on Windows");
        eprintln!("Enable Developer Mode or run as Administrator");
        std::process::exit(0); // Exit gracefully for CI
    }
}

/// Unix version (no-op)
#[cfg(unix)]
pub fn skip_if_no_symlink_permission() {
    // Unix always supports symlinks
}

/// Get a test directory for symlink tests (skips if no permission on Windows)
pub fn get_test_symlink_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        use std::fs;
        use std::os::windows::fs as windows_fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().ok()?;
        let target_file = temp_dir.path().join("target.txt");
        let link_file = temp_dir.path().join("link.txt");

        fs::File::create(&target_file).ok()?;

        // Try to create symlink - if fails, return None
        windows_fs::symlink_file(&target_file, &link_file).ok()?;

        Some(temp_dir.path().to_path_buf())
    }

    #[cfg(unix)]
    {
        use tempfile::TempDir;
        Some(TempDir::new().ok()?.path().to_path_buf())
    }
}
