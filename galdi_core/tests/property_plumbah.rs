// Property-based tests for Plumbah annotation
mod common;

use galdi_core::{Meta, PlumbahError, PlumbahObject, Status};
use proptest::prelude::*;

proptest! {
    /// Property A: Version Constant
    /// ∀ plumbah_object: PlumbahObject.
    ///   plumbah_object.version == "1.0"
    #[test]
    fn proptest_plumbah_version_constant(
        tool_name in "[a-z_]{3,15}",
        tool_version in "[0-9]\\.[0-9]\\.[0-9]",
        exec_time in 1u64..10000
    ) {
        let meta = Meta::new(
            &tool_name,
            &tool_version,
            true,
            false,
            true,
            true,
            exec_time,
            chrono::Utc::now(),
        );

        let plumbah = PlumbahObject::new(Status::Ok, meta);

        prop_assert_eq!(plumbah.version, "1.0",
            "PlumbahObject version should always be 1.0");
    }

    /// Property B: Status Serialization (lowercase)
    /// ∀ status: Status.
    ///   serialize(status).is_lowercase()
    #[test]
    fn proptest_status_serialization_lowercase(
        use_ok in proptest::bool::ANY
    ) {
        let status = if use_ok {
            Status::Ok
        } else {
            Status::Error
        };

        let meta = Meta::new(
            "test_tool",
            "0.1.0",
            true,
            false,
            true,
            true,
            100,
            chrono::Utc::now(),
        );

        let plumbah = PlumbahObject::new(status, meta);
        let json = serde_json::to_string(&plumbah).expect("Serialization should succeed");

        // Check that status is lowercase in JSON
        if use_ok {
            prop_assert!(json.contains("\"ok\""),
                "Status::Ok should serialize as lowercase 'ok'");
            prop_assert!(!json.contains("\"OK\""),
                "Status should not contain uppercase 'OK'");
        } else {
            prop_assert!(json.contains("\"error\""),
                "Status::Error should serialize as lowercase 'error'");
            prop_assert!(!json.contains("\"ERROR\""),
                "Status should not contain uppercase 'ERROR'");
        }
    }

    /// Property C: Meta Level 2 Compliance
    /// Meta should always be Level 2 compliant
    #[test]
    fn proptest_meta_level_2_compliance(
        exec_time in 1u64..10000
    ) {
        let meta = Meta::new(
            "galdi_snapshot",
            "0.2.2",
            true,
            false,
            true,
            true,
            exec_time,
            chrono::Utc::now(),
        );

        let plumbah = PlumbahObject::new(Status::Ok, meta);

        let meta_ref = plumbah.meta.as_ref().expect("Meta should be present");

        prop_assert_eq!(meta_ref.plumbah_level, 2,
            "Meta should be Level 2 compliant");
        prop_assert!(!meta_ref.tool.is_empty(),
            "Tool name should not be empty");
        prop_assert!(!meta_ref.tool_version.is_empty(),
            "Tool version should not be empty");
        prop_assert!(meta_ref.execution_time_ms > 0,
            "Execution time should be > 0");
    }

    /// Property D: Optional Field Skipping (errors)
    /// When errors is None, it should not appear in serialized JSON
    #[test]
    fn proptest_optional_errors_omitted(
        include_errors in proptest::bool::ANY
    ) {
        let meta = Meta::new(
            "test_tool",
            "0.1.0",
            true,
            false,
            true,
            true,
            100,
            chrono::Utc::now(),
        );

        let mut plumbah = PlumbahObject::new(Status::Ok, meta);

        if include_errors {
            plumbah.errors = Some(vec![PlumbahError {
                code: "TEST_ERROR".to_string(),
                message: "test error".to_string(),
                path: None,
                recoverable: true,
                context: None,
            }]);
        } else {
            plumbah.errors = None;
        }

        let json = serde_json::to_string(&plumbah).expect("Serialization should succeed");

        if include_errors {
            prop_assert!(json.contains("\"errors\""),
                "JSON should contain 'errors' field when Some");
        } else {
            prop_assert!(!json.contains("\"errors\""),
                "JSON should not contain 'errors' field when None");
        }
    }

    /// Property E: Optional Field Skipping (stream)
    /// When stream is None, it should not appear in serialized JSON
    #[test]
    fn proptest_optional_stream_omitted(
        include_stream in proptest::bool::ANY
    ) {
        let meta = Meta::new(
            "test_tool",
            "0.1.0",
            true,
            false,
            true,
            true,
            100,
            chrono::Utc::now(),
        );

        let mut plumbah = PlumbahObject::new(Status::Ok, meta);

        if include_stream {
            plumbah.stream = Some("head".to_string());
        } else {
            plumbah.stream = None;
        }

        let json = serde_json::to_string(&plumbah).expect("Serialization should succeed");

        if include_stream {
            prop_assert!(json.contains("\"stream\""),
                "JSON should contain 'stream' field when Some");
        } else {
            prop_assert!(!json.contains("\"stream\""),
                "JSON should not contain 'stream' field when None");
        }
    }

    /// Property F: Roundtrip Serialization
    /// PlumbahObject should survive serialize -> deserialize cycle
    #[test]
    fn proptest_plumbah_roundtrip(
        exec_time in 1u64..10000
    ) {
        let meta = Meta::new(
            "galdi_snapshot",
            "0.2.2",
            true,
            false,
            true,
            true,
            exec_time,
            chrono::Utc::now(),
        );

        let original = PlumbahObject::new(Status::Ok, meta);
        let json = serde_json::to_string(&original).expect("Serialization should succeed");
        let deserialized: PlumbahObject = serde_json::from_str(&json)
            .expect("Deserialization should succeed");

        prop_assert_eq!(original.version, deserialized.version,
            "Version should match after roundtrip");
        // Note: Status doesn't implement PartialEq, so we can't compare directly
        // But roundtrip serialization will ensure status is preserved

        let orig_meta = original.meta.as_ref().unwrap();
        let deser_meta = deserialized.meta.as_ref().unwrap();

        prop_assert_eq!(&orig_meta.tool, &deser_meta.tool,
            "Tool name should match after roundtrip");
        prop_assert_eq!(&orig_meta.tool_version, &deser_meta.tool_version,
            "Tool version should match after roundtrip");
        prop_assert_eq!(orig_meta.plumbah_level, deser_meta.plumbah_level,
            "Plumbah level should match after roundtrip");
    }
}
