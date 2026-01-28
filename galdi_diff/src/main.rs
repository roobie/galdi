//! CLI entry point for the `galdi_diff` tool.
//!
//! This tool compares two filesystem snapshots (or live directories)
//! and reports on differences. The output includes a `$plumbah`
//! annotation property for Plumbah compliance.
//!
//! Note: For MCP server mode, use the unified `galdi --serve` binary instead.

use clap::Parser;

// Use modules from the library
use galdi_diff::{app, cli};

use cli::ToolArgs;

/// Main entry point.
///
/// 1. Parses CLI arguments.
/// 2. Executes the diff operation via app::run().
/// 3. Outputs the result to stdout.
/// 4. Exits with appropriate status code.
fn main() -> anyhow::Result<()> {
    let args = ToolArgs::parse();
    let result = app::run(args)?;
    println!("{}", result.output);
    std::process::exit(result.exit_code);
}
