#![allow(dead_code)]
// Custom assertions for galdi_core tests
use galdi_core::{ChecksumAlgorithm, PlumbahObject, Snapshot, SnapshotEntry};

/// Assert that two snapshots are deterministic (same except timestamp)
pub fn assert_snapshot_deterministic(s1: &Snapshot, s2: &Snapshot) {
    assert_eq!(s1.root, s2.root, "Snapshot roots differ");
    assert_eq!(
        s1.checksum_algorithm, s2.checksum_algorithm,
        "Checksum algorithms differ"
    );
    assert_eq!(s1.entries.len(), s2.entries.len(), "Entry counts differ");

    // Timestamps may differ, so we don't compare them
    // Compare entries (should be identical and sorted)
    for (e1, e2) in s1.entries.iter().zip(s2.entries.iter()) {
        assert_eq!(e1.path, e2.path, "Entry paths differ");
        assert_eq!(e1.entry_type, e2.entry_type, "Entry types differ");
        assert_eq!(e1.size, e2.size, "Entry sizes differ");
        assert_eq!(e1.mode, e2.mode, "Entry modes differ");
        assert_eq!(e1.checksum, e2.checksum, "Entry checksums differ");
        assert_eq!(e1.target, e2.target, "Entry targets differ");
        // mtime may differ slightly, so we don't compare it
    }
}

/// Assert that snapshot entries are sorted by path
pub fn assert_entries_sorted(entries: &[SnapshotEntry]) {
    for i in 1..entries.len() {
        assert!(
            entries[i - 1].path <= entries[i].path,
            "Entries not sorted: {:?} should come before {:?}",
            entries[i - 1].path,
            entries[i].path
        );
    }
}

/// Assert that a checksum string has the correct format
pub fn assert_checksum_format(checksum: &str, algo: ChecksumAlgorithm) {
    match algo {
        ChecksumAlgorithm::XXH3_64 => {
            assert!(
                checksum.starts_with("xxh3_64:"),
                "XXH3_64 checksum should start with 'xxh3_64:'"
            );
            let hex_part = &checksum[8..];
            assert_eq!(
                hex_part.len(),
                16,
                "XXH3_64 hex part should be 16 characters, got {}",
                hex_part.len()
            );
            assert!(
                hex_part.chars().all(|c| c.is_ascii_hexdigit()),
                "XXH3_64 hex part should contain only hex digits"
            );
        }
        ChecksumAlgorithm::Sha256 => {
            assert!(
                checksum.starts_with("sha256:"),
                "SHA256 checksum should start with 'sha256:'"
            );
            let hex_part = &checksum[7..];
            assert_eq!(
                hex_part.len(),
                64,
                "SHA256 hex part should be 64 characters, got {}",
                hex_part.len()
            );
            assert!(
                hex_part.chars().all(|c| c.is_ascii_hexdigit()),
                "SHA256 hex part should contain only hex digits"
            );
            assert!(
                hex_part.chars().all(|c| !c.is_uppercase()),
                "SHA256 hex should be lowercase"
            );
        }
    }
}

/// Assert that a Plumbah object has all required Level 2 fields
pub fn assert_plumbah_compliant<T>(object: &PlumbahObject) {
    assert_eq!(object.version, "1.0", "Plumbah version should be 1.0");

    // Check that meta has all required semantic flags
    let meta = object
        .meta
        .as_ref()
        .expect("Meta should be present for non-streaming output");
    assert_eq!(meta.plumbah_level, 2, "Should be Level 2 compliant");

    // Verify tool name and version are present
    assert!(!meta.tool.is_empty(), "Tool name should not be empty");
    assert!(
        !meta.tool_version.is_empty(),
        "Tool version should not be empty"
    );

    // execution_time_ms should be present
    assert!(meta.execution_time_ms > 0, "Execution time should be > 0");
}

/// Assert JSONL output is valid
pub fn assert_valid_jsonl(output: &str) {
    let lines: Vec<&str> = output.lines().collect();
    assert!(
        lines.len() >= 2,
        "JSONL must have at least head and tail, got {} lines",
        lines.len()
    );

    // Verify each line is valid JSON
    for (i, line) in lines.iter().enumerate() {
        serde_json::from_str::<serde_json::Value>(line)
            .unwrap_or_else(|e| panic!("Line {} invalid JSON: {} - line: {}", i, e, line));
    }

    // Verify head marker
    let head: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(
        head["$plumbah"]["stream"], "head",
        "First line should be head marker"
    );

    // Verify tail marker
    let tail: serde_json::Value = serde_json::from_str(lines.last().unwrap()).unwrap();
    assert_eq!(
        tail["$plumbah"]["stream"], "tail",
        "Last line should be tail marker"
    );
}
