use chrono::{DateTime, Utc};
/// Plumbah annotation data structures for JSON output.
/// This module defines the Plumbah annotation structure that is added to JSON output
/// via the `$plumbah` property, including metadata, status, and error information.
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

/// The current Plumbah annotation version.
pub const PLUMBAH_VERSION: &str = "1.0";

#[derive(Debug, Serialize, Deserialize)]
pub struct PlumbahObject {
    pub version: String, // "1.0"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<String>, // "head" or "tail" for streaming
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>, // Present in head, absent in middle
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<PlumbahError>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<StreamSummary>, // Present in tail only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_time_ms: Option<u64>, // In tail (also in Meta for regular output)
}

impl PlumbahObject {
    pub fn new(status: Status, meta: Meta) -> Self {
        PlumbahObject {
            version: PLUMBAH_VERSION.to_string(),
            stream: None,
            status,
            meta: Some(meta),
            errors: None,
            summary: None,
            execution_time_ms: None,
        }
    }
    pub fn with_errors(mut self, errors: Vec<PlumbahError>) -> Self {
        self.errors = Some(errors);
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Error,
    Partial,
}

/// Streaming summary for tail line
#[derive(Debug, Serialize, Deserialize)]
pub struct StreamSummary {
    pub total: usize,
    pub processed: usize,
    pub errors: usize,
}

/// Profile metadata for streaming profile support
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileMetadata {
    pub name: String,
    #[serde(flatten)]
    pub data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    /// Whether the invocation is idempotent, and as such can be safely retried without side effects.
    pub idempotent: bool,
    /// Whether the invocation mutates (external) state.
    pub mutates: bool,
    /// Whether the invocation is safe to run (i.e., non-destructive).
    pub safe: bool,
    /// Whether the invocation is deterministic (i.e., produces the same output for the same input).
    /// I.e. for galdi_snapshot, this is always false, because the file system can change between runs.
    /// For galdi_diff, this is true iif both inputs are serialized snapshots (i.e. read from file or stdin).
    pub deterministic: bool,
    pub plumbah_level: u8, // 0, 1, or 2
    pub execution_time_ms: u64,
    /// The name of the tool that produced this output.
    pub tool: String,
    /// The version of the tool that produced this output. Must follow semver.
    pub tool_version: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
    /// Timestamp of when the output was produced.
    pub timestamp: DateTime<Utc>,
    /// Optional profiles metadata for streaming and other profiles
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: Option<Vec<ProfileMetadata>>,
}

impl Meta {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tool: &str,
        tool_version: &str,
        idempotent: bool,
        mutates: bool,
        safe: bool,
        deterministic: bool,
        execution_time_ms: u64,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Meta {
            idempotent,
            mutates,
            safe,
            deterministic,
            plumbah_level: 2,
            execution_time_ms,
            tool: tool.to_string(),
            tool_version: tool_version.to_string(),
            extra: HashMap::new(),
            timestamp,
            profiles: None,
        }
    }

    pub fn with_profiles(mut self, profiles: Vec<ProfileMetadata>) -> Self {
        self.profiles = Some(profiles);
        self
    }

    pub fn with_default_profiles(self) -> Self {
        self.with_profiles(vec![ProfileMetadata {
            name: "_determinism".to_string(),
            data: HashMap::from([(
                "depends_on".to_string(),
                serde_json::json!(vec!["file_system"]),
            )]),
        }])
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlumbahError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    pub recoverable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, serde_json::Value>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_status_serializes_lowercase() {
        // Test that Status enum serializes to lowercase
        let ok_json = serde_json::to_string(&Status::Ok).unwrap();
        assert_eq!(ok_json, r#""ok""#);

        let error_json = serde_json::to_string(&Status::Error).unwrap();
        assert_eq!(error_json, r#""error""#);

        let partial_json = serde_json::to_string(&Status::Partial).unwrap();
        assert_eq!(partial_json, r#""partial""#);
    }

    #[test]
    fn test_plumbah_version_constant() {
        assert_eq!(PLUMBAH_VERSION, "1.0");
    }

    #[test]
    fn test_meta_has_all_semantic_flags() {
        let meta = Meta::new(
            "test_tool",
            "1.0.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );

        // Verify all four semantic flags are present
        assert_eq!(meta.idempotent, true);
        assert_eq!(meta.mutates, false);
        assert_eq!(meta.safe, true);
        assert_eq!(meta.deterministic, true);
    }

    #[test]
    fn test_errors_skip_when_empty() {
        let meta = Meta::new(
            "test_tool",
            "1.0.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );
        let plumbah = PlumbahObject::new(Status::Ok, meta);

        let json = serde_json::to_string(&plumbah).unwrap();

        // When errors is None, it should not appear in JSON
        assert!(!json.contains("errors"));
    }

    #[test]
    fn test_plumbah_object_ok_serialization() {
        let meta = Meta::new(
            "test_tool",
            "1.0.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );
        let plumbah = PlumbahObject::new(Status::Ok, meta);

        let json = serde_json::to_value(&plumbah).unwrap();

        assert_eq!(json["version"], "1.0");
        assert_eq!(json["status"], "ok");
        assert_eq!(json["meta"]["idempotent"], true);
        assert_eq!(json["meta"]["plumbah_level"], 2);
    }

    #[test]
    fn test_plumbah_object_error_serialization() {
        let meta = Meta::new(
            "test_tool",
            "1.0.0",
            false,
            false,
            true,
            false,
            100,
            Utc::now(),
        );
        let plumbah = PlumbahObject::new(Status::Error, meta);

        let json = serde_json::to_value(&plumbah).unwrap();

        assert_eq!(json["version"], "1.0");
        assert_eq!(json["status"], "error");
    }

    #[test]
    fn test_plumbah_object_partial_serialization() {
        let meta = Meta::new(
            "test_tool",
            "1.0.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );
        let plumbah = PlumbahObject::new(Status::Partial, meta);

        let json = serde_json::to_value(&plumbah).unwrap();

        assert_eq!(json["version"], "1.0");
        assert_eq!(json["status"], "partial");
    }

    #[test]
    fn test_plumbah_object_level2_fields() {
        let meta = Meta::new(
            "test_tool",
            "1.2.3",
            true,
            false,
            true,
            true,
            245,
            Utc::now(),
        );

        // Verify Level 2 specific fields
        assert_eq!(meta.plumbah_level, 2);
        assert_eq!(meta.execution_time_ms, 245);
        assert_eq!(meta.tool, "test_tool");
        assert_eq!(meta.tool_version, "1.2.3");
    }

    #[test]
    fn test_plumbah_object_with_errors() {
        let meta = Meta::new(
            "test_tool",
            "1.0.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );

        let error = PlumbahError {
            code: "TEST_ERROR".to_string(),
            message: "Test error message".to_string(),
            path: Some(PathBuf::from("/test")),
            recoverable: false,
            context: None,
        };

        let plumbah = PlumbahObject::new(Status::Error, meta).with_errors(vec![error]);

        let json = serde_json::to_value(&plumbah).unwrap();

        assert!(json["errors"].is_array());
        assert_eq!(json["errors"][0]["code"], "TEST_ERROR");
    }

    #[test]
    fn test_plumbah_object_extra_fields_preserved() {
        let mut meta = Meta::new(
            "test_tool",
            "1.0.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );

        // Add extra fields
        meta.extra.insert(
            "custom_field".to_string(),
            serde_json::json!("custom_value"),
        );
        meta.extra
            .insert("entries_scanned".to_string(), serde_json::json!(42));

        let json = serde_json::to_value(&meta).unwrap();

        // Verify extra fields are flattened into meta
        assert_eq!(json["custom_field"], "custom_value");
        assert_eq!(json["entries_scanned"], 42);
    }

    #[test]
    fn test_annotation_does_not_wrap_data() {
        // This test verifies the annotation pattern concept
        // PlumbahObject should be used as a sibling field, not a wrapper

        #[derive(Serialize)]
        struct AnnotatedOutput {
            #[serde(rename = "$plumbah")]
            plumbah: PlumbahObject,
            data_field: String,
            count: u32,
        }

        let meta = Meta::new(
            "test_tool",
            "1.0.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );

        let output = AnnotatedOutput {
            plumbah: PlumbahObject::new(Status::Ok, meta),
            data_field: "test".to_string(),
            count: 42,
        };

        let json = serde_json::to_value(&output).unwrap();

        // Data should be at top level, not nested
        assert!(json.get("$plumbah").is_some());
        assert_eq!(json["data_field"], "test");
        assert_eq!(json["count"], 42);

        // Should NOT have a "data" wrapper
        assert!(json.get("data").is_none());
    }

    // Property-based tests would go here if proptest is added
    // For now, we'll add basic roundtrip tests

    #[test]
    fn test_plumbah_object_roundtrip() {
        let meta = Meta::new(
            "test_tool",
            "1.0.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );
        let original = PlumbahObject::new(Status::Ok, meta);

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: PlumbahObject = serde_json::from_str(&json).unwrap();

        assert_eq!(original.version, deserialized.version);
        assert_eq!(
            serde_json::to_string(&original.status).unwrap(),
            serde_json::to_string(&deserialized.status).unwrap()
        );
    }

    #[test]
    fn test_meta_extra_fields_roundtrip() {
        let mut meta = Meta::new(
            "test_tool",
            "1.0.0",
            true,
            false,
            true,
            true,
            100,
            Utc::now(),
        );

        meta.extra
            .insert("custom".to_string(), serde_json::json!({"nested": "value"}));

        let json = serde_json::to_string(&meta).unwrap();
        let deserialized: Meta = serde_json::from_str(&json).unwrap();

        assert_eq!(meta.extra.len(), deserialized.extra.len());
        assert_eq!(meta.extra.get("custom"), deserialized.extra.get("custom"));
    }
}
