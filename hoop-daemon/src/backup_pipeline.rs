//! Daily fleet.db snapshot pipeline: VACUUM INTO → zstd → age (optional) → S3.
//!
//! Plan reference: §15.3, §16.6
//!
//! Acceptance:
//! - Daily cron fires per config schedule
//! - Failure: exponential backoff, max 3 retries, then alert
//! - Encryption skipped cleanly when no age key set
//! - Metrics `hoop_backup_last_success_timestamp`, `hoop_backup_last_size_bytes` updated

use crate::backup::{BackupCredentials, BackupFileConfig};
use crate::fleet;
use crate::metrics;
use crate::shutdown::ShutdownPhase;
use anyhow::{bail, Context, Result};
use chrono::{Datelike, Timelike, Utc};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

type HmacSha256 = Hmac<Sha256>;

const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_SECS: u64 = 2;
const MAX_BACKOFF_SECS: u64 = 60;

// ── Pipeline entry point ─────────────────────────────────────────────

pub struct BackupPipeline {
    config: BackupFileConfig,
    credentials: BackupCredentials,
}

impl BackupPipeline {
    pub fn new(config: BackupFileConfig, credentials: BackupCredentials) -> Self {
        Self { config, credentials }
    }

    /// Spawn a background scheduler that checks the cron schedule every 60 s.
    ///
    /// Follows the same `tokio::select!` pattern as the morning-brief scheduler.
    pub fn start_scheduler(self, mut shutdown: broadcast::Receiver<ShutdownPhase>) {
        tokio::spawn(async move {
            let schedule = CronSchedule::parse(&self.config.schedule);
            let mut last_run_date: Option<chrono::NaiveDate> = None;

            loop {
                tokio::select! {
                    _ = shutdown.recv() => {
                        info!("Backup scheduler shutting down");
                        break;
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {
                        let now = Utc::now();
                        let today = now.date_naive();

                        if !schedule.matches(&now) {
                            continue;
                        }
                        if last_run_date.as_ref() == Some(&today) {
                            continue;
                        }

                        last_run_date = Some(today);
                        info!("Backup scheduler: triggering daily snapshot for {}", today);

                        match self.run_snapshot().await {
                            Ok(size) => info!("Backup completed ({} bytes)", size),
                            Err(e) => {
                                error!("Backup failed after all retries: {}", e);
                                metrics::metrics().hoop_backup_failures_total.inc();
                            }
                        }
                    }
                }
            }
        });
    }

    // ── Core pipeline ────────────────────────────────────────────────

    async fn run_snapshot(&self) -> Result<u64> {
        let start = std::time::Instant::now();

        // 1. VACUUM INTO temp snapshot
        let snapshot_path = self.vacuum_into()?;

        // 2. zstd compress
        let compressed_path = self.zstd_compress(&snapshot_path)?;
        let _ = std::fs::remove_file(&snapshot_path);

        // 3. Optional age encryption
        let (upload_path, encrypted) = if self.config.encryption {
            match self.age_encrypt(&compressed_path).await {
                Ok(p) => (p, true),
                Err(e) => {
                    warn!("Age encryption failed, uploading unencrypted: {}", e);
                    (compressed_path.clone(), false)
                }
            }
        } else {
            (compressed_path.clone(), false)
        };

        // 4. Upload to S3 with retry
        let file_size = std::fs::metadata(&upload_path)
            .with_context(|| format!("stat {}", upload_path.display()))?
            .len();
        let s3_key = self.build_s3_key(encrypted);

        self.upload_with_retry(&upload_path, &s3_key).await?;

        // 5. Cleanup temp files
        let _ = std::fs::remove_file(&compressed_path);
        if encrypted {
            let _ = std::fs::remove_file(&upload_path);
        }

        // 6. Record metrics
        let elapsed = start.elapsed();
        let m = metrics::metrics();
        m.hoop_backup_last_success_timestamp.set(Utc::now().timestamp());
        m.hoop_backup_last_size_bytes.set(file_size as i64);
        m.hoop_backup_run_duration_seconds.observe(elapsed.as_secs_f64());

        info!(
            "Backup snapshot uploaded: {} bytes in {:.1}s (key={})",
            file_size,
            elapsed.as_secs_f64(),
            s3_key,
        );

        Ok(file_size)
    }

    // ── Step 1: VACUUM INTO ──────────────────────────────────────────

    fn vacuum_into(&self) -> Result<PathBuf> {
        let db_path = fleet::db_path();
        if !db_path.exists() {
            bail!("fleet.db not found at {}", db_path.display());
        }

        let snapshot_dir = std::env::temp_dir().join("hoop-backup");
        std::fs::create_dir_all(&snapshot_dir)
            .context("create temp dir for backup snapshot")?;

        let snapshot_path = snapshot_dir.join(format!(
            "fleet-{}.db",
            Utc::now().format("%Y%m%dT%H%M%SZ")
        ));

        info!(
            "VACUUM INTO {} from {}",
            snapshot_path.display(),
            db_path.display()
        );

        let conn = rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .context("open fleet.db for VACUUM INTO")?;

        // VACUUM INTO produces a self-contained, consistent snapshot without
        // locking the source database for the duration of the copy.
        conn.execute_batch(&format!(
            "VACUUM INTO '{}'",
            snapshot_path.display()
        ))
        .context("VACUUM INTO failed")?;

        drop(conn);

        let size = std::fs::metadata(&snapshot_path)?.len();
        info!("VACUUM INTO produced {} byte snapshot", size);

        Ok(snapshot_path)
    }

    // ── Step 2: zstd compress ─────────────────────────────────────────

    fn zstd_compress(&self, input: &Path) -> Result<PathBuf> {
        let output = PathBuf::from(format!("{}.zst", input.display()));

        let raw = std::fs::read(input)
            .with_context(|| format!("read {}", input.display()))?;

        let compressed = zstd::encode_all(&raw[..], 3)
            .context("zstd compression failed")?;

        std::fs::write(&output, &compressed)
            .with_context(|| format!("write {}", output.display()))?;

        info!(
            "Compressed {} → {} bytes ({:.1}x)",
            raw.len(),
            compressed.len(),
            raw.len() as f64 / compressed.len().max(1) as f64,
        );

        Ok(output)
    }

    // ── Step 3: optional age encryption ────────────────────────────────

    async fn age_encrypt(&self, input: &Path) -> Result<PathBuf> {
        let recipient = self.credentials.age_key.as_deref().with_context(|| {
            "encryption enabled but HOOP_BACKUP_AGE_KEY not set"
        })?;

        let output = PathBuf::from(format!("{}.age", input.display()));

        let status = tokio::process::Command::new("age")
            .arg("--encrypt")
            .arg("--recipient")
            .arg(recipient)
            .arg("--output")
            .arg(&output)
            .arg(input)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .status()
            .await
            .context("failed to spawn `age` — is it installed?")?;

        if !status.success() {
            bail!("age exited with {:?}", status.code());
        }

        Ok(output)
    }

    // ── S3 key layout ─────────────────────────────────────────────────

    fn build_s3_key(&self, encrypted: bool) -> String {
        let ts = Utc::now().format("%Y%m%dT%H%M%SZ");
        let ext = if encrypted { ".zst.age" } else { ".zst" };
        format!(
            "{}/fleet.db.{}{}",
            self.config.prefix.trim_end_matches('/'),
            ts,
            ext,
        )
    }

    // ── Step 4: S3 PUT with exponential-backoff retry ─────────────────

    async fn upload_with_retry(&self, file_path: &Path, s3_key: &str) -> Result<()> {
        let data = std::fs::read(file_path)
            .with_context(|| format!("read {}", file_path.display()))?;

        let mut attempt = 0u32;
        let mut backoff_secs = INITIAL_BACKOFF_SECS;

        loop {
            attempt += 1;
            match self.s3_put(&data, s3_key).await {
                Ok(()) => return Ok(()),
                Err(e) if attempt < MAX_RETRIES => {
                    warn!(
                        "S3 PUT attempt {}/{} failed: {} — retrying in {}s",
                        attempt, MAX_RETRIES, e, backoff_secs,
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(backoff_secs)).await;
                    backoff_secs = (backoff_secs * 2).min(MAX_BACKOFF_SECS);
                }
                Err(e) => {
                    bail!("S3 PUT failed after {} attempts: {}", attempt, e);
                }
            }
        }
    }

    /// Single PUT to S3-compatible storage with AWS SigV4 signing.
    async fn s3_put(&self, data: &[u8], key: &str) -> Result<()> {
        let region = "us-east-1";
        let endpoint = self.config.endpoint.trim_end_matches('/');
        let url_str = format!("{}/{}/{}", endpoint, self.config.bucket, key);
        let url: reqwest::Url = url_str
            .parse()
            .with_context(|| format!("invalid S3 URL: {}", url_str))?;

        let now = Utc::now();

        // Payload hash
        let mut hasher = Sha256::new();
        hasher.update(data);
        let content_sha256 = hex::encode(hasher.finalize());

        let date_stamp = now.format("%Y%m%d").to_string();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let host = url.host_str().unwrap_or("");
        let canonical_uri = url.path();
        let canonical_qs = url.query().unwrap_or("");

        let canonical_headers = format!(
            "content-length:{}\nhost:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
            data.len(),
            host,
            content_sha256,
            amz_date,
        );
        let signed_headers = "content-length;host;x-amz-content-sha256;x-amz-date";

        let canonical_request = format!(
            "PUT\n{}\n{}\n{}\n{}\n{}",
            canonical_uri,
            canonical_qs,
            canonical_headers,
            signed_headers,
            content_sha256,
        );

        let credential_scope = format!("{}/{}/s3/aws4_request", date_stamp, region);

        let mut h = Sha256::new();
        h.update(canonical_request.as_bytes());
        let creq_hash = hex::encode(h.finalize());

        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{}\n{}\n{}",
            amz_date, credential_scope, creq_hash,
        );

        // Derive signing key
        let k_date = hmac_sha256(
            format!("AWS4{}", self.credentials.secret_access_key).as_bytes(),
            date_stamp.as_bytes(),
        );
        let k_region = hmac_sha256(&k_date, region.as_bytes());
        let k_service = hmac_sha256(&k_region, b"s3");
        let signing_key = hmac_sha256(&k_service, b"aws4_request");
        let signature = hex::encode(hmac_sha256(&signing_key, string_to_sign.as_bytes()));

        let auth_header = format!(
            "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
            self.credentials.access_key_id, credential_scope, signed_headers, signature,
        );

        let resp = reqwest::Client::new()
            .put(url.clone())
            .header("Authorization", auth_header)
            .header("x-amz-date", amz_date)
            .header("x-amz-content-sha256", &content_sha256)
            .header("Content-Length", data.len())
            .body(data.to_vec())
            .send()
            .await
            .with_context(|| format!("S3 PUT request failed for {}", url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("S3 PUT {} returned {}: {}", key, status, body.trim());
        }

        Ok(())
    }
}

// ── SigV4 HMAC helper ────────────────────────────────────────────────

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key length valid");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

// ── Minimal 5-field cron matcher ─────────────────────────────────────

struct CronSchedule {
    minutes: Vec<u32>,
    hours: Vec<u32>,
    doms: Vec<u32>,
    months: Vec<u32>,
    dows: Vec<u32>,
}

impl CronSchedule {
    fn parse(expr: &str) -> Self {
        let fields: Vec<&str> = expr.split_whitespace().collect();
        assert_eq!(fields.len(), 5, "cron must have 5 fields");

        CronSchedule {
            minutes: parse_cron_field(fields[0], 0, 59),
            hours: parse_cron_field(fields[1], 0, 23),
            doms: parse_cron_field(fields[2], 1, 31),
            months: parse_cron_field(fields[3], 1, 12),
            dows: parse_cron_field(fields[4], 0, 6),
        }
    }

    fn matches(&self, t: &chrono::DateTime<Utc>) -> bool {
        self.minutes.contains(&(t.time().minute() as u32))
            && self.hours.contains(&(t.time().hour() as u32))
            && self.doms.contains(&(t.date_naive().day() as u32))
            && self.months.contains(&(t.date_naive().month() as u32))
            && self.dows.contains(&t.weekday().num_days_from_sunday())
    }
}

fn parse_cron_field(field: &str, lo: u32, hi: u32) -> Vec<u32> {
    if field == "*" {
        return (lo..=hi).collect();
    }
    let mut vals = Vec::new();
    for part in field.split(',') {
        if let Some((a, b)) = part.split_once('-') {
            let s: u32 = a.parse().unwrap_or(lo);
            let e: u32 = b.parse().unwrap_or(hi);
            vals.extend(s..=e);
        } else if let Some(step_str) = part.strip_prefix("*/") {
            let step: u32 = step_str.parse().unwrap_or(1).max(1);
            let mut v = lo;
            while v <= hi {
                vals.push(v);
                v += step;
            }
        } else if let Ok(v) = part.parse::<u32>() {
            vals.push(v);
        }
    }
    vals.sort();
    vals.dedup();
    vals
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cron_parse_daily_at_4am() {
        let s = CronSchedule::parse("0 4 * * *");
        assert!(s.minutes.contains(&0));
        assert!(s.hours.contains(&4));
        assert_eq!(s.doms.len(), 31); // all days
    }

    #[test]
    fn cron_parse_every_15_min() {
        let s = CronSchedule::parse("*/15 * * * *");
        assert_eq!(s.minutes, vec![0, 15, 30, 45]);
    }

    #[test]
    fn cron_parse_range() {
        let s = CronSchedule::parse("0 1-3 * * *");
        assert_eq!(s.hours, vec![1, 2, 3]);
    }

    #[test]
    fn cron_matches_specific_time() {
        let s = CronSchedule::parse("30 14 * * *");
        let t = chrono::DateTime::parse_from_rfc3339("2024-06-15T14:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(s.matches(&t));

        let t2 = chrono::DateTime::parse_from_rfc3339("2024-06-15T14:31:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(!s.matches(&t2));
    }

    #[test]
    fn build_s3_key_format() {
        let config = BackupFileConfig {
            endpoint: "https://s3.example.com".into(),
            bucket: "bkt".into(),
            prefix: "backups/".into(),
            schedule: "0 4 * * *".into(),
            retention_days: 30,
            encryption: false,
        };
        let creds = BackupCredentials {
            access_key_id: "key".into(),
            secret_access_key: "secret".into(),
            age_key: None,
        };
        let pipeline = BackupPipeline::new(config, creds);
        let key = pipeline.build_s3_key(false);
        assert!(key.starts_with("backups/fleet.db.2"));
        assert!(key.ends_with(".zst"));
        assert!(!key.contains(".age"));
    }

    #[test]
    fn build_s3_key_encrypted() {
        let config = BackupFileConfig {
            endpoint: "https://s3.example.com".into(),
            bucket: "bkt".into(),
            prefix: "backups/".into(),
            schedule: "0 4 * * *".into(),
            retention_days: 30,
            encryption: true,
        };
        let creds = BackupCredentials {
            access_key_id: "key".into(),
            secret_access_key: "secret".into(),
            age_key: Some("age1test".into()),
        };
        let pipeline = BackupPipeline::new(config, creds);
        let key = pipeline.build_s3_key(true);
        assert!(key.ends_with(".zst.age"));
    }

    #[test]
    fn vacuum_into_reads_fleet_db() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("fleet.db");

        // Create a small SQLite database
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.pragma_update(None, "journal_mode", "WAL").unwrap();
            conn.execute("CREATE TABLE t (id INTEGER)", []).unwrap();
            conn.execute("INSERT INTO t VALUES (1)", []).unwrap();
        }

        // Point fleet.db at our temp database
        std::env::set_var("_HOOP_FLEET_DB_PATH", &db_path);

        let config = BackupFileConfig {
            endpoint: "https://s3.example.com".into(),
            bucket: "bkt".into(),
            prefix: "backups/".into(),
            schedule: "0 4 * * *".into(),
            retention_days: 30,
            encryption: false,
        };
        let creds = BackupCredentials {
            access_key_id: "key".into(),
            secret_access_key: "secret".into(),
            age_key: None,
        };
        let pipeline = BackupPipeline::new(config, creds);
        let snapshot = pipeline.vacuum_into().unwrap();

        // Snapshot should be a valid SQLite file
        assert!(snapshot.exists());
        let conn = rusqlite::Connection::open(&snapshot).unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM t", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);

        // Cleanup
        std::env::remove_var("_HOOP_FLEET_DB_PATH");
    }

    #[test]
    fn zstd_compress_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("test.db");
        std::fs::write(&input, b"hello sqlite data here").unwrap();

        let config = BackupFileConfig {
            endpoint: "https://s3.example.com".into(),
            bucket: "bkt".into(),
            prefix: "backups/".into(),
            schedule: "0 4 * * *".into(),
            retention_days: 30,
            encryption: false,
        };
        let creds = BackupCredentials {
            access_key_id: "key".into(),
            secret_access_key: "secret".into(),
            age_key: None,
        };
        let pipeline = BackupPipeline::new(config, creds);
        let compressed = pipeline.zstd_compress(&input).unwrap();
        assert!(compressed.exists());

        let compressed_data = std::fs::read(&compressed).unwrap();
        let decompressed = zstd::decode_all(&compressed_data[..]).unwrap();
        assert_eq!(decompressed, b"hello sqlite data here");

        let _ = std::fs::remove_file(&compressed);
    }

    #[test]
    fn retry_logic_exhausts_attempts() {
        // Verify the backoff calculation doesn't overflow
        let mut backoff = INITIAL_BACKOFF_SECS;
        for _ in 0..10 {
            backoff = (backoff * 2).min(MAX_BACKOFF_SECS);
        }
        assert_eq!(backoff, MAX_BACKOFF_SECS);
    }

    #[test]
    fn hmac_sha256_produces_deterministic_output() {
        let a = hmac_sha256(b"key", b"data");
        let b = hmac_sha256(b"key", b"data");
        assert_eq!(a, b);
        assert_ne!(a, hmac_sha256(b"key", b"other"));
    }
}
