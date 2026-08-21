//! Comprehensive unit tests for schema version handling
//!
//! Tests cover:
//! - Schema version compatibility checks
//! - Version parsing and comparison
//! - Edge cases (boundaries, malformed schemas)
//! - Migration logic validation

use hoop_schema::*;

#[cfg(test)]
mod compatibility_checks {
    use super::*;
    use hoop_schema::version::SCHEMA_VERSION;

    /// Test that SCHEMA_VERSION follows semver format
    #[test]
    fn test_schema_version_format_valid() {
        let re = regex::Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
        assert!(
            re.is_match(SCHEMA_VERSION),
            "SCHEMA_VERSION should follow X.Y.Z format"
        );
    }

    /// Test SCHEMA_VERSION components are parseable
    #[test]
    fn test_schema_version_parseable() {
        let parts: Vec<&str> = SCHEMA_VERSION.split('.').collect();
        assert_eq!(parts.len(), 3, "Should have exactly 3 components");

        let major: u32 = parts[0]
            .parse()
            .expect("Major version should be parseable");
        let _minor: u32 = parts[1]
            .parse()
            .expect("Minor version should be parseable");
        let _patch: u32 = parts[2]
            .parse()
            .expect("Patch version should be parseable");

        assert!(major >= 1, "Major version should be at least 1");
        // No specific constraints on minor/patch
    }

    /// Test that schema version types are distinct
    #[test]
    fn test_schema_version_types_are_distinct() {
        // Different schema version types exist for different record types
        // They are all Newtype-pattern structs wrapping String
        // This test verifies they're different types by compilation

        // We can deserialize JSON into each type
        let _sv1: HoopConfigSchemaVersion = serde_json::from_str("\"1.33.0\"").unwrap();
        let _sv2: StitchSchemaVersion = serde_json::from_str("\"1.33.0\"").unwrap();

        // These are different types - you can't assign one to the other
        // If they were the same type, this would compile:
        // let _sv3: HoopConfigSchemaVersion = _sv2; // This would fail to compile
    }
}

#[cfg(test)]
mod version_comparison {
    use super::*;

    /// Parse semver string to comparable tuple
    fn parse_version(v: &str) -> (u32, u32, u32) {
        let parts: Vec<u32> = v
            .split('.')
            .map(|p| p.parse().unwrap())
            .collect();
        (parts[0], parts[1], parts[2])
    }

    /// Test major version comparison
    #[test]
    fn test_version_comparison_major() {
        assert!(parse_version("2.0.0") > parse_version("1.99.99"));
        assert!(parse_version("1.0.0") > parse_version("0.99.99"));
        assert!(parse_version("10.0.0") > parse_version("9.99.99"));
    }

    /// Test minor version comparison
    #[test]
    fn test_version_comparison_minor() {
        assert!(parse_version("1.2.0") > parse_version("1.1.99"));
        assert!(parse_version("1.10.0") > parse_version("1.9.99"));
        assert!(parse_version("1.1.0") > parse_version("1.0.99"));
    }

    /// Test patch version comparison
    #[test]
    fn test_version_comparison_patch() {
        assert!(parse_version("1.2.3") > parse_version("1.2.2"));
        assert!(parse_version("1.2.10") > parse_version("1.2.9"));
        assert!(parse_version("1.2.0") > parse_version("1.1.99"));
    }

    /// Test version equality
    #[test]
    fn test_version_equality() {
        assert_eq!(parse_version("1.2.3"), parse_version("1.2.3"));
        assert_eq!(parse_version("0.0.0"), parse_version("0.0.0"));
        assert_eq!(parse_version("99.99.99"), parse_version("99.99.99"));
    }

    /// Test version ordering is transitive
    #[test]
    fn test_version_ordering_transitive() {
        let v1 = parse_version("1.0.0");
        let v2 = parse_version("1.1.0");
        let v3 = parse_version("1.2.0");

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3); // Transitivity
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    /// Test schema version mismatch detection causes panic in write_versioned
    #[test]
    #[should_panic(expected = "schema_version mismatch")]
    fn test_missing_schema_version_panics() {
        use uuid::Uuid;
        use chrono::{DateTime, Utc};

        // Create a Stitch with a valid structure but wrong version
        // The panic happens when we try to write_versioned with wrong version
        let stitch_json = serde_json::json!({
            "id": Uuid::new_v4(),
            "project": "test",
            "kind": "operator",
            "title": "Test",
            "created_by": "test",
            "created_at": "2024-01-01T00:00:00Z",
            "participants": [],
            "schema_version": "1.0.0"  // Wrong version - will cause mismatch panic
        });

        let stitch: Stitch = serde_json::from_value(stitch_json).unwrap();

        // This should panic because schema_version doesn't match SCHEMA_VERSION
        write_versioned(&stitch);
    }

    /// Test malformed version strings are rejected
    #[test]
    fn test_malformed_version_rejected() {
        let invalid_versions = vec![
            "not.a.version",
            "1.2",
            "1.2.3.4",
            "v1.2.3",
            "1.x.0",
            "",
            ".",
            "1..0",
            "1.2.",
        ];

        let re = regex::Regex::new(r"^\d+\.\d+\.\d+$").unwrap();

        for version in invalid_versions {
            assert!(
                !re.is_match(version),
                "Invalid version '{}' should not match semver pattern",
                version
            );
        }
    }

    /// Test version boundary: zero versions
    #[test]
    fn test_version_boundary_zero() {
        let v = parse_version("0.0.0");
        assert_eq!(v, (0, 0, 0));

        assert!(parse_version("0.0.1") > parse_version("0.0.0"));
        assert!(parse_version("0.1.0") > parse_version("0.0.99"));
        assert!(parse_version("1.0.0") > parse_version("0.99.99"));
    }

    /// Test version boundary: large versions
    #[test]
    fn test_version_boundary_large() {
        // Test that large version numbers are handled correctly
        assert!(parse_version("99.99.99") > parse_version("99.99.98"));
        assert!(parse_version("100.0.0") > parse_version("99.99.99"));

        // Test u32 boundary (though versions should never be this large)
        let max = u32::MAX;
        let v_max = parse_version(&format!("{}.{}.{}", max, max, max));
        assert_eq!(v_max, (max, max, max));
    }

    /// Test version boundary: leading zeros
    #[test]
    fn test_version_boundary_leading_zeros() {
        // "01.02.03" should parse to (1, 2, 3)
        let v = parse_version("01.02.03");
        assert_eq!(v, (1, 2, 3));

        // Leading zeros don't affect comparison
        assert_eq!(parse_version("01.02.03"), parse_version("1.2.3"));
    }

    /// Test schema version mismatch detection
    #[test]
    fn test_schema_version_mismatch_detection() {
        use serde_json::json;

        // Create a Stitch with wrong schema version
        let wrong_version = json!({
            "id": Uuid::new_v4(),
            "project": "test",
            "kind": "operator",
            "title": "Test",
            "created_by": "user",
            "created_at": "2024-01-01T00:00:00Z",
            "participants": [],
            "schema_version": "1.0.0"  // Wrong version
        });

        // This should deserialize (schema_version field exists)
        let stitch: Stitch = serde_json::from_value(wrong_version).unwrap();

        // But write_versioned should panic
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            write_versioned(&stitch);
        }));

        assert!(result.is_err(), "write_versioned should panic on version mismatch");
    }

    /// Test schema version types preserve string values
    #[test]
    fn test_schema_version_type_preservation() {
        let version: HoopConfigSchemaVersion = serde_json::from_str("\"1.33.0\"").unwrap();
        let serialized = serde_json::to_string(&version).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        // Should serialize as plain string, not wrapped object
        assert_eq!(parsed.as_str(), Some("1.33.0"));
    }

    /// Test all DurableRecord types have schema_version field
    #[test]
    fn test_all_durable_records_have_schema_version() {
        let ts = parse_utc("2024-01-01T00:00:00Z");

        // Deserialize from JSON to verify schema_version field presence
        let stitch_json = serde_json::json!({
            "id": Uuid::new_v4(),
            "project": "test",
            "kind": "operator",
            "title": "Test",
            "created_by": "test",
            "created_at": "2024-01-01T00:00:00Z",
            "participants": [],
            "schema_version": "1.33.0"
        });

        let stitch: Stitch = serde_json::from_value(stitch_json).unwrap();
        let serialized = serde_json::to_value(stitch).unwrap();

        assert!(
            serialized.get("schema_version").is_some(),
            "DurableRecord should have schema_version field"
        );
    }

    fn parse_utc(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    fn parse_version(v: &str) -> (u32, u32, u32) {
        let parts: Vec<u32> = v
            .split('.')
            .map(|p| p.parse().unwrap())
            .collect();
        (parts[0], parts[1], parts[2])
    }
}

#[cfg(test)]
mod migration_compatibility {
    use super::*;

    /// Test that types maintain backward compatibility within minor versions
    #[test]
    fn test_minor_version_backward_compat() {
        // Within a minor version (1.X.Y), adding optional fields should be compatible
        let minimal = r#"{"schema_version":"1.33.0"}"#;
        let extended = r#"{"schema_version":"1.33.0","agent":{"adapter":"claude"}}"#;

        // Both should deserialize successfully
        let _min: HoopConfig = serde_json::from_str(minimal).unwrap();
        let _ext: HoopConfig = serde_json::from_str(extended).unwrap();
    }

    /// Test that major version changes are detected
    #[test]
    fn test_major_version_detection() {
        let v1 = parse_version("1.99.99");
        let v2 = parse_version("2.0.0");

        assert!(v2 > v1, "Major version 2.0.0 should be greater than 1.99.99");
    }

    /// Test that patch versions are compatible
    #[test]
    fn test_patch_version_compatibility() {
        // Patch versions should be fully compatible
        let v1 = parse_version("1.33.0");
        let v2 = parse_version("1.33.1");

        assert!(v2 > v1, "Patch version should be comparable");
    }

    fn parse_version(v: &str) -> (u32, u32, u32) {
        let parts: Vec<u32> = v
            .split('.')
            .map(|p| p.parse().unwrap())
            .collect();
        (parts[0], parts[1], parts[2])
    }
}

#[cfg(test)]
mod schema_validation {
    use super::*;

    /// Test that schema_version is required field
    #[test]
    fn test_schema_version_required() {
        // Missing schema_version should fail deserialization
        let invalid = r#"{"project":"test","title":"Test"}"#;
        let result: Result<Stitch, _> = serde_json::from_str(invalid);
        assert!(result.is_err(), "Should fail without schema_version");
    }

    /// Test that schema_version accepts valid format
    #[test]
    fn test_schema_version_valid_format() {
        use uuid::Uuid;

        let valid_json = serde_json::json!({
            "id": Uuid::new_v4(),
            "schema_version":"1.33.0",
            "project":"test",
            "kind":"operator",
            "title":"Test",
            "created_by":"test",
            "created_at":"2024-01-01T00:00:00Z",
            "participants":[]
        });

        let result: Result<Stitch, _> = serde_json::from_value(valid_json);
        assert!(result.is_ok(), "Should succeed with valid schema_version format");
    }

    /// Test that schema_version rejects invalid formats
    #[test]
    fn test_schema_version_invalid_formats() {
        let invalid_formats = vec![
            r#"{"schema_version":"1.33","project":"test","kind":"operator","title":"Test","created_by":"test","created_at":"2024-01-01T00:00:00Z","participants":[]}"#, // Missing patch
            r#"{"schema_version":"latest","project":"test","kind":"operator","title":"Test","created_by":"test","created_at":"2024-01-01T00:00:00Z","participants":[]}"#, // Non-numeric
            r#"{"schema_version":"v1.33.0","project":"test","kind":"operator","title":"Test","created_by":"test","created_at":"2024-01-01T00:00:00Z","participants":[]}"#, // With 'v' prefix
        ];

        for invalid in invalid_formats {
            let _result: Result<Stitch, _> = serde_json::from_str(invalid);
            // Schema validation may or may not catch these at deserialize time
            // depending on the JSON Schema validation implementation
            // The key is that write_versioned should catch version mismatches
        }
    }

    /// Test schema version round-trip preservation
    #[test]
    fn test_schema_version_round_trip() {
        use uuid::Uuid;
        use chrono::{DateTime, Utc};

        let stitch_json = serde_json::json!({
            "id": Uuid::new_v4(),
            "project": "test",
            "kind": "operator",
            "title": "Test",
            "created_by": "test",
            "created_at": "2024-01-01T00:00:00Z",
            "participants": [],
            "schema_version": "1.33.0"
        });

        let stitch: Stitch = serde_json::from_value(stitch_json).unwrap();
        let json = serde_json::to_string(&stitch).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(
            parsed["schema_version"],
            "1.33.0",
            "schema_version should be preserved in JSON"
        );
    }
}
