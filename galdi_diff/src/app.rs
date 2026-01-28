//! Application logic separated from `main` so the CLI entry is thin.
//!
//! This module exposes `run()` which performs the diff and returns
//! an exit code. `main.rs` remains responsible only for argument parsing
//! and process-level concerns.

use chrono::Utc;
use galdi_core::*;
use serde::Serialize;
use std::fs::File;
use std::io;
use std::path::Path;
use std::time::Instant;

use crate::cli::ToolArgs;
use crate::diff::{DiffEngine, DiffOptions};

/// Envelope pattern for clean error handling.
///
/// This enum allows us to return either a successful DiffResult
/// or a PlumbahObject containing error information, both serialized
/// to the same output format.
#[derive(Serialize)]
#[serde(untagged)]
enum Envelope {
    DiffResult(DiffResult),
    Error(PlumbahObject),
}

/// Result of running the diff operation.
///
/// Contains the exit code and the JSON output string.
pub struct RunResult {
    pub exit_code: i32,
    pub output: String,
}

/// Run the diff tool logic and return a result.
///
/// This function:
/// 1. Loads source and target snapshots (from JSON files, stdin, or live filesystem)
/// 2. Performs the diff operation using DiffEngine
/// 3. Formats the output (JSON or human-readable)
/// 4. Returns RunResult with exit code and output string
///
/// # Errors
///
/// Returns an error if:
/// - JSON parsing fails
/// - Output serialization fails
/// - Other I/O errors occur
pub fn run(args: ToolArgs) -> anyhow::Result<RunResult> {
    let start = Instant::now();

    // Handle --plumbah-info flag: return only metadata without diffing
    if args.plumbah_info {
        let elapsed = start.elapsed();
        let plumbah = PlumbahObject::new(
            Status::Ok,
            Meta::new(
                "galdi_diff",
                env!("CARGO_PKG_VERSION"),
                true,  // idempotent
                false, // mutates
                true,  // safe
                true,  // deterministic (same inputs → same outputs)
                elapsed.as_millis() as u64,
                Utc::now(),
            )
            .with_default_profiles(),
        );

        let output = serde_json::to_string_pretty(&plumbah)?;

        return Ok(RunResult {
            exit_code: 0,
            output,
        });
    }

    // Load source and target (either from filesystem or JSON)
    let source_result = load_snapshot(&args.source, &args);
    let target_result = load_snapshot(&args.target, &args);

    // Build envelope based on results
    let envelope = match (source_result, target_result) {
        (Ok(source), Ok(target)) => {
            // Perform diff
            let engine = DiffEngine::new(DiffOptions {
                ignore_time: args.ignore_time,
                ignore_mode: args.ignore_mode,
                structure_only: args.structure_only,
            });
            let diff_result = engine.diff(&source, &target);
            Envelope::DiffResult(diff_result)
        }
        (Err(e), _) | (_, Err(e)) => {
            // Error loading snapshots
            let elapsed = start.elapsed();
            Envelope::Error(
                PlumbahObject::new(
                    Status::Error,
                    Meta::new(
                        "galdi_diff",
                        env!("CARGO_PKG_VERSION"),
                        true,
                        false,
                        true,
                        true,
                        elapsed.as_millis() as u64,
                        Utc::now(),
                    )
                    .with_default_profiles(),
                )
                .with_errors(vec![PlumbahError {
                    code: "LOAD_ERROR".to_string(),
                    message: e.to_string(),
                    path: None,
                    recoverable: false,
                    context: None,
                }]),
            )
        }
    };

    // Output to stdout
    let output = if args.human {
        format_human(&envelope)?
    } else {
        serde_json::to_string_pretty(&envelope)?
    };

    // Compute exit code from envelope
    let exit_code = match envelope {
        Envelope::DiffResult(ref result) => match result.plumbah.status {
            Status::Ok => 0,
            Status::Partial => 2,
            Status::Error => 1,
        },
        Envelope::Error(ref error) => match error.status {
            Status::Ok => 0,
            Status::Partial => 2,
            Status::Error => 1,
        },
    };

    Ok(RunResult { exit_code, output })
}

/// Formats the output for human consumption.
///
/// Currently, this just returns the pretty-printed JSON implementation,
/// but it serves as a hook for future human-readable text output.
fn format_human(envelope: &Envelope) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&envelope)
}

/// Load a snapshot from a path (JSON file, stdin, or live filesystem).
///
/// This function detects the input type based on the path:
/// - "-" → Read JSON from stdin
/// - "*.json" → Read JSON from file
/// - Otherwise → Scan live filesystem
///
/// The `is_serialized` flag is set to indicate whether the snapshot
/// came from a JSON file (vs. live scan), which affects diff behavior.
fn load_snapshot(path: &Path, args: &ToolArgs) -> anyhow::Result<Snapshot> {
    if path == Path::new("-") {
        // Read from stdin
        let stdin = io::stdin();
        let snapshot: Snapshot = serde_json::from_reader(stdin)?;
        Ok(snapshot)
    } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
        // Load JSON snapshot
        let file = File::open(path)?;
        Ok(serde_json::from_reader(file)?)
    } else {
        // Scan live filesystem
        let scanner = Scanner::new(ScanOptions {
            root: path.to_path_buf(),
            checksum_algorithm: ChecksumAlgorithm::XXH3_64,
            follow_symlinks: args.follow_symlinks,
            max_depth: args.max_depth,
            exclude_patterns: args.exclude.clone(),
            timeout_ms: args.timeout_ms,
            threads: None, // Use default auto-detect
            normalize_paths: args.normalize_paths,
        });
        Ok(scanner.scan()?)
    }
}
