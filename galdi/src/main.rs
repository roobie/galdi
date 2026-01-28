//! Unified CLI entry point for the `galdi` tool.
//!
//! Provides subcommands for `snapshot` and `diff` operations, as well as
//! a unified MCP server mode that exposes both tools.

use clap::Parser;
use rmcp::ServiceExt;

mod cli;
mod mcp;

use cli::{Args, Command};

/// Main entry point for the unified galdi tool.
///
/// 1. Parses CLI arguments
/// 2. If --serve flag: starts unified MCP server
/// 3. Otherwise, routes to appropriate subcommand:
///    - `galdi snapshot` → delegates to galdi_snapshot::app::run()
///    - `galdi diff` → delegates to galdi_diff::app::run()
/// 4. Prints output to stdout
/// 5. Exits with appropriate code (0=ok, 1=error, 2=partial)
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // If --serve flag is set, start unified MCP server
    if args.serve {
        let service = mcp::GaldiUnifiedService::new()
            .serve(rmcp::transport::stdio())
            .await?;
        service.waiting().await?;
        return Ok(());
    }

    // Route to appropriate subcommand
    match args.command {
        Some(Command::Snapshot(snapshot_args)) => {
            // Convert facade args to library args and delegate
            let lib_args: galdi_snapshot::cli::ToolArgs = snapshot_args.into();
            let result = galdi_snapshot::app::run(lib_args)?;
            println!("{}", result.output);
            std::process::exit(result.exit_code);
        }
        Some(Command::Diff(diff_args)) => {
            // Convert facade args to library args and delegate
            let lib_args: galdi_diff::cli::ToolArgs = diff_args.into();
            let result = galdi_diff::app::run(lib_args)?;
            println!("{}", result.output);
            std::process::exit(result.exit_code);
        }
        None => {
            // No subcommand provided - show help
            eprintln!("Error: No subcommand provided");
            eprintln!();
            eprintln!("Usage: galdi <COMMAND>");
            eprintln!();
            eprintln!("Commands:");
            eprintln!("  snapshot  Create a filesystem snapshot");
            eprintln!("  diff      Compare two snapshots or directories");
            eprintln!();
            eprintln!("Options:");
            eprintln!("  --serve   Start MCP server mode");
            eprintln!("  --help    Print help information");
            std::process::exit(1);
        }
    }
}
