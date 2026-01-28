//! Library exports for galdi_snapshot
//!
//! This module exposes internal components for testing purposes.

pub mod app;
pub mod cli;
pub mod output;

// Re-export commonly used types
pub use output::StreamingOutput;
