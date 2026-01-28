// Property-based tests for Scanner
mod common;

use common::*;
use galdi_core::{ChecksumAlgorithm, ScanOptions, Scanner};
use proptest::prelude::*;

proptest! {
    /// Property A: Sorting Invariant
    /// ∀ scan_result where skip_sort=false.
    ///   is_sorted(scan_result.entries, by=path)
    #[test]
    fn proptest_scan_output_sorted(file_count in 1usize..20) {
        let temp_dir = create_test_dir();

        // Create random files
        for i in 0..file_count {
            // Create files with random names to test sorting
            let name = format!("file_{:03}.txt", file_count - i); // Reverse order names
            create_file_with_content(temp_dir.path(), &name, b"test");
        }

        let scanner = Scanner::new(ScanOptions {
            root: temp_dir.path().to_path_buf(),
            checksum_algorithm: ChecksumAlgorithm::XXH3_64,
            follow_symlinks: false,
            max_depth: None,
            exclude_patterns: vec![],
            timeout_ms: None,
            threads: Some(1), // Single-threaded for determinism
            normalize_paths: false,
        });

        let snapshot = scanner.scan().expect("Scan should succeed");

        // Verify entries are sorted by path
        assert_entries_sorted(&snapshot.entries);

        // Also verify using proptest assertions
        for window in snapshot.entries.windows(2) {
            prop_assert!(window[0].path <= window[1].path,
                "Entries should be sorted: {:?} should come before {:?}",
                window[0].path, window[1].path);
        }
    }

    /// Property B: Count Matches Entries
    /// The count field should always match the actual entries length
    #[test]
    fn proptest_scanner_count_matches(file_count in 1usize..30) {
        let temp_dir = create_test_dir();

        // Create files and directories
        for i in 0..file_count {
            if i % 2 == 0 {
                create_file_with_content(temp_dir.path(), &format!("file_{}.txt", i), b"test");
            } else {
                create_dir(temp_dir.path(), &format!("dir_{}", i));
            }
        }

        let scanner = Scanner::new(ScanOptions {
            root: temp_dir.path().to_path_buf(),
            checksum_algorithm: ChecksumAlgorithm::XXH3_64,
            follow_symlinks: false,
            max_depth: None,
            exclude_patterns: vec![],
            timeout_ms: None,
            threads: None,
            normalize_paths: false,
        });

        let snapshot = scanner.scan().expect("Scan should succeed");

        prop_assert_eq!(snapshot.count, snapshot.entries.len(),
            "Count field should match entries length");
    }

    /// Property C: Thread Count Invariant (Determinism)
    /// Results should be identical regardless of thread count (after sorting)
    #[test]
    fn proptest_parallel_scan_deterministic(
        file_count in 5usize..15,
        thread_count in 2usize..5
    ) {
        let temp_dir = create_test_dir();

        // Create files with predictable content
        for i in 0..file_count {
            let content = format!("content_{}", i);
            create_file_with_content(temp_dir.path(), &format!("file_{:02}.txt", i), content.as_bytes());
        }

        // Scan with single thread
        let scanner_single = Scanner::new(ScanOptions {
            root: temp_dir.path().to_path_buf(),
            checksum_algorithm: ChecksumAlgorithm::XXH3_64,
            follow_symlinks: false,
            max_depth: None,
            exclude_patterns: vec![],
            timeout_ms: None,
            threads: Some(1),
            normalize_paths: false,
        });

        let snapshot_single = scanner_single.scan().expect("Single-threaded scan should succeed");

        // Scan with multiple threads
        let scanner_multi = Scanner::new(ScanOptions {
            root: temp_dir.path().to_path_buf(),
            checksum_algorithm: ChecksumAlgorithm::XXH3_64,
            follow_symlinks: false,
            max_depth: None,
            exclude_patterns: vec![],
            timeout_ms: None,
            threads: Some(thread_count),
            normalize_paths: false,
        });

        let snapshot_multi = scanner_multi.scan().expect("Multi-threaded scan should succeed");

        // After sorting, should be identical
        prop_assert_eq!(snapshot_single.entries.len(), snapshot_multi.entries.len(),
            "Entry counts should match between single and multi-threaded scans");

        for (e1, e2) in snapshot_single.entries.iter().zip(snapshot_multi.entries.iter()) {
            prop_assert_eq!(&e1.path, &e2.path,
                "Paths should match at same index");
            prop_assert_eq!(&e1.checksum, &e2.checksum,
                "Checksums should match for same file");
            prop_assert_eq!(e1.entry_type, e2.entry_type,
                "Entry types should match");
        }
    }

    /// Property D: All Paths Are Relative
    /// Scanner should never produce absolute paths
    #[test]
    fn proptest_scanner_paths_relative(file_count in 1usize..20) {
        let temp_dir = create_test_dir();

        for i in 0..file_count {
            create_file_with_content(temp_dir.path(), &format!("file_{}.txt", i), b"test");
        }

        let scanner = Scanner::new(ScanOptions {
            root: temp_dir.path().to_path_buf(),
            checksum_algorithm: ChecksumAlgorithm::XXH3_64,
            follow_symlinks: false,
            max_depth: None,
            exclude_patterns: vec![],
            timeout_ms: None,
            threads: None,
            normalize_paths: false,
        });

        let snapshot = scanner.scan().expect("Scan should succeed");

        for entry in &snapshot.entries {
            prop_assert!(!entry.path.is_absolute(),
                "Entry path {:?} should be relative, not absolute", entry.path);
        }
    }

    /// Property E: Checksum Algorithm Consistency
    /// All file checksums should use the configured algorithm
    #[test]
    fn proptest_checksum_algorithm_consistency(
        file_count in 1usize..15,
        use_sha256 in proptest::bool::ANY
    ) {
        let temp_dir = create_test_dir();

        for i in 0..file_count {
            create_file_with_content(temp_dir.path(), &format!("file_{}.txt", i), b"test content");
        }

        let algo = if use_sha256 {
            ChecksumAlgorithm::Sha256
        } else {
            ChecksumAlgorithm::XXH3_64
        };

        let scanner = Scanner::new(ScanOptions {
            root: temp_dir.path().to_path_buf(),
            checksum_algorithm: algo,
            follow_symlinks: false,
            max_depth: None,
            exclude_patterns: vec![],
            timeout_ms: None,
            threads: None,
            normalize_paths: false,
        });

        let snapshot = scanner.scan().expect("Scan should succeed");

        let expected_prefix = if use_sha256 { "sha256:" } else { "xxh3_64:" };

        for entry in &snapshot.entries {
            if let Some(ref checksum) = entry.checksum {
                prop_assert!(checksum.starts_with(expected_prefix),
                    "Checksum {:?} should start with {:?}", checksum, expected_prefix);
            }
        }
    }
}
