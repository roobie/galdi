//! Application logic separated from `main` so the CLI entry is thin.
//!
//! This module exposes `run()` which performs the scan and returns an
//! exit code. `main.rs` remains responsible only for argument parsing
//! and process-level concerns.

use chrono::Utc;
use galdi_core::*;
use serde::Serialize;
use std::time::Instant;

use crate::cli::ToolArgs;

#[derive(Serialize)]
#[serde(untagged)]
enum Envelope {
    Snapshot(Snapshot),
    Error(PlumbahObject),
}

/// Run the snapshot tool logic and return an exit code.
pub fn run(args: ToolArgs) -> anyhow::Result<RunResult> {
    let start = Instant::now();

    // Handle --plumbah-info flag: return only metadata without scanning
    if args.plumbah_info {
        let elapsed = start.elapsed();
        let plumbah = PlumbahObject::new(
            Status::Ok,
            Meta::new(
                "galdi_snapshot",
                env!("CARGO_PKG_VERSION"),
                true,  // idempotent
                false, // mutates
                true,  // safe
                true, // deterministic (paths are required input, and output depends only on these paths, so output depends only on input, i.e. deterministic)
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

    // Build scanner - always uses parallel walking
    let scanner = Scanner::new(ScanOptions {
        root: args.path.clone(),
        checksum_algorithm: args.checksum,
        follow_symlinks: args.follow_symlinks,
        max_depth: args.max_depth,
        exclude_patterns: args.exclude.clone(),
        timeout_ms: args.timeout_ms,
        threads: args.threads,
        normalize_paths: args.normalize_paths,
    });

    if args.jsonl {
        run_jsonl(scanner, start, &args)
    } else {
        run_json(scanner, start, &args)
    }
}

/// Run in JSON mode (original behavior)
fn run_json(scanner: Scanner, start: Instant, args: &ToolArgs) -> anyhow::Result<RunResult> {
    // Perform scan
    let result = scanner.scan();
    let elapsed = start.elapsed();

    // Build output with Plumbah annotation
    let envelope = match result {
        Ok(snapshot) => Envelope::Snapshot(snapshot),
        Err(e) => Envelope::Error(
            PlumbahObject::new(
                Status::Error,
                Meta::new(
                    "galdi_snapshot",
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
            .with_errors(vec![e.to_plumbah_error()]),
        ),
    };

    // Output to stdout
    let output = if args.human {
        format_human(&envelope)?
    } else {
        serde_json::to_string_pretty(&envelope)?
    };

    // Compute exit code from envelope
    let code = match envelope {
        Envelope::Snapshot(snapshot) => {
            match &snapshot.plumbah.meta {
                Some(_meta) => 0, // Success
                None => 1,        // Should not happen in JSON mode
            }
        }
        Envelope::Error(error) => match error.status {
            Status::Ok => 0,
            Status::Partial => 2,
            Status::Error => 1,
        },
    };

    Ok(RunResult {
        exit_code: code,
        output,
    })
}

/// Run in JSONL streaming mode
fn run_jsonl(scanner: Scanner, _start: Instant, _args: &ToolArgs) -> anyhow::Result<RunResult> {
    use crate::output::StreamingOutput;
    use std::io;

    let stdout = io::stdout();
    let mut streaming = StreamingOutput::new(stdout.lock());

    // Deterministic flag: false (filesystem can change between runs)
    let deterministic = false;

    // Write head
    streaming.write_head(
        &scanner.options.root,
        scanner.options.checksum_algorithm,
        deterministic,
    )?;

    // Stream entries one at a time using the iterator
    for result in scanner.scan_iter() {
        match result {
            Ok(entry) => {
                streaming.write_entry(&entry)?;
            }
            Err(e) => {
                // Write error as middle line
                streaming.write_error(&e.to_plumbah_error())?;
            }
        }
    }

    // Always write tail
    streaming.write_tail()?;

    Ok(RunResult {
        exit_code: streaming.exit_code(),
        output: String::new(), // Output already written to stdout
    })
}

pub struct RunResult {
    pub exit_code: i32,
    pub output: String,
}

fn format_human(envelope: &Envelope) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&envelope)
}
