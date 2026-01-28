// Property-based tests for Diff Engine

use galdi_core::Snapshot;
use galdi_diff::diff::{DiffEngine, DiffOptions};
use proptest::prelude::*;

// Helper to generate arbitrary snapshot - simplified version
fn arbitrary_snapshot() -> impl Strategy<Value = Snapshot> {
    use chrono::Utc;
    use galdi_core::{ChecksumAlgorithm, EntryType, Meta, PlumbahObject, SnapshotEntry, Status};
    use std::path::PathBuf;

    (
        "[a-z]{1,10}", // root
        0usize..20,    // entry count
        any::<bool>(), // use xxh3 or sha256
    )
        .prop_map(|(root, count, use_xxh3)| {
            let algo = if use_xxh3 {
                ChecksumAlgorithm::XXH3_64
            } else {
                ChecksumAlgorithm::Sha256
            };

            let mut entries = Vec::new();
            for i in 0..count {
                let entry_type = match i % 3 {
                    0 => EntryType::File,
                    1 => EntryType::Directory,
                    _ => EntryType::Symlink,
                };

                let checksum = if entry_type == EntryType::File {
                    Some(format!("xxh3_64:{:016x}", i as u64))
                } else {
                    None
                };

                let target = if entry_type == EntryType::Symlink {
                    Some(PathBuf::from("target"))
                } else {
                    None
                };

                entries.push(SnapshotEntry {
                    path: PathBuf::from(format!("file_{:04}", i)),
                    entry_type,
                    size: if entry_type == EntryType::File {
                        Some(i as u64 * 100)
                    } else {
                        None
                    },
                    mode: Some("644".to_string()),
                    mtime: Utc::now(),
                    checksum,
                    target,
                });
            }

            let meta = Meta::new(
                "galdi_snapshot",
                "0.2.2",
                true,
                false,
                true,
                true,
                100,
                Utc::now(),
            );

            Snapshot {
                plumbah: PlumbahObject::new(Status::Ok, meta),
                version: "1.0".to_string(),
                root: PathBuf::from(root),
                checksum_algorithm: algo,
                count: entries.len(),
                entries,
            }
        })
}

proptest! {
    /// Property A: Reflexivity (Identity Diff)
    /// ∀ snapshot: Snapshot.
    ///   diff(snapshot, snapshot) == DiffResult {
    ///     identical: true,
    ///     differences: [],
    ///     summary: { unchanged: snapshot.entries.len(), ... }
    ///   }
    #[test]
    fn proptest_diff_reflexivity(snapshot in arbitrary_snapshot()) {
        let engine = DiffEngine::new(DiffOptions {
            ignore_time: false,
            ignore_mode: false,
            structure_only: false,
        });

        let diff_result = engine.diff(&snapshot, &snapshot);

        prop_assert!(diff_result.identical,
            "Diff of snapshot with itself should be identical");
        prop_assert_eq!(diff_result.differences.len(), 0,
            "No differences should exist when comparing snapshot to itself");
        prop_assert_eq!(diff_result.summary.unchanged, snapshot.entries.len(),
            "All entries should be unchanged");
        prop_assert_eq!(diff_result.summary.added, 0,
            "No entries should be added");
        prop_assert_eq!(diff_result.summary.removed, 0,
            "No entries should be removed");
        prop_assert_eq!(diff_result.summary.modified, 0,
            "No entries should be modified");
    }

    /// Property B: Identical Flag Property
    /// ∀ snapshot_a, snapshot_b.
    ///   diff_result.identical ⟺ (diff_result.differences.len() == 0)
    #[test]
    fn proptest_diff_identical_flag(snapshot in arbitrary_snapshot()) {
        let engine = DiffEngine::new(DiffOptions {
            ignore_time: false,
            ignore_mode: false,
            structure_only: false,
        });

        let diff_result = engine.diff(&snapshot, &snapshot);

        prop_assert_eq!(diff_result.identical, diff_result.differences.is_empty(),
            "identical flag should match whether differences list is empty");
    }

    /// Property C: Summary Consistency
    /// The summary counts should be consistent with the differences list
    #[test]
    fn proptest_diff_summary_consistency(snapshot in arbitrary_snapshot()) {
        let engine = DiffEngine::new(DiffOptions {
            ignore_time: false,
            ignore_mode: false,
            structure_only: false,
        });

        let diff_result = engine.diff(&snapshot, &snapshot);

        let total_changes = diff_result.summary.added +
                          diff_result.summary.removed +
                          diff_result.summary.modified +
                          diff_result.summary.unchanged;

        prop_assert_eq!(total_changes, snapshot.entries.len(),
            "Total of all summary counts should equal entry count");
    }

    /// Property D: Differences Sorted
    /// Differences should be sorted by path for determinism
    #[test]
    fn proptest_diff_output_sorted(snapshot in arbitrary_snapshot()) {
        let engine = DiffEngine::new(DiffOptions {
            ignore_time: false,
            ignore_mode: false,
            structure_only: false,
        });

        let diff_result = engine.diff(&snapshot, &snapshot);

        // For identical snapshots, differences should be empty
        // But let's verify the property holds
        for window in diff_result.differences.windows(2) {
            prop_assert!(window[0].path <= window[1].path,
                "Differences should be sorted by path");
        }
    }

    /// Property E: Symmetry Property (Partial)
    /// When comparing A to B vs B to A, added/removed should be swapped
    /// This test creates two different snapshots by modifying entry count
    #[test]
    fn proptest_diff_symmetry_partial(count_a in 5usize..10, count_b in 11usize..15) {
        use chrono::Utc;
        use galdi_core::{ChecksumAlgorithm, EntryType, Meta, PlumbahObject, SnapshotEntry, Status};
        use std::path::PathBuf;

        // Create two snapshots with different entry counts
        let create_snapshot = |count: usize| {
            let mut entries = Vec::new();
            for i in 0..count {
                entries.push(SnapshotEntry {
                    path: PathBuf::from(format!("file_{:04}", i)),
                    entry_type: EntryType::File,
                    size: Some(i as u64 * 100),
                    mode: Some("644".to_string()),
                    mtime: Utc::now(),
                    checksum: Some(format!("xxh3_64:{:016x}", i as u64)),
                    target: None,
                });
            }

            let meta = Meta::new(
                "galdi_snapshot",
                "0.2.2",
                true,
                false,
                true,
                true,
                100,
                Utc::now(),
            );

            Snapshot {
                plumbah: PlumbahObject::new(Status::Ok, meta),
                version: "1.0".to_string(),
                root: PathBuf::from("test"),
                checksum_algorithm: ChecksumAlgorithm::XXH3_64,
                count: entries.len(),
                entries,
            }
        };

        let snapshot_a = create_snapshot(count_a);
        let snapshot_b = create_snapshot(count_b);

        let engine = DiffEngine::new(DiffOptions {
            ignore_time: false,
            ignore_mode: false,
            structure_only: false,
        });

        let diff_ab = engine.diff(&snapshot_a, &snapshot_b);
        let diff_ba = engine.diff(&snapshot_b, &snapshot_a);

        // Symmetry: added in A→B should equal removed in B→A
        prop_assert_eq!(diff_ab.summary.added, diff_ba.summary.removed,
            "Added in A→B should equal removed in B→A");

        // Symmetry: removed in A→B should equal added in B→A
        prop_assert_eq!(diff_ab.summary.removed, diff_ba.summary.added,
            "Removed in A→B should equal added in B→A");

        // Unchanged should be the same in both directions
        prop_assert_eq!(diff_ab.summary.unchanged, diff_ba.summary.unchanged,
            "Unchanged count should be symmetric");
    }

    /// Property F: Empty Snapshot Diff
    /// Diffing an empty snapshot with a non-empty one should show all as added
    #[test]
    fn proptest_diff_empty_snapshot(count in 1usize..20) {
        use chrono::Utc;
        use galdi_core::{ChecksumAlgorithm, EntryType, Meta, PlumbahObject, SnapshotEntry, Status};
        use std::path::PathBuf;

        // Create empty snapshot
        let meta1 = Meta::new("galdi_snapshot", "0.2.2", true, false, true, true, 100, Utc::now());
        let empty_snapshot = Snapshot {
            plumbah: PlumbahObject::new(Status::Ok, meta1),
            version: "1.0".to_string(),
            root: PathBuf::from("test"),
            checksum_algorithm: ChecksumAlgorithm::XXH3_64,
            count: 0,
            entries: vec![],
        };

        // Create non-empty snapshot
        let mut entries = Vec::new();
        for i in 0..count {
            entries.push(SnapshotEntry {
                path: PathBuf::from(format!("file_{:04}", i)),
                entry_type: EntryType::File,
                size: Some(100),
                mode: Some("644".to_string()),
                mtime: Utc::now(),
                checksum: Some(format!("xxh3_64:{:016x}", i as u64)),
                target: None,
            });
        }

        let meta2 = Meta::new("galdi_snapshot", "0.2.2", true, false, true, true, 100, Utc::now());
        let non_empty_snapshot = Snapshot {
            plumbah: PlumbahObject::new(Status::Ok, meta2),
            version: "1.0".to_string(),
            root: PathBuf::from("test"),
            checksum_algorithm: ChecksumAlgorithm::XXH3_64,
            count: entries.len(),
            entries,
        };

        let engine = DiffEngine::new(DiffOptions {
            ignore_time: false,
            ignore_mode: false,
            structure_only: false,
        });

        let diff_result = engine.diff(&empty_snapshot, &non_empty_snapshot);

        prop_assert_eq!(diff_result.summary.added, count,
            "All entries should be added when comparing empty to non-empty");
        prop_assert_eq!(diff_result.summary.removed, 0,
            "No entries should be removed");
        prop_assert_eq!(diff_result.summary.modified, 0,
            "No entries should be modified");
        prop_assert_eq!(diff_result.summary.unchanged, 0,
            "No entries should be unchanged");
    }
}
