use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Snapshot data structures for galdi.
///
/// This module defines the data structures used to represent filesystem snapshots,
/// including files, directories, and symlinks, along with their metadata.

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    #[serde(rename = "$plumbah")]
    pub plumbah: PlumbahObject,

    /// Snapshot (structural) version, e.g., "1.0" - used by galdi tools to manage compatibility.
    pub version: String,

    pub root: PathBuf,
    pub checksum_algorithm: ChecksumAlgorithm,
    pub count: usize,
    pub entries: Vec<SnapshotEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotEntry {
    pub path: PathBuf, // Relative to root
    #[serde(rename = "type")]
    pub entry_type: EntryType,
    pub size: Option<u64>,
    /// for unix e.g ."0644" - posix file mode as octal string
    /// for windows e.g. "00000020" - win32 file attributes bitfield as hex string
    pub mode: Option<String>,
    pub mtime: DateTime<Utc>,
    pub checksum: Option<String>, // "xxhash64:abc123"
    pub target: Option<PathBuf>,  // For symlinks
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    /// For special files, not yet mapped specifically, like device nodes, FIFOs, sockets, etc.
    Undefined = 0,
    File,
    Directory,
    Symlink,
}

/**
 * GPT 5.2 (2025-01-13)
 * Practical recommendations
 * For new 64‑bit code:
 * Prefer XXH3_64bits for most non‑cryptographic uses (hash maps, Bloom filters, dedup, checksums).
 * When you want an extra‑large space:
 * Use XXH128 (e.g., content IDs, long‑term storage, cross‑system fingerprints).
 *
 * So, in a loose order of preference for general purpose checksums for galdi purposes:
 * 1. XXH3_64
 * 3. Sha256
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChecksumAlgorithm {
    XXH3_64,
    Sha256,
}

use std::str::FromStr;

use crate::PlumbahObject;

impl FromStr for ChecksumAlgorithm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "xxh3_64" => Ok(ChecksumAlgorithm::XXH3_64),
            "sha256" => Ok(ChecksumAlgorithm::Sha256),
            _ => Err(format!("Invalid checksum algorithm: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Meta, Status};
    use serde_json;

    #[test]
    fn test_entry_type_serializes_lowercase() {
        // Test that EntryType enum serializes to lowercase
        let file_json = serde_json::to_string(&EntryType::File).unwrap();
        assert_eq!(file_json, r#""file""#);

        let dir_json = serde_json::to_string(&EntryType::Directory).unwrap();
        assert_eq!(dir_json, r#""directory""#);

        let symlink_json = serde_json::to_string(&EntryType::Symlink).unwrap();
        assert_eq!(symlink_json, r#""symlink""#);

        let undefined_json = serde_json::to_string(&EntryType::Undefined).unwrap();
        assert_eq!(undefined_json, r#""undefined""#);
    }

    #[test]
    fn test_entry_type_values() {
        // Verify discriminants
        assert_eq!(EntryType::Undefined as u8, 0);
        assert_eq!(EntryType::File as u8, 1);
        assert_eq!(EntryType::Directory as u8, 2);
        assert_eq!(EntryType::Symlink as u8, 3);
    }

    #[test]
    fn test_checksum_algorithm_from_str_xxh3() {
        let result = ChecksumAlgorithm::from_str("xxh3_64").unwrap();
        assert_eq!(result, ChecksumAlgorithm::XXH3_64);
    }

    #[test]
    fn test_checksum_algorithm_from_str_sha256() {
        let result = ChecksumAlgorithm::from_str("sha256").unwrap();
        assert_eq!(result, ChecksumAlgorithm::Sha256);
    }

    #[test]
    fn test_checksum_algorithm_case_insensitive() {
        assert_eq!(
            ChecksumAlgorithm::from_str("XXH3_64").unwrap(),
            ChecksumAlgorithm::XXH3_64
        );
        assert_eq!(
            ChecksumAlgorithm::from_str("SHA256").unwrap(),
            ChecksumAlgorithm::Sha256
        );
        assert_eq!(
            ChecksumAlgorithm::from_str("Sha256").unwrap(),
            ChecksumAlgorithm::Sha256
        );
    }

    #[test]
    fn test_checksum_algorithm_invalid_returns_err() {
        let result = ChecksumAlgorithm::from_str("md5");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid checksum algorithm"));
    }

    #[test]
    fn test_snapshot_serialization_roundtrip() {
        let meta = Meta::new(
            "galdi_snapshot",
            "0.1.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );

        let snapshot = Snapshot {
            plumbah: PlumbahObject::new(Status::Ok, meta),
            version: "1.0".to_string(),
            root: PathBuf::from("/test"),
            checksum_algorithm: ChecksumAlgorithm::XXH3_64,
            count: 0,
            entries: vec![],
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: Snapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(snapshot.version, deserialized.version);
        assert_eq!(snapshot.root, deserialized.root);
        assert_eq!(snapshot.checksum_algorithm, deserialized.checksum_algorithm);
        assert_eq!(snapshot.count, deserialized.count);
    }

    #[test]
    fn test_snapshot_entry_serialization_roundtrip() {
        let entry = SnapshotEntry {
            path: PathBuf::from("test.txt"),
            entry_type: EntryType::File,
            size: Some(1024),
            mode: Some("644".to_string()),
            mtime: Utc::now(),
            checksum: Some("xxh3_64:abc123".to_string()),
            target: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: SnapshotEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.path, deserialized.path);
        assert_eq!(entry.entry_type, deserialized.entry_type);
        assert_eq!(entry.size, deserialized.size);
        assert_eq!(entry.checksum, deserialized.checksum);
    }

    #[test]
    fn test_snapshot_with_file_entries() {
        let meta = Meta::new(
            "galdi_snapshot",
            "0.1.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );

        let file_entry = SnapshotEntry {
            path: PathBuf::from("file.txt"),
            entry_type: EntryType::File,
            size: Some(512),
            mode: Some("644".to_string()),
            mtime: Utc::now(),
            checksum: Some("xxh3_64:abc123".to_string()),
            target: None,
        };

        let snapshot = Snapshot {
            plumbah: PlumbahObject::new(Status::Ok, meta),
            version: "1.0".to_string(),
            root: PathBuf::from("/test"),
            checksum_algorithm: ChecksumAlgorithm::XXH3_64,
            count: 1,
            entries: vec![file_entry.clone()],
        };

        assert_eq!(snapshot.entries.len(), 1);
        assert_eq!(snapshot.entries[0].entry_type, EntryType::File);
        assert!(snapshot.entries[0].checksum.is_some());
    }

    #[test]
    fn test_snapshot_with_directory_entries() {
        let meta = Meta::new(
            "galdi_snapshot",
            "0.1.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );

        let dir_entry = SnapshotEntry {
            path: PathBuf::from("subdir"),
            entry_type: EntryType::Directory,
            size: Some(0),
            mode: Some("755".to_string()),
            mtime: Utc::now(),
            checksum: None,
            target: None,
        };

        let snapshot = Snapshot {
            plumbah: PlumbahObject::new(Status::Ok, meta),
            version: "1.0".to_string(),
            root: PathBuf::from("/test"),
            checksum_algorithm: ChecksumAlgorithm::XXH3_64,
            count: 1,
            entries: vec![dir_entry.clone()],
        };

        assert_eq!(snapshot.entries[0].entry_type, EntryType::Directory);
        assert!(snapshot.entries[0].checksum.is_none());
    }

    #[test]
    fn test_snapshot_with_symlink_entries() {
        let meta = Meta::new(
            "galdi_snapshot",
            "0.1.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );

        let symlink_entry = SnapshotEntry {
            path: PathBuf::from("link"),
            entry_type: EntryType::Symlink,
            size: Some(0),
            mode: Some("777".to_string()),
            mtime: Utc::now(),
            checksum: None,
            target: Some(PathBuf::from("target.txt")),
        };

        let snapshot = Snapshot {
            plumbah: PlumbahObject::new(Status::Ok, meta),
            version: "1.0".to_string(),
            root: PathBuf::from("/test"),
            checksum_algorithm: ChecksumAlgorithm::XXH3_64,
            count: 1,
            entries: vec![symlink_entry.clone()],
        };

        assert_eq!(snapshot.entries[0].entry_type, EntryType::Symlink);
        assert_eq!(
            snapshot.entries[0].target,
            Some(PathBuf::from("target.txt"))
        );
    }

    #[test]
    fn test_snapshot_relative_path_handling() {
        let meta = Meta::new(
            "galdi_snapshot",
            "0.1.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );

        let entry = SnapshotEntry {
            path: PathBuf::from("relative/path/file.txt"),
            entry_type: EntryType::File,
            size: Some(100),
            mode: Some("644".to_string()),
            mtime: Utc::now(),
            checksum: Some("xxh3_64:123".to_string()),
            target: None,
        };

        let snapshot = Snapshot {
            plumbah: PlumbahObject::new(Status::Ok, meta),
            version: "1.0".to_string(),
            root: PathBuf::from("/absolute/root"),
            checksum_algorithm: ChecksumAlgorithm::XXH3_64,
            count: 1,
            entries: vec![entry],
        };

        // Verify path is relative (doesn't start with /)
        assert!(!snapshot.entries[0].path.is_absolute());
    }

    #[test]
    fn test_snapshot_deterministic_field_order() {
        let meta = Meta::new(
            "galdi_snapshot",
            "0.1.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );

        let snapshot = Snapshot {
            plumbah: PlumbahObject::new(Status::Ok, meta),
            version: "1.0".to_string(),
            root: PathBuf::from("/test"),
            checksum_algorithm: ChecksumAlgorithm::XXH3_64,
            count: 0,
            entries: vec![],
        };

        let json = serde_json::to_value(&snapshot).unwrap();

        // Verify field order in JSON
        let keys: Vec<&str> = json
            .as_object()
            .unwrap()
            .keys()
            .map(|s| s.as_str())
            .collect();

        // $plumbah should be first
        assert_eq!(keys[0], "$plumbah");

        // Verify all expected fields are present
        assert!(json.get("$plumbah").is_some());
        assert!(json.get("version").is_some());
        assert!(json.get("root").is_some());
        assert!(json.get("checksum_algorithm").is_some());
        assert!(json.get("entries").is_some());
    }

    #[test]
    fn test_snapshot_has_plumbah_field() {
        let meta = Meta::new(
            "galdi_snapshot",
            "0.1.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );

        let snapshot = Snapshot {
            plumbah: PlumbahObject::new(Status::Ok, meta),
            version: "1.0".to_string(),
            root: PathBuf::from("/test"),
            checksum_algorithm: ChecksumAlgorithm::XXH3_64,
            count: 0,
            entries: vec![],
        };

        let json = serde_json::to_value(&snapshot).unwrap();

        // Verify $plumbah field is present
        assert!(json.get("$plumbah").is_some());
        assert_eq!(json["$plumbah"]["version"], "1.0");
        assert_eq!(json["$plumbah"]["status"], "ok");
    }

    #[test]
    fn test_snapshot_data_at_top_level() {
        let meta = Meta::new(
            "galdi_snapshot",
            "0.1.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );

        let snapshot = Snapshot {
            plumbah: PlumbahObject::new(Status::Ok, meta),
            version: "1.0".to_string(),
            root: PathBuf::from("/test"),
            checksum_algorithm: ChecksumAlgorithm::XXH3_64,
            count: 0,
            entries: vec![],
        };

        let json = serde_json::to_value(&snapshot).unwrap();

        // Data fields should be at top level, not nested
        assert_eq!(json["version"], "1.0");
        assert_eq!(json["root"], "/test");
        assert!(json.get("entries").is_some());
    }

    #[test]
    fn test_snapshot_no_data_wrapper() {
        let meta = Meta::new(
            "galdi_snapshot",
            "0.1.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );

        let snapshot = Snapshot {
            plumbah: PlumbahObject::new(Status::Ok, meta),
            version: "1.0".to_string(),
            root: PathBuf::from("/test"),
            checksum_algorithm: ChecksumAlgorithm::XXH3_64,
            count: 0,
            entries: vec![],
        };

        let json = serde_json::to_value(&snapshot).unwrap();

        // Should NOT have a "data" wrapper - annotation pattern
        assert!(json.get("data").is_none());

        // Should have both $plumbah and data fields at same level
        assert!(json.get("$plumbah").is_some());
        assert!(json.get("version").is_some());
        assert!(json.get("entries").is_some());
    }

    // Property-based tests (basic versions without proptest)
    #[test]
    fn test_snapshot_serialization_preserves_data() {
        let meta = Meta::new(
            "galdi_snapshot",
            "0.1.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );

        let entries = vec![
            SnapshotEntry {
                path: PathBuf::from("a.txt"),
                entry_type: EntryType::File,
                size: Some(100),
                mode: Some("644".to_string()),
                mtime: Utc::now(),
                checksum: Some("xxh3_64:abc".to_string()),
                target: None,
            },
            SnapshotEntry {
                path: PathBuf::from("dir"),
                entry_type: EntryType::Directory,
                size: Some(0),
                mode: Some("755".to_string()),
                mtime: Utc::now(),
                checksum: None,
                target: None,
            },
        ];

        let snapshot = Snapshot {
            plumbah: PlumbahObject::new(Status::Ok, meta),
            version: "1.0".to_string(),
            root: PathBuf::from("/test"),
            checksum_algorithm: ChecksumAlgorithm::XXH3_64,
            count: 2,
            entries: entries.clone(),
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: Snapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(snapshot.entries.len(), deserialized.entries.len());
        assert_eq!(snapshot.count, deserialized.count);

        for (orig, deser) in snapshot.entries.iter().zip(deserialized.entries.iter()) {
            assert_eq!(orig.path, deser.path);
            assert_eq!(orig.entry_type, deser.entry_type);
        }
    }

    #[test]
    fn test_entry_type_roundtrip() {
        let types = vec![
            EntryType::File,
            EntryType::Directory,
            EntryType::Symlink,
            EntryType::Undefined,
        ];

        for entry_type in types {
            let json = serde_json::to_string(&entry_type).unwrap();
            let deserialized: EntryType = serde_json::from_str(&json).unwrap();
            assert_eq!(entry_type, deserialized);
        }
    }
}
