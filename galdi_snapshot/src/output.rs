//! JSONL streaming output handler for galdi_snapshot.
//!
//! Implements the Plumbah Streaming Profile v1.0 with head/middle/tail structure.

use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Path;
use std::time::Instant;

use chrono::Utc;
use galdi_core::{
    ChecksumAlgorithm, Meta, PlumbahError, PlumbahObject, ProfileMetadata, SnapshotEntry, Status,
    StreamSummary,
};

/// Handler for streaming JSONL output
pub struct StreamingOutput<W: Write> {
    writer: W,
    start_time: Instant,
    total_entries: usize,
    error_count: usize,
}

impl<W: Write> StreamingOutput<W> {
    /// Create a new streaming output handler
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            start_time: Instant::now(),
            total_entries: 0,
            error_count: 0,
        }
    }

    /// Write head line with metadata
    pub fn write_head(
        &mut self,
        root: &Path,
        checksum: ChecksumAlgorithm,
        deterministic: bool,
    ) -> io::Result<()> {
        let meta = Meta {
            idempotent: true,
            mutates: false,
            safe: true,
            deterministic, // false if --no-sort, false otherwise (filesystem changes)
            plumbah_level: 2,
            execution_time_ms: 0, // Set in tail
            tool: "galdi_snapshot".to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            extra: HashMap::new(),
            timestamp: Utc::now(),
            profiles: Some(vec![ProfileMetadata {
                name: "streaming".to_string(),
                data: HashMap::new(),
            }]),
        };

        let head = serde_json::json!({
            "$plumbah": PlumbahObject {
                version: "1.0".to_string(),
                stream: Some("head".to_string()),
                status: Status::Ok,
                meta: Some(meta),
                errors: None,
                summary: None,
                execution_time_ms: None,
            },
            "version": "1.0",
            "root": root,
            "checksum_algorithm": checksum,
        });

        writeln!(self.writer, "{}", serde_json::to_string(&head)?)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Write data entry (middle line - no $plumbah)
    pub fn write_entry(&mut self, entry: &SnapshotEntry) -> io::Result<()> {
        writeln!(self.writer, "{}", serde_json::to_string(entry)?)?;
        self.writer.flush()?;
        self.total_entries += 1;
        Ok(())
    }

    /// Write error line (middle line with $plumbah.errors)
    pub fn write_error(&mut self, error: &PlumbahError) -> io::Result<()> {
        let error_line = serde_json::json!({
            "$plumbah": {
                "errors": vec![error],
            }
        });
        writeln!(self.writer, "{}", serde_json::to_string(&error_line)?)?;
        self.writer.flush()?;
        self.error_count += 1;
        Ok(())
    }

    /// Write tail line with summary and final status
    pub fn write_tail(&mut self) -> io::Result<()> {
        let status = if self.error_count > 0 {
            Status::Partial
        } else {
            Status::Ok
        };

        let tail = serde_json::json!({
            "$plumbah": PlumbahObject {
                version: "1.0".to_string(),
                stream: Some("tail".to_string()),
                status,
                meta: None,
                errors: None,
                summary: Some(StreamSummary {
                    total: self.total_entries,
                    processed: self.total_entries,
                    errors: self.error_count,
                }),
                execution_time_ms: Some(self.start_time.elapsed().as_millis() as u64),
            }
        });

        writeln!(self.writer, "{}", serde_json::to_string(&tail)?)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Determine exit code: 0 (ok/partial), 1 (error)
    pub fn exit_code(&self) -> i32 {
        // Streaming profile: exit 0 for ok/partial, 1 for error
        0
    }
}
