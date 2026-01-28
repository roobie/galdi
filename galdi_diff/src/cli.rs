//! Command-line interface for `galdi_diff`.
//!
//! `galdi_diff` compares two filesystem snapshots (or live directories, or any combination thereof)
//! and emits a Plumbah-compliant diff report showing all differences.
//!
//! Example usage:
//! ```text
//! galdi_diff before.json after.json --ignore-time --human
//! ```
//!
//! For filter pattern syntax see `globwalk`'s glob patterns.

use std::path::PathBuf;

use clap::Parser;
use galdi_core::ChecksumAlgorithm;

#[derive(Parser, Debug)]
#[command(name = "galdi_diff")]
#[command(
    about = "Compare two galdi filesystem references or snapshots reporting on the differences, if any.",
    long_about = None
)]
pub struct ToolArgs {
    pub source: PathBuf,

    pub target: PathBuf,

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

    /// Ignore timestamp differences when comparing entries.
    #[arg(long)]
    pub ignore_time: bool,

    /// Ignore permission mode differences when comparing entries.
    #[arg(long)]
    pub ignore_mode: bool,

    /// Only compare structure (paths and types), skip checksums and metadata.
    #[arg(long)]
    pub structure_only: bool,

    /// Timeout in milliseconds for each snapshot operation.
    #[arg(long)]
    pub timeout_ms: Option<u64>,

    /// Normalize paths to use '/' as separator (useful on Windows).
    #[arg(long)]
    pub normalize_paths: bool,

    /// Return only Plumbah metadata (dry-run for semantic introspection).
    #[arg(long)]
    pub plumbah_info: bool,
}
