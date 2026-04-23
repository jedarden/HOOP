//! Attachment storage layout
//!
//! Bead attachments: `<workspace>/.beads/attachments/<bead-id>/<filename>`
//! Stitch/Note attachments: `~/.hoop/attachments/<stitch-id>/<filename>`
//!
//! Directories are created lazily on first attach. All paths are canonicalized
//! after creation and prefix-checked against the expected root to prevent
//! path-traversal attacks (§13 Security).

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

// ── Size limits ──────────────────────────────────────────────────────────────

/// Per-type size limits for attachments.  All values are in bytes.
#[derive(Debug, Clone, Copy)]
pub struct SizeLimits {
    pub image_bytes: u64,
    pub audio_bytes: u64,
    pub video_bytes: u64,
    pub pdf_bytes: u64,
}

impl Default for SizeLimits {
    fn default() -> Self {
        Self {
            image_bytes: 50 * 1024 * 1024,   //  50 MB
            audio_bytes: 100 * 1024 * 1024,  // 100 MB
            video_bytes: 500 * 1024 * 1024,  // 500 MB
            pdf_bytes: 50 * 1024 * 1024,     //  50 MB
        }
    }
}

// ── Attachment kind ──────────────────────────────────────────────────────────

/// MIME-based attachment kind, detected from magic bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachmentKind {
    Image,
    Audio,
    Video,
    Pdf,
}

impl AttachmentKind {
    /// Detect kind from a file's leading bytes. Returns `None` for unknown types.
    pub fn from_magic(data: &[u8]) -> Option<Self> {
        // JPEG
        if data.starts_with(b"\xff\xd8\xff") {
            return Some(Self::Image);
        }
        // PNG
        if data.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Some(Self::Image);
        }
        // GIF
        if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            return Some(Self::Image);
        }
        // WebP: RIFF....WEBP
        if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
            return Some(Self::Image);
        }
        // PDF
        if data.starts_with(b"%PDF-") {
            return Some(Self::Pdf);
        }
        // MP3: ID3v2 tag or MPEG sync
        if data.starts_with(b"ID3")
            || (data.len() >= 2 && data[0] == 0xff && (data[1] & 0xe0) == 0xe0)
        {
            return Some(Self::Audio);
        }
        // OGG container (Vorbis, Opus, FLAC)
        if data.starts_with(b"OggS") {
            return Some(Self::Audio);
        }
        // FLAC
        if data.starts_with(b"fLaC") {
            return Some(Self::Audio);
        }
        // WAVE
        if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WAVE" {
            return Some(Self::Audio);
        }
        // ISO Base Media (MP4/M4A/MOV) — check for ftyp box at offset 4
        if data.len() >= 8 && &data[4..8] == b"ftyp" {
            return Some(Self::Video);
        }
        // WebM / Matroska (EBML header)
        if data.starts_with(b"\x1a\x45\xdf\xa3") {
            return Some(Self::Video);
        }
        // AVI: RIFF....AVI
        if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"AVI " {
            return Some(Self::Video);
        }
        None
    }

    /// Maximum allowed byte count for this kind, given a limit configuration.
    pub fn size_limit(self, limits: &SizeLimits) -> u64 {
        match self {
            Self::Image => limits.image_bytes,
            Self::Audio => limits.audio_bytes,
            Self::Video => limits.video_bytes,
            Self::Pdf => limits.pdf_bytes,
        }
    }
}

// ── Input validation ─────────────────────────────────────────────────────────

/// Validate a bead ID.
///
/// Bead IDs produced by `br` look like `hoop-ttb.4.12` — alphanumeric, hyphens,
/// dots, underscores, not starting with a hyphen or dot.
fn is_valid_bead_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 256
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_')
        && !id.starts_with('-')
        && !id.starts_with('.')
}

/// Validate a stitch ID.
///
/// Stitch IDs are lowercase UUID v4 strings: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`.
fn is_valid_stitch_id(id: &str) -> bool {
    let b = id.as_bytes();
    if b.len() != 36 {
        return false;
    }
    let dashes = [8, 13, 18, 23];
    for (i, &byte) in b.iter().enumerate() {
        if dashes.contains(&i) {
            if byte != b'-' {
                return false;
            }
        } else if !byte.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

/// Validate an attachment filename.
///
/// Must be a flat name (no path separators), non-empty, ≤255 bytes, no null bytes,
/// and not the special names `.` or `..`.
fn is_valid_filename(filename: &str) -> bool {
    !filename.is_empty()
        && filename.len() <= 255
        && filename != "."
        && filename != ".."
        && !filename.contains('/')
        && !filename.contains('\\')
        && !filename.contains('\0')
}

// ── Directory helpers ─────────────────────────────────────────────────────────

/// Return (and lazily create) the attachment directory for a bead.
///
/// Path: `<workspace>/.beads/attachments/<bead-id>/`
pub fn bead_attachment_dir(workspace: &Path, bead_id: &str) -> Result<PathBuf> {
    if !is_valid_bead_id(bead_id) {
        anyhow::bail!("invalid bead id: {:?}", bead_id);
    }
    let dir = workspace
        .join(".beads")
        .join("attachments")
        .join(bead_id);
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create bead attachment dir: {}", dir.display()))?;

    let canonical = dir
        .canonicalize()
        .with_context(|| format!("failed to canonicalize: {}", dir.display()))?;

    // Prefix guard: resolved path must be under <workspace>/.beads/attachments/
    let prefix = workspace
        .join(".beads")
        .join("attachments")
        .canonicalize()
        .context("failed to canonicalize bead attachments prefix")?;
    if !canonical.starts_with(&prefix) {
        anyhow::bail!("path traversal detected for bead id: {:?}", bead_id);
    }

    Ok(canonical)
}

/// Return (and lazily create) the attachment directory for a stitch/Note.
///
/// Path: `~/.hoop/attachments/<stitch-id>/`
pub fn stitch_attachment_dir(stitch_id: &str) -> Result<PathBuf> {
    if !is_valid_stitch_id(stitch_id) {
        anyhow::bail!("invalid stitch id: {:?}", stitch_id);
    }
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("home directory not found"))?;
    let dir = home.join(".hoop").join("attachments").join(stitch_id);
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create stitch attachment dir: {}", dir.display()))?;

    let canonical = dir
        .canonicalize()
        .with_context(|| format!("failed to canonicalize: {}", dir.display()))?;

    let prefix = home
        .join(".hoop")
        .join("attachments")
        .canonicalize()
        .context("failed to canonicalize stitch attachments prefix")?;
    if !canonical.starts_with(&prefix) {
        anyhow::bail!("path traversal detected for stitch id: {:?}", stitch_id);
    }

    Ok(canonical)
}

// ── Path resolution ───────────────────────────────────────────────────────────

/// Resolve the destination path for a bead attachment file.
///
/// The directory is created lazily. Returns an error if the bead ID or filename
/// fails validation.
pub fn bead_attachment_path(
    workspace: &Path,
    bead_id: &str,
    filename: &str,
) -> Result<PathBuf> {
    if !is_valid_filename(filename) {
        anyhow::bail!("invalid attachment filename: {:?}", filename);
    }
    let dir = bead_attachment_dir(workspace, bead_id)?;
    let dest = dir.join(filename);

    // Defense-in-depth: the parent of the resolved path must equal the dir.
    let parent = dest
        .parent()
        .ok_or_else(|| anyhow::anyhow!("attachment path has no parent"))?;
    if parent.canonicalize().ok().as_deref() != Some(dir.as_path()) {
        anyhow::bail!("path traversal detected in filename: {:?}", filename);
    }

    Ok(dest)
}

/// Resolve the destination path for a stitch attachment file.
pub fn stitch_attachment_path(stitch_id: &str, filename: &str) -> Result<PathBuf> {
    if !is_valid_filename(filename) {
        anyhow::bail!("invalid attachment filename: {:?}", filename);
    }
    let dir = stitch_attachment_dir(stitch_id)?;
    let dest = dir.join(filename);

    let parent = dest
        .parent()
        .ok_or_else(|| anyhow::anyhow!("attachment path has no parent"))?;
    if parent.canonicalize().ok().as_deref() != Some(dir.as_path()) {
        anyhow::bail!("path traversal detected in filename: {:?}", filename);
    }

    Ok(dest)
}

// ── Storage ───────────────────────────────────────────────────────────────────

/// Store bytes as a bead attachment using an atomic write (tmp → rename).
///
/// The attachment kind is inferred from magic bytes and checked against
/// `limits`. Returns the canonical destination path.
pub fn store_bead_attachment(
    workspace: &Path,
    bead_id: &str,
    filename: &str,
    data: &[u8],
    limits: &SizeLimits,
) -> Result<PathBuf> {
    check_size(data, limits)?;
    let dest = bead_attachment_path(workspace, bead_id, filename)?;
    write_atomic(&dest, data)?;
    Ok(dest)
}

/// Store bytes as a stitch attachment using an atomic write (tmp → rename).
pub fn store_stitch_attachment(
    stitch_id: &str,
    filename: &str,
    data: &[u8],
    limits: &SizeLimits,
) -> Result<PathBuf> {
    check_size(data, limits)?;
    let dest = stitch_attachment_path(stitch_id, filename)?;
    write_atomic(&dest, data)?;
    Ok(dest)
}

/// Enforce size limit if the kind is recognisable; unknown types are uncapped.
fn check_size(data: &[u8], limits: &SizeLimits) -> Result<()> {
    if let Some(kind) = AttachmentKind::from_magic(data) {
        let max = kind.size_limit(limits);
        if data.len() as u64 > max {
            anyhow::bail!(
                "attachment too large: {} bytes exceeds {} byte limit for {:?}",
                data.len(),
                max,
                kind
            );
        }
    }
    Ok(())
}

/// Atomic write: write to a uniquely-named `.tmp` sibling, then rename into place.
fn write_atomic(dest: &Path, data: &[u8]) -> Result<()> {
    let tmp_name = format!(
        "{}.{}.tmp",
        dest.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("attachment"),
        uuid::Uuid::new_v4()
    );
    let tmp = dest
        .parent()
        .ok_or_else(|| anyhow::anyhow!("dest path has no parent"))?
        .join(tmp_name);

    std::fs::write(&tmp, data)
        .with_context(|| format!("failed to write tmp file: {}", tmp.display()))?;
    std::fs::rename(&tmp, dest)
        .with_context(|| format!("failed to rename {} -> {}", tmp.display(), dest.display()))?;

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ── Validation ────────────────────────────────────────────────────────────

    #[test]
    fn bead_id_valid() {
        assert!(is_valid_bead_id("hoop-ttb.4.12"));
        assert!(is_valid_bead_id("abc"));
        assert!(is_valid_bead_id("a1_b2-c3.d4"));
    }

    #[test]
    fn bead_id_invalid() {
        assert!(!is_valid_bead_id(""));
        assert!(!is_valid_bead_id("-starts-with-dash"));
        assert!(!is_valid_bead_id(".starts-with-dot"));
        assert!(!is_valid_bead_id("has/slash"));
        assert!(!is_valid_bead_id("has space"));
        assert!(!is_valid_bead_id(&"x".repeat(257)));
    }

    #[test]
    fn stitch_id_valid() {
        assert!(is_valid_stitch_id(
            "550e8400-e29b-41d4-a716-446655440000"
        ));
        assert!(is_valid_stitch_id(
            "00000000-0000-0000-0000-000000000000"
        ));
    }

    #[test]
    fn stitch_id_invalid() {
        assert!(!is_valid_stitch_id(""));
        assert!(!is_valid_stitch_id("not-a-uuid"));
        assert!(!is_valid_stitch_id(
            "550e8400-e29b-41d4-a716-44665544000g" // 'g' not hex
        ));
        assert!(!is_valid_stitch_id(
            "550e8400e29b41d4a716446655440000" // no dashes
        ));
        assert!(!is_valid_stitch_id(
            "550e8400-e29b-41d4-a716-4466554400" // too short
        ));
    }

    #[test]
    fn filename_valid() {
        assert!(is_valid_filename("image.png"));
        assert!(is_valid_filename("recording.mp3"));
        assert!(is_valid_filename("document.pdf"));
        assert!(is_valid_filename("a"));
    }

    #[test]
    fn filename_invalid() {
        assert!(!is_valid_filename(""));
        assert!(!is_valid_filename("."));
        assert!(!is_valid_filename(".."));
        assert!(!is_valid_filename("foo/bar.png"));
        assert!(!is_valid_filename("foo\\bar.png"));
        assert!(!is_valid_filename("nul\0byte"));
        assert!(!is_valid_filename(&"x".repeat(256)));
    }

    // ── Magic bytes / AttachmentKind ──────────────────────────────────────────

    #[test]
    fn magic_jpeg() {
        let jpeg = b"\xff\xd8\xff\xe0\x00\x10JFIF";
        assert_eq!(AttachmentKind::from_magic(jpeg), Some(AttachmentKind::Image));
    }

    #[test]
    fn magic_png() {
        let png = b"\x89PNG\r\n\x1a\n\x00\x00";
        assert_eq!(AttachmentKind::from_magic(png), Some(AttachmentKind::Image));
    }

    #[test]
    fn magic_pdf() {
        let pdf = b"%PDF-1.4 header";
        assert_eq!(AttachmentKind::from_magic(pdf), Some(AttachmentKind::Pdf));
    }

    #[test]
    fn magic_mp3_id3() {
        let mp3 = b"ID3\x03\x00\x00rest";
        assert_eq!(AttachmentKind::from_magic(mp3), Some(AttachmentKind::Audio));
    }

    #[test]
    fn magic_ogg() {
        let ogg = b"OggS\x00rest";
        assert_eq!(AttachmentKind::from_magic(ogg), Some(AttachmentKind::Audio));
    }

    #[test]
    fn magic_mp4() {
        // ftyp box at offset 4
        let mut mp4 = [0u8; 12];
        mp4[4..8].copy_from_slice(b"ftyp");
        mp4[8..12].copy_from_slice(b"isom");
        assert_eq!(
            AttachmentKind::from_magic(&mp4),
            Some(AttachmentKind::Video)
        );
    }

    #[test]
    fn magic_webm() {
        let webm = b"\x1a\x45\xdf\xa3rest";
        assert_eq!(
            AttachmentKind::from_magic(webm),
            Some(AttachmentKind::Video)
        );
    }

    #[test]
    fn magic_unknown() {
        assert_eq!(AttachmentKind::from_magic(b"random bytes"), None);
    }

    // ── Size limits ───────────────────────────────────────────────────────────

    #[test]
    fn size_limit_defaults() {
        let limits = SizeLimits::default();
        assert_eq!(AttachmentKind::Audio.size_limit(&limits), 100 * 1024 * 1024);
        assert_eq!(AttachmentKind::Video.size_limit(&limits), 500 * 1024 * 1024);
        assert_eq!(AttachmentKind::Image.size_limit(&limits), 50 * 1024 * 1024);
        assert_eq!(AttachmentKind::Pdf.size_limit(&limits), 50 * 1024 * 1024);
    }

    #[test]
    fn check_size_rejects_oversized_audio() {
        let limits = SizeLimits {
            audio_bytes: 10,
            ..Default::default()
        };
        // Fake an OGG magic so the kind is detected
        let mut data = vec![0u8; 20];
        data[..4].copy_from_slice(b"OggS");
        assert!(check_size(&data, &limits).is_err());
    }

    #[test]
    fn check_size_allows_unknown_type() {
        let limits = SizeLimits {
            audio_bytes: 1,
            video_bytes: 1,
            image_bytes: 1,
            pdf_bytes: 1,
        };
        // Unknown magic — no kind detected, no limit applied
        assert!(check_size(b"random bytes that are larger than 1 byte", &limits).is_ok());
    }

    // ── Bead attachment directory ─────────────────────────────────────────────

    #[test]
    fn bead_attachment_dir_created_lazily() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let dir = bead_attachment_dir(ws, "test-bead.1.0").unwrap();
        assert!(dir.is_dir());
        assert!(dir.ends_with("test-bead.1.0"));
    }

    #[test]
    fn bead_attachment_dir_idempotent() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let dir1 = bead_attachment_dir(ws, "my-bead.1").unwrap();
        let dir2 = bead_attachment_dir(ws, "my-bead.1").unwrap();
        assert_eq!(dir1, dir2);
    }

    #[test]
    fn bead_attachment_dir_rejects_invalid_id() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        assert!(bead_attachment_dir(ws, "").is_err());
        assert!(bead_attachment_dir(ws, "-bad").is_err());
        assert!(bead_attachment_dir(ws, "../escape").is_err());
    }

    #[test]
    fn bead_attachment_path_rejects_bad_filename() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        assert!(bead_attachment_path(ws, "bead.1", "..").is_err());
        assert!(bead_attachment_path(ws, "bead.1", "sub/dir.png").is_err());
        assert!(bead_attachment_path(ws, "bead.1", "").is_err());
    }

    #[test]
    fn bead_attachment_path_ok() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let p = bead_attachment_path(ws, "bead.1", "image.png").unwrap();
        // Should end with bead.1/image.png
        assert!(p.ends_with("image.png"));
        assert!(p.parent().unwrap().ends_with("bead.1"));
    }

    // ── Stitch attachment directory ───────────────────────────────────────────

    #[test]
    fn stitch_id_format_gate() {
        assert!(stitch_attachment_dir("not-a-uuid").is_err());
        assert!(stitch_attachment_dir("").is_err());
    }

    // ── store_bead_attachment ─────────────────────────────────────────────────

    #[test]
    fn store_bead_attachment_writes_file() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let data = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR"; // PNG header stub
        let limits = SizeLimits::default();
        let dest = store_bead_attachment(ws, "bead.1", "test.png", data, &limits).unwrap();
        assert!(dest.exists());
        assert_eq!(std::fs::read(&dest).unwrap(), data);
    }

    #[test]
    fn store_bead_attachment_rejects_oversized_video() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let limits = SizeLimits {
            video_bytes: 4,
            ..Default::default()
        };
        // WebM magic + extra bytes to exceed the limit
        let mut data = vec![0u8; 10];
        data[..4].copy_from_slice(b"\x1a\x45\xdf\xa3");
        assert!(store_bead_attachment(ws, "bead.1", "clip.webm", &data, &limits).is_err());
    }

    #[test]
    fn store_bead_attachment_atomic_no_tmp_left() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let data = b"%PDF-1.4 minimal";
        let limits = SizeLimits::default();
        let dest = store_bead_attachment(ws, "bead.1", "doc.pdf", data, &limits).unwrap();
        let parent = dest.parent().unwrap();
        // No stale .tmp files should remain
        let tmps: Vec<_> = std::fs::read_dir(parent)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .contains(".tmp")
            })
            .collect();
        assert!(tmps.is_empty(), "stale tmp files left: {:?}", tmps);
    }
}
