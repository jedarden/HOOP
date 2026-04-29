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
