//! Command-line interface for unified `galdi` tool.
//!
//! Provides subcommands for `snapshot` and `diff` operations.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use galdi_core::ChecksumAlgorithm;

/// Unified galdi tool for filesystem snapshots and diffs
#[derive(Parser, Debug)]
#[command(name = "galdi")]
#[command(about = "Plumbah-compliant filesystem snapshot and diff tool", long_about = None)]
pub struct Args {
    /// Start MCP server mode (exposes both snapshot and diff tools)
    #[arg(long, global = true)]
    pub serve: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create a filesystem snapshot
    Snapshot(SnapshotArgs),
    /// Compare two snapshots or directories
    Diff(DiffArgs),
}

/// Arguments for the snapshot subcommand
#[derive(Parser, Debug)]
pub struct SnapshotArgs {
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

/// Arguments for the diff subcommand
#[derive(Parser, Debug)]
pub struct DiffArgs {
    pub source: PathBuf,

    pub target: PathBuf,

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

// Conversion from facade args to library args
impl From<SnapshotArgs> for galdi_snapshot::cli::ToolArgs {
    fn from(args: SnapshotArgs) -> Self {
        galdi_snapshot::cli::ToolArgs {
            path: args.path,
            output: args.output,
            checksum: args.checksum,
            follow_symlinks: args.follow_symlinks,
            max_depth: args.max_depth,
            exclude: args.exclude,
            shallow: args.shallow,
            human: args.human,
            timeout_ms: args.timeout_ms,
            jsonl: args.jsonl,
            threads: args.threads,
            normalize_paths: args.normalize_paths,
            plumbah_info: args.plumbah_info,
        }
    }
}

impl From<DiffArgs> for galdi_diff::cli::ToolArgs {
    fn from(args: DiffArgs) -> Self {
        galdi_diff::cli::ToolArgs {
            source: args.source,
            target: args.target,
            output: args.output,
            checksum: args.checksum,
            follow_symlinks: args.follow_symlinks,
            max_depth: args.max_depth,
            exclude: args.exclude,
            shallow: args.shallow,
            human: args.human,
            ignore_time: args.ignore_time,
            ignore_mode: args.ignore_mode,
            structure_only: args.structure_only,
            timeout_ms: args.timeout_ms,
            normalize_paths: args.normalize_paths,
            plumbah_info: args.plumbah_info,
        }
    }
}
