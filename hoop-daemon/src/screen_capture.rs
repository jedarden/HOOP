//! Screen capture stitch support
//!
//! Reads frame_samples.json and the video file from the stitch attachments directory.
//! Screen captures are stored at ~/.hoop/attachments/<stitch_id>/:
//!   screen.{mp4,webm,mov}   — the video recording
//!   frame_samples.json      — chapter markers derived from UI-change frame samples
//!   transcript.json         — word-level Whisper transcript from the audio track
//!   meta.json               — stitch metadata (title, project, recorded_at)
//!
//! Streaming uploads are stored at ~/.hoop/streaming_uploads/<stream_id>/:
//!   partial.{ext}           — partial video file being streamed
//!   metadata.json           — stream session metadata
//!
//! All functions accept `ValidStitchId` for compile-time path-traversal protection (§13).
//! Paths are canonicalized and prefix-checked against an allowlist (§13, §K2).

use crate::atomic_write;
use crate::id_validators::ValidStitchId;
use crate::path_security::{canonicalize_and_check, PathAllowlist};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use uuid::Uuid;

/// A frame sample representing a UI change captured during screen recording.
/// These become chapter markers in the video player.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FrameSample {
    pub timestamp_secs: f64,
    pub label: String,
}

/// Word-level transcript entry (from Whisper)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TranscriptWord {
    pub word: String,
    pub start: f64,
    pub end: f64,
}

/// Transcript with word-level timestamps
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ScreenCaptureTranscript {
    pub text: String,
    pub words: Vec<TranscriptWord>,
}

/// Sidecar metadata written alongside the video file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ScreenCaptureMeta {
    pub stitch_id: String,
    pub project: String,
    pub title: String,
    pub recorded_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<f64>,
}

/// Screen capture API response payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ScreenCaptureData {
    pub stitch_id: String,
    pub title: String,
    pub project: String,
    pub recorded_at: String,
    pub video_url: String,
    pub duration_secs: Option<f64>,
    pub chapters: Vec<FrameSample>,
    pub transcript: Option<ScreenCaptureTranscript>,
}

/// Summary used for the project list endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ScreenCaptureSummary {
    pub stitch_id: String,
    pub project: String,
    pub title: String,
    pub recorded_at: String,
    pub duration_secs: Option<f64>,
    pub chapter_count: usize,
    pub has_transcript: bool,
}

/// Return the stitch attachments directory.
///
/// Path-traversal protection: `stitch_id` is a `ValidStitchId` which guarantees
/// it's a lowercase UUID format (no `..`, `/`, or other path components).
/// The directory is created lazily, then canonicalized and prefix-checked
/// against the allowlist to catch symlink escapes (§13, §K2).
pub fn attachments_dir(stitch_id: &ValidStitchId) -> Result<PathBuf> {
    let allowlist = PathAllowlist::for_stitch_attachments()
        .context("failed to build path allowlist for stitch attachments")?;

    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("home directory not found"))?;
    let dir = home
        .join(".hoop")
        .join("attachments")
        .join(stitch_id.as_str());
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create attachment dir: {}", dir.display()))?;

    let canonical = canonicalize_and_check(&dir, &allowlist)
        .map_err(|_| anyhow::anyhow!("path traversal detected for stitch id"))?;

    Ok(canonical)
}

/// Check if a screen capture video exists for a stitch.
pub fn has_video(stitch_id: &ValidStitchId) -> bool {
    video_path(stitch_id).is_some()
}

/// Get the video file path — checks screen.mp4, screen.webm, screen.mov in order.
pub fn video_path(stitch_id: &ValidStitchId) -> Option<PathBuf> {
    let dir = attachments_dir(stitch_id).ok()?;
    for ext in &["mp4", "webm", "mov"] {
        let path = dir.join(format!("screen.{}", ext));
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// Load frame samples from frame_samples.json. Returns empty vec if not found.
pub fn load_frame_samples(stitch_id: &ValidStitchId) -> Vec<FrameSample> {
    let dir = match attachments_dir(stitch_id) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    let path = dir.join("frame_samples.json");
    if !path.exists() {
        return Vec::new();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default()
}

/// Load transcript from transcript.json. Returns None if not found.
pub fn load_transcript(stitch_id: &ValidStitchId) -> Option<ScreenCaptureTranscript> {
    let dir = attachments_dir(stitch_id).ok()?;
    let path = dir.join("transcript.json");
    if !path.exists() {
        return None;
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
}

/// Load meta.json sidecar. Returns a default if not found.
pub fn load_meta(stitch_id: &ValidStitchId) -> ScreenCaptureMeta {
    let dir = match attachments_dir(stitch_id) {
        Ok(d) => d,
        Err(_) => {
            return ScreenCaptureMeta {
                stitch_id: stitch_id.to_string(),
                project: String::new(),
                title: format!("Screen capture {}", stitch_id),
                recorded_at: String::new(),
                duration_secs: None,
            }
        }
    };
    let path = dir.join("meta.json");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_else(|| ScreenCaptureMeta {
            stitch_id: stitch_id.to_string(),
            project: String::new(),
            title: format!("Screen capture {}", stitch_id),
            recorded_at: String::new(),
            duration_secs: None,
        })
}

/// List all screen-capture stitches for a project by scanning the attachments directory.
///
/// Path-traversal protection: only reads from the pre-canonicalized `~/.hoop/attachments/`
/// directory. Stitch IDs found on disk are validated before use (defense-in-depth).
pub fn list_for_project(project: &str) -> Vec<ScreenCaptureSummary> {
    let mut base = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    base.push(".hoop");
    base.push("attachments");

    let Ok(entries) = std::fs::read_dir(&base) else {
        return Vec::new();
    };

    let mut results = Vec::new();
    for entry in entries.flatten() {
        let stitch_id_raw = entry.file_name().to_string_lossy().to_string();
        // Only consider directories that look like UUIDs (36-char lowercase)
        if stitch_id_raw.len() != 36 {
            continue;
        }
        // Validate the stitch_id before using it (defense-in-depth)
        let Ok(stitch_id) = ValidStitchId::parse(&stitch_id_raw) else {
            continue;
        };
        if !has_video(&stitch_id) {
            continue;
        }
        let meta = load_meta(&stitch_id);
        if !project.is_empty() && meta.project != project {
            continue;
        }
        let chapters = load_frame_samples(&stitch_id);
        let has_transcript = attachments_dir(&stitch_id)
            .ok()
            .map(|dir| dir.join("transcript.json").exists())
            .unwrap_or(false);
        results.push(ScreenCaptureSummary {
            stitch_id: stitch_id_raw,
            project: meta.project,
            title: meta.title,
            recorded_at: meta.recorded_at,
            duration_secs: meta.duration_secs,
            chapter_count: chapters.len(),
            has_transcript,
        });
    }

    // Sort newest first (recorded_at is ISO 8601 so lexicographic works)
    results.sort_by(|a, b| b.recorded_at.cmp(&a.recorded_at));
    results
}

/// Streaming upload session for screen captures
///
/// Used for real-time chunked upload during recording when the total
/// size is not known upfront.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct StreamingUploadSession {
    pub stream_id: String,
    pub stitch_id: String,
    pub project: String,
    pub video_content_type: String,
    pub file_extension: String,
    pub received_bytes: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Response when starting a streaming upload
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct StartStreamingUploadResponse {
    pub stream_id: String,
    pub stitch_id: String,
    pub upload_url: String,
    pub complete_url: String,
}

/// Streaming upload registry for managing active streaming sessions
pub struct StreamingUploadRegistry {
    streaming_dir: PathBuf,
    allowlist: PathAllowlist,
}

impl StreamingUploadRegistry {
    pub fn new() -> Result<Self> {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".hoop");
        home.push("streaming_uploads");
        let streaming_dir = home.clone();

        // Create directory if it doesn't exist
        fs::create_dir_all(&streaming_dir)
            .context("failed to create streaming uploads directory")?;

        let allowlist = PathAllowlist::for_uploads(&streaming_dir)?;

        Ok(Self {
            streaming_dir,
            allowlist,
        })
    }

    /// Get the directory for a streaming upload session
    fn stream_dir(&self, stream_id: &str) -> Result<PathBuf> {
        let path = self.streaming_dir.join(stream_id);
        canonicalize_and_check(&path, &self.allowlist)
            .context("stream directory failed path validation")
    }

    /// Get the metadata file path for a streaming session
    fn metadata_path(&self, stream_id: &str) -> Result<PathBuf> {
        Ok(self.stream_dir(stream_id)?.join("metadata.json"))
    }

    /// Get the partial video file path for a streaming session
    fn partial_path(&self, stream_id: &str) -> Result<PathBuf> {
        Ok(self.stream_dir(stream_id)?.join("partial.bin"))
    }

    /// Start a new streaming upload session
    pub fn start_session(
        &self,
        project: String,
        video_content_type: String,
    ) -> Result<StartStreamingUploadResponse> {
        let stitch_id = Uuid::new_v4().to_string();
        let stream_id = Uuid::new_v4().to_string();
        let valid_stitch_id =
            ValidStitchId::parse(&stitch_id).context("generated invalid stitch ID")?;

        // Determine file extension from content type
        let file_extension = if video_content_type.contains("mp4") {
            "mp4"
        } else if video_content_type.contains("mov") {
            "mov"
        } else {
            "webm"
        };

        // Create stream directory
        let stream_dir = self.streaming_dir.join(&stream_id);
        fs::create_dir_all(&stream_dir).context("failed to create stream directory")?;

        // Create the attachments directory for the final stitch
        let _attachments_dir = attachments_dir(&valid_stitch_id)?;

        // Create session metadata
        let now = chrono::Utc::now();
        let session = StreamingUploadSession {
            stream_id: stream_id.clone(),
            stitch_id: stitch_id.clone(),
            project,
            video_content_type,
            file_extension: file_extension.to_string(),
            received_bytes: 0,
            created_at: now,
            updated_at: now,
        };

        // Write metadata
        let metadata_path = stream_dir.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&session)
            .context("failed to serialize session metadata")?;
        atomic_write::atomic_write_file_str(&metadata_path, &metadata_json)
            .context("failed to write session metadata")?;

        // Create empty partial file
        let partial_path = stream_dir.join("partial.bin");
        atomic_write::atomic_write_file(&partial_path, b"")
            .context("failed to create partial file")?;

        Ok(StartStreamingUploadResponse {
            stream_id: stream_id.clone(),
            stitch_id,
            upload_url: format!("/api/p/{{project}}/screen-captures/stream/{}", stream_id),
            complete_url: format!(
                "/api/p/{{project}}/screen-captures/stream/{}/complete",
                stream_id
            ),
        })
    }

    /// Append a chunk to a streaming upload
    pub fn append_chunk(&self, stream_id: &str, data: &[u8]) -> Result<u64> {
        let partial_path = self.partial_path(stream_id)?;

        // Append to partial file
        let mut file = OpenOptions::new()
            .write(true)
            .open(&partial_path)
            .context("failed to open partial file")?;

        file.seek(SeekFrom::End(0))
            .context("failed to seek to end of partial file")?;
        file.write_all(data).context("failed to write chunk")?;
        file.sync_all().context("failed to sync chunk to disk")?;

        // Update metadata
        let new_size = file
            .metadata()
            .context("failed to get file metadata")?
            .len();

        let mut session = self.load_session(stream_id)?;
        session.received_bytes = new_size;
        session.updated_at = chrono::Utc::now();
        self.save_session(stream_id, &session)?;

        Ok(new_size)
    }

    /// Load a streaming upload session
    pub fn load_session(&self, stream_id: &str) -> Result<StreamingUploadSession> {
        let metadata_path = self.metadata_path(stream_id)?;
        let content = fs::read_to_string(&metadata_path)
            .with_context(|| format!("stream session not found: {}", stream_id))?;
        let session: StreamingUploadSession =
            serde_json::from_str(&content).context("failed to parse session metadata")?;
        Ok(session)
    }

    /// Save a streaming upload session
    fn save_session(&self, stream_id: &str, session: &StreamingUploadSession) -> Result<()> {
        let metadata_path = self.metadata_path(stream_id)?;
        let metadata_json = serde_json::to_string_pretty(session)
            .context("failed to serialize session metadata")?;
        atomic_write::atomic_write_file_str(&metadata_path, &metadata_json)
            .context("failed to write session metadata")?;
        Ok(())
    }

    /// Complete a streaming upload and move to final location
    pub fn complete_session(
        &self,
        stream_id: &str,
        duration_secs: f64,
        frame_samples: Vec<FrameSample>,
        state: &crate::DaemonState,
    ) -> Result<ScreenCaptureData> {
        let session = self.load_session(stream_id)?;
        let valid_stitch_id = ValidStitchId::parse(&session.stitch_id)?;

        // Move partial file to final location
        let partial_path = self.partial_path(stream_id)?;
        let attachments_dir = attachments_dir(&valid_stitch_id)?;
        let video_path = attachments_dir.join(format!("screen.{}", session.file_extension));

        // Atomic rename
        fs::rename(&partial_path, &video_path).with_context(|| {
            format!(
                "failed to move {} to {}",
                partial_path.display(),
                video_path.display()
            )
        })?;

        // Write frame_samples.json
        let frame_samples_path = attachments_dir.join("frame_samples.json");
        let frame_samples_json = serde_json::to_string_pretty(&frame_samples)
            .context("failed to serialize frame samples")?;
        atomic_write::atomic_write_file_str(&frame_samples_path, &frame_samples_json)
            .context("failed to write frame samples")?;

        // §18.1 secrets scan: screen capture text (frame labels)
        {
            let frame_labels: String = frame_samples
                .iter()
                .map(|f| f.label.as_str())
                .collect::<Vec<_>>()
                .join(" ");

            let findings = crate::redaction::scan_screen_capture_text(&frame_labels);
            if !findings.is_empty() {
                tracing::warn!(
                    stitch_id = %session.stitch_id,
                    project = %session.project,
                    findings = findings.len(),
                    "Screen capture frame labels contain potential secrets — flagged for operator review (§18.1)"
                );
                crate::redaction::audit_findings(
                    "screen_capture",
                    &findings,
                    crate::redaction_policy::RedactionAction::FlaggedOnly,
                    &session.stitch_id,
                    Some(&session.project),
                    "system",
                );
            }
        }

        // Write meta.json
        let now = chrono::Utc::now();
        let title = format!("Screen capture {}", now.format("%Y-%m-%d %H:%M"));
        let meta = ScreenCaptureMeta {
            stitch_id: session.stitch_id.clone(),
            project: session.project.clone(),
            title: title.clone(),
            recorded_at: now.to_rfc3339(),
            duration_secs: Some(duration_secs),
        };
        let meta_path = attachments_dir.join("meta.json");
        let meta_json =
            serde_json::to_string_pretty(&meta).context("failed to serialize metadata")?;
        atomic_write::atomic_write_file_str(&meta_path, &meta_json)
            .context("failed to write metadata")?;

        // Insert into fleet.db
        let db_path = crate::fleet::db_path();
        let conn = rusqlite::Connection::open(&db_path).context("failed to open database")?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .context("failed to set WAL mode")?;

        conn.execute(
            "INSERT INTO stitches (id, project, kind, title, created_by, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                &session.stitch_id,
                &session.project,
                "screen-capture",
                &title,
                "operator",
                now.to_rfc3339(),
            ],
        ).context("failed to create stitch row")?;

        // Evaluate pattern queries for the new stitch
        if let Err(e) = crate::pattern_query_evaluator::sync_and_emit_pattern_queries(
            &session.stitch_id,
            &session.project,
            "screen-capture",
            &title,
            &state.pattern_tx,
        ) {
            tracing::warn!(
                "Failed to sync pattern queries for stitch {}: {}",
                session.stitch_id,
                e
            );
        }

        tracing::info!(
            "Completed streaming screen capture {} in project {} (duration: {:.1}s, size: {} bytes)",
            session.stitch_id,
            session.project,
            duration_secs,
            session.received_bytes
        );

        // Clean up streaming directory
        let stream_dir = self.stream_dir(stream_id)?;
        fs::remove_dir_all(&stream_dir).context("failed to clean up stream directory")?;

        Ok(ScreenCaptureData {
            video_url: format!("/api/screen-capture/{}/video", session.stitch_id),
            stitch_id: session.stitch_id,
            title,
            project: session.project,
            recorded_at: now.to_rfc3339(),
            duration_secs: Some(duration_secs),
            chapters: frame_samples,
            transcript: None,
        })
    }

    /// Cancel and cleanup a streaming upload session
    pub fn cancel_session(&self, stream_id: &str) -> Result<()> {
        let stream_dir = self.stream_dir(stream_id)?;
        if stream_dir.exists() {
            fs::remove_dir_all(&stream_dir).context("failed to remove stream directory")?;
        }
        Ok(())
    }
}

impl Default for StreamingUploadRegistry {
    fn default() -> Self {
        Self::new().expect("failed to create streaming upload registry")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_capture_meta_default() {
        let meta = ScreenCaptureMeta {
            stitch_id: "test-stitch".to_string(),
            project: String::new(),
            title: String::new(),
            recorded_at: String::new(),
            duration_secs: None,
        };

        assert_eq!(meta.stitch_id, "test-stitch");
        assert!(meta.project.is_empty());
        assert!(meta.title.is_empty());
        assert!(meta.duration_secs.is_none());
    }

    #[test]
    fn test_screen_capture_meta_serialization() {
        let meta = ScreenCaptureMeta {
            stitch_id: "st-123".to_string(),
            project: "test-project".to_string(),
            title: "Test Capture".to_string(),
            recorded_at: "2026-04-29T12:00:00Z".to_string(),
            duration_secs: Some(60.5),
        };

        let json = serde_json::to_string(&meta).unwrap();
        assert!(json.contains("st-123"));
        assert!(json.contains("test-project"));
        assert!(json.contains("60.5"));

        let parsed: ScreenCaptureMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.stitch_id, meta.stitch_id);
        assert_eq!(parsed.project, meta.project);
        assert_eq!(parsed.duration_secs, meta.duration_secs);
    }

    #[test]
    fn test_frame_sample_serialization() {
        let sample = FrameSample {
            timestamp_secs: 10.5,
            label: "Chapter 1".to_string(),
        };

        let json = serde_json::to_string(&sample).unwrap();
        assert!(json.contains("10.5"));
        assert!(json.contains("Chapter 1"));

        let parsed: FrameSample = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.timestamp_secs, 10.5);
        assert_eq!(parsed.label, "Chapter 1");
    }

    #[test]
    fn test_transcript_word_serialization() {
        let word = TranscriptWord {
            word: "hello".to_string(),
            start: 0.0,
            end: 0.5,
        };

        let json = serde_json::to_string(&word).unwrap();
        assert!(json.contains("hello"));

        let parsed: TranscriptWord = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.word, "hello");
        assert_eq!(parsed.start, 0.0);
        assert_eq!(parsed.end, 0.5);
    }

    #[test]
    fn test_screen_capture_transcript_serialization() {
        let transcript = ScreenCaptureTranscript {
            text: "hello world".to_string(),
            words: vec![
                TranscriptWord {
                    word: "hello".to_string(),
                    start: 0.0,
                    end: 0.5,
                },
                TranscriptWord {
                    word: "world".to_string(),
                    start: 0.5,
                    end: 1.0,
                },
            ],
        };

        let json = serde_json::to_string(&transcript).unwrap();
        assert!(json.contains("hello world"));

        let parsed: ScreenCaptureTranscript = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.text, "hello world");
        assert_eq!(parsed.words.len(), 2);
    }

    #[test]
    fn test_screen_capture_summary_serialization() {
        let summary = ScreenCaptureSummary {
            stitch_id: "st-123".to_string(),
            project: "test-project".to_string(),
            title: "Test".to_string(),
            recorded_at: "2026-04-29T12:00:00Z".to_string(),
            duration_secs: Some(60.0),
            chapter_count: 5,
            has_transcript: true,
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("st-123"));
        assert!(json.contains("true"));

        let parsed: ScreenCaptureSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.chapter_count, 5);
        assert!(parsed.has_transcript);
    }

    #[test]
    fn test_screen_capture_data_serialization() {
        let data = ScreenCaptureData {
            stitch_id: "st-123".to_string(),
            title: "Test".to_string(),
            project: "test-project".to_string(),
            recorded_at: "2026-04-29T12:00:00Z".to_string(),
            video_url: "http://example.com/video.mp4".to_string(),
            duration_secs: Some(60.0),
            chapters: vec![],
            transcript: None,
        };

        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("video_url"));

        let parsed: ScreenCaptureData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.video_url, "http://example.com/video.mp4");
    }
}
