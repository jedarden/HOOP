//! Per-account configuration for OpenCode ZAI proxy limits.
//!
//! This module loads and parses accounts.yaml which contains
//! per-account prompt limits for OpenCode/ZAI proxy usage.
//!
//! Format:
//! ```yaml
//! accounts:
//!   opencode-default:
//!     adapter: opencode
//!     limits:
//!       opencode:
//!         prompts_per_5h: 1600
//!         prompts_per_7d: 8000
//!   opencode-work:
//!     adapter: opencode
//!     limits:
//!       opencode:
//!         prompts_per_5h: 3200
//!         prompts_per_7d: 16000
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// OpenCode prompt limits for a single account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeLimits {
    /// Prompt budget per 5-hour window
    pub prompts_per_5h: u64,
    /// Prompt budget per 7-day window
    pub prompts_per_7d: u64,
}

/// Per-account configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    /// Adapter name (e.g., "opencode", "claude", "gemini")
    pub adapter: String,
    /// Rate limit configuration (adapter-specific)
    #[serde(default)]
    pub limits: AccountLimits,
}

/// Rate limit configuration for an account.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountLimits {
    /// OpenCode prompt limits (applicable when adapter == "opencode")
    #[serde(default)]
    pub opencode: Option<OpenCodeLimits>,
}

/// Top-level accounts.yaml structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountsConfig {
    /// Per-account configurations keyed by account ID
    #[serde(default)]
    pub accounts: HashMap<String, AccountConfig>,
}

impl Default for AccountsConfig {
    fn default() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }
}

impl AccountsConfig {
    /// Load accounts configuration from a YAML file.
    ///
    /// Returns an empty config if the file doesn't exist.
    pub fn load_from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            debug!("Accounts config file not found at {}, using empty config", path.display());
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse accounts.yaml: {}", e))?;

        info!(
            "Loaded accounts config for {} accounts from {}",
            config.accounts.len(),
            path.display()
        );

        Ok(config)
    }

    /// Get OpenCode limits for a specific account.
    ///
    /// Returns None if the account is not configured or doesn't have OpenCode limits.
    pub fn get_opencode_limits(&self, account_id: &str) -> Option<&OpenCodeLimits> {
        self.accounts
            .get(account_id)
            .and_then(|account| {
                if account.adapter == "opencode" {
                    account.limits.opencode.as_ref()
                } else {
                    None
                }
            })
    }

    /// Get default OpenCode limits for accounts not in the config.
    ///
    /// Returns the ZAI proxy Max tier defaults: 1600/5h, 8000/7d.
    pub fn default_opencode_limits() -> OpenCodeLimits {
        OpenCodeLimits {
            prompts_per_5h: 1600,
            prompts_per_7d: 8000,
        }
    }

    /// Get OpenCode limits for an account, or default if not configured.
    pub fn get_opencode_limits_or_default(&self, account_id: &str) -> OpenCodeLimits {
        self.get_opencode_limits(account_id)
            .cloned()
            .unwrap_or_else(Self::default_opencode_limits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_accounts_config() {
        let yaml = r#"
accounts:
  opencode-default:
    adapter: opencode
    limits:
      opencode:
        prompts_per_5h: 1600
        prompts_per_7d: 8000
  opencode-work:
    adapter: opencode
    limits:
      opencode:
        prompts_per_5h: 3200
        prompts_per_7d: 16000
"#;

        let config: AccountsConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.accounts.len(), 2);

        let default_limits = config.get_opencode_limits("opencode-default").unwrap();
        assert_eq!(default_limits.prompts_per_5h, 1600);
        assert_eq!(default_limits.prompts_per_7d, 8000);

        let work_limits = config.get_opencode_limits("opencode-work").unwrap();
        assert_eq!(work_limits.prompts_per_5h, 3200);
        assert_eq!(work_limits.prompts_per_7d, 16000);
    }

    #[test]
    fn test_empty_config_returns_none() {
        let config = AccountsConfig::default();
        assert!(config.get_opencode_limits("opencode-default").is_none());
    }

    #[test]
    fn test_get_opencode_limits_or_default() {
        let config = AccountsConfig::default();
        let limits = config.get_opencode_limits_or_default("opencode-default");
        assert_eq!(limits.prompts_per_5h, 1600);
        assert_eq!(limits.prompts_per_7d, 8000);
    }

    #[test]
    fn test_non_opencode_account_returns_none() {
        let yaml = r#"
accounts:
  claude-default:
    adapter: claude
    limits: {}
"#;

        let config: AccountsConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.get_opencode_limits("claude-default").is_none());
    }

    #[test]
    fn test_default_opencode_limits() {
        let limits = AccountsConfig::default_opencode_limits();
        assert_eq!(limits.prompts_per_5h, 1600);
        assert_eq!(limits.prompts_per_7d, 8000);
    }
}
