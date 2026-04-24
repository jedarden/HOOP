//! `hoop restore --from s3://<bucket>/<prefix>/<snapshot-id>`
//!
//! Fetches a snapshot from S3-compatible storage, validates the manifest,
//! moves the existing `~/.hoop/` aside for rollback, restores fleet.db +
//! attachments + config, then runs schema migrations.

use anyhow::{bail, Context, Result};
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

type HmacSha256 = Hmac<Sha256>;

// ── Manifest types ──────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct SnapshotManifest {
    pub snapshot_id: String,
    pub timestamp: String,
    pub schema_version: String,
    pub hoop_version: Option<String>,
    pub files: std::collections::HashMap<String, FileEntry>,
}

#[derive(Debug, serde::Deserialize)]
pub struct FileEntry {
    pub sha256: String,
    pub size: u64,
}

// ── S3 URI parsing ──────────────────────────────────────────────────

#[derive(Debug)]
struct S3Locator {
    bucket: String,
    key: String,
}

fn parse_s3_uri(uri: &str) -> Result<S3Locator> {
    let stripped = uri.strip_prefix("s3://").context("URI must start with s3://")?;
    let (bucket, key) = stripped
        .split_once('/')
        .context("URI must be s3://<bucket>/<key>")?;
    Ok(S3Locator {
        bucket: bucket.to_string(),
        key: key.to_string(),
    })
}

// ── S3 configuration from environment ───────────────────────────────

#[derive(Debug)]
struct S3Config {
    endpoint: String,
    region: String,
    access_key: String,
    secret_key: String,
}

fn load_s3_config() -> Result<S3Config> {
    let endpoint = std::env::var("HOOP_BACKUP_ENDPOINT")
        .or_else(|_| std::env::var("AWS_ENDPOINT_URL"))
        .context(
            "Set HOOP_BACKUP_ENDPOINT or AWS_ENDPOINT_URL to your S3-compatible endpoint",
        )?;
    let region =
        std::env::var("AWS_REGION").or_else(|_| std::env::var("AWS_DEFAULT_REGION")).unwrap_or_else(|_| "us-east-1".into());
    let access_key = std::env::var("AWS_ACCESS_KEY_ID")
        .context("Set AWS_ACCESS_KEY_ID for S3 authentication")?;
    let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY")
        .context("Set AWS_SECRET_ACCESS_KEY for S3 authentication")?;

    Ok(S3Config {
        endpoint: endpoint.trim_end_matches('/').to_string(),
        region,
        access_key,
        secret_key,
    })
}

// ── AWS SigV4 (minimal GET-only signer) ─────────────────────────────

fn sign_request(
    config: &S3Config,
    method: &str,
    url: &reqwest::Url,
    bucket: &str,
    object_key: &str,
    now: &chrono::DateTime<Utc>,
) -> Result<String> {
    let date_stamp = now.format("%Y%m%d").to_string();
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let service = "s3";

    let host = url.host_str().unwrap_or("");
    let canonical_uri = url.path();
    let canonical_querystring = url.query().unwrap_or("");

    let canonical_headers = format!("host:{host}\nx-amz-content-sha256:UNSIGNED-PAYLOAD\nx-amz-date:{amz_date}\n");
    let signed_headers = "host;x-amz-content-sha256;x-amz-date";

    let canonical_request = format!(
        "{method}\n{canonical_uri}\n{canonical_querystring}\n{canonical_headers}\n{signed_headers}\nUNSIGNED-PAYLOAD"
    );

    let credential_scope = format!("{date_stamp}/{}/{service}/aws4_request", config.region);

    let mut hasher = Sha256::new();
    hasher.update(canonical_request.as_bytes());
    let canonical_request_hash = hex::encode(hasher.finalize());

    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{canonical_request_hash}"
    );

    // Derive signing key
    let mut mac = HmacSha256::new_from_slice(format!("AWS4{}", config.secret_key).as_bytes())
        .expect("HMAC key length is valid");
    mac.update(date_stamp.as_bytes());
    let k_date = mac.finalize().into_bytes();

    let mut mac = HmacSha256::new_from_slice(&k_date).expect("HMAC key length is valid");
    mac.update(config.region.as_bytes());
    let k_region = mac.finalize().into_bytes();

    let mut mac = HmacSha256::new_from_slice(&k_region).expect("HMAC key length is valid");
    mac.update(service.as_bytes());
    let k_service = mac.finalize().into_bytes();

    let mut mac = HmacSha256::new_from_slice(&k_service).expect("HMAC key length is valid");
    mac.update(b"aws4_request");
    let signing_key = mac.finalize().into_bytes();

    let mut mac = HmacSha256::new_from_slice(&signing_key).expect("HMAC key length is valid");
    mac.update(string_to_sign.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    Ok(format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        config.access_key, credential_scope, signed_headers, signature
    ))
}

// ── S3 download helpers ─────────────────────────────────────────────

fn build_s3_url(config: &S3Config, bucket: &str, key: &str) -> Result<reqwest::Url> {
    // Path-style: https://<endpoint>/<bucket>/<key>
    let url_str = format!("{}/{}/{}", config.endpoint, bucket, key);
    Ok(url_str.parse()?)
}

async fn s3_get(config: &S3Config, bucket: &str, key: &str) -> Result<bytes::Bytes> {
    let url = build_s3_url(config, bucket, key)?;
    let now = Utc::now();
    let auth = sign_request(config, "GET", &url, bucket, key, &now)?;

    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();

    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("Authorization", auth)
        .header("x-amz-date", amz_date)
        .header("x-amz-content-sha256", "UNSIGNED-PAYLOAD")
        .send()
        .await
        .with_context(|| format!("Failed to download s3://{}/{}", bucket, key))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        bail!(
            "S3 GET s3://{}/{} returned {}: {}",
            bucket,
            key,
            status,
            body.trim()
        );
    }

    let bytes = resp.bytes().await?;
    Ok(bytes)
}

// ── File verification ────────────────────────────────────────────────

fn verify_sha256(data: &[u8], expected: &str) -> Result<()> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let actual = hex::encode(hasher.finalize());
    if actual != expected {
        bail!(
            "SHA-256 mismatch: expected {}, got {}",
            expected,
            actual
        );
    }
    Ok(())
}

// ── Daemon check ────────────────────────────────────────────────────

fn is_daemon_running() -> bool {
    // Check the control socket
    let mut hoop_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    hoop_dir.push(".hoop");
    let sock = hoop_dir.join("control.sock");
    if sock.exists() {
        // Try to connect — if it connects, daemon is running
        if std::os::unix::net::UnixStream::connect(&sock).is_ok() {
            return true;
        }
    }

    // Also try the TCP port as a fallback
    if std::net::TcpStream::connect("127.0.0.1:3000").is_ok() {
        return true;
    }

    false
}

// ── Rollback directory ───────────────────────────────────────────────

fn hoop_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hoop")
}

fn rollback_dir() -> PathBuf {
    let ts = Utc::now().format("%Y%m%dT%H%M%SZ");
    hoop_dir().with_extension(format!("rollback.{}", ts))
}

fn move_aside_for_rollback() -> Result<PathBuf> {
    let src = hoop_dir();
    let dst = rollback_dir();

    if src.exists() {
        std::fs::rename(&src, &dst)
            .with_context(|| format!("Failed to move {} -> {}", src.display(), dst.display()))?;
    }

    Ok(dst)
}

fn rollback(backup_dir: &Path) -> Result<()> {
    let target = hoop_dir();
    // Remove partial restore if it exists
    if target.exists() {
        std::fs::remove_dir_all(&target)
            .with_context(|| format!("Failed to remove partial restore at {}", target.display()))?;
    }
    // Move backup back
    if backup_dir.exists() {
        std::fs::rename(backup_dir, &target)
            .with_context(|| format!("Failed to restore rollback {} -> {}", backup_dir.display(), target.display()))?;
    }
    Ok(())
}

// ── Main restore logic ──────────────────────────────────────────────

pub async fn run_restore(from_uri: &str) -> Result<()> {
    // 1. Precondition: daemon must not be running
    if is_daemon_running() {
        bail!("HOOP daemon is running. Stop it before restoring:\n  systemctl --user stop hoop");
    }

    // 2. Parse S3 URI and load config
    let locator = parse_s3_uri(from_uri)?;
    let s3_config = load_s3_config()?;

    println!("Fetching manifest from s3://{}/{} ...", locator.bucket, locator.key);

    // 3. Download and parse manifest
    let manifest_bytes = s3_get(&s3_config, &locator.bucket, &format!("{}/manifest.json", locator.key)).await?;
    let manifest: SnapshotManifest = serde_json::from_slice(&manifest_bytes)
        .context("Failed to parse manifest.json")?;

    println!(
        "Snapshot: {} (schema {}, {})",
        manifest.snapshot_id,
        manifest.schema_version,
        manifest.timestamp
    );

    // 4. Validate schema version — refuse newer-than-current (§20)
    let current = hoop_daemon::fleet::SCHEMA_VERSION;
    if is_newer(&manifest.schema_version, current) {
        bail!(
            "Snapshot schema version {} is newer than this binary's {}. \
             Upgrade HOOP before restoring this snapshot.",
            manifest.schema_version,
            current
        );
    }

    // Verify manifest has fleet.db
    let db_entry = manifest.files.get("fleet.db").context(
        "Manifest does not contain fleet.db — snapshot is incomplete",
    )?;

    // 5. Move existing ~/.hoop/ aside (destructive action follows)
    let backup_dir = move_aside_for_rollback()
        .context("Failed to move existing ~/.hoop/ aside for rollback")?;

    println!(
        "Moved existing state to {}",
        backup_dir.display()
    );

    // Helper to rollback on failure
    let restore_with_rollback = |result: Result<()>| -> Result<()> {
        if let Err(e) = &result {
            eprintln!("Restore failed: {e}");
            eprintln!("Rolling back...");
            if let Err(re) = rollback(&backup_dir) {
                eprintln!("Rollback also failed: {re}");
                eprintln!("Manual recovery: rename {} to ~/.hoop/", backup_dir.display());
            } else {
                eprintln!("Rollback complete — original state restored.");
            }
        }
        result
    };

    // 6. Create fresh ~/.hoop/
    let hoop = hoop_dir();
    std::fs::create_dir_all(&hoop)
        .with_context(|| format!("Failed to create {}", hoop.display()))?;

    // 7. Download and restore fleet.db
    let result = async {
        println!("Downloading fleet.db ...");
        let db_data = s3_get(&s3_config, &locator.bucket, &format!("{}/fleet.db", locator.key)).await?;
        verify_sha256(&db_data, &db_entry.sha256)
            .context("fleet.db integrity check failed")?;
        let db_path = hoop.join("fleet.db");
        std::fs::write(&db_path, &db_data)
            .with_context(|| format!("Failed to write {}", db_path.display()))?;
        println!("fleet.db restored ({} bytes)", db_data.len());

        // 8. Download and restore attachments (if present)
        let attachments_dir = hoop.join("attachments");
        for (name, entry) in &manifest.files {
            if name.starts_with("attachments/") {
                let rel_path = name.strip_prefix("attachments/").unwrap();
                let file_path = attachments_dir.join(rel_path);
                if let Some(parent) = file_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                println!("Downloading {} ...", name);
                let data = s3_get(&s3_config, &locator.bucket, &format!("{}/{}", locator.key, name)).await?;
                verify_sha256(&data, &entry.sha256)
                    .with_context(|| format!("{} integrity check failed", name))?;
                std::fs::write(&file_path, &data)?;
            }
        }

        // 9. Download and restore config (if present)
        if let Some(entry) = manifest.files.get("config.json") {
            println!("Downloading config.json ...");
            let config_data = s3_get(&s3_config, &locator.bucket, &format!("{}/config.json", locator.key)).await?;
            verify_sha256(&config_data, &entry.sha256)
                .context("config.json integrity check failed")?;
            std::fs::write(hoop.join("config.json"), &config_data)?;
        }

        // 10. Restore projects.yaml from backup if the new snapshot doesn't include one
        let projects_backup = backup_dir.join("projects.yaml");
        if projects_backup.exists() && !manifest.files.contains_key("projects.yaml") {
            let dst = hoop.join("projects.yaml");
            std::fs::copy(&projects_backup, &dst).with_context(|| {
                format!(
                    "Failed to copy projects.yaml from backup {}",
                    projects_backup.display()
                )
            })?;
            println!("Preserved projects.yaml from previous state");
        }

        // 11. Run schema migrations on restored fleet.db
        println!("Running schema migrations ...");
        let db_path = hoop.join("fleet.db");
        let pre_version = hoop_daemon::fleet::restore_and_migrate(&db_path)
            .context("Schema migration failed")?;
        if pre_version != current {
            println!("Migrated schema {} -> {}", pre_version, current);
        }

        Ok(())
    }
    .await;

    restore_with_rollback(result)?;

    println!("Restore complete.");
    println!("Start the daemon with: systemctl --user start hoop");

    Ok(())
}

/// Compare two semver-like strings; true if `a` > `b`.
fn is_newer(a: &str, b: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|p| p.parse().ok())
            .collect()
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
    fn test_parse_s3_uri() {
        let loc = parse_s3_uri("s3://mybucket/backups/snap-001").unwrap();
        assert_eq!(loc.bucket, "mybucket");
        assert_eq!(loc.key, "backups/snap-001");
    }

    #[test]
    fn test_parse_s3_uri_no_key() {
        assert!(parse_s3_uri("s3://mybucket").is_err());
    }

    #[test]
    fn test_parse_s3_uri_no_scheme() {
        assert!(parse_s3_uri("https://example.com/bucket/key").is_err());
    }

    #[test]
    fn test_is_newer() {
        assert!(is_newer("2.0.0", "1.11.0"));
        assert!(is_newer("1.12.0", "1.11.0"));
        assert!(is_newer("1.11.1", "1.11.0"));
        assert!(!is_newer("1.11.0", "1.11.0"));
        assert!(!is_newer("1.10.0", "1.11.0"));
        assert!(!is_newer("0.9.0", "1.0.0"));
    }

    #[test]
    fn test_verify_sha256() {
        let data = b"hello world";
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hex::encode(hasher.finalize());
        assert!(verify_sha256(data, &hash).is_ok());
        assert!(verify_sha256(data, "0000").is_err());
    }

    #[test]
    fn test_manifest_parsing() {
        let json = r#"{
            "snapshot_id": "snap-001",
            "timestamp": "2024-01-15T04:00:00Z",
            "schema_version": "1.11.0",
            "hoop_version": "0.1.0",
            "files": {
                "fleet.db": { "sha256": "abc123", "size": 4096 },
                "config.json": { "sha256": "def456", "size": 128 },
                "attachments/note.wav": { "sha256": "ghi789", "size": 8192 }
            }
        }"#;
        let m: SnapshotManifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.snapshot_id, "snap-001");
        assert_eq!(m.files.len(), 3);
        assert!(m.files.contains_key("fleet.db"));
        assert_eq!(m.files["fleet.db"].sha256, "abc123");
    }
}
