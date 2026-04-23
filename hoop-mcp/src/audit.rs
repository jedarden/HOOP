//! Audit logging for MCP tool calls
//!
//! Every tool call is recorded in ~/.hoop/mcp_audit.log with:
//! - Timestamp
//! - Tool name
//! - Args hash (SHA-256 for integrity)
//! - Actor (MCP client identifier)

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub actor: String,
    pub tool_name: String,
    pub args: Option<Value>,
    pub args_hash: String,
    #[serde(flatten)]
    pub result: AuditResultWrapper,
}

/// Wrapper for serialization that flattens the result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AuditResultWrapper {
    Success { result: String },
    Failure { result: String, error: String },
}

#[derive(Debug, Clone)]
pub enum AuditResult {
    Success,
    Failure(String),
}

/// Audit log writer
pub struct AuditLog {
    file: Mutex<File>,
}

impl AuditLog {
    /// Open the audit log at ~/.hoop/mcp_audit.log
    pub fn open() -> Result<Self> {
        let mut path = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
        path.push(".hoop");

        std::fs::create_dir_all(&path)?;
        path.push("mcp_audit.log");

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        Ok(Self {
            file: Mutex::new(file),
        })
    }

    /// Record a tool call
    pub fn record(&self, actor: &str, tool_name: &str, args: Option<&Value>, result: &AuditResult) -> Result<()> {
        let timestamp = Utc::now().to_rfc3339();
        let args_hash = compute_args_hash(args);

        let result_wrapper = match result {
            AuditResult::Success => AuditResultWrapper::Success {
                result: "success".to_string(),
            },
            AuditResult::Failure(msg) => AuditResultWrapper::Failure {
                result: "failure".to_string(),
                error: msg.clone(),
            },
        };

        let entry = AuditEntry {
            timestamp,
            actor: actor.to_string(),
            tool_name: tool_name.to_string(),
            args: args.cloned(),
            args_hash,
            result: result_wrapper,
        };

        let json = serde_json::to_string(&entry)?;
        let line = format!("{}\n", json);

        let mut file = self.file.lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        file.write_all(line.as_bytes())?;
        file.flush()?;

        Ok(())
    }
}

/// Compute SHA-256 hash of args for integrity verification
fn compute_args_hash(args: Option<&Value>) -> String {
    let json = match args {
        Some(v) => serde_json::to_string(v).unwrap_or_default(),
        None => "null".to_string(),
    };

    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    hex::encode(hasher.finalize())
}

impl serde::Serialize for AuditResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            AuditResult::Success => {
                serializer.serialize_str("success")
            }
            AuditResult::Failure(msg) => {
                serializer.serialize_str(&format!("failure: {}", msg))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_args_hash() {
        let args = serde_json::json!({
            "project": "test",
            "limit": 10
        });

        let hash1 = compute_args_hash(Some(&args));
        let hash2 = compute_args_hash(Some(&args));

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 hex is 64 chars

        // Different args produce different hash
        let args2 = serde_json::json!({
            "project": "other",
            "limit": 10
        });
        let hash3 = compute_args_hash(Some(&args2));
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_compute_args_hash_null() {
        let hash = compute_args_hash(None);
        assert_eq!(hash.len(), 64);
    }
}
