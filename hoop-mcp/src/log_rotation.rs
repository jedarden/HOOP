//! File-based log rotation for hoop-mcp (§18.4, §6 Phase 6 deliverable 3)
//!
//! Same rotation parameters as the daemon: 100 MB / 24 h / 14-day retention.
//! Writes to `~/.hoop/logs/hoop-mcp.YYYY-MM-DD.log`.

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
const PREFIX: &str = "hoop-mcp";

// ── Redaction (same patterns as daemon) ───────────────────────────────────────

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
                eprintln!("hoop-mcp log rotation failed: {e}");
            }
        }
        let redacted = redact(buf);
        self.file.write_all(&redacted)?;
        self.size += redacted.len() as u64;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
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

/// Initialise file-based log rotation with stderr mirror and redaction.
///
/// Uses stderr (not stdout) for the console mirror because stdout carries
/// JSON-RPC traffic in stdio mode.
pub fn init_logging() {
    let dir = log_dir();
    let filter = EnvFilter::from_default_env().add_directive(Level::INFO.into());

    let file_writer = match RotatingFileWriter::open(dir) {
        Ok(w) => SharedWriter {
            inner: Arc::new(Mutex::new(w)),
        },
        Err(e) => {
            eprintln!("hoop-mcp: log rotation init failed ({e}), falling back to stderr");
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_writer(std::io::stderr)
                .init();
            return;
        }
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(file_writer).with_ansi(false))
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(filter)
        .init();
}
