use std::path::{PathBuf, StripPrefixError};

use crate::plumbah::PlumbahError;

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Symlink loop detected at: {0}")]
    SymlinkLoop(PathBuf),
}

impl ScanError {
    pub fn to_plumbah_error(&self) -> PlumbahError {
        match self {
            Self::PathNotFound(path) => PlumbahError {
                code: "PATH_NOT_FOUND".to_string(),
                message: format!("Path not found: {}", path.display()),
                path: Some(path.clone()),
                recoverable: false,
                context: None,
            },
            Self::PermissionDenied(path) => PlumbahError {
                code: "PERMISSION_DENIED".to_string(),
                message: format!("Permission denied: {}", path.display()),
                path: Some(path.clone()),
                recoverable: false,
                context: None,
            },
            Self::Io(error) => PlumbahError {
                code: "IO_ERROR".to_string(),
                message: error.to_string(),
                path: None,
                recoverable: false,
                context: None,
            },
            Self::SymlinkLoop(path) => PlumbahError {
                code: "SYMLINK_LOOP".to_string(),
                message: format!("Symlink loop detected at: {}", path.display()),
                path: Some(path.clone()),
                recoverable: false,
                context: None,
            },
        }
    }
}

impl From<StripPrefixError> for ScanError {
    fn from(err: StripPrefixError) -> Self {
        // GlobError contains either WalkDir or Pattern errors
        // Convert to IO error with appropriate context
        ScanError::Io(std::io::Error::other(err.to_string()))
    }
}

impl From<ignore::Error> for ScanError {
    fn from(err: ignore::Error) -> Self {
        // Convert ignore::Error to ScanError::Io
        ScanError::Io(std::io::Error::other(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_path_not_found_to_plumbah_error() {
        let path = PathBuf::from("/nonexistent");
        let error = ScanError::PathNotFound(path.clone());
        let plumbah_error = error.to_plumbah_error();

        assert_eq!(plumbah_error.code, "PATH_NOT_FOUND");
        assert!(plumbah_error.message.contains("Path not found"));
        assert_eq!(plumbah_error.path, Some(path));
        assert!(!plumbah_error.recoverable);
    }

    #[test]
    fn test_permission_denied_to_plumbah_error() {
        let path = PathBuf::from("/etc/shadow");
        let error = ScanError::PermissionDenied(path.clone());
        let plumbah_error = error.to_plumbah_error();

        assert_eq!(plumbah_error.code, "PERMISSION_DENIED");
        assert!(plumbah_error.message.contains("Permission denied"));
        assert_eq!(plumbah_error.path, Some(path));
        assert!(!plumbah_error.recoverable);
    }

    #[test]
    fn test_io_error_to_plumbah_error() {
        let io_error = std::io::Error::other("test error");
        let error = ScanError::Io(io_error);
        let plumbah_error = error.to_plumbah_error();

        assert_eq!(plumbah_error.code, "IO_ERROR");
        assert!(plumbah_error.message.contains("test error"));
        assert_eq!(plumbah_error.path, None);
        assert!(!plumbah_error.recoverable);
    }

    #[test]
    fn test_symlink_loop_to_plumbah_error() {
        let path = PathBuf::from("/loop/link");
        let error = ScanError::SymlinkLoop(path.clone());
        let plumbah_error = error.to_plumbah_error();

        assert_eq!(plumbah_error.code, "SYMLINK_LOOP");
        assert!(plumbah_error.message.contains("Symlink loop detected"));
        assert_eq!(plumbah_error.path, Some(path));
        assert!(!plumbah_error.recoverable);
    }

    #[test]
    fn test_plumbah_error_code_format() {
        // Verify all error codes are UPPER_CASE with underscores
        let test_cases = vec![
            ScanError::PathNotFound(PathBuf::from("/test")),
            ScanError::PermissionDenied(PathBuf::from("/test")),
            ScanError::Io(std::io::Error::other("test")),
            ScanError::SymlinkLoop(PathBuf::from("/test")),
        ];

        for error in test_cases {
            let plumbah_error = error.to_plumbah_error();
            // Verify code is uppercase and uses underscores
            assert_eq!(
                plumbah_error.code,
                plumbah_error.code.to_uppercase(),
                "Error code should be uppercase"
            );
            assert!(
                !plumbah_error.code.contains('-'),
                "Error code should use underscores, not hyphens"
            );
        }
    }

    #[test]
    fn test_plumbah_error_includes_path() {
        // PathNotFound should include path
        let path = PathBuf::from("/test/path");
        let error = ScanError::PathNotFound(path.clone());
        let plumbah_error = error.to_plumbah_error();
        assert_eq!(plumbah_error.path, Some(path));

        // IO error should not include path
        let io_error = ScanError::Io(std::io::Error::other("test"));
        let plumbah_error = io_error.to_plumbah_error();
        assert_eq!(plumbah_error.path, None);
    }

    #[test]
    fn test_plumbah_error_recoverable_flag() {
        // All current errors are non-recoverable
        let test_cases = vec![
            ScanError::PathNotFound(PathBuf::from("/test")),
            ScanError::PermissionDenied(PathBuf::from("/test")),
            ScanError::Io(std::io::Error::other("test")),
            ScanError::SymlinkLoop(PathBuf::from("/test")),
        ];

        for error in test_cases {
            let plumbah_error = error.to_plumbah_error();
            assert!(
                !plumbah_error.recoverable,
                "All scan errors should be non-recoverable"
            );
        }
    }

    #[test]
    fn test_error_conversions_from_strip_prefix() {
        // Create a StripPrefixError
        let base = Path::new("/base");
        let other = Path::new("/other");

        if let Err(strip_error) = other.strip_prefix(base) {
            let scan_error: ScanError = strip_error.into();
            match scan_error {
                ScanError::Io(_) => {
                    // Successfully converted to IO error
                }
                _ => panic!("StripPrefixError should convert to ScanError::Io"),
            }
        }
    }

    #[test]
    fn test_error_serialization_roundtrip() {
        use serde_json;

        let path = PathBuf::from("/test/path");
        let error = ScanError::PathNotFound(path);
        let plumbah_error = error.to_plumbah_error();

        // Serialize to JSON
        let json = serde_json::to_string(&plumbah_error).expect("Failed to serialize");

        // Deserialize back
        let deserialized: PlumbahError =
            serde_json::from_str(&json).expect("Failed to deserialize");

        // Verify fields match
        assert_eq!(plumbah_error.code, deserialized.code);
        assert_eq!(plumbah_error.message, deserialized.message);
        assert_eq!(plumbah_error.path, deserialized.path);
        assert_eq!(plumbah_error.recoverable, deserialized.recoverable);
    }
}
