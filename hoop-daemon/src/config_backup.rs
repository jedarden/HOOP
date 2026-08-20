//! Config file backup: backs up config.yml and projects.yaml on every change.
//!
//! Config files are backed up to S3 on every successful config reload and
//! daily as part of the scheduled backup. Each backup includes a SHA-256
//! hash for integrity verification.
//!
//! Plan reference: §15.3

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tracing::info;

/// Config file backup metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigBackup {
    /// config.yml content (SHA-256 hashed for storage).
    pub config_yml_hash: String,
    /// config.yml size in bytes.
    pub config_yml_size: u64,
    /// projects.yaml content (SHA-256 hashed for storage).
    pub projects_yaml_hash: String,
    /// projects.yaml size in bytes.
    pub projects_yaml_size: u64,
    /// ISO 8601 timestamp of when the config was backed up.
    pub backed_up_at: String,
}

impl ConfigBackup {
    /// Create a new config backup from the current config files.
    pub fn from_hoop_dir() -> Result<Self> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let hoop_dir = home.join(".hoop");

        let config_path = hoop_dir.join("config.yml");
        let projects_path = hoop_dir.join("projects.yaml");

        let (config_yml_hash, config_yml_size) = if config_path.exists() {
            let data = std::fs::read(&config_path)
                .with_context(|| format!("read config.yml from {}", config_path.display()))?;
            let hash = hex::encode(Sha256::digest(&data));
            let size = data.len() as u64;
            (hash, size)
        } else {
            (String::new(), 0)
        };

        let (projects_yaml_hash, projects_yaml_size) = if projects_path.exists() {
            let data = std::fs::read(&projects_path)
                .with_context(|| format!("read projects.yaml from {}", projects_path.display()))?;
            let hash = hex::encode(Sha256::digest(&data));
            let size = data.len() as u64;
            (hash, size)
        } else {
            (String::new(), 0)
        };

        Ok(Self {
            config_yml_hash,
            config_yml_size,
            projects_yaml_hash,
            projects_yaml_size,
            backed_up_at: Utc::now().to_rfc3339(),
        })
    }

    /// Get the path to config.yml.
    pub fn config_yml_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".hoop").join("config.yml")
    }

    /// Get the path to projects.yaml.
    pub fn projects_yaml_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".hoop").join("projects.yaml")
    }
}

/// Upload config files to S3 as part of a snapshot.
pub async fn upload_config_to_snapshot(
    s3_client: &crate::backup_pipeline::BackupPipeline,
    snapshot_id: &str,
) -> Result<Option<ConfigBackup>> {
    let config_backup = ConfigBackup::from_hoop_dir()?;

    // Skip if both files are empty (don't exist)
    if config_backup.config_yml_size == 0 && config_backup.projects_yaml_size == 0 {
        info!("No config files to backup");
        return Ok(None);
    }

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let hoop_dir = home.join(".hoop");

    // Upload config.yml if it exists
    if config_backup.config_yml_size > 0 {
        let config_path = hoop_dir.join("config.yml");
        let data = std::fs::read(&config_path)?;
        let compressed = zstd::encode_all(&data[..], 3)?;

        let s3_key = format!(
            "{}/{}/config.yml.zst",
            s3_client.snapshot_prefix(snapshot_id),
            snapshot_id
        );

        s3_client
            .upload_with_retry_from_bytes(&compressed, &s3_key)
            .await?;

        info!("Uploaded config.yml to {}", s3_key);
    }

    // Upload projects.yaml if it exists
    if config_backup.projects_yaml_size > 0 {
        let projects_path = hoop_dir.join("projects.yaml");
        let data = std::fs::read(&projects_path)?;
        let compressed = zstd::encode_all(&data[..], 3)?;

        let s3_key = format!(
            "{}/{}/projects.yaml.zst",
            s3_client.snapshot_prefix(snapshot_id),
            snapshot_id
        );

        s3_client
            .upload_with_retry_from_bytes(&compressed, &s3_key)
            .await?;

        info!("Uploaded projects.yaml to {}", s3_key);
    }

    Ok(Some(config_backup))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_backup_from_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        // Temporarily override the home directory
        let original_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", tmp.path());

        let backup = ConfigBackup::from_hoop_dir().unwrap();
        assert_eq!(backup.config_yml_size, 0);
        assert_eq!(backup.projects_yaml_size, 0);
        assert!(backup.config_yml_hash.is_empty());
        assert!(backup.projects_yaml_hash.is_empty());

        // Restore original HOME
        if let Some(original) = original_home {
            std::env::set_var("HOME", original);
        } else {
            std::env::remove_var("HOME");
        }
    }

    #[test]
    fn test_config_backup_with_files() {
        let tmp = tempfile::tempdir().unwrap();
        let hoop_dir = tmp.path().join(".hoop");
        std::fs::create_dir_all(&hoop_dir).unwrap();

        // Create config.yml
        let config_path = hoop_dir.join("config.yml");
        std::fs::write(
            &config_path,
            b"schema_version: \"1.0.0\"\nagent:\n  adapter: claude\n",
        )
        .unwrap();

        // Create projects.yaml
        let projects_path = hoop_dir.join("projects.yaml");
        std::fs::write(
            &projects_path,
            b"projects:\n  - name: test\n    path: /test\n",
        )
        .unwrap();

        let original_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", tmp.path());

        let backup = ConfigBackup::from_hoop_dir().unwrap();
        assert_eq!(backup.config_yml_size, 52);
        assert_eq!(backup.projects_yaml_size, 40);
        assert!(!backup.config_yml_hash.is_empty());
        assert!(!backup.projects_yaml_hash.is_empty());

        if let Some(original) = original_home {
            std::env::set_var("HOME", original);
        } else {
            std::env::remove_var("HOME");
        }
    }

    #[test]
    fn test_config_backup_hash_is_deterministic() {
        let tmp = tempfile::tempdir().unwrap();
        let hoop_dir = tmp.path().join(".hoop");
        std::fs::create_dir_all(&hoop_dir).unwrap();

        let config_path = hoop_dir.join("config.yml");
        let content = b"test content";
        std::fs::write(&config_path, content).unwrap();

        let original_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", tmp.path());

        let backup1 = ConfigBackup::from_hoop_dir().unwrap();
        let backup2 = ConfigBackup::from_hoop_dir().unwrap();

        assert_eq!(backup1.config_yml_hash, backup2.config_yml_hash);
        assert_eq!(backup1.config_yml_size, backup2.config_yml_size);

        // Verify the hash is correct SHA-256
        let mut hasher = Sha256::new();
        hasher.update(content);
        let expected_hash = hex::encode(hasher.finalize());
        assert_eq!(backup1.config_yml_hash, expected_hash);

        if let Some(original) = original_home {
            std::env::set_var("HOME", original);
        } else {
            std::env::remove_var("HOME");
        }
    }
}
