// Property-based tests for Snapshot serialization
mod common;

use common::*;
use galdi_core::{EntryType, Snapshot};
use proptest::prelude::*;

proptest! {
    /// Property A: Roundtrip (Isomorphism)
    /// ∀ snapshot: Snapshot.
    ///   deserialize(serialize(snapshot)) == snapshot
    #[test]
    fn proptest_snapshot_roundtrip(snapshot in arbitrary_snapshot()) {
        // Serialize to JSON
        let json = serde_json::to_string(&snapshot)
            .expect("Serialization should succeed");

        // Deserialize back
        let deserialized: Snapshot = serde_json::from_str(&json)
            .expect("Deserialization should succeed");

        // Core properties should match
        prop_assert_eq!(snapshot.version, deserialized.version,
            "Version mismatch in roundtrip");
        prop_assert_eq!(snapshot.root, deserialized.root,
            "Root mismatch in roundtrip");
        prop_assert_eq!(snapshot.count, deserialized.count,
            "Count mismatch in roundtrip");
        prop_assert_eq!(snapshot.checksum_algorithm, deserialized.checksum_algorithm,
            "Checksum algorithm mismatch in roundtrip");
        prop_assert_eq!(snapshot.entries.len(), deserialized.entries.len(),
            "Entry count mismatch in roundtrip");

        // Verify each entry
        for (orig, deser) in snapshot.entries.iter().zip(deserialized.entries.iter()) {
            prop_assert_eq!(&orig.path, &deser.path, "Entry path mismatch");
            prop_assert_eq!(orig.entry_type, deser.entry_type, "Entry type mismatch");
            prop_assert_eq!(orig.size, deser.size, "Entry size mismatch");
            prop_assert_eq!(&orig.mode, &deser.mode, "Entry mode mismatch");
            prop_assert_eq!(&orig.checksum, &deser.checksum, "Entry checksum mismatch");
            prop_assert_eq!(&orig.target, &deser.target, "Entry target mismatch");
        }
    }

    /// Property B: Count Invariant
    /// ∀ snapshot: Snapshot.
    ///   snapshot.count == snapshot.entries.len()
    #[test]
    fn proptest_snapshot_count_matches_entries(snapshot in arbitrary_snapshot()) {
        prop_assert_eq!(snapshot.count, snapshot.entries.len(),
            "Count field should match entries length");
    }

    /// Property C: Path Relativity Invariant
    /// ∀ entry in snapshot.entries.
    ///   !entry.path.is_absolute()
    #[test]
    fn proptest_all_paths_relative(snapshot in arbitrary_snapshot()) {
        for entry in &snapshot.entries {
            prop_assert!(!entry.path.is_absolute(),
                "Entry path {:?} should be relative, not absolute", entry.path);
        }
    }

    /// Property D: Checksum Presence Invariant
    /// ∀ entry in snapshot.entries.
    ///   (entry.entry_type == File) ⟹ (entry.checksum.is_some())
    ///   (entry.entry_type != File) ⟹ (entry.checksum.is_none())
    #[test]
    fn proptest_checksum_presence_invariant(snapshot in arbitrary_snapshot()) {
        for entry in &snapshot.entries {
            match entry.entry_type {
                EntryType::File => {
                    prop_assert!(entry.checksum.is_some(),
                        "File entry {:?} should have a checksum", entry.path);
                }
                EntryType::Directory | EntryType::Symlink | EntryType::Undefined => {
                    prop_assert!(entry.checksum.is_none(),
                        "Non-file entry {:?} (type: {:?}) should not have a checksum",
                        entry.path, entry.entry_type);
                }
            }
        }
    }

    /// Property E: Symlink Target Invariant
    /// ∀ entry in snapshot.entries.
    ///   (entry.entry_type == Symlink) ⟹ (entry.target.is_some())
    ///   (entry.entry_type != Symlink) ⟹ (entry.target.is_none())
    #[test]
    fn proptest_symlink_target_invariant(snapshot in arbitrary_snapshot()) {
        for entry in &snapshot.entries {
            match entry.entry_type {
                EntryType::Symlink => {
                    prop_assert!(entry.target.is_some(),
                        "Symlink entry {:?} should have a target", entry.path);
                }
                EntryType::File | EntryType::Directory | EntryType::Undefined => {
                    prop_assert!(entry.target.is_none(),
                        "Non-symlink entry {:?} (type: {:?}) should not have a target",
                        entry.path, entry.entry_type);
                }
            }
        }
    }

    /// Property F: Size Presence for Files
    /// Files should have size, directories typically don't
    #[test]
    fn proptest_file_size_invariant(snapshot in arbitrary_snapshot()) {
        for entry in &snapshot.entries {
            if entry.entry_type == EntryType::File {
                prop_assert!(entry.size.is_some(),
                    "File entry {:?} should have a size", entry.path);
            }
        }
    }

    /// Property G: Version Constant
    /// All snapshots should have version "1.0"
    #[test]
    fn proptest_snapshot_version_constant(snapshot in arbitrary_snapshot()) {
        prop_assert_eq!(snapshot.version, "1.0",
            "Snapshot version should always be 1.0");
    }
}
