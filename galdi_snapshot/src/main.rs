//! CLI entry point for the `galdi_snapshot` tool.
//!
//! This tool scans a filesystem directory and generates a snapshot
//! representing the state of the files (paths, sizes, checksums, etc.).
//! The output includes a `$plumbah` annotation property for Plumbah compliance.
//!
//! Note: For MCP server mode, use the unified `galdi --serve` binary instead.

use clap::Parser;

// Use modules from the library
use galdi_snapshot::{app, cli};

use cli::ToolArgs;

/// Main entry point.
///
/// 1. Parses CLI arguments.
/// 2. Configures the filesystem scanner.
/// 3. Executes the scan.
/// 4. Outputs the result with Plumbah annotation.
/// 5. Prints the result to stdout.
fn main() -> anyhow::Result<()> {
    let args = ToolArgs::parse();
    let result = app::run(args)?;
    println!("{}", result.output);
    std::process::exit(result.exit_code);
}
