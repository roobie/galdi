//! Integration tests for JSONL streaming output
//!
//! These property-based tests verify critical invariants of the JSONL format:
//! - Line count matches entries + 2 (head + tail)
//! - Every line is valid JSON
//! - Head/tail markers are correct
//! - Summary counts are accurate

use chrono::Utc;
use galdi_core::{ChecksumAlgorithm, EntryType, SnapshotEntry};
use proptest::prelude::*;
use std::path::PathBuf;

// Import the output module from galdi_snapshot
// Note: This requires making output module public or adding test helpers
// For now, we'll test through the public API

/// Generate a valid SnapshotEntry for testing (reserved for future use)
#[allow(dead_code)]
fn arbitrary_entry() -> impl Strategy<Value = SnapshotEntry> {
    (
        prop::string::string_regex("[a-z0-9_]{1,20}").unwrap(),
        prop_oneof![
            Just(EntryType::File),
            Just(EntryType::Directory),
            Just(EntryType::Symlink),
        ],
        any::<u64>(),
        prop::option::of("[0-7]{3,4}"),
    )
        .prop_map(|(path, entry_type, size, mode)| SnapshotEntry {
            path: PathBuf::from(path),
            entry_type,
            size: Some(size),
            mode,
            mtime: Utc::now(),
            checksum: match entry_type {
                EntryType::File => Some(format!("xxh3_64:{:016x}", size % 0xFFFFFFFF)),
                _ => None,
            },
            target: match entry_type {
                EntryType::Symlink => Some(PathBuf::from("target.txt")),
                _ => None,
            },
        })
}

/// Parse JSONL output and extract components
fn parse_jsonl(output: &str) -> (serde_json::Value, Vec<serde_json::Value>, serde_json::Value) {
    let lines: Vec<&str> = output.lines().collect();
    assert!(lines.len() >= 2, "JSONL must have at least head and tail");

    let head: serde_json::Value =
        serde_json::from_str(lines[0]).expect("Head line should be valid JSON");

    let mut middle = Vec::new();
    for line in &lines[1..lines.len() - 1] {
        let value: serde_json::Value =
            serde_json::from_str(line).expect("Middle line should be valid JSON");
        middle.push(value);
    }

    let tail: serde_json::Value =
        serde_json::from_str(lines[lines.len() - 1]).expect("Tail line should be valid JSON");

    (head, middle, tail)
}

// Property-based tests
proptest! {
    /// Property: Line count equals entries + 2 (head + tail)
    #[test]
    fn proptest_jsonl_line_count(
        entry_count in 0usize..100,
    ) {
        // Generate entries
        let entries: Vec<SnapshotEntry> = (0..entry_count)
            .map(|i| SnapshotEntry {
                path: PathBuf::from(format!("file_{}.txt", i)),
                entry_type: EntryType::File,
                size: Some(100),
                mode: Some("644".to_string()),
                mtime: Utc::now(),
                checksum: Some(format!("xxh3_64:{:016x}", i)),
                target: None,
            })
            .collect();

        // Create JSONL output
        let mut buf = Vec::new();
        {
            use galdi_snapshot::output::StreamingOutput;
            let mut output = StreamingOutput::new(&mut buf);

            output.write_head(
                &PathBuf::from("/test"),
                ChecksumAlgorithm::XXH3_64,
                false,
            ).unwrap();

            for entry in &entries {
                output.write_entry(entry).unwrap();
            }

            output.write_tail().unwrap();
        }

        let output_str = String::from_utf8(buf).unwrap();
        let line_count = output_str.lines().count();

        prop_assert_eq!(
            line_count,
            entry_count + 2,
            "Expected {} lines (head + {} entries + tail), got {}",
            entry_count + 2,
            entry_count,
            line_count
        );
    }

    /// Property: Every line is valid JSON
    #[test]
    fn proptest_jsonl_all_lines_valid_json(
        entry_count in 1usize..50,
    ) {
        let entries: Vec<SnapshotEntry> = (0..entry_count)
            .map(|i| SnapshotEntry {
                path: PathBuf::from(format!("file_{}.txt", i)),
                entry_type: EntryType::File,
                size: Some(100),
                mode: Some("644".to_string()),
                mtime: Utc::now(),
                checksum: Some(format!("xxh3_64:{:016x}", i)),
                target: None,
            })
            .collect();

        let mut buf = Vec::new();
        {
            use galdi_snapshot::output::StreamingOutput;
            let mut output = StreamingOutput::new(&mut buf);

            output.write_head(&PathBuf::from("/test"), ChecksumAlgorithm::XXH3_64, false).unwrap();
            for entry in &entries {
                output.write_entry(entry).unwrap();
            }
            output.write_tail().unwrap();
        }

        let output_str = String::from_utf8(buf).unwrap();

        for (i, line) in output_str.lines().enumerate() {
            let parse_result: Result<serde_json::Value, _> = serde_json::from_str(line);
            prop_assert!(
                parse_result.is_ok(),
                "Line {} should be valid JSON: {}",
                i,
                line
            );
        }
    }

    /// Property: Head line has correct stream marker and structure
    #[test]
    fn proptest_jsonl_head_structure(
        entry_count in 0usize..30,
    ) {
        let entries: Vec<SnapshotEntry> = (0..entry_count)
            .map(|i| SnapshotEntry {
                path: PathBuf::from(format!("file_{}.txt", i)),
                entry_type: EntryType::File,
                size: Some(100),
                mode: Some("644".to_string()),
                mtime: Utc::now(),
                checksum: Some(format!("xxh3_64:{:016x}", i)),
                target: None,
            })
            .collect();

        let mut buf = Vec::new();
        {
            use galdi_snapshot::output::StreamingOutput;
            let mut output = StreamingOutput::new(&mut buf);

            output.write_head(&PathBuf::from("/test"), ChecksumAlgorithm::XXH3_64, false).unwrap();
            for entry in &entries {
                output.write_entry(entry).unwrap();
            }
            output.write_tail().unwrap();
        }

        let output_str = String::from_utf8(buf).unwrap();
        let (head, _middle, _tail) = parse_jsonl(&output_str);

        // Verify head structure
        prop_assert_eq!(head["$plumbah"]["stream"].as_str(), Some("head"));
        prop_assert_eq!(head["$plumbah"]["status"].as_str(), Some("ok"));
        prop_assert!(head["$plumbah"]["meta"].is_object());
        prop_assert_eq!(head["$plumbah"]["meta"]["profiles"][0]["name"].as_str(), Some("streaming"));
        prop_assert_eq!(head["version"].as_str(), Some("1.0"));
        prop_assert_eq!(head["checksum_algorithm"].as_str(), Some("xxh3_64"));
    }

    /// Property: Tail line has correct stream marker and summary
    #[test]
    fn proptest_jsonl_tail_structure(
        entry_count in 0usize..30,
    ) {
        let entries: Vec<SnapshotEntry> = (0..entry_count)
            .map(|i| SnapshotEntry {
                path: PathBuf::from(format!("file_{}.txt", i)),
                entry_type: EntryType::File,
                size: Some(100),
                mode: Some("644".to_string()),
                mtime: Utc::now(),
                checksum: Some(format!("xxh3_64:{:016x}", i)),
                target: None,
            })
            .collect();

        let mut buf = Vec::new();
        {
            use galdi_snapshot::output::StreamingOutput;
            let mut output = StreamingOutput::new(&mut buf);

            output.write_head(&PathBuf::from("/test"), ChecksumAlgorithm::XXH3_64, false).unwrap();
            for entry in &entries {
                output.write_entry(entry).unwrap();
            }
            output.write_tail().unwrap();
        }

        let output_str = String::from_utf8(buf).unwrap();
        let (_head, _middle, tail) = parse_jsonl(&output_str);

        // Verify tail structure
        prop_assert_eq!(tail["$plumbah"]["stream"].as_str(), Some("tail"));
        prop_assert_eq!(tail["$plumbah"]["status"].as_str(), Some("ok"));
        prop_assert_eq!(tail["$plumbah"]["summary"]["total"].as_u64(), Some(entry_count as u64));
        prop_assert_eq!(tail["$plumbah"]["summary"]["processed"].as_u64(), Some(entry_count as u64));
        prop_assert_eq!(tail["$plumbah"]["summary"]["errors"].as_u64(), Some(0));
        prop_assert!(tail["$plumbah"]["execution_time_ms"].is_number());
    }

    /// Property: Middle lines have no $plumbah field (pure data)
    #[test]
    fn proptest_jsonl_middle_no_plumbah(
        entry_count in 1usize..30,
    ) {
        let entries: Vec<SnapshotEntry> = (0..entry_count)
            .map(|i| SnapshotEntry {
                path: PathBuf::from(format!("file_{}.txt", i)),
                entry_type: EntryType::File,
                size: Some(100),
                mode: Some("644".to_string()),
                mtime: Utc::now(),
                checksum: Some(format!("xxh3_64:{:016x}", i)),
                target: None,
            })
            .collect();

        let mut buf = Vec::new();
        {
            use galdi_snapshot::output::StreamingOutput;
            let mut output = StreamingOutput::new(&mut buf);

            output.write_head(&PathBuf::from("/test"), ChecksumAlgorithm::XXH3_64, false).unwrap();
            for entry in &entries {
                output.write_entry(entry).unwrap();
            }
            output.write_tail().unwrap();
        }

        let output_str = String::from_utf8(buf).unwrap();
        let (_head, middle, _tail) = parse_jsonl(&output_str);

        // Verify middle entries have no $plumbah field
        for (i, entry) in middle.iter().enumerate() {
            prop_assert!(
                entry.get("$plumbah").is_none(),
                "Middle entry {} should not have $plumbah field",
                i
            );
            // Should have data fields instead
            prop_assert!(entry.get("path").is_some(), "Entry {} missing path field", i);
            prop_assert!(entry.get("type").is_some(), "Entry {} missing type field", i);
        }
    }

    /// Property: Summary counts are accurate
    #[test]
    fn proptest_jsonl_summary_accuracy(
        entry_count in 0usize..50,
    ) {
        let entries: Vec<SnapshotEntry> = (0..entry_count)
            .map(|i| SnapshotEntry {
                path: PathBuf::from(format!("file_{}.txt", i)),
                entry_type: EntryType::File,
                size: Some(100),
                mode: Some("644".to_string()),
                mtime: Utc::now(),
                checksum: Some(format!("xxh3_64:{:016x}", i)),
                target: None,
            })
            .collect();

        let mut buf = Vec::new();
        {
            use galdi_snapshot::output::StreamingOutput;
            let mut output = StreamingOutput::new(&mut buf);

            output.write_head(&PathBuf::from("/test"), ChecksumAlgorithm::XXH3_64, false).unwrap();
            for entry in &entries {
                output.write_entry(entry).unwrap();
            }
            output.write_tail().unwrap();
        }

        let output_str = String::from_utf8(buf).unwrap();
        let (_head, middle, tail) = parse_jsonl(&output_str);

        // Summary should match actual counts
        let summary_total = tail["$plumbah"]["summary"]["total"].as_u64().unwrap();
        let summary_processed = tail["$plumbah"]["summary"]["processed"].as_u64().unwrap();
        let summary_errors = tail["$plumbah"]["summary"]["errors"].as_u64().unwrap();

        prop_assert_eq!(summary_total as usize, entry_count);
        prop_assert_eq!(summary_processed as usize, entry_count);
        prop_assert_eq!(summary_errors, 0);
        prop_assert_eq!(middle.len(), entry_count);
    }

    /// Property: Execution time is always present and reasonable
    #[test]
    fn proptest_jsonl_execution_time(
        entry_count in 0usize..20,
    ) {
        let entries: Vec<SnapshotEntry> = (0..entry_count)
            .map(|i| SnapshotEntry {
                path: PathBuf::from(format!("file_{}.txt", i)),
                entry_type: EntryType::File,
                size: Some(100),
                mode: Some("644".to_string()),
                mtime: Utc::now(),
                checksum: Some(format!("xxh3_64:{:016x}", i)),
                target: None,
            })
            .collect();

        let mut buf = Vec::new();
        {
            use galdi_snapshot::output::StreamingOutput;
            let mut output = StreamingOutput::new(&mut buf);

            output.write_head(&PathBuf::from("/test"), ChecksumAlgorithm::XXH3_64, false).unwrap();
            for entry in &entries {
                output.write_entry(entry).unwrap();
            }
            output.write_tail().unwrap();
        }

        let output_str = String::from_utf8(buf).unwrap();
        let (_head, _middle, tail) = parse_jsonl(&output_str);

        // Execution time should be present and reasonable (< 10 seconds)
        let exec_time = tail["$plumbah"]["execution_time_ms"].as_u64().unwrap();
        prop_assert!(exec_time < 10000, "Execution time should be < 10s for {} entries", entry_count);
    }
}

// Standard unit tests for edge cases
#[test]
fn test_jsonl_empty_entries() {
    // Empty snapshot (0 entries) should still have valid head and tail
    let mut buf = Vec::new();
    {
        use galdi_snapshot::output::StreamingOutput;
        let mut output = StreamingOutput::new(&mut buf);

        output
            .write_head(&PathBuf::from("/test"), ChecksumAlgorithm::XXH3_64, false)
            .unwrap();
        output.write_tail().unwrap();
    }

    let output_str = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output_str.lines().collect();

    assert_eq!(lines.len(), 2, "Empty snapshot should have exactly 2 lines");

    let head: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    let tail: serde_json::Value = serde_json::from_str(lines[1]).unwrap();

    assert_eq!(head["$plumbah"]["stream"], "head");
    assert_eq!(tail["$plumbah"]["stream"], "tail");
    assert_eq!(tail["$plumbah"]["summary"]["total"], 0);
}

#[test]
fn test_jsonl_single_entry() {
    // Single entry should produce exactly 3 lines
    let mut buf = Vec::new();
    {
        use galdi_snapshot::output::StreamingOutput;
        let mut output = StreamingOutput::new(&mut buf);

        output
            .write_head(&PathBuf::from("/test"), ChecksumAlgorithm::XXH3_64, false)
            .unwrap();

        let entry = SnapshotEntry {
            path: PathBuf::from("single.txt"),
            entry_type: EntryType::File,
            size: Some(100),
            mode: Some("644".to_string()),
            mtime: Utc::now(),
            checksum: Some("xxh3_64:abc123".to_string()),
            target: None,
        };
        output.write_entry(&entry).unwrap();

        output.write_tail().unwrap();
    }

    let output_str = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output_str.lines().collect();

    assert_eq!(lines.len(), 3);

    let head: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    let middle: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    let tail: serde_json::Value = serde_json::from_str(lines[2]).unwrap();

    assert_eq!(head["$plumbah"]["stream"], "head");
    assert!(middle.get("$plumbah").is_none());
    assert_eq!(middle["path"], "single.txt");
    assert_eq!(tail["$plumbah"]["stream"], "tail");
}

#[test]
fn test_jsonl_with_errors() {
    // Test that error lines work correctly
    use galdi_core::PlumbahError;

    let mut buf = Vec::new();
    {
        use galdi_snapshot::output::StreamingOutput;
        let mut output = StreamingOutput::new(&mut buf);

        output
            .write_head(&PathBuf::from("/test"), ChecksumAlgorithm::XXH3_64, false)
            .unwrap();

        let error = PlumbahError {
            code: "TEST_ERROR".to_string(),
            message: "Test error message".to_string(),
            path: Some(PathBuf::from("/error/path")),
            recoverable: true,
            context: None,
        };
        output.write_error(&error).unwrap();

        output.write_tail().unwrap();
    }

    let output_str = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output_str.lines().collect();

    assert_eq!(lines.len(), 3); // head + error + tail

    let error_line: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    assert!(error_line.get("$plumbah").is_some());
    assert!(error_line["$plumbah"].get("errors").is_some());

    let tail: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
    assert_eq!(tail["$plumbah"]["summary"]["errors"], 1);
    assert_eq!(tail["$plumbah"]["status"], "partial");
}
