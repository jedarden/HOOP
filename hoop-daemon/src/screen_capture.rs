//! Screen capture stitch support
//!
//! Reads frame_samples.json and the video file from the stitch attachments directory.
//! Screen captures are stored at ~/.hoop/attachments/<stitch_id>/:
//!   screen.{mp4,webm,mov}   — the video recording
//!   frame_samples.json      — chapter markers derived from UI-change frame samples
//!   transcript.json         — word-level Whisper transcript from the audio track
//!   meta.json               — stitch metadata (title, project, recorded_at)
//!
//! All functions accept `ValidStitchId` for compile-time path-traversal protection (§13).
//! Paths are canonicalized and prefix-checked against an allowlist (§13, §K2).

use crate::id_validators::ValidStitchId;
use crate::path_security::{canonicalize_and_check, PathAllowlist};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A frame sample representing a UI change captured during screen recording.
/// These become chapter markers in the video player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameSample {
    pub timestamp_secs: f64,
    pub label: String,
}

/// Word-level transcript entry (from Whisper)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptWord {
    pub word: String,
    pub start: f64,
    pub end: f64,
}

/// Transcript with word-level timestamps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenCaptureTranscript {
    pub text: String,
    pub words: Vec<TranscriptWord>,
}

/// Sidecar metadata written alongside the video file
#[derive(Debug, Clone, Serialize, Deserialize)]
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
