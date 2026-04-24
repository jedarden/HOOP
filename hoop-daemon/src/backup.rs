//! Backup configuration parser and credential resolver.
//!
//! Reads the `backup:` section from `~/.hoop/config.yml` and resolves
//! S3 credentials from environment variables. Credentials are **never**
//! stored in the YAML config or written to the audit log.
//!
//! Plan reference: §15.2

use serde::Deserialize;
use std::path::PathBuf;
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// Config structs (from config.yml)
// ---------------------------------------------------------------------------

/// Parsed `backup:` section from `~/.hoop/config.yml`.
#[derive(Debug, Clone, Deserialize)]
pub struct BackupFileConfig {
    /// S3-compatible endpoint URL (e.g. `https://s3.us-east-1.amazonaws.com`).
    pub endpoint: String,
    /// S3 bucket name.
    pub bucket: String,
    /// Key prefix for all backup objects.
    pub prefix: String,
    /// Cron schedule for automatic backups (default: `0 4 * * *`).
    #[serde(default = "default_schedule")]
    pub schedule: String,
    /// Days to retain backups before pruning (default: 30).
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
    /// Encrypt backups with age (default: false).
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
// Credentials (environment variables only)
// ---------------------------------------------------------------------------

/// S3 credentials resolved from environment variables.
///
/// These are **never** read from config.yml and **never** written to logs.
#[derive(Clone)]
pub struct BackupCredentials {
    /// From `HOOP_BACKUP_ACCESS_KEY_ID`.
    pub access_key_id: String,
    /// From `HOOP_BACKUP_SECRET_ACCESS_KEY`.
    pub secret_access_key: String,
    /// From `HOOP_BACKUP_AGE_KEY` (required only when encryption is enabled).
    pub age_key: Option<String>,
}

impl std::fmt::Debug for BackupCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackupCredentials")
            .field("access_key_id", &format!("{}…{}", &self.access_key_id[..self.access_key_id.len().min(4)], &self.access_key_id[self.access_key_id.len().saturating_sub(4)..]))
            .field("secret_access_key", &"[REDACTED]")
            .field("age_key", &self.age_key.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

impl BackupCredentials {
    /// Read credentials from environment variables.
    ///
    /// Returns `None` (with a warning log) if either required variable is missing.
    pub fn from_env(encryption: bool) -> Option<Self> {
        let access_key_id = match std::env::var("HOOP_BACKUP_ACCESS_KEY_ID") {
            Ok(v) => v,
            Err(_) => {
                warn!(
                    "HOOP_BACKUP_ACCESS_KEY_ID not set — backups disabled. \
                     Set this env var to enable scheduled backups."
                );
                return None;
            }
        };

        let secret_access_key = match std::env::var("HOOP_BACKUP_SECRET_ACCESS_KEY") {
            Ok(v) => v,
            Err(_) => {
                warn!(
                    "HOOP_BACKUP_SECRET_ACCESS_KEY not set — backups disabled. \
                     Set this env var to enable scheduled backups."
                );
                return None;
            }
        };

        let age_key = if encryption {
            match std::env::var("HOOP_BACKUP_AGE_KEY") {
                Ok(v) => Some(v),
                Err(_) => {
                    warn!(
                        "encryption is enabled but HOOP_BACKUP_AGE_KEY not set — \
                         backups disabled. Set this env var to enable encrypted backups."
                    );
                    return None;
                }
            }
        } else {
            std::env::var("HOOP_BACKUP_AGE_KEY").ok()
        };

        Some(Self {
            access_key_id,
            secret_access_key,
            age_key,
        })
    }
}

// ---------------------------------------------------------------------------
// Resolved state
// ---------------------------------------------------------------------------

/// Runtime state of the backup subsystem after config + credential resolution.
#[derive(Debug, Clone)]
pub enum BackupState {
    /// Backup section missing or empty in config.yml — feature not configured.
    NotConfigured,
    /// Credentials not available — daemon started but backups are disabled.
    Disabled {
        config: BackupFileConfig,
        reason: String,
    },
    /// Fully configured and ready to run scheduled backups.
    Ready {
        config: BackupFileConfig,
        credentials: BackupCredentials,
    },
}

impl BackupState {
    pub fn is_ready(&self) -> bool {
        matches!(self, BackupState::Ready { .. })
    }
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

/// Config path for `~/.hoop/config.yml`.
fn config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".hoop").join("config.yml")
}

/// Load and validate the backup config section, resolving credentials from env vars.
///
/// Follows the same pattern as `agent_adapter::load_adapter_config()`:
/// - Missing file or missing `backup:` section → `NotConfigured`
/// - Parse error → `NotConfigured` with warning
/// - Missing env vars → `Disabled` with clear log message
/// - Everything present → `Ready`
pub fn load_backup_config() -> BackupState {
    let path = config_path();
    if !path.exists() {
        return BackupState::NotConfigured;
    }

    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to read config.yml: {} — backup not configured", e);
            return BackupState::NotConfigured;
        }
    };

    let root: serde_yaml::Value = match serde_yaml::from_str(&contents) {
        Ok(v) => v,
        Err(e) => {
            warn!("Failed to parse config.yml: {} — backup not configured", e);
            return BackupState::NotConfigured;
        }
    };

    let backup_section = match root.get("backup") {
        Some(v) => v,
        None => return BackupState::NotConfigured,
    };

    let config: BackupFileConfig = match serde_json::from_value(
        serde_json::to_value(backup_section).unwrap_or_default(),
    ) {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to parse backup config section: {} — backup not configured", e);
            return BackupState::NotConfigured;
        }
    };

    // Validate cron schedule is parseable
    if let Err(e) = validate_cron(&config.schedule) {
        let reason = format!("invalid cron schedule '{}': {}", config.schedule, e);
        warn!("Backup disabled: {}", reason);
        return BackupState::Disabled {
            config,
            reason,
        };
    }

    // Validate endpoint looks like a URL
    if !config.endpoint.starts_with("http://") && !config.endpoint.starts_with("https://") {
        let reason = format!("endpoint must start with http:// or https:// (got '{}')", config.endpoint);
        warn!("Backup disabled: {}", reason);
        return BackupState::Disabled {
            config,
            reason,
        };
    }

    // Resolve credentials from env vars
    let credentials = match BackupCredentials::from_env(config.encryption) {
        Some(c) => c,
        None => {
            return BackupState::Disabled {
                config,
                reason: "missing required environment variables (HOOP_BACKUP_ACCESS_KEY_ID, HOOP_BACKUP_SECRET_ACCESS_KEY)".to_string(),
            };
        }
    };

    info!(
        "Backup configured: endpoint={}, bucket={}, prefix={}, schedule={}, retention={}d, encryption={}",
        config.endpoint,
        config.bucket,
        config.prefix,
        config.schedule,
        config.retention_days,
        config.encryption,
    );

    BackupState::Ready { config, credentials }
}

/// Basic cron field count validation (5-field cron).
fn validate_cron(expr: &str) -> Result<(), String> {
    let fields: Vec<&str> = expr.split_whitespace().collect();
    if fields.len() != 5 {
        return Err(format!(
            "expected 5 fields (min hour dom mon dow), got {}",
            fields.len()
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cron_validation_accepts_standard() {
        assert!(validate_cron("0 4 * * *").is_ok());
        assert!(validate_cron("*/15 * * * *").is_ok());
        assert!(validate_cron("0 0 1 1 *").is_ok());
    }

    #[test]
    fn cron_validation_rejects_wrong_field_count() {
        assert!(validate_cron("0 4 * *").is_err());
        assert!(validate_cron("0 4 * * * *").is_err());
        assert!(validate_cron("").is_err());
        assert!(validate_cron("garbage").is_err());
    }

    #[test]
    fn endpoint_validation_rejects_non_url() {
        assert!(validate_endpoint("s3.amazonaws.com").is_err());
        assert!(validate_endpoint("ftp://bad").is_err());
    }

    fn validate_endpoint(endpoint: &str) -> Result<(), String> {
        if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
            return Err(format!(
                "endpoint must start with http:// or https:// (got '{}')",
                endpoint
            ));
        }
        Ok(())
    }

    #[test]
    fn file_config_deserializes_minimal() {
        let yaml = serde_yaml::from_str::<serde_yaml::Value>(
            "endpoint: https://s3.example.com\nbucket: my-bucket\nprefix: backups/",
        )
        .unwrap();
        let config: BackupFileConfig = serde_json::from_value(
            serde_json::to_value(yaml).unwrap(),
        )
        .unwrap();
        assert_eq!(config.endpoint, "https://s3.example.com");
        assert_eq!(config.bucket, "my-bucket");
        assert_eq!(config.prefix, "backups/");
        assert_eq!(config.schedule, "0 4 * * *"); // default
        assert_eq!(config.retention_days, 30); // default
        assert!(!config.encryption); // default
    }

    #[test]
    fn file_config_deserializes_full() {
        let yaml = serde_yaml::from_str::<serde_yaml::Value>(
            "endpoint: https://s3.example.com\nbucket: my-bucket\nprefix: backups/\nschedule: '*/30 * * * *'\nretention_days: 14\nencryption: true",
        )
        .unwrap();
        let config: BackupFileConfig = serde_json::from_value(
            serde_json::to_value(yaml).unwrap(),
        )
        .unwrap();
        assert_eq!(config.schedule, "*/30 * * * *");
        assert_eq!(config.retention_days, 14);
        assert!(config.encryption);
    }

    #[test]
    fn backup_state_is_ready_only_when_ready() {
        assert!(BackupState::Ready {
            config: BackupFileConfig {
                endpoint: "https://s3.example.com".into(),
                bucket: "b".into(),
                prefix: "p".into(),
                schedule: default_schedule(),
                retention_days: 30,
                encryption: false,
            },
            credentials: BackupCredentials {
                access_key_id: "key".into(),
                secret_access_key: "secret".into(),
                age_key: None,
            },
        }
        .is_ready());

        assert!(!BackupState::NotConfigured.is_ready());
        assert!(!BackupState::Disabled {
            config: BackupFileConfig {
                endpoint: "https://s3.example.com".into(),
                bucket: "b".into(),
                prefix: "p".into(),
                schedule: default_schedule(),
                retention_days: 30,
                encryption: false,
            },
            reason: "test".into(),
        }
        .is_ready());
    }

    #[test]
    fn credentials_from_env_missing_keys() {
        // Clear env vars to ensure they're not set
        std::env::remove_var("HOOP_BACKUP_ACCESS_KEY_ID");
        std::env::remove_var("HOOP_BACKUP_SECRET_ACCESS_KEY");
        std::env::remove_var("HOOP_BACKUP_AGE_KEY");
        assert!(BackupCredentials::from_env(false).is_none());
    }

    #[test]
    fn credentials_debug_redacts_secrets() {
        let creds = BackupCredentials {
            access_key_id: "AKIAIOSFODNN7EXAMPLE".into(),
            secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".into(),
            age_key: Some("AGE-SECRET-KEY-1XYZ".into()),
        };
        let debug = format!("{:?}", creds);
        assert!(!debug.contains("wJalrXUtnFEMI"), "secret_access_key leaked in Debug");
        assert!(!debug.contains("AGE-SECRET-KEY"), "age_key leaked in Debug");
        assert!(debug.contains("[REDACTED]"), "expected [REDACTED] placeholder");
        assert!(debug.contains("AKIA"), "access_key_id prefix should be visible");
    }
}
