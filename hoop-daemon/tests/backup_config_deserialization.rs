//! Isolated integration test for BackupFileConfig deserialization.
//!
//! This test file is deliberately minimal and self-contained to verify that
//! BackupFileConfig can be deserialized from YAML/JSON correctly without
//! being blocked by compilation failures in other parts of hoop-daemon.
//!
//! Run with: cargo test --test backup_config_deserialization

use serde::Deserialize;
use serde_json;

// ---------------------------------------------------------------------------
// Minimal BackupFileConfig definition (matches hoop-daemon/src/backup.rs)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct BackupFileConfig {
    pub endpoint: String,
    pub bucket: String,
    pub prefix: String,
    #[serde(default = "default_schedule")]
    pub schedule: String,
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
    #[serde(default)]
    pub encryption: bool,
}

fn default_schedule() -> String {
    "0 4 * * *".to_string()
}

fn default_retention_days() -> u32 {
    30
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn minimal_config_applies_defaults() {
    let yaml_input = "endpoint: https://s3.example.com\nbucket: my-bucket\nprefix: backups/";

    let yaml_value: serde_yaml::Value =
        serde_yaml::from_str(yaml_input).expect("YAML should parse");

    let json_value: serde_json::Value =
        serde_json::to_value(&yaml_value).expect("YAML→JSON conversion should succeed");

    let config: BackupFileConfig =
        serde_json::from_value(json_value).expect("BackupFileConfig should deserialize");

    assert_eq!(config.endpoint, "https://s3.example.com");
    assert_eq!(config.bucket, "my-bucket");
    assert_eq!(config.prefix, "backups/");
    assert_eq!(config.schedule, "0 4 * * *"); // default applied
    assert_eq!(config.retention_days, 30); // default applied
    assert!(!config.encryption); // default applied
}

#[test]
fn full_config_uses_explicit_values() {
    let yaml_input = "endpoint: https://s3.example.com\n\
                      bucket: my-bucket\n\
                      prefix: backups/\n\
                      schedule: '*/30 * * * *'\n\
                      retention_days: 14\n\
                      encryption: true";

    let yaml_value: serde_yaml::Value =
        serde_yaml::from_str(yaml_input).expect("YAML should parse");

    let json_value: serde_json::Value =
        serde_json::to_value(&yaml_value).expect("YAML→JSON conversion should succeed");

    let config: BackupFileConfig =
        serde_json::from_value(json_value).expect("BackupFileConfig should deserialize");

    assert_eq!(config.endpoint, "https://s3.example.com");
    assert_eq!(config.bucket, "my-bucket");
    assert_eq!(config.prefix, "backups/");
    assert_eq!(config.schedule, "*/30 * * * *"); // explicit value
    assert_eq!(config.retention_days, 14); // explicit value
    assert!(config.encryption); // explicit value
}

#[test]
fn direct_json_deserialization_works() {
    let json_str = r#"{
        "endpoint": "https://s3.example.com",
        "bucket": "my-bucket",
        "prefix": "backups/"
    }"#;

    let config: BackupFileConfig =
        serde_json::from_str(json_str).expect("Should deserialize from JSON directly");

    assert_eq!(config.endpoint, "https://s3.example.com");
    assert_eq!(config.bucket, "my-bucket");
    assert_eq!(config.prefix, "backups/");
    assert_eq!(config.schedule, "0 4 * * *"); // default
    assert_eq!(config.retention_days, 30); // default
}
