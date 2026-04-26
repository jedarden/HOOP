//! File-based log rotation with redaction (§18.4, §6 Phase 6 deliverable 3)
//!
//! - Default path: `~/.hoop/logs/`
//! - Rotation on 100 MB or 24 h (whichever first)
//! - 14-day retention with startup cleanup
//! - Regex redaction applied at write time for API keys, tokens, and secrets

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use chrono::Local;
use regex::Regex;
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100 MB
const MAX_AGE_DAYS: i64 = 14;
const PREFIX: &str = "hoop";

// ── Redaction ────────────────────────────────────────────────────────────────

fn redaction_regexes() -> &'static [Regex] {
    static RES: OnceLock<Vec<Regex>> = OnceLock::new();
    RES.get_or_init(|| {
        vec![
            Regex::new(r"sk-[a-zA-Z0-9_-]{20,}").unwrap(),
            Regex::new(r"Bearer\s+[A-Za-z0-9._-]+").unwrap(),
            Regex::new(r"(?i)(api[_-]?key|secret|token|password|passwd)\s*[:=]\s*\S{8,}")
                .unwrap(),
            Regex::new(r#"(?i)"(api[_-]?key|secret|token|password)"\s*:\s*"[^"]{8,}""#)
                .unwrap(),
        ]
    })
}

fn redact(input: &[u8]) -> Vec<u8> {
    let Ok(s) = std::str::from_utf8(input) else {
        return input.to_vec();
    };
    let mut out = s.to_owned();
    for re in redaction_regexes() {
        out = re.replace_all(&out, "[REDACTED]").into_owned();
    }
    out.into_bytes()
}

// ── Rotation helpers ─────────────────────────────────────────────────────────

fn log_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hoop")
        .join("logs")
}

fn file_path(dir: &Path, date: chrono::NaiveDate, seq: u32) -> PathBuf {
    if seq == 0 {
        dir.join(format!("{PREFIX}.{date}.log"))
    } else {
        dir.join(format!("{PREFIX}.{date}.{seq}.log"))
    }
}

fn cleanup_old_logs(dir: &Path) -> io::Result<()> {
    let cutoff = (Local::now() - chrono::Duration::days(MAX_AGE_DAYS)).date_naive();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if let Some(rest) = name_str.strip_prefix(&format!("{PREFIX}.")) {
            let date_part = rest.split('.').next().unwrap_or("");
            if let Ok(d) = chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
                if d < cutoff {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }
    Ok(())
}

// ── Rotating writer ──────────────────────────────────────────────────────────

struct RotatingFileWriter {
    file: File,
    path: PathBuf,
    size: u64,
    date: chrono::NaiveDate,
    seq: u32,
    dir: PathBuf,
}

impl RotatingFileWriter {
    fn open(dir: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&dir)?;
        cleanup_old_logs(&dir)?;

        let today = Local::now().date_naive();
        let mut seq = 0u32;
        while file_path(&dir, today, seq + 1).exists() {
            seq += 1;
        }

        let path = file_path(&dir, today, seq);
        let (file, size) = if path.exists() {
            let f = OpenOptions::new().append(true).open(&path)?;
            let s = f.metadata()?.len();
            (f, s)
        } else {
            (File::create(&path)?, 0)
        };

        if size >= MAX_FILE_SIZE {
            seq += 1;
            let path = file_path(&dir, today, seq);
            return Ok(Self {
                file: File::create(&path)?,
                path,
                size: 0,
                date: today,
                seq,
                dir,
            });
        }

        Ok(Self {
            file,
            path,
            size,
            date: today,
            seq,
            dir,
        })
    }

    fn rotate(&mut self) -> io::Result<()> {
        let today = Local::now().date_naive();
        if self.date < today {
            self.date = today;
            self.seq = 0;
        } else {
            self.seq += 1;
        }
        let path = file_path(&self.dir, self.date, self.seq);
        self.file = File::create(&path)?;
        self.path = path;
        self.size = 0;
        cleanup_old_logs(&self.dir)?;
        Ok(())
    }
}

impl Write for RotatingFileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let today = Local::now().date_naive();
        if self.date < today || self.size >= MAX_FILE_SIZE {
            if let Err(e) = self.rotate() {
                eprintln!("hoop log rotation failed: {e}");
            }
        }
        let redacted = redact(buf);
        self.file.write_all(&redacted)?;
        self.size += redacted.len() as u64;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // Use sync_all() instead of flush() to ensure log data reaches disk.
        // This prevents partial/corrupted log entries on crash (crash-safe logging).
        self.file.sync_all()
    }
}

// ── MakeWriter adapter ───────────────────────────────────────────────────────

/// Newtype wrapper because `MutexGuard<T>: Write` requires Rust ≥1.76.
struct WriteGuard<'a>(std::sync::MutexGuard<'a, RotatingFileWriter>);

impl Write for WriteGuard<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

struct SharedWriter {
    inner: Arc<Mutex<RotatingFileWriter>>,
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for SharedWriter {
    type Writer = WriteGuard<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        WriteGuard(self.inner.lock().expect("log writer mutex poisoned"))
    }
}

// ── Public init ──────────────────────────────────────────────────────────────

/// Initialise file-based log rotation with stdout mirror and redaction.
///
/// Falls back to stdout-only if the log directory cannot be created.
pub fn init_logging() {
    let dir = log_dir();
    let filter = EnvFilter::from_default_env().add_directive(Level::INFO.into());

    let file_writer = match RotatingFileWriter::open(dir) {
        Ok(w) => SharedWriter {
            inner: Arc::new(Mutex::new(w)),
        },
        Err(e) => {
            eprintln!("hoop: log rotation init failed ({e}), falling back to stdout");
            tracing_subscriber::fmt().with_env_filter(filter).init();
            return;
        }
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(file_writer).with_ansi(false))
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(filter)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_api_key() {
        let input = b"Using key sk-ant-api03-abcdefghijklmnopqrstuvwxyz1234567890 for request";
        let out = redact(input);
        let s = std::str::from_utf8(&out).unwrap();
        assert!(!s.contains("sk-ant-api03"));
        assert!(s.contains("[REDACTED]"));
    }

    #[test]
    fn redact_bearer_token() {
        let input = b"Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.payload.signature";
        let out = redact(input);
        let s = std::str::from_utf8(&out).unwrap();
        assert!(!s.contains("eyJhbGci"));
        assert!(s.contains("[REDACTED]"));
    }

    #[test]
    fn redact_key_value() {
        let input = b"config api_key=supersecretvalue12345 loaded";
        let out = redact(input);
        let s = std::str::from_utf8(&out).unwrap();
        assert!(!s.contains("supersecretvalue12345"));
        assert!(s.contains("[REDACTED]"));
    }

    #[test]
    fn redact_json_key() {
        let input = br#"{"api_key": "sk-ant-supersecretkey1234567890abcdef"}"#;
        let out = redact(input);
        let s = std::str::from_utf8(&out).unwrap();
        assert!(!s.contains("sk-ant-supersecretkey"));
    }

    #[test]
    fn no_redact_normal_text() {
        let input = b"Server started on 127.0.0.1:3000";
        let out = redact(input);
        assert_eq!(out, input);
    }

    #[test]
    fn no_redact_short_values() {
        let input = b"status=ok count=5 name=test";
        let out = redact(input);
        assert_eq!(out, input);
    }

    #[test]
    fn rotation_file_naming() {
        let dir = PathBuf::from("/tmp/hoop-test");
        let date = chrono::NaiveDate::from_ymd_opt(2026, 4, 23).unwrap();
        assert_eq!(
            file_path(&dir, date, 0),
            PathBuf::from("/tmp/hoop-test/hoop.2026-04-23.log")
        );
        assert_eq!(
            file_path(&dir, date, 1),
            PathBuf::from("/tmp/hoop-test/hoop.2026-04-23.1.log")
        );
    }
}
