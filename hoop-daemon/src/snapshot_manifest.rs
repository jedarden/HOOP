//! Snapshot manifest: ties all backup pieces together.
//!
//! Each backup writes a `manifest.json` uploaded **last** (after fleet.db and
//! attachments) so that a partial upload is never mistaken for a complete
//! snapshot. Restore validates the manifest before taking any destructive
//! action and rejects snapshots newer than the running binary.
//!
//! Plan reference: §15.3, §20.1

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// Per-snapshot manifest uploaded to S3 alongside backup pieces.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SnapshotManifest {
    /// Unique snapshot identifier (ISO timestamp, e.g. `20240615T040000Z`).
    pub snapshot_id: String,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// HOOP schema version at snapshot time (from `fleet::SCHEMA_VERSION`).
    pub schema_version: String,
    /// S3 key for the compressed (and optionally encrypted) fleet.db backup.
    pub fleet_db_key: String,
    /// S3 key for the incremental attachments manifest (if present).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments_manifest_key: Option<String>,
    /// Encryption mode: `"none"` or `"age"`.
    pub encryption: String,
    /// HOOP binary version at snapshot time.
    pub hoop_version: String,
    /// SHA-256 of the compressed fleet.db blob for integrity verification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fleet_db_sha256: Option<String>,
    /// Size in bytes of the compressed fleet.db blob.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fleet_db_size: Option<u64>,
}

impl SnapshotManifest {
    /// Validate the manifest against the running binary.
    ///
    /// Returns `Ok(())` when the manifest is usable, `Err` otherwise.
    /// Currently checks:
    /// - `schema_version` is not newer than `current_schema` (§20.1).
    pub fn validate(&self, current_schema: &str) -> Result<()> {
        if is_newer_version(&self.schema_version, current_schema) {
            bail!(
                "Snapshot schema version {} is newer than this binary's {}. \
                 Upgrade HOOP before restoring this snapshot.",
                self.schema_version,
                current_schema
            );
        }
        Ok(())
    }
}

/// Compare two semver-like strings; returns `true` when `a > b`.
pub fn is_newer_version(a: &str, b: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.').filter_map(|p| p.parse().ok()).collect()
    };
    let va = parse(a);
    let vb = parse(b);
    for i in 0..std::cmp::max(va.len(), vb.len()) {
        let na = va.get(i).unwrap_or(&0);
        let nb = vb.get(i).unwrap_or(&0);
        if na > nb {
            return true;
        }
        if na < nb {
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_serializes_expected_fields() {
        let m = SnapshotManifest {
            snapshot_id: "20240615T040000Z".into(),
            created_at: "2024-06-15T04:00:00Z".into(),
            schema_version: "1.11.0".into(),
            fleet_db_key: "backups/20240615T040000Z/fleet.db.zst".into(),
            attachments_manifest_key: Some(
                "backups/20240615T040000Z/attachments.manifest.json".into(),
            ),
            encryption: "none".into(),
            hoop_version: "0.1.0".into(),
            fleet_db_sha256: Some("abc123".into()),
            fleet_db_size: Some(4096),
        };
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("\"snapshot_id\""));
        assert!(json.contains("\"fleet_db_key\""));
        assert!(json.contains("\"encryption\""));
    }

    #[test]
    fn manifest_skips_optional_none_fields() {
        let m = SnapshotManifest {
            snapshot_id: "snap".into(),
            created_at: "2024-01-01T00:00:00Z".into(),
            schema_version: "1.0.0".into(),
            fleet_db_key: "fleet.db.zst".into(),
            attachments_manifest_key: None,
            encryption: "none".into(),
            hoop_version: "0.1.0".into(),
            fleet_db_sha256: None,
            fleet_db_size: None,
        };
        let json = serde_json::to_string(&m).unwrap();
        assert!(!json.contains("attachments_manifest_key"));
        assert!(!json.contains("fleet_db_sha256"));
    }

    #[test]
    fn manifest_roundtrips_through_json() {
        let m = SnapshotManifest {
            snapshot_id: "20240615T040000Z".into(),
            created_at: "2024-06-15T04:00:00Z".into(),
            schema_version: "1.11.0".into(),
            fleet_db_key: "backups/20240615T040000Z/fleet.db.zst.age".into(),
            attachments_manifest_key: None,
            encryption: "age".into(),
            hoop_version: "0.1.0".into(),
            fleet_db_sha256: Some("deadbeef".into()),
            fleet_db_size: Some(2048),
        };
        let json = serde_json::to_string(&m).unwrap();
        let parsed: SnapshotManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, m);
    }

    #[test]
    fn validate_accepts_current_version() {
        let m = SnapshotManifest {
            snapshot_id: "snap".into(),
            created_at: "2024-01-01T00:00:00Z".into(),
            schema_version: "1.11.0".into(),
            fleet_db_key: "fleet.db.zst".into(),
            attachments_manifest_key: None,
            encryption: "none".into(),
            hoop_version: "0.1.0".into(),
            fleet_db_sha256: None,
            fleet_db_size: None,
        };
        assert!(m.validate("1.11.0").is_ok());
        assert!(m.validate("2.0.0").is_ok());
    }

    #[test]
    fn validate_rejects_newer_version() {
        let m = SnapshotManifest {
            snapshot_id: "snap".into(),
            created_at: "2024-01-01T00:00:00Z".into(),
            schema_version: "2.0.0".into(),
            fleet_db_key: "fleet.db.zst".into(),
            attachments_manifest_key: None,
            encryption: "none".into(),
            hoop_version: "0.1.0".into(),
            fleet_db_sha256: None,
            fleet_db_size: None,
        };
        let err = m.validate("1.11.0").unwrap_err();
        assert!(err.to_string().contains("newer than"));
    }

    #[test]
    fn is_newer_version_cases() {
        assert!(is_newer_version("2.0.0", "1.11.0"));
        assert!(is_newer_version("1.12.0", "1.11.0"));
        assert!(is_newer_version("1.11.1", "1.11.0"));
        assert!(!is_newer_version("1.11.0", "1.11.0"));
        assert!(!is_newer_version("1.10.0", "1.11.0"));
        assert!(!is_newer_version("0.9.0", "1.0.0"));
    }
}
