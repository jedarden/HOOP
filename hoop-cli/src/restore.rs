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

/// Remove all `~/.hoop.rollback.*` directories left from previous or current restore.
/// Called after a successful restore so the daemon doesn't refuse to start
/// (audit interprets leftover rollback dirs as an interrupted restore).
fn cleanup_rollback_dirs_in(base: &Path) -> Result<u32> {
    let mut removed = 0u32;
    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with(".hoop.rollback.") {
                let path = entry.path();
                std::fs::remove_dir_all(&path)
                    .with_context(|| format!("Failed to remove rollback dir {}", path.display()))?;
                removed += 1;
            }
        }
    }
    Ok(removed)
}

fn cleanup_rollback_dirs() -> Result<u32> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    cleanup_rollback_dirs_in(&home)
}

/// Move `backup_dir` back to `target`, removing any partial state at `target`.
fn rollback_to(backup_dir: &Path, target: &Path) -> Result<()> {
    if target.exists() {
        std::fs::remove_dir_all(target)
            .with_context(|| format!("Failed to remove partial restore at {}", target.display()))?;
    }
    if backup_dir.exists() {
        std::fs::rename(backup_dir, target)
            .with_context(|| format!("Failed to restore rollback {} -> {}", backup_dir.display(), target.display()))?;
    }
    Ok(())
}

fn rollback(backup_dir: &Path) -> Result<()> {
    rollback_to(backup_dir, &hoop_dir())
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

    // 3. Download and parse manifest (uploaded last by backup pipeline)
    let manifest_bytes = s3_get(&s3_config, &locator.bucket, &format!("{}/manifest.json", locator.key)).await?;
    let manifest: hoop_daemon::snapshot_manifest::SnapshotManifest = serde_json::from_slice(&manifest_bytes)
        .context("Failed to parse manifest.json")?;

    println!(
        "Snapshot: {} (schema {}, {})",
        manifest.snapshot_id,
        manifest.schema_version,
        manifest.created_at
    );

    // 4. Validate manifest before any destructive action (§20.1)
    let current = hoop_daemon::fleet::SCHEMA_VERSION;
    manifest.validate(current)
        .context("Manifest validation failed")?;

    // 5. Move existing ~/.hoop/ aside (destructive action follows)
    let backup_dir = move_aside_for_rollback()
        .context("Failed to move existing ~/.hoop/ aside for rollback")?;

    println!(
        "Moved existing state to {}",
        backup_dir.display()
    );

    // Everything after move_aside is protected: any failure triggers rollback.
    let result: Result<()> = async {
        // 6. Create fresh ~/.hoop/
        let hoop = hoop_dir();
        std::fs::create_dir_all(&hoop)
            .with_context(|| format!("Failed to create {}", hoop.display()))?;

        // 7. Download and restore fleet.db (compressed, optionally encrypted)
        println!("Downloading {} ...", manifest.fleet_db_key);
        let db_compressed = s3_get(&s3_config, &locator.bucket, &manifest.fleet_db_key).await?;

        // Integrity check using manifest hash
        if let Some(ref expected_sha) = manifest.fleet_db_sha256 {
            verify_sha256(&db_compressed, expected_sha)
                .context("fleet.db integrity check failed")?;
        }

        // Decompress
        let db_data = if manifest.fleet_db_key.ends_with(".zst") || manifest.fleet_db_key.ends_with(".age") {
            // If encrypted, try to decrypt first
            let compressed = if manifest.encryption == "age" {
                decrypt_with_age(&db_compressed)?
            } else {
                db_compressed
            };
            zstd::decode_all(&compressed[..])
                .context("zstd decompress fleet.db")?
        } else {
            db_compressed.to_vec()
        };

        let db_path = hoop.join("fleet.db");
        std::fs::write(&db_path, &db_data)
            .with_context(|| format!("Failed to write {}", db_path.display()))?;
        println!("fleet.db restored ({} bytes)", db_data.len());

        // 8. Download and restore attachments (if manifest references them)
        if let Some(ref att_key) = manifest.attachments_manifest_key {
            println!("Fetching attachment manifest from {} ...", att_key);
            let att_manifest_bytes = s3_get(&s3_config, &locator.bucket, att_key).await?;
            let att_manifest: serde_json::Value = serde_json::from_slice(&att_manifest_bytes)
                .context("Failed to parse attachment manifest")?;

            // Download each tracked attachment file
            if let Some(files) = att_manifest.get("files").and_then(|f| f.as_object()) {
                let attachments_dir = hoop.join("attachments");
                for (rel_path, _entry) in files {
                    let file_path = attachments_dir.join(rel_path);
                    if let Some(parent) = file_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }

                    let s3_key = format!("{}/attachments/{}.zst", locator.key, rel_path);
                    println!("Downloading attachment {} ...", rel_path);
                    let data = s3_get(&s3_config, &locator.bucket, &s3_key).await?;
                    let decompressed = zstd::decode_all(&data[..])
                        .with_context(|| format!("decompress attachment {}", rel_path))?;
                    std::fs::write(&file_path, &decompressed)?;
                }
            }
        }

        // 9. Restore projects.yaml from backup if available
        let projects_backup = backup_dir.join("projects.yaml");
        if projects_backup.exists() {
            let dst = hoop.join("projects.yaml");
            std::fs::copy(&projects_backup, &dst).with_context(|| {
                format!(
                    "Failed to copy projects.yaml from backup {}",
                    projects_backup.display()
                )
            })?;
            println!("Preserved projects.yaml from previous state");
        }

        // 10. Run schema migrations on restored fleet.db
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

    // Rollback on any failure after move_aside (§15.4)
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
    result?;

    // Clean up rollback dir(s) so the daemon doesn't refuse to start
    // (audit interprets leftover .hoop.rollback.* as interrupted restore).
    match cleanup_rollback_dirs() {
        Ok(0) => {}
        Ok(n) => println!("Cleaned up {} rollback backup(s).", n),
        Err(e) => {
            eprintln!("Warning: could not clean up rollback directory: {e}");
            eprintln!("The daemon may refuse to start until manually removed.");
        }
    }

    println!("Restore complete.");
    println!("Start the daemon with: systemctl --user start hoop");

    Ok(())
}

/// Decrypt age-encrypted data using the identity from the environment.
fn decrypt_with_age(data: &[u8]) -> Result<bytes::Bytes> {
    let identity = std::env::var("HOOP_BACKUP_AGE_IDENTITY")
        .or_else(|_| std::env::var("AGE_IDENTITY"))
        .context("Set HOOP_BACKUP_AGE_IDENTITY or AGE_IDENTITY to decrypt age-encrypted backups")?;

    let mut child = std::process::Command::new("age")
        .arg("--decrypt")
        .arg("--identity")
        .arg(&identity)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("failed to spawn `age` — is it installed?")?;

    use std::io::Write;
    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(data)?;
    }
    drop(child.stdin.take());

    let output = child.wait_with_output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("age decrypt failed: {}", stderr.trim());
    }

    Ok(bytes::Bytes::from(output.stdout))
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
            "created_at": "2024-01-15T04:00:00Z",
            "schema_version": "1.11.0",
            "fleet_db_key": "backups/snap-001/fleet.db.zst",
            "attachments_manifest_key": "backups/snap-001/attachments.manifest.json",
            "encryption": "none",
            "hoop_version": "0.1.0",
            "fleet_db_sha256": "abc123",
            "fleet_db_size": 4096
        }"#;
        let m: hoop_daemon::snapshot_manifest::SnapshotManifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.snapshot_id, "snap-001");
        assert_eq!(m.fleet_db_key, "backups/snap-001/fleet.db.zst");
        assert_eq!(m.encryption, "none");
        assert_eq!(m.fleet_db_sha256.as_deref(), Some("abc123"));
    }

    #[test]
    fn test_newer_version_rejection_diagnostic() {
        use hoop_daemon::snapshot_manifest::SnapshotManifest;
        let m = SnapshotManifest {
            snapshot_id: "snap-future".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
            schema_version: "99.0.0".into(),
            fleet_db_key: "backups/snap-future/fleet.db.zst".into(),
            attachments_manifest_key: None,
            encryption: "none".into(),
            hoop_version: "99.0.0".into(),
            fleet_db_sha256: None,
            fleet_db_size: None,
        };
        let current = hoop_daemon::fleet::SCHEMA_VERSION;
        let err = m.validate(current).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("99.0.0"),
            "diagnostic should mention snapshot version: {msg}"
        );
        assert!(
            msg.contains("newer than"),
            "diagnostic should say 'newer than': {msg}"
        );
        assert!(
            msg.contains("Upgrade HOOP"),
            "diagnostic should say 'Upgrade HOOP': {msg}"
        );
    }

    #[test]
    fn test_rollback_restores_original() {
        let tmp = tempfile::tempdir().unwrap();
        let hoop = tmp.path().join(".hoop");
        let backup = tmp.path().join(".hoop.rollback.20240101T000000Z");

        // Set up original state
        std::fs::create_dir_all(&hoop).unwrap();
        std::fs::write(hoop.join("fleet.db"), "original data").unwrap();
        std::fs::write(hoop.join("projects.yaml"), "projects: []").unwrap();

        // Move aside (simulates move_aside_for_rollback)
        std::fs::rename(&hoop, &backup).unwrap();

        // Simulate partial restore — new incomplete state
        std::fs::create_dir_all(&hoop).unwrap();
        std::fs::write(hoop.join("fleet.db"), "partial garbage").unwrap();

        // Rollback
        rollback_to(&backup, &hoop).unwrap();

        // Original state is restored
        assert!(hoop.exists(), "~/.hoop/ should exist after rollback");
        assert!(
            !backup.exists(),
            "rollback dir should be gone after rollback"
        );
        assert_eq!(
            std::fs::read_to_string(hoop.join("fleet.db")).unwrap(),
            "original data",
            "fleet.db should contain original data"
        );
        assert_eq!(
            std::fs::read_to_string(hoop.join("projects.yaml")).unwrap(),
            "projects: []",
            "projects.yaml should be preserved"
        );
    }

    #[test]
    fn test_rollback_no_partial_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let hoop = tmp.path().join(".hoop");
        let backup = tmp.path().join(".hoop.rollback.20240101T000000Z");

        // Original state moved aside
        std::fs::create_dir_all(&backup).unwrap();
        std::fs::write(backup.join("fleet.db"), "original").unwrap();
        // No partial ~/.hoop/ exists (e.g., create_dir_all failed)

        rollback_to(&backup, &hoop).unwrap();

        assert!(hoop.exists());
        assert_eq!(
            std::fs::read_to_string(hoop.join("fleet.db")).unwrap(),
            "original"
        );
    }

    // ── Acceptance: cleanup on success ──────────────────────────────────

    #[test]
    fn test_cleanup_rollback_dirs_removes_leftovers() {
        let tmp = tempfile::tempdir().unwrap();
        let rollback_a = tmp.path().join(".hoop.rollback.20240101T000000Z");
        let rollback_b = tmp.path().join(".hoop.rollback.20240102T000000Z");
        let normal_hoop = tmp.path().join(".hoop");
        let unrelated = tmp.path().join(".other");

        std::fs::create_dir_all(&rollback_a).unwrap();
        std::fs::write(rollback_a.join("fleet.db"), "old-a").unwrap();
        std::fs::create_dir_all(&rollback_b).unwrap();
        std::fs::write(rollback_b.join("fleet.db"), "old-b").unwrap();
        std::fs::create_dir_all(&normal_hoop).unwrap();
        std::fs::write(normal_hoop.join("fleet.db"), "current").unwrap();
        std::fs::create_dir_all(&unrelated).unwrap();

        let removed = cleanup_rollback_dirs_in(tmp.path()).unwrap();
        assert_eq!(removed, 2, "should remove exactly 2 rollback dirs");
        assert!(!rollback_a.exists(), "rollback A should be removed");
        assert!(!rollback_b.exists(), "rollback B should be removed");
        assert!(normal_hoop.exists(), "~/.hoop/ should be untouched");
        assert!(unrelated.exists(), "unrelated dirs should be untouched");
        assert_eq!(
            std::fs::read_to_string(normal_hoop.join("fleet.db")).unwrap(),
            "current",
            "~/.hoop/ content should be intact"
        );
    }

    #[test]
    fn test_cleanup_rollback_dirs_noop_when_clean() {
        let tmp = tempfile::tempdir().unwrap();
        let normal_hoop = tmp.path().join(".hoop");
        std::fs::create_dir_all(&normal_hoop).unwrap();

        let removed = cleanup_rollback_dirs_in(tmp.path()).unwrap();
        assert_eq!(removed, 0, "should remove nothing when clean");
    }

    // ── Acceptance: newer-version rejection diagnostic ──────────────────

    #[test]
    fn test_newer_version_rejection_clear_diagnostic() {
        use hoop_daemon::snapshot_manifest::SnapshotManifest;
        let m = SnapshotManifest {
            snapshot_id: "snap-future".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
            schema_version: "99.0.0".into(),
            fleet_db_key: "backups/snap-future/fleet.db.zst".into(),
            attachments_manifest_key: None,
            encryption: "none".into(),
            hoop_version: "99.0.0".into(),
            fleet_db_sha256: None,
            fleet_db_size: None,
        };
        let current = hoop_daemon::fleet::SCHEMA_VERSION;
        let err = m.validate(current).unwrap_err();
        let msg = err.to_string();

        // Diagnostic must identify both versions
        assert!(
            msg.contains("99.0.0"),
            "must mention snapshot version: {msg}"
        );
        assert!(
            msg.contains(current),
            "must mention binary's current version ({current}): {msg}"
        );
        assert!(
            msg.contains("newer than"),
            "must say 'newer than': {msg}"
        );
        assert!(
            msg.contains("Upgrade HOOP"),
            "must suggest upgrading: {msg}"
        );

        // Validate also wraps cleanly through context()
        let wrapped = Err::<(), _>(err).context("Manifest validation failed");
        let wrapped_msg = format!("{:#}", wrapped.unwrap_err());
        assert!(
            wrapped_msg.contains("Manifest validation failed"),
            "context wrapper preserved: {wrapped_msg}"
        );
        assert!(
            wrapped_msg.contains("99.0.0"),
            "inner diagnostic preserved through context: {wrapped_msg}"
        );
    }

    #[test]
    fn test_newer_version_rejection_happens_before_move_aside() {
        // Verify the order in run_restore(): validate() is called before
        // move_aside_for_rollback(). A newer-version manifest must never
        // reach the destructive rename step.
        //
        // This is a structural guarantee — if the code order changes,
        // this test documents the intended invariant.
        use hoop_daemon::snapshot_manifest::SnapshotManifest;
        let m = SnapshotManifest {
            snapshot_id: "snap-future".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
            schema_version: "99.0.0".into(),
            fleet_db_key: "backups/snap-future/fleet.db.zst".into(),
            attachments_manifest_key: None,
            encryption: "none".into(),
            hoop_version: "99.0.0".into(),
            fleet_db_sha256: None,
            fleet_db_size: None,
        };

        // Validation fails — this runs *before* any fs::rename
        assert!(m.validate("1.0.0").is_err());

        // Source code order check: find the call sites inside run_restore()
        let src = include_str!("restore.rs");

        // Find run_restore() function body
        let fn_start = src
            .find("pub async fn run_restore(")
            .expect("restore.rs must define run_restore()");

        // Within run_restore, the validate call must precede the move_aside call
        let validate_pos = src[fn_start..]
            .find("manifest.validate(current)")
            .expect("run_restore must call manifest.validate(current)");
        let move_aside_pos = src[fn_start..]
            .find("move_aside_for_rollback()\n")
            .expect("run_restore must call move_aside_for_rollback()");

        assert!(
            validate_pos < move_aside_pos,
            "manifest.validate() must be called before move_aside_for_rollback() \
             (validate at offset {validate_pos}, move_aside at offset {move_aside_pos} from fn start)"
        );
    }

    // ── Acceptance: rollback on mid-flight failure ──────────────────────

    #[test]
    fn test_mid_failure_rollback_full_cycle() {
        // Simulates the full restore lifecycle:
        //   1. Original ~/.hoop/ exists with real data
        //   2. move_aside_for_rollback()
        //   3. Partial restore writes incomplete data
        //   4. Simulated failure triggers rollback_to()
        //   5. Original state is fully restored
        let tmp = tempfile::tempdir().unwrap();
        let hoop = tmp.path().join(".hoop");
        let backup = tmp.path().join(".hoop.rollback.20240615T040000Z");

        // Step 1: Original state
        std::fs::create_dir_all(&hoop).unwrap();
        std::fs::write(hoop.join("fleet.db"), "original-fleet-data").unwrap();
        std::fs::write(hoop.join("projects.yaml"), "projects:\n  - name: test\n    path: /test").unwrap();
        std::fs::create_dir_all(hoop.join("attachments/sessions")).unwrap();
        std::fs::write(hoop.join("attachments/sessions/log.txt"), "session data").unwrap();

        // Step 2: Move aside
        std::fs::rename(&hoop, &backup).unwrap();
        assert!(!hoop.exists(), "~/.hoop/ gone after move_aside");
        assert!(backup.exists(), "backup dir exists after move_aside");

        // Step 3: Partial restore (simulates failure mid-download)
        std::fs::create_dir_all(&hoop).unwrap();
        std::fs::write(hoop.join("fleet.db"), "incomplete-garbage").unwrap();
        // projects.yaml and attachments NOT written — partial state

        // Step 4: Failure detected → rollback
        rollback_to(&backup, &hoop).unwrap();

        // Step 5: Original state fully restored
        assert!(hoop.exists(), "~/.hoop/ must exist after rollback");
        assert!(!backup.exists(), "backup dir consumed by rollback");
        assert_eq!(
            std::fs::read_to_string(hoop.join("fleet.db")).unwrap(),
            "original-fleet-data",
            "fleet.db must contain original data"
        );
        assert_eq!(
            std::fs::read_to_string(hoop.join("projects.yaml")).unwrap(),
            "projects:\n  - name: test\n    path: /test",
            "projects.yaml must be original"
        );
        assert_eq!(
            std::fs::read_to_string(hoop.join("attachments/sessions/log.txt")).unwrap(),
            "session data",
            "attachments must be original"
        );
    }

    #[test]
    fn test_successful_restore_then_cleanup_allows_daemon_start() {
        // After a successful restore + cleanup, no .hoop.rollback.* dirs remain.
        // The daemon's audit check (check_restore_state) would therefore pass.
        let tmp = tempfile::tempdir().unwrap();
        let hoop = tmp.path().join(".hoop");
        let rollback = tmp.path().join(".hoop.rollback.20240615T040000Z");

        // Simulate: original state, moved aside, then restored successfully
        std::fs::create_dir_all(&rollback).unwrap();
        std::fs::write(rollback.join("fleet.db"), "original").unwrap();
        std::fs::rename(&rollback, &hoop).unwrap();

        // At this point restore "succeeded" but rollback dir was consumed by rename.
        // cleanup_rollback_dirs_in is a no-op (nothing to clean).
        let removed = cleanup_rollback_dirs_in(tmp.path()).unwrap();
        assert_eq!(removed, 0);

        // No .hoop.rollback.* exists → daemon audit would pass
        let has_leftovers = std::fs::read_dir(tmp.path())
            .unwrap()
            .flatten()
            .any(|e| e.file_name().to_string_lossy().starts_with(".hoop.rollback."));
        assert!(!has_leftovers, "no leftover rollback dirs after cleanup");
    }
}
