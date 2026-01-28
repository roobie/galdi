//! Unified MCP server for both snapshot and diff operations.
//!
//! Exposes both `take_filesystem_snapshot` and `compare_snapshots` tools
//! through a single MCP server instance.

use galdi_core::ChecksumAlgorithm;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ErrorData as McpError, *},
    schemars, tool, tool_handler, tool_router,
};
use serde::Deserialize;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct GaldiUnifiedService {
    tool_router: ToolRouter<GaldiUnifiedService>,
}

// Re-export request types from individual binaries for consistency
// These will be made public in the library crates
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TakeFilesystemSnapshotRequest {
    #[schemars(description = "The path to take a snapshot of")]
    pub path: String,

    #[schemars(description = "Glob patterns to include or exclude files and directories")]
    pub exclude_patterns: Option<Vec<String>>,

    #[schemars(description = "Timeout in milliseconds for the snapshot operation")]
    pub timeout_ms: Option<u64>,

    #[schemars(
        description = "Output format: 'json' (default, pretty-printed) or 'jsonl' (streaming, one entry per line)"
    )]
    pub format: Option<String>,

    #[schemars(
        description = "Number of threads for parallel scanning (default: auto-detect based on CPU cores)"
    )]
    pub threads: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CompareSnapshotsRequest {
    #[schemars(description = "Path to the source snapshot (JSON file) or live directory")]
    pub source: String,

    #[schemars(description = "Path to the target snapshot (JSON file) or live directory")]
    pub target: String,

    #[schemars(description = "Ignore timestamp differences when comparing entries")]
    pub ignore_time: Option<bool>,

    #[schemars(description = "Ignore permission mode differences when comparing entries")]
    pub ignore_mode: Option<bool>,

    #[schemars(
        description = "Only compare structure (paths and types), skip checksums and metadata"
    )]
    pub structure_only: Option<bool>,

    #[schemars(description = "Glob patterns to exclude when scanning live directories")]
    pub exclude_patterns: Option<Vec<String>>,

    #[schemars(description = "Timeout in milliseconds for snapshot operations")]
    pub timeout_ms: Option<u64>,

    #[schemars(description = "Follow symbolic links when scanning live directories")]
    pub follow_symlinks: Option<bool>,

    #[schemars(description = "Maximum recursion depth for live directory scans")]
    pub max_depth: Option<usize>,

    #[schemars(description = "Normalize paths to use '/' as separator (useful on Windows)")]
    pub normalize_paths: Option<bool>,
}

#[tool_router]
impl GaldiUnifiedService {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Respond with a filesystem snapshot of the specified directory path. Supports both JSON (default) and JSONL (streaming) formats. JSONL format is recommended for large directories (1000+ files) for better memory efficiency and progressive parsing. Set 'stream: true' with JSONL format to enable true streaming with progress notifications."
    )]
    async fn take_filesystem_snapshot(
        &self,
        Parameters(request): Parameters<TakeFilesystemSnapshotRequest>,
    ) -> Result<CallToolResult, McpError> {
        let use_jsonl = request.format.as_deref() == Some("jsonl");

        // Delegate to galdi_snapshot::app::run
        let result = galdi_snapshot::app::run(galdi_snapshot::cli::ToolArgs {
            path: std::path::PathBuf::from(request.path),
            output: None,
            shallow: false,
            checksum: ChecksumAlgorithm::XXH3_64,
            follow_symlinks: false,
            human: false,
            max_depth: None,
            exclude: request.exclude_patterns.unwrap_or_default(),
            timeout_ms: request.timeout_ms,
            jsonl: use_jsonl,
            threads: request.threads,
            normalize_paths: false,
            plumbah_info: false,
        });

        match result {
            Err(e) => Err(McpError {
                message: Cow::Owned(format!("Failed to take filesystem snapshot: {}", e)),
                data: None,
                code: ErrorCode::INTERNAL_ERROR,
            }),
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(result.output)])),
        }
    }

    #[tool(
        description = "Compare two filesystem snapshots or live directories and report differences. Accepts JSON snapshot files, live filesystem paths, or any combination. Returns a Plumbah-compliant diff report with detailed change information including file additions, deletions, modifications, and attribute changes (content, permissions, timestamps, etc.)."
    )]
    async fn compare_snapshots(
        &self,
        Parameters(request): Parameters<CompareSnapshotsRequest>,
    ) -> Result<CallToolResult, McpError> {
        // Delegate to galdi_diff::app::run
        let result = galdi_diff::app::run(galdi_diff::cli::ToolArgs {
            source: std::path::PathBuf::from(request.source),
            target: std::path::PathBuf::from(request.target),
            output: None,
            checksum: ChecksumAlgorithm::XXH3_64,
            follow_symlinks: request.follow_symlinks.unwrap_or(false),
            max_depth: request.max_depth,
            exclude: request.exclude_patterns.unwrap_or_default(),
            shallow: false,
            human: false,
            ignore_time: request.ignore_time.unwrap_or(false),
            ignore_mode: request.ignore_mode.unwrap_or(false),
            structure_only: request.structure_only.unwrap_or(false),
            timeout_ms: request.timeout_ms,
            normalize_paths: request.normalize_paths.unwrap_or(false),
            plumbah_info: false,
        });

        match result {
            Err(e) => Err(McpError {
                message: Cow::Owned(format!("Failed to compare snapshots: {}", e)),
                data: None,
                code: ErrorCode::INTERNAL_ERROR,
            }),
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(result.output)])),
        }
    }
}

#[tool_handler]
impl ServerHandler for GaldiUnifiedService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Unified galdi service providing filesystem snapshot and diff capabilities. Use take_filesystem_snapshot to create snapshots and compare_snapshots to diff two filesystem states (JSON snapshots or live directories). All outputs are Plumbah-compliant with semantic annotations.".to_string()),
        }
    }
}
