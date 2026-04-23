//! Local Whisper transcription service
//!
//! CPU-bound, failure-tolerant transcription using whisper.cpp CLI.
//! Features:
//! - Async job queue with tokio
//! - Word-level timestamps from Whisper
//! - Fallback to per-utterance timestamps if word-level fails
//! - Retry on failure (max 3 attempts)
//! - Partial transcript saved on failure
//! - Supports wav, mp3, m4a inputs

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinSet;
use tracing::{debug, error, info, warn};

use crate::dictated_notes::{TranscriptWord, TranscriptionResult, TranscriptionStatus};

/// Supported audio formats for input
pub const SUPPORTED_AUDIO_FORMATS: &[&str] = &["wav", "mp3", "m4a", "ogg", "flac", "webm", "opus"];

/// Voice/transcription config loaded from ~/.hoop/voice.yml
#[derive(Debug, Clone, serde::Deserialize)]
pub struct VoiceConfigFile {
    pub whisper_model_path: Option<String>,
    pub whisper_cli_path: Option<String>,
    pub max_concurrent: Option<usize>,
    pub max_retries: Option<u32>,
}

/// Load voice config from ~/.hoop/voice.yml (returns defaults if file missing)
pub fn load_voice_config() -> VoiceConfigFile {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let path = home.join(".hoop").join("voice.yml");

    if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_yaml::from_str(&content) {
                Ok(config) => {
                    info!("Loaded voice config from {}", path.display());
                    return config;
                }
                Err(e) => warn!("Failed to parse {}: {}, using defaults", path.display(), e),
            },
            Err(e) => warn!("Failed to read {}: {}, using defaults", path.display(), e),
        }
    }
    VoiceConfigFile {
        whisper_model_path: None,
        whisper_cli_path: None,
        max_concurrent: None,
        max_retries: None,
    }
}

/// Build TranscriptionConfig from voice config file + defaults
pub fn build_transcription_config(voice_config: &VoiceConfigFile) -> TranscriptionConfig {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let default_model_path = home.join(".hoop").join("models").join("ggml-base.en.bin");

    TranscriptionConfig {
        whisper_cli_path: voice_config.whisper_cli_path.as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("whisper")),
        whisper_model_path: voice_config.whisper_model_path.as_ref()
            .map(PathBuf::from)
            .unwrap_or(default_model_path),
        max_concurrent: voice_config.max_concurrent.unwrap_or(MAX_CONCURRENT_JOBS),
        max_retries: voice_config.max_retries.unwrap_or(MAX_RETRIES),
    }
}

/// Maximum concurrent transcription jobs
const MAX_CONCURRENT_JOBS: usize = 2;

/// Maximum retry attempts per job
const MAX_RETRIES: u32 = 3;

/// Transcription job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    /// Job is queued and waiting to start
    Pending,
    /// Job is currently being transcribed
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed after all retries
    Failed,
}

/// A transcription job in the queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionJob {
    pub id: String,
    pub stitch_id: String,
    pub audio_path: PathBuf,
    pub status: JobStatus,
    pub attempts: u32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

/// Configuration for the transcription service
#[derive(Debug, Clone)]
pub struct TranscriptionConfig {
    /// Path to whisper.cpp CLI binary
    pub whisper_cli_path: PathBuf,
    /// Path to Whisper model file (.gguf)
    pub whisper_model_path: PathBuf,
    /// Maximum jobs to run concurrently
    pub max_concurrent: usize,
    /// Maximum retry attempts
    pub max_retries: u32,
}

impl Default for TranscriptionConfig {
    fn default() -> Self {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".hoop");

        Self {
            whisper_cli_path: PathBuf::from("whisper"), // Assume in PATH
            whisper_model_path: home.join("models").join("ggml-base.en.bin"),
            max_concurrent: MAX_CONCURRENT_JOBS,
            max_retries: MAX_RETRIES,
        }
    }
}

/// Transcription service - manages async job queue
#[derive(Debug, Clone)]
pub struct TranscriptionService {
    config: TranscriptionConfig,
    jobs: Arc<RwLock<Vec<TranscriptionJob>>>,
    job_tx: mpsc::Sender<String>,
}

impl TranscriptionService {
    /// Create a new transcription service
    pub fn new(config: TranscriptionConfig) -> Self {
        let (job_tx, mut job_rx) = mpsc::channel::<String>(100);
        let jobs = Arc::new(RwLock::new(Vec::new()));
        let service = Self {
            config,
            jobs,
            job_tx,
        };

        // Spawn the job processor
        let processor = TranscriptionJobProcessor::new(
            service.config.clone(),
            service.jobs.clone(),
        );
        tokio::spawn(async move {
            processor.run(job_rx).await;
        });

        service
    }

    /// Submit a new transcription job
    pub async fn submit_job(
        &self,
        stitch_id: String,
        audio_path: PathBuf,
    ) -> Result<String> {
        let job_id = uuid::Uuid::new_v4().to_string();
        let job = TranscriptionJob {
            id: job_id.clone(),
            stitch_id,
            audio_path,
            status: JobStatus::Pending,
            attempts: 0,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
        };

        // Add to in-memory store
        self.jobs.write().await.push(job.clone());

        // Persist to database
        self.persist_job(&job).await?;

        // Send to job queue
        self.job_tx.send(job_id.clone()).await
            .context("Failed to send job to queue")?;

        info!("Submitted transcription job {} for stitch {}", job_id, job.stitch_id);
        Ok(job_id)
    }

    /// Get a job by ID
    pub async fn get_job(&self, job_id: &str) -> Option<TranscriptionJob> {
        self.jobs.read().await
            .iter()
            .find(|j| j.id == job_id)
            .cloned()
    }

    /// Get all jobs for a stitch
    pub async fn get_jobs_for_stitch(&self, stitch_id: &str) -> Vec<TranscriptionJob> {
        self.jobs.read().await
            .iter()
            .filter(|j| j.stitch_id == stitch_id)
            .cloned()
            .collect()
    }

    /// Persist a job to the database
    async fn persist_job(&self, job: &TranscriptionJob) -> Result<()> {
        let db_path = crate::fleet::db_path();
        let job_id = job.id.clone();
        let stitch_id = job.stitch_id.clone();
        let audio_path = job.audio_path.to_string_lossy().to_string();
        let status = format!("{:?}", job.status);
        let attempts = job.attempts;
        let created_at = job.created_at.to_rfc3339();
        let started_at = job.started_at.map(|dt| dt.to_rfc3339());
        let completed_at = job.completed_at.map(|dt| dt.to_rfc3339());
        let error_message = job.error_message.clone();

        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path)?;
            conn.pragma_update(None, "journal_mode", "WAL")?;

            conn.execute(
                r#"
                INSERT INTO transcription_jobs (id, stitch_id, audio_path, status, attempts,
                                               created_at, started_at, completed_at, error_message)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                ON CONFLICT(id) DO UPDATE SET
                    status = excluded.status,
                    attempts = excluded.attempts,
                    started_at = excluded.started_at,
                    completed_at = excluded.completed_at,
                    error_message = excluded.error_message
                "#,
                params![
                    job_id, stitch_id, audio_path, status, attempts,
                    created_at, started_at, completed_at, error_message
                ],
            )?;
            Ok::<(), anyhow::Error>(())
        }).await??;

        Ok(())
    }

    /// Update job status in memory and database
    async fn update_job_status(&self, job_id: &str, status: JobStatus, error: Option<String>) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            job.status = status.clone();
            if status == JobStatus::Running && job.started_at.is_none() {
                job.started_at = Some(Utc::now());
            }
            if matches!(status, JobStatus::Completed | JobStatus::Failed) {
                job.completed_at = Some(Utc::now());
            }
            job.error_message = error;

            // Persist to database
            let job_clone = job.clone();
            let jobs_ref = jobs.clone(); // Clone the lock reference for the async task
            drop(jobs); // Release the write lock before the async task

            let self_jobs = self.jobs.clone();
            tokio::task::spawn_blocking(move || {
                let db_path = crate::fleet::db_path();
                let conn = Connection::open(&db_path)?;
                conn.pragma_update(None, "journal_mode", "WAL")?;

                let status_str = format!("{:?}", status);
                let started_at = job_clone.started_at.map(|dt| dt.to_rfc3339());
                let completed_at = job_clone.completed_at.map(|dt| dt.to_rfc3339());

                conn.execute(
                    r#"
                    UPDATE transcription_jobs
                    SET status = ?1, attempts = ?2, started_at = ?3, completed_at = ?4, error_message = ?5
                    WHERE id = ?6
                    "#,
                    params![
                        status_str, job_clone.attempts, started_at, completed_at,
                        job_clone.error_message, job_clone.id
                    ],
                )?;
                Ok::<(), anyhow::Error>(())
            }).await??;
        }
        Ok(())
    }
}

/// Check if an audio file needs conversion to WAV
fn needs_conversion(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        !ext.eq_ignore_ascii_case("wav")
    } else {
        true
    }
}

/// Convert audio file to WAV format using ffmpeg
///
/// Returns the path to the converted WAV file (in temp directory)
fn convert_to_wav(audio_path: &Path) -> Result<PathBuf> {
    let temp_wav = std::env::temp_dir()
        .join(format!("hoop_convert_{}.wav", uuid::Uuid::new_v4()));

    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(audio_path)
        .arg("-ar")
        .arg("16000")  // Whisper prefers 16kHz
        .arg("-ac")
        .arg("1")      // Mono
        .arg("-acodec")
        .arg("pcm_s16le")  // 16-bit PCM
        .arg("-y")      // Overwrite output file
        .arg(&temp_wav)
        .output()
        .context("Failed to execute ffmpeg for audio conversion")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg conversion failed: {}", stderr);
    }

    Ok(temp_wav)
}

/// Transcribe with fallback to segment-level timestamps
///
/// If word-level timestamps fail or are empty, falls back to segment-level timestamps.
async fn transcribe_with_fallback(
    audio_path: &Path,
    config: &TranscriptionConfig,
) -> TranscriptionResult {
    // First attempt: full word-level transcription
    match transcribe_with_whisper_internal(audio_path, config).await {
        Ok(result) => {
            // Check if we got word-level timestamps
            if result.words.is_empty() && !result.transcript.is_empty() {
                debug!("Word-level timestamps empty, will use segment-level fallback");
                // Fall back to segment-level timestamps
                if let Ok(segment_result) = transcribe_segment_level_internal(audio_path, config).await {
                    info!("Using segment-level timestamps for {}", audio_path.display());
                    return segment_result;
                }
            }
            result
        }
        Err(e) => {
            warn!("Full transcription failed: {}, trying segment-level fallback", e);
            // Fallback to segment-level transcription
            match transcribe_segment_level_internal(audio_path, config).await {
                Ok(result) => result,
                Err(e2) => {
                    // Last resort: return a minimal result with error info
                    error!("Both word-level and segment-level transcription failed");
                    TranscriptionResult {
                        transcript: String::new(),
                        words: vec![],
                        duration_secs: None,
                        language: None,
                    }
                }
            }
        }
    }
}

/// Internal function to transcribe with word-level timestamps
async fn transcribe_with_whisper_internal(
    audio_path: &Path,
    config: &TranscriptionConfig,
) -> Result<TranscriptionResult> {
    let audio_path = audio_path.to_path_buf();
    let whisper_cli = config.whisper_cli_path.clone();
    let model_path = config.whisper_model_path.clone();

    tokio::task::spawn_blocking(move || {
        if !model_path.exists() {
            return Err(anyhow::anyhow!("Whisper model not found at {}", model_path.display()));
        }

        if !audio_path.exists() {
            return Err(anyhow::anyhow!("Audio file not found: {}", audio_path.display()));
        }

        let working_audio_path = if needs_conversion(&audio_path) {
            debug!("Converting audio file {} to WAV format", audio_path.display());
            convert_to_wav(&audio_path)?
        } else {
            audio_path.clone()
        };

        let temp_output = std::env::temp_dir().join(format!("whisper_{}", uuid::Uuid::new_v4()));

        let output = Command::new(&whisper_cli)
            .arg("-m")
            .arg(&model_path)
            .arg("-f")
            .arg(&working_audio_path)
            .arg("-oj")
            .arg("--output-file")
            .arg(&temp_output)
            .output()
            .context("Failed to execute whisper.cpp")?;

        if working_audio_path != audio_path {
            let _ = std::fs::remove_file(&working_audio_path);
        }

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("whisper.cpp failed: {}", stderr));
        }

        let json_path = temp_output.with_extension("json");
        let json_str = std::fs::read_to_string(&json_path)?;
        let _ = std::fs::remove_file(&json_path);
        let _ = std::fs::remove_file(temp_output.with_extension("txt"));

        let whisper_output: WhisperOutput = serde_json::from_str(&json_str)?;

        let mut words = Vec::new();
        for segment in &whisper_output.segments {
            for word_data in &segment.words {
                if let Some(word) = &word_data.word {
                    words.push(TranscriptWord {
                        word: word.trim().to_string(),
                        start: word_data.start,
                        end: word_data.end,
                    });
                }
            }
        }

        let transcript = whisper_output.segments
            .iter()
            .map(|s| s.text.trim().to_string())
            .collect::<Vec<_>>()
            .join(" ");

        let duration_secs = whisper_output.segments
            .last()
            .map(|s| s.end)
            .or_else(|| whisper_output.segments.iter().map(|s| s.end).reduce(f64::max))
            .or(Some(0.0));

        Ok(TranscriptionResult {
            transcript,
            words,
            duration_secs,
            language: whisper_output.language,
        })
    }).await?
}

/// Internal function to transcribe with segment-level timestamps (fallback)
async fn transcribe_segment_level_internal(
    audio_path: &Path,
    config: &TranscriptionConfig,
) -> Result<TranscriptionResult> {
    let audio_path = audio_path.to_path_buf();
    let whisper_cli = config.whisper_cli_path.clone();
    let model_path = config.whisper_model_path.clone();

    tokio::task::spawn_blocking(move || {
        if !model_path.exists() {
            return Err(anyhow::anyhow!("Whisper model not found at {}", model_path.display()));
        }

        let working_audio_path = if needs_conversion(&audio_path) {
            convert_to_wav(&audio_path)?
        } else {
            audio_path.clone()
        };

        let temp_output = std::env::temp_dir().join(format!("whisper_{}", uuid::Uuid::new_v4()));

        let output = Command::new(&whisper_cli)
            .arg("-m")
            .arg(&model_path)
            .arg("-f")
            .arg(&working_audio_path)
            .arg("-oj")  // Still request JSON output
            .arg("--output-file")
            .arg(&temp_output)
            .output()?;

        if working_audio_path != audio_path {
            let _ = std::fs::remove_file(&working_audio_path);
        }

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("whisper.cpp failed: {}", stderr));
        }

        let json_path = temp_output.with_extension("json");
        let json_str = std::fs::read_to_string(&json_path)?;
        let _ = std::fs::remove_file(&json_path);
        let _ = std::fs::remove_file(temp_output.with_extension("txt"));

        let whisper_output: WhisperOutput = serde_json::from_str(&json_str)?;

        // Build word-level timestamps from segments
        let mut words = Vec::new();
        for segment in &whisper_output.segments {
            // Split segment text into words
            let segment_words: Vec<&str> = segment.text
                .split_whitespace()
                .collect();

            if segment_words.is_empty() {
                continue;
            }

            let segment_duration = segment.end - segment.start;
            let word_duration = segment_duration / segment_words.len() as f64;

            for (i, word) in segment_words.iter().enumerate() {
                words.push(TranscriptWord {
                    word: word.to_string(),
                    start: segment.start + (i as f64 * word_duration),
                    end: segment.start + ((i + 1) as f64 * word_duration),
                });
            }
        }

        let transcript = whisper_output.segments
            .iter()
            .map(|s| s.text.trim().to_string())
            .collect::<Vec<_>>()
            .join(" ");

        let duration_secs = whisper_output.segments
            .last()
            .map(|s| s.end)
            .or_else(|| whisper_output.segments.iter().map(|s| s.end).reduce(f64::max))
            .or(Some(0.0));

        Ok(TranscriptionResult {
            transcript,
            words,
            duration_secs,
            language: whisper_output.language,
        })
    }).await?
}

/// Job processor - runs transcription jobs in a limited pool
struct TranscriptionJobProcessor {
    config: TranscriptionConfig,
    jobs: Arc<RwLock<Vec<TranscriptionJob>>>,
    running: Arc<AtomicBool>,
}

impl TranscriptionJobProcessor {
    fn new(config: TranscriptionConfig, jobs: Arc<RwLock<Vec<TranscriptionJob>>>) -> Self {
        Self {
            config,
            jobs,
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    async fn run(&self, mut job_rx: mpsc::Receiver<String>) {
        let mut join_set = JoinSet::new();

        info!("Transcription job processor started");

        while self.running.load(Ordering::Relaxed) {
            tokio::select! {
                // Accept new jobs
                Some(job_id) = job_rx.recv() => {
                    // Wait for capacity
                    while join_set.len() >= self.config.max_concurrent {
                        if join_set.join_next().await.is_none() {
                            break;
                        }
                    }

                    // Spawn job task
                    let job_id_clone = job_id.clone();
                    let jobs = self.jobs.clone();
                    let config = self.config.clone();

                    join_set.spawn(async move {
                        Self::process_job(job_id_clone, jobs, config).await;
                    });
                }
                // Clean up completed jobs
                Some(_) = join_set.join_next() => {}
                else => {
                    break;
                }
            }
        }

        // Wait for remaining jobs
        while let Some(result) = join_set.join_next().await {
            if let Err(e) = result {
                error!("Job task failed: {}", e);
            }
        }

        info!("Transcription job processor stopped");
    }

    async fn process_job(
        job_id: String,
        jobs: Arc<RwLock<Vec<TranscriptionJob>>>,
        config: TranscriptionConfig,
    ) {
        // Get job details
        let (stitch_id, audio_path) = {
            let jobs_read = jobs.read().await;
            let job = match jobs_read.iter().find(|j| j.id == job_id) {
                Some(j) => j.clone(),
                None => {
                    warn!("Job {} not found", job_id);
                    return;
                }
            };
            (job.stitch_id.clone(), job.audio_path.clone())
        };

        info!("Starting transcription job {} for stitch {}", job_id, stitch_id);

        // Update status to running
        let service = TranscriptionService {
            config: config.clone(),
            jobs: jobs.clone(),
            job_tx: mpsc::channel(1).0, // Dummy sender
        };
        if let Err(e) = service.update_job_status(&job_id, JobStatus::Running, None).await {
            error!("Failed to update job status: {}", e);
        }

        // Run transcription with retries
        let mut result: Option<TranscriptionResult> = None;
        let mut last_error: Option<String> = None;

        for attempt in 1..=config.max_retries {
            debug!("Transcription attempt {} for job {}", attempt, job_id);

            // Increment attempt counter
            {
                let mut jobs_write = jobs.write().await;
                if let Some(job) = jobs_write.iter_mut().find(|j| j.id == job_id) {
                    job.attempts = attempt;
                }
            }

            let transcription = transcribe_with_fallback(&audio_path, &config).await;
            if !transcription.transcript.is_empty() {
                info!("Transcription succeeded for job {} on attempt {}", job_id, attempt);
                result = Some(transcription);
                break;
            } else {
                warn!("Transcription attempt {} produced empty transcript for job {}", attempt, job_id);
                last_error = Some("Empty transcript produced".to_string());
            }

            if attempt < config.max_retries {
                tokio::time::sleep(Duration::from_secs(5 * attempt as u64)).await;
            }
        }

        // Update job with result
        match result {
            Some(transcription) => {
                if let Err(e) = Self::store_transcription_result(
                    &stitch_id, &transcription, TranscriptionStatus::Completed
                ).await {
                    error!("Failed to store transcription result: {}", e);
                    let _ = service.update_job_status(
                        &job_id,
                        JobStatus::Failed,
                        Some(format!("Failed to store result: {}", e))
                    ).await;
                } else {
                    let _ = service.update_job_status(&job_id, JobStatus::Completed, None).await;
                }
            }
            None => {
                // All retries failed - store partial transcript if available
                let error_msg = last_error.unwrap_or_else(|| "Unknown error".to_string());
                error!("Transcription failed for job {} after {} attempts: {}", job_id, config.max_retries, error_msg);

                let partial_result = TranscriptionResult {
                    transcript: format!("[Transcription failed: {}]", error_msg),
                    words: vec![],
                    duration_secs: None,
                    language: None,
                };

                if let Err(e) = Self::store_transcription_result(
                    &stitch_id, &partial_result, TranscriptionStatus::Failed
                ).await {
                    error!("Failed to store partial transcription result: {}", e);
                }

                let _ = service.update_job_status(
                    &job_id,
                    JobStatus::Failed,
                    Some(format!("Failed after {} attempts: {}", config.max_retries, error_msg))
                ).await;
            }
        }
    }

    /// Transcribe audio using whisper.cpp CLI
    async fn transcribe_with_whisper(
        audio_path: &Path,
        config: &TranscriptionConfig,
    ) -> Result<TranscriptionResult> {
        let audio_path = audio_path.to_path_buf();
        let whisper_cli = config.whisper_cli_path.clone();
        let model_path = config.whisper_model_path.clone();

        // Run in blocking task since transcription is CPU-bound
        tokio::task::spawn_blocking(move || {
            // Check if model exists
            if !model_path.exists() {
                return Err(anyhow::anyhow!(
                    "Whisper model not found at {}. Download a model from https://huggingface.co/ggerganov/whisper.cpp",
                    model_path.display()
                ));
            }

            // Check if audio file exists
            if !audio_path.exists() {
                return Err(anyhow::anyhow!("Audio file not found: {}", audio_path.display()));
            }

            // Convert audio to WAV if necessary (whisper.cpp prefers WAV)
            let working_audio_path = if needs_conversion(&audio_path) {
                debug!("Converting audio file {} to WAV format", audio_path.display());
                convert_to_wav(&audio_path)
                    .context("Failed to convert audio to WAV")?
            } else {
                audio_path.clone()
            };

            // Build whisper.cpp command
            // -m: model path
            // -f: input file
            // -oj: output JSON with word timestamps
            // -l: language (auto-detect if not specified)
            // --output-file: temp file path prefix
            let temp_output = std::env::temp_dir().join(format!("whisper_{}", uuid::Uuid::new_v4()));

            let output = Command::new(&whisper_cli)
                .arg("-m")
                .arg(&model_path)
                .arg("-f")
                .arg(&working_audio_path)
                .arg("-oj")  // Output JSON with word timestamps
                .arg("--output-file")
                .arg(&temp_output)
                .output()
                .context("Failed to execute whisper.cpp")?;

            // Clean up converted file if we created one
            if working_audio_path != audio_path {
                let _ = std::fs::remove_file(&working_audio_path);
            }

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("whisper.cpp failed: {}", stderr));
            }

            // Parse the JSON output
            let json_path = temp_output.with_extension("json");
            let json_str = std::fs::read_to_string(&json_path)
                .context("Failed to read whisper output JSON")?;

            // Clean up temp files
            let _ = std::fs::remove_file(&json_path);
            let _ = std::fs::remove_file(temp_output.with_extension("txt"));

            // Parse Whisper JSON output
            let whisper_output: WhisperOutput = serde_json::from_str(&json_str)
                .context("Failed to parse Whisper JSON output")?;

            // Extract word-level timestamps
            let mut words = Vec::new();
            for segment in &whisper_output.segments {
                for word_data in &segment.words {
                    if let Some(word) = &word_data.word {
                        words.push(TranscriptWord {
                            word: word.trim().to_string(),
                            start: word_data.start,
                            end: word_data.end,
                        });
                    }
                }
            }

            // Get full transcript
            let transcript = whisper_output.segments
                .iter()
                .map(|s| s.text.trim().to_string())
                .collect::<Vec<_>>()
                .join(" ");

            // Get duration from last segment or total_duration
            let duration_secs = whisper_output.segments
                .last()
                .map(|s| s.end)
                .or_else(|| whisper_output.segments.iter().map(|s| s.end).reduce(f64::max))
                .or(Some(0.0));

            // Detect language from output
            let language = whisper_output.language;

            Ok(TranscriptionResult {
                transcript,
                words,
                duration_secs,
                language,
            })
        }).await?
    }

    /// Store transcription result in dictated_notes table
    async fn store_transcription_result(
        stitch_id: &str,
        result: &TranscriptionResult,
        status: TranscriptionStatus,
    ) -> Result<()> {
        let stitch_id = stitch_id.to_string();
        let transcript = result.transcript.clone();
        let words_json = serde_json::to_string(&result.words)?;
        let duration_secs = result.duration_secs;
        let language = result.language.clone();
        let transcribed_at = Utc::now().to_rfc3339();
        let status_json = serde_json::to_string(&status)?;

        tokio::task::spawn_blocking(move || {
            let db_path = crate::fleet::db_path();
            let conn = Connection::open(&db_path)?;
            conn.pragma_update(None, "journal_mode", "WAL")?;

            conn.execute(
                r#"
                UPDATE dictated_notes
                SET transcript = ?1, transcript_words = ?2, transcribed_at = ?3,
                    duration_secs = ?4, language = ?5, transcription_status = ?6
                WHERE stitch_id = ?7
                "#,
                params![transcript, words_json, transcribed_at,
                        duration_secs, language, status_json, stitch_id],
            )?;

            Ok::<(), anyhow::Error>(())
        }).await??;

        Ok(())
    }
}

/// Whisper JSON output structure
#[derive(Debug, Serialize, Deserialize)]
struct WhisperOutput {
    language: Option<String>,
    duration: Option<f64>,
    segments: Vec<WhisperSegment>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WhisperSegment {
    start: f64,
    end: f64,
    text: String,
    words: Vec<WhisperWord>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WhisperWord {
    word: Option<String>,
    start: f64,
    end: f64,
    probability: Option<f64>,
}

/// Initialize the transcription_jobs table
pub fn init_transcription_table(conn: &mut Connection) -> Result<()> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS transcription_jobs (
            id TEXT PRIMARY KEY NOT NULL,
            stitch_id TEXT NOT NULL,
            audio_path TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            attempts INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            started_at TEXT,
            completed_at TEXT,
            error_message TEXT
        )
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_transcription_jobs_stitch_id
        ON transcription_jobs(stitch_id)
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_transcription_jobs_status
        ON transcription_jobs(status)
        "#,
        [],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = TranscriptionConfig::default();
        assert_eq!(config.max_concurrent, MAX_CONCURRENT_JOBS);
        assert_eq!(config.max_retries, MAX_RETRIES);
    }

    #[test]
    fn test_job_status_serialization() {
        let status = JobStatus::Pending;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: JobStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, status);
    }

    #[test]
    fn test_transcription_job_serialization() {
        let job = TranscriptionJob {
            id: "test-id".to_string(),
            stitch_id: "test-stitch".to_string(),
            audio_path: PathBuf::from("/tmp/test.wav"),
            status: JobStatus::Pending,
            attempts: 0,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
        };

        let json = serde_json::to_string(&job).unwrap();
        let deserialized: TranscriptionJob = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, job.id);
        assert_eq!(deserialized.stitch_id, job.stitch_id);
    }
}
