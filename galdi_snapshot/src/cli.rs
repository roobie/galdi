//! Command-line interface for `galdi_snapshot`.
//!
//! `galdi_snapshot` scans a directory and emits a Plumbah-compliant
//! filesystem snapshot (JSON) that can be consumed by `galdi` for diffing.
//!
//! Example usage:
//! ```text
//! galdi_snapshot /path/to/dir --checksum xxh3_64 --human
//! ```
//!
//! For filter pattern syntax see `globwalk`'s glob patterns.

use std::path::PathBuf;

use clap::Parser;
use galdi_core::ChecksumAlgorithm;

#[derive(Parser, Debug)]
#[command(name = "galdi_snapshot")]
#[command(
    about = "Create Plumbah-compliant filesystem snapshots for use with galdi.",
    long_about = None
)]
pub struct ToolArgs {
    /// Directory to snapshot (required).
    pub path: PathBuf,

    /// Write snapshot to this file; stdout if omitted.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Checksum algorithm to use for file content.
    /// Defaults to `xxh3_64`. See `galdi_core::ChecksumAlgorithm`.
    #[arg(long, default_value = "xxh3_64")]
    pub checksum: ChecksumAlgorithm,

    /// Follow symbolic links when scanning.
    #[arg(long)]
    pub follow_symlinks: bool,

    /// Maximum recursion depth for the scan. `None` = unlimited.
    #[arg(long)]
    pub max_depth: Option<usize>,

    /// Glob-style include/exclude patterns (supports `globwalk` syntax).
    /// Use `!` prefix to exclude. Multiple patterns may be provided.
    /// Examples: `**/*.log` (include .log files), `!temp/` (exclude directories named `temp`).
    #[arg(long)]
    pub exclude: Vec<String>,

    /// Perform a shallow scan (skip content checksums).
    #[arg(long)]
    pub shallow: bool,

    /// Output human-friendly representation instead of JSON.
    #[arg(long)]
    pub human: bool,

    #[arg(long)]
    pub timeout_ms: Option<u64>,

    /// Output JSONL format (streaming) instead of JSON.
    #[arg(long)]
    pub jsonl: bool,

    /// Number of threads for parallel scanning (default: auto-detect).
    #[arg(long)]
    pub threads: Option<usize>,

    /// Normalize paths to use '/' as separator (useful on Windows).
    #[arg(long)]
    pub normalize_paths: bool,

    /// Return only Plumbah metadata (dry-run for semantic introspection).
    #[arg(long)]
    pub plumbah_info: bool,
}
