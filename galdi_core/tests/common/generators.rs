#![allow(dead_code)]
// Property test generators for galdi_core

use chrono::{DateTime, Utc};
use galdi_core::{
    ChecksumAlgorithm, EntryType, Meta, PlumbahObject, Snapshot, SnapshotEntry, Status,
};
use proptest::prelude::*;
use std::path::PathBuf;

/// Generate arbitrary EntryType (excluding Undefined for simplicity)
pub fn arbitrary_entry_type() -> impl Strategy<Value = EntryType> {
    prop_oneof![
        Just(EntryType::File),
        Just(EntryType::Directory),
        Just(EntryType::Symlink),
    ]
}

/// Generate arbitrary ChecksumAlgorithm
pub fn arbitrary_checksum_algorithm() -> impl Strategy<Value = ChecksumAlgorithm> {
    prop_oneof![
        Just(ChecksumAlgorithm::XXH3_64),
        Just(ChecksumAlgorithm::Sha256),
        Just(ChecksumAlgorithm::Blake3),
    ]
}

/// Generate a valid checksum string for a given algorithm
fn format_checksum(algo: ChecksumAlgorithm, value: u64) -> String {
    match algo {
        ChecksumAlgorithm::XXH3_64 => format!("xxh3_64:{:016x}", value),
        ChecksumAlgorithm::Sha256 => {
            // Generate a fake but valid-looking SHA256
            format!("sha256:{:064x}", value as u128 * 0xdeadbeef)
        }
        ChecksumAlgorithm::Blake3 => {
            // Generate a fake but valid-looking Blake3
            format!("blake3:{:064x}", value as u128 * 0xcafebabe)
        }
    }
}

/// Generate random but valid SnapshotEntry
pub fn arbitrary_entry() -> impl Strategy<Value = SnapshotEntry> {
    (
        "[a-z0-9_]{1,20}",                // path component
        arbitrary_entry_type(),           // entry type
        any::<u64>(),                     // size/hash seed
        proptest::option::of("[0-7]{3}"), // mode
        arbitrary_checksum_algorithm(),   // checksum algorithm
    )
        .prop_map(|(path, entry_type, size_seed, mode, algo)| {
            let size = match entry_type {
                EntryType::File => Some(size_seed % 1_000_000), // Reasonable file size
                EntryType::Directory => None,
                EntryType::Symlink => None,
                EntryType::Undefined => None,
            };

            let checksum = match entry_type {
                EntryType::File => Some(format_checksum(algo, size_seed)),
                _ => None,
            };

            let target = match entry_type {
                EntryType::Symlink => Some(PathBuf::from("target/path")),
                _ => None,
            };

            SnapshotEntry {
                path: PathBuf::from(path),
                entry_type,
                size,
                mode,
                mtime: Utc::now(),
                checksum,
                target,
            }
        })
}

/// Generate random but valid SnapshotEntry with consistent checksum algorithm
pub fn arbitrary_entry_with_algo(algo: ChecksumAlgorithm) -> impl Strategy<Value = SnapshotEntry> {
    (
        "[a-z0-9_]{1,20}",                // path component
        arbitrary_entry_type(),           // entry type
        any::<u64>(),                     // size/hash seed
        proptest::option::of("[0-7]{3}"), // mode
    )
        .prop_map(move |(path, entry_type, size_seed, mode)| {
            let size = match entry_type {
                EntryType::File => Some(size_seed % 1_000_000),
                EntryType::Directory => None,
                EntryType::Symlink => None,
                EntryType::Undefined => None,
            };

            let checksum = match entry_type {
                EntryType::File => Some(format_checksum(algo, size_seed)),
                _ => None,
            };

            let target = match entry_type {
                EntryType::Symlink => Some(PathBuf::from("target/path")),
                _ => None,
            };

            SnapshotEntry {
                path: PathBuf::from(path),
                entry_type,
                size,
                mode,
                mtime: Utc::now(),
                checksum,
                target,
            }
        })
}

/// Generate a vector of entries with unique paths (to avoid duplicates)
pub fn arbitrary_entries(max_count: usize) -> impl Strategy<Value = Vec<SnapshotEntry>> {
    let count = if max_count == 0 { 0 } else { max_count };
    proptest::collection::vec(arbitrary_entry(), 0..=count).prop_map(|mut entries| {
        // Make paths unique by appending index
        for (i, entry) in entries.iter_mut().enumerate() {
            entry.path = PathBuf::from(format!("file_{:04}", i));
        }
        entries
    })
}

/// Generate a vector of entries with a specific checksum algorithm
pub fn arbitrary_entries_with_algo(
    algo: ChecksumAlgorithm,
    max_count: usize,
) -> impl Strategy<Value = Vec<SnapshotEntry>> {
    let count = if max_count == 0 { 0 } else { max_count };
    proptest::collection::vec(arbitrary_entry_with_algo(algo), 0..=count).prop_map(|mut entries| {
        // Make paths unique by appending index
        for (i, entry) in entries.iter_mut().enumerate() {
            entry.path = PathBuf::from(format!("file_{:04}", i));
        }
        entries
    })
}

/// Generate random but valid Snapshot
pub fn arbitrary_snapshot() -> impl Strategy<Value = Snapshot> {
    (
        "[a-z]{1,10}",                  // root
        arbitrary_checksum_algorithm(), // checksum algorithm
        0usize..=50,                    // entry count
    )
        .prop_flat_map(|(root, algo, count)| {
            arbitrary_entries_with_algo(algo, count).prop_map(move |entries| {
                let actual_count = entries.len();
                let meta = Meta::new(
                    "galdi_snapshot",
                    "0.2.2",
                    true,  // idempotent
                    false, // mutates
                    true,  // safe
                    true,  // deterministic
                    100,   // execution_time_ms
                    Utc::now(),
                );

                Snapshot {
                    plumbah: PlumbahObject::new(Status::Ok, meta),
                    version: "1.0".to_string(),
                    root: PathBuf::from(root.clone()),
                    checksum_algorithm: algo,
                    count: actual_count,
                    entries,
                }
            })
        })
}

/// Generate a snapshot with a specific entry count
pub fn arbitrary_snapshot_with_count(count: usize) -> impl Strategy<Value = Snapshot> {
    (
        "[a-z]{1,10}",                  // root
        arbitrary_checksum_algorithm(), // checksum algorithm
    )
        .prop_flat_map(move |(root, algo)| {
            arbitrary_entries_with_algo(algo, count).prop_map(move |entries| {
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
                    root: PathBuf::from(root.clone()),
                    checksum_algorithm: algo,
                    count: entries.len(),
                    entries,
                }
            })
        })
}

/// Generate a snapshot with a specific root and count
pub fn snapshot_with_root_and_count(root: String, count: usize) -> impl Strategy<Value = Snapshot> {
    let root_clone = root.clone();
    arbitrary_checksum_algorithm().prop_flat_map(move |algo| {
        let root_inner = root_clone.clone();
        arbitrary_entries_with_algo(algo, count).prop_map(move |entries| {
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
                root: PathBuf::from(root_inner.clone()),
                checksum_algorithm: algo,
                count: entries.len(),
                entries,
            }
        })
    })
}
