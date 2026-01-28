//! Library exports for galdi_diff.
//!
//! This module exposes internal components for testing and future
//! MCP server integration.

pub mod app;
pub mod cli;
pub mod diff;

// Re-export commonly used types
pub use app::{RunResult, run};
