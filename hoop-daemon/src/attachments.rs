//! Attachment storage layout
//!
//! Bead attachments: `<workspace>/.beads/attachments/<bead-id>/<filename>`
//! Stitch/Note attachments: `~/.hoop/attachments/<stitch-id>/<filename>`
//!
//! Directories are created lazily on first attach. All paths are canonicalized
//! after creation and prefix-checked against the expected root to prevent
//! path-traversal attacks (§13 Security).

use crate::id_validators::{ValidBeadId, ValidStitchId};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

// ── Content-type sniffing (§13) ────────────────────────────────────────────────

/// Sniff the MIME type from leading bytes using the `infer` crate.
pub fn sniff_mime(data: &[u8]) -> Option<String> {
    infer::get(data).map(|t| t.mime_type().to_string())
}

/// Map a file extension (without leading dot) to its expected MIME type.
pub fn extension_to_mime(ext: &str) -> Option<&'static str> {
    match ext.to_lowercase().as_str() {
        // Images
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "svg" => Some("image/svg+xml"),
        "bmp" => Some("image/bmp"),
        "ico" => Some("image/x-icon"),
        "tiff" | "tif" => Some("image/tiff"),
        // Audio
        "mp3" => Some("audio/mpeg"),
        "wav" => Some("audio/wav"),
        "ogg" | "oga" => Some("audio/ogg"),
        "flac" => Some("audio/flac"),
        "aac" => Some("audio/aac"),
        "m4a" => Some("audio/mp4"),
        "wma" => Some("audio/x-ms-wma"),
        "opus" => Some("audio/opus"),
        // Video
        "mp4" | "m4v" => Some("video/mp4"),
        "webm" => Some("video/webm"),
        "mov" => Some("video/quicktime"),
        "avi" => Some("video/x-msvideo"),
        "mkv" => Some("video/x-matroska"),
        "wmv" => Some("video/x-ms-wmv"),
        "flv" => Some("video/x-flv"),
        // Documents
        "pdf" => Some("application/pdf"),
        // Archives
        "zip" => Some("application/zip"),
        "gz" | "gzip" => Some("application/gzip"),
        "tar" => Some("application/x-tar"),
        "rar" => Some("application/vnd.rar"),
        "7z" => Some("application/x-7z-compressed"),
        _ => None,
    }
}

/// Record of SVG sanitization, embedded in the sidecar `.meta.json` when the
/// sanitizer modified the file.  Present only on the *sanitized* copy; absent
/// on the `_unsafe` original copy.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SvgSanitizeRecord {
    /// Filename of the preserved unsafe original (e.g. `diagram_unsafe.svg`).
    pub unsafe_filename: String,
    /// Element names that were removed (e.g. `["script", "foreignObject"]`).
    pub removed_elements: Vec<String>,
    /// Attribute descriptions that were removed (e.g. `["onclick", "href=http://…"]`).
    pub removed_attrs: Vec<String>,
}

/// Record of PDF sanitization, embedded in the sidecar `.meta.json` when the
/// sanitizer modified the file.  Present only on the *sanitized* copy; absent
/// on the `_unsafe` original copy.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PdfSanitizeRecord {
    /// Filename of the preserved unsafe original (e.g. `report_unsafe.pdf`).
    pub unsafe_filename: String,
    /// Threats that were neutralised (e.g. `["/JS string", "JavaScript action type"]`).
    pub removed_threats: Vec<String>,
}

/// Metadata persisted alongside an attachment recording both declared and sniffed types.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttachmentMetadata {
    /// Original filename as declared by the uploader.
    pub filename: String,
    /// Extension extracted from the filename.
    pub declared_extension: String,
    /// MIME type expected from the declared extension.
    pub declared_mime: Option<String>,
    /// MIME type sniffed from magic bytes via `infer`.
    pub sniffed_mime: Option<String>,
    /// Timestamp when the metadata was verified and written.
    pub verified_at: chrono::DateTime<chrono::Utc>,
    /// SVG sanitization record, present only when an SVG was modified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svg_sanitize: Option<SvgSanitizeRecord>,
    /// PDF sanitization record, present only when a PDF was modified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_sanitize: Option<PdfSanitizeRecord>,
}

/// Validate that the declared filename extension matches the actual content.
///
/// Returns attachment metadata on success, or an error on mismatch.
pub fn validate_content_match(filename: &str, data: &[u8]) -> Result<AttachmentMetadata> {
    let ext = filename.rsplit('.').next().unwrap_or("").to_string();
    let declared_mime = extension_to_mime(&ext).map(|s| s.to_string());
    let sniffed_mime = sniff_mime(data);

    if let (Some(ref declared), Some(ref sniffed)) = (&declared_mime, &sniffed_mime) {
        if declared != sniffed {
            anyhow::bail!(
                "content-type mismatch: file claims to be {:?} ({}) but actual content is {}",
                ext,
                declared,
                sniffed
            );
        }
    }

    Ok(AttachmentMetadata {
        filename: filename.to_string(),
        declared_extension: ext,
        declared_mime,
        sniffed_mime,
        verified_at: chrono::Utc::now(),
        svg_sanitize: None,
        pdf_sanitize: None,
    })
}

/// Build the filename used for the preserved unsafe copy of a sanitized SVG.
///
/// `diagram.svg` → `diagram_unsafe.svg`
/// `ICON.SVG`    → `ICON_unsafe.SVG`
pub fn make_unsafe_svg_filename(filename: &str) -> String {
    if let Some(stem) = filename.strip_suffix(".svg") {
        // Avoid double-marking an already-unsafe copy.
        if stem.to_ascii_lowercase().ends_with("_unsafe") {
            return filename.to_owned();
        }
        format!("{stem}_unsafe.svg")
    } else if let Some(stem) = filename.strip_suffix(".SVG") {
        if stem.to_ascii_lowercase().ends_with("_unsafe") {
            return filename.to_owned();
        }
        format!("{stem}_unsafe.SVG")
    } else {
        format!("{filename}_unsafe")
    }
}

/// Build the filename used for the preserved unsafe copy of a sanitized PDF.
///
/// `report.pdf` → `report_unsafe.pdf`
/// `REPORT.PDF` → `REPORT_unsafe.PDF`
pub fn make_unsafe_pdf_filename(filename: &str) -> String {
    for suffix in &[".pdf", ".PDF"] {
        if let Some(stem) = filename.strip_suffix(suffix) {
            if stem.to_ascii_lowercase().ends_with("_unsafe") {
                return filename.to_owned();
            }
            return format!("{stem}_unsafe{suffix}");
        }
    }
    format!("{filename}_unsafe")
}

/// Write attachment metadata as a sidecar `<filename>.meta.json` file.
pub fn write_attachment_meta(dest: &Path, meta: &AttachmentMetadata) -> Result<()> {
    let meta_name = format!(
        "{}.meta.json",
        dest.file_name().and_then(|n| n.to_str()).unwrap_or("attachment")
    );
    let meta_path = dest.parent()
        .ok_or_else(|| anyhow::anyhow!("attachment path has no parent"))?
        .join(meta_name);
    let json = serde_json::to_string_pretty(meta)?;
    std::fs::write(&meta_path, json)
        .with_context(|| format!("failed to write attachment metadata: {}", meta_path.display()))?;
    Ok(())
}

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
///
/// Accepts a `ValidBeadId` to enforce compile-time proof that the ID has been
/// validated before reaching any filesystem path construction.
pub fn bead_attachment_dir(workspace: &Path, bead_id: &ValidBeadId) -> Result<PathBuf> {
    let dir = workspace
        .join(".beads")
        .join("attachments")
        .join(bead_id.as_str());
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
///
/// Accepts a `ValidStitchId` to enforce compile-time proof that the ID has been
/// validated before reaching any filesystem path construction.
pub fn stitch_attachment_dir(stitch_id: &ValidStitchId) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("home directory not found"))?;
    let dir = home.join(".hoop").join("attachments").join(stitch_id.as_str());
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
/// The directory is created lazily. Returns an error if the filename
/// fails validation.
pub fn bead_attachment_path(
    workspace: &Path,
    bead_id: &ValidBeadId,
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
pub fn stitch_attachment_path(stitch_id: &ValidStitchId, filename: &str) -> Result<PathBuf> {
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
/// `limits`. Content-type is sniffed and validated against the declared
/// extension (§13).  SVG and PDF files are sanitized (§13) before storage;
/// the original unsafe copy is also retained when modified.
/// Returns the canonical destination path.
pub fn store_bead_attachment(
    workspace: &Path,
    bead_id: &ValidBeadId,
    filename: &str,
    data: &[u8],
    limits: &SizeLimits,
) -> Result<PathBuf> {
    check_size(data, limits)?;
    let meta = validate_content_match(filename, data)?;

    if meta.declared_extension.eq_ignore_ascii_case("svg") {
        return store_svg_with_sanitization(data, filename, meta, |name| {
            bead_attachment_path(workspace, bead_id, name)
        });
    }

    if meta.declared_extension.eq_ignore_ascii_case("pdf") {
        return store_pdf_with_sanitization(data, filename, meta, |name| {
            bead_attachment_path(workspace, bead_id, name)
        });
    }

    let dest = bead_attachment_path(workspace, bead_id, filename)?;
    write_atomic(&dest, data)?;
    write_attachment_meta(&dest, &meta)?;
    Ok(dest)
}

/// Store bytes as a stitch attachment using an atomic write (tmp → rename).
///
/// Content-type is sniffed and validated against the declared extension (§13).
/// SVG and PDF files are sanitized before storage.
pub fn store_stitch_attachment(
    stitch_id: &ValidStitchId,
    filename: &str,
    data: &[u8],
    limits: &SizeLimits,
) -> Result<PathBuf> {
    check_size(data, limits)?;
    let meta = validate_content_match(filename, data)?;

    if meta.declared_extension.eq_ignore_ascii_case("svg") {
        return store_svg_with_sanitization(data, filename, meta, |name| {
            stitch_attachment_path(stitch_id, name)
        });
    }

    if meta.declared_extension.eq_ignore_ascii_case("pdf") {
        return store_pdf_with_sanitization(data, filename, meta, |name| {
            stitch_attachment_path(stitch_id, name)
        });
    }

    let dest = stitch_attachment_path(stitch_id, filename)?;
    write_atomic(&dest, data)?;
    write_attachment_meta(&dest, &meta)?;
    Ok(dest)
}

/// Internal helper: sanitize an SVG and store both the safe and unsafe copies.
///
/// `resolve_path(filename)` must return the canonical destination for any
/// filename within the same attachment directory (used for both the sanitized
/// and unsafe files).
fn store_svg_with_sanitization<F>(
    data: &[u8],
    filename: &str,
    meta: AttachmentMetadata,
    resolve_path: F,
) -> Result<PathBuf>
where
    F: Fn(&str) -> Result<PathBuf>,
{
    let result = crate::svg_sanitize::sanitize(data)
        .with_context(|| format!("SVG sanitization failed for {:?}", filename))?;

    // Sanitized version is always stored at the declared filename.
    let dest = resolve_path(filename)?;
    write_atomic(&dest, &result.safe_bytes)?;

    let svg_record = if result.record.was_modified {
        // Store original as _unsafe sibling.
        let unsafe_name = make_unsafe_svg_filename(filename);
        let unsafe_dest = resolve_path(&unsafe_name)?;
        write_atomic(&unsafe_dest, data)?;
        let unsafe_meta = AttachmentMetadata {
            filename: unsafe_name.clone(),
            svg_sanitize: None,
            ..meta.clone()
        };
        write_attachment_meta(&unsafe_dest, &unsafe_meta)?;

        Some(SvgSanitizeRecord {
            unsafe_filename: unsafe_name,
            removed_elements: result.record.removed_elements,
            removed_attrs: result.record.removed_attrs,
        })
    } else {
        None
    };

    let final_meta = AttachmentMetadata {
        svg_sanitize: svg_record,
        ..meta
    };
    write_attachment_meta(&dest, &final_meta)?;

    Ok(dest)
}

/// Internal helper: sanitize a PDF and store both the safe and unsafe copies.
///
/// `resolve_path(filename)` must return the canonical destination for any
/// filename within the same attachment directory.
fn store_pdf_with_sanitization<F>(
    data: &[u8],
    filename: &str,
    meta: AttachmentMetadata,
    resolve_path: F,
) -> Result<PathBuf>
where
    F: Fn(&str) -> Result<PathBuf>,
{
    let result = crate::pdf_sanitize::sanitize(data)
        .with_context(|| format!("PDF sanitization failed for {:?}", filename))?;

    // Sanitized version is always stored at the declared filename.
    let dest = resolve_path(filename)?;
    write_atomic(&dest, &result.safe_bytes)?;

    let pdf_record = if result.record.was_modified {
        // Store original as _unsafe sibling.
        let unsafe_name = make_unsafe_pdf_filename(filename);
        let unsafe_dest = resolve_path(&unsafe_name)?;
        write_atomic(&unsafe_dest, data)?;
        let unsafe_meta = AttachmentMetadata {
            filename: unsafe_name.clone(),
            pdf_sanitize: None,
            ..meta.clone()
        };
        write_attachment_meta(&unsafe_dest, &unsafe_meta)?;

        Some(PdfSanitizeRecord {
            unsafe_filename: unsafe_name,
            removed_threats: result.record.removed_threats,
        })
    } else {
        None
    };

    let final_meta = AttachmentMetadata {
        pdf_sanitize: pdf_record,
        ..meta
    };
    write_attachment_meta(&dest, &final_meta)?;

    Ok(dest)
}
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
    use crate::id_validators::{validate_bead_id, validate_stitch_id, ValidBeadId, ValidStitchId};
    use tempfile::TempDir;

    // ── Validation ────────────────────────────────────────────────────────────

    #[test]
    fn bead_id_valid() {
        assert!(validate_bead_id("hoop-ttb.4.12").is_ok());
        assert!(validate_bead_id("abc").is_ok());
        assert!(validate_bead_id("a1_b2-c3.d4").is_ok());
    }

    #[test]
    fn bead_id_invalid() {
        assert!(validate_bead_id("").is_err());
        assert!(validate_bead_id("-starts-with-dash").is_err());
        assert!(validate_bead_id(".starts-with-dot").is_err());
        assert!(validate_bead_id("has/slash").is_err());
        assert!(validate_bead_id("has space").is_err());
        assert!(validate_bead_id(&"x".repeat(257)).is_err());
    }

    #[test]
    fn stitch_id_valid() {
        assert!(validate_stitch_id(
            "550e8400-e29b-41d4-a716-446655440000"
        ).is_ok());
        assert!(validate_stitch_id(
            "00000000-0000-0000-0000-000000000000"
        ).is_ok());
    }

    #[test]
    fn stitch_id_invalid() {
        assert!(validate_stitch_id("").is_err());
        assert!(validate_stitch_id("not-a-uuid").is_err());
        assert!(validate_stitch_id(
            "550e8400-e29b-41d4-a716-44665544000g" // 'g' not hex
        ).is_err());
        assert!(validate_stitch_id(
            "550e8400e29b41d4a716446655440000" // no dashes
        ).is_err());
        assert!(validate_stitch_id(
            "550e8400-e29b-41d4-a716-4466554400" // too short
        ).is_err());
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
        let bid = ValidBeadId::parse("test-bead.1.0").unwrap();
        let dir = bead_attachment_dir(ws, &bid).unwrap();
        assert!(dir.is_dir());
        assert!(dir.ends_with("test-bead.1.0"));
    }

    #[test]
    fn bead_attachment_dir_idempotent() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let bid = ValidBeadId::parse("my-bead.1").unwrap();
        let dir1 = bead_attachment_dir(ws, &bid).unwrap();
        let dir2 = bead_attachment_dir(ws, &bid).unwrap();
        assert_eq!(dir1, dir2);
    }

    #[test]
    fn bead_attachment_dir_rejects_invalid_id() {
        assert!(ValidBeadId::parse("").is_err());
        assert!(ValidBeadId::parse("-bad").is_err());
        assert!(ValidBeadId::parse("../escape").is_err());
    }

    #[test]
    fn bead_attachment_path_rejects_bad_filename() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let bid = ValidBeadId::parse("bead.1").unwrap();
        assert!(bead_attachment_path(ws, &bid, "..").is_err());
        assert!(bead_attachment_path(ws, &bid, "sub/dir.png").is_err());
        assert!(bead_attachment_path(ws, &bid, "").is_err());
    }

    #[test]
    fn bead_attachment_path_ok() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let bid = ValidBeadId::parse("bead.1").unwrap();
        let p = bead_attachment_path(ws, &bid, "image.png").unwrap();
        // Should end with bead.1/image.png
        assert!(p.ends_with("image.png"));
        assert!(p.parent().unwrap().ends_with("bead.1"));
    }

    // ── Stitch attachment directory ───────────────────────────────────────────

    #[test]
    fn stitch_id_format_gate() {
        assert!(ValidStitchId::parse("not-a-uuid").is_err());
        assert!(ValidStitchId::parse("").is_err());
    }

    // ── store_bead_attachment ─────────────────────────────────────────────────

    #[test]
    fn store_bead_attachment_writes_file() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let data = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR"; // PNG header stub
        let limits = SizeLimits::default();
        let bead_id = ValidBeadId::parse("bead.1").unwrap();
        let dest = store_bead_attachment(ws, &bead_id, "test.png", data, &limits).unwrap();
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
        let bead_id = ValidBeadId::parse("bead.1").unwrap();
        assert!(store_bead_attachment(ws, &bead_id, "clip.webm", &data, &limits).is_err());
    }

    #[test]
    fn store_bead_attachment_atomic_no_tmp_left() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let data = b"%PDF-1.4 minimal";
        let limits = SizeLimits::default();
        let bead_id = ValidBeadId::parse("bead.1").unwrap();
        let dest = store_bead_attachment(ws, &bead_id, "doc.pdf", data, &limits).unwrap();
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

    // ── Content-type sniffing (§13) ─────────────────────────────────────────────

    #[test]
    fn sniff_mime_png() {
        let png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR";
        assert_eq!(sniff_mime(png).as_deref(), Some("image/png"));
    }

    #[test]
    fn sniff_mime_jpeg() {
        let jpeg = b"\xff\xd8\xff\xe0\x00\x10JFIF";
        assert_eq!(sniff_mime(jpeg).as_deref(), Some("image/jpeg"));
    }

    #[test]
    fn sniff_mime_pdf() {
        let pdf = b"%PDF-1.4 some content here";
        assert_eq!(sniff_mime(pdf).as_deref(), Some("application/pdf"));
    }

    #[test]
    fn sniff_mime_zip() {
        let zip = b"\x50\x4b\x03\x04\x00\x00\x00\x00";
        assert_eq!(sniff_mime(zip).as_deref(), Some("application/zip"));
    }

    #[test]
    fn sniff_mime_exe() {
        // Windows PE (MZ header)
        let exe = b"\x4d\x5a\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        assert!(sniff_mime(exe).is_some());
        assert!(sniff_mime(exe).unwrap().contains("dosexec") || sniff_mime(exe).unwrap().contains("executable"));
    }

    #[test]
    fn sniff_mime_unknown() {
        assert!(sniff_mime(b"random text data").is_none());
    }

    #[test]
    fn extension_to_mime_coverage() {
        assert_eq!(extension_to_mime("png"), Some("image/png"));
        assert_eq!(extension_to_mime("jpg"), Some("image/jpeg"));
        assert_eq!(extension_to_mime("jpeg"), Some("image/jpeg"));
        assert_eq!(extension_to_mime("pdf"), Some("application/pdf"));
        assert_eq!(extension_to_mime("svg"), Some("image/svg+xml"));
        assert_eq!(extension_to_mime("zip"), Some("application/zip"));
        assert_eq!(extension_to_mime("mp3"), Some("audio/mpeg"));
        assert_eq!(extension_to_mime("mp4"), Some("video/mp4"));
        assert_eq!(extension_to_mime("webm"), Some("video/webm"));
        assert_eq!(extension_to_mime("txt"), None);
        assert_eq!(extension_to_mime("xyz"), None);
    }

    // ── Content-type mismatch rejection (§13 acceptance) ────────────────────────

    #[test]
    fn reject_png_claimed_as_jpg() {
        let png_bytes = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR";
        let result = validate_content_match("photo.jpg", png_bytes);
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("content-type mismatch"),
            "expected mismatch error, got: {err}"
        );
        assert!(
            err.contains("jpg") && err.contains("image/png"),
            "error should mention both declared and sniffed: {err}"
        );
    }

    #[test]
    fn reject_zip_claimed_as_svg() {
        let zip_bytes = b"\x50\x4b\x03\x04\x00\x00\x00\x00";
        let result = validate_content_match("diagram.svg", zip_bytes);
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("content-type mismatch"),
            "expected mismatch error, got: {err}"
        );
        assert!(
            err.contains("svg") && err.contains("application/zip"),
            "error should mention both declared and sniffed: {err}"
        );
    }

    #[test]
    fn reject_exe_claimed_as_pdf() {
        // Windows PE (MZ header)
        let exe_bytes = b"\x4d\x5a\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let result = validate_content_match("report.pdf", exe_bytes);
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("content-type mismatch"),
            "expected mismatch error, got: {err}"
        );
        assert!(
            err.contains("pdf") && err.contains("portable-executable"),
            "error should mention both declared and sniffed: {err}"
        );
    }

    #[test]
    fn accept_legit_png() {
        let png_bytes = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR";
        let meta = validate_content_match("image.png", png_bytes).unwrap();
        assert_eq!(meta.declared_mime.as_deref(), Some("image/png"));
        assert_eq!(meta.sniffed_mime.as_deref(), Some("image/png"));
    }

    #[test]
    fn accept_legit_pdf() {
        let pdf_bytes = b"%PDF-1.4 document body follows";
        let meta = validate_content_match("document.pdf", pdf_bytes).unwrap();
        assert_eq!(meta.declared_mime.as_deref(), Some("application/pdf"));
        assert_eq!(meta.sniffed_mime.as_deref(), Some("application/pdf"));
    }

    #[test]
    fn accept_unknown_extension_with_unknown_content() {
        // Neither extension nor content is recognized → no conflict → allow
        let meta = validate_content_match("notes.txt", b"hello world").unwrap();
        assert_eq!(meta.declared_mime, None);
        assert_eq!(meta.sniffed_mime, None);
    }

    // ── Sidecar metadata persistence ────────────────────────────────────────────

    #[test]
    fn store_bead_attachment_writes_sidecar_meta() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let png_data = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR";
        let limits = SizeLimits::default();

        let bead_id = ValidBeadId::parse("bead.1").unwrap();
        let dest = store_bead_attachment(ws, &bead_id, "image.png", png_data, &limits).unwrap();

        let meta_path = dest.parent().unwrap().join("image.png.meta.json");
        assert!(meta_path.exists(), "sidecar .meta.json should exist");

        let meta: AttachmentMetadata =
            serde_json::from_str(&std::fs::read_to_string(&meta_path).unwrap()).unwrap();
        assert_eq!(meta.filename, "image.png");
        assert_eq!(meta.declared_extension, "png");
        assert_eq!(meta.declared_mime.as_deref(), Some("image/png"));
        assert_eq!(meta.sniffed_mime.as_deref(), Some("image/png"));
    }

    #[test]
    fn store_bead_attachment_rejects_mismatch_no_file_written() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        // PNG bytes claimed as .jpg → should be rejected
        let png_data = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR";
        let limits = SizeLimits::default();

        let bead_id = ValidBeadId::parse("bead.1").unwrap();
        let result = store_bead_attachment(ws, &bead_id, "photo.jpg", png_data, &limits);
        assert!(result.is_err());

        // Neither the attachment nor the sidecar should exist
        let dir = ws.join(".beads").join("attachments").join("bead.1");
        if dir.exists() {
            let entries: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).collect();
            assert!(entries.is_empty(), "no files should be written on rejection");
        }
    }

    // ── §13 Acceptance: SVG script tag stripped with clear record ──────────────

    #[test]
    fn svg_with_script_tag_stripped_and_recorded() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let limits = SizeLimits::default();
        let bead_id = ValidBeadId::parse("bead.1").unwrap();

        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg"><script>alert('xss')</script><rect width="10" height="10"/></svg>"#;
        let dest = store_bead_attachment(ws, &bead_id, "diagram.svg", svg, &limits).unwrap();

        // The sanitized file should NOT contain script or alert
        let stored = std::fs::read_to_string(&dest).unwrap();
        assert!(!stored.contains("script"), "script tag should be stripped");
        assert!(!stored.contains("alert"), "script content should be stripped");
        assert!(stored.contains("rect"), "non-dangerous content should survive");

        // The sidecar metadata should record what was removed
        let meta_path = dest.parent().unwrap().join("diagram.svg.meta.json");
        let meta: AttachmentMetadata =
            serde_json::from_str(&std::fs::read_to_string(&meta_path).unwrap()).unwrap();
        let rec = meta.svg_sanitize.expect("SVG sanitize record should be present");
        assert!(rec.removed_elements.iter().any(|e| e == "script"));
        assert!(!rec.unsafe_filename.is_empty());

        // The unsafe original should be preserved
        let unsafe_path = dest.parent().unwrap().join(&rec.unsafe_filename);
        assert!(unsafe_path.exists(), "unsafe copy should be preserved");
        let unsafe_content = std::fs::read_to_string(&unsafe_path).unwrap();
        assert!(unsafe_content.contains("script"), "unsafe copy should have original script");
    }

    // ── §13 Acceptance: mismatch (claimed PNG, actual EXE) rejected ────────────

    #[test]
    fn exe_claimed_as_png_rejected_with_clear_error() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let limits = SizeLimits::default();
        let bead_id = ValidBeadId::parse("bead.1").unwrap();

        // Windows PE (MZ header) disguised as PNG
        let exe_bytes = b"\x4d\x5a\x90\x00\x03\x00\x00\x00\x04\x00\x00\x00\xff\xff\x00\x00";
        let result = store_bead_attachment(ws, &bead_id, "photo.png", exe_bytes, &limits);
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("content-type mismatch"),
            "should reject with mismatch error, got: {err}"
        );
        assert!(
            err.contains("png") && (err.contains("executable") || err.contains("dosexec")),
            "error should reference both declared and actual type: {err}"
        );
    }

    // ── §13 Acceptance: PDF with embedded JS sanitized to no-JS version ───────

    #[test]
    fn pdf_with_js_sanitized_through_storage() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let limits = SizeLimits::default();
        let bead_id = ValidBeadId::parse("bead.1").unwrap();

        let pdf = b"%PDF-1.4\n1 0 obj\n<< /Type /Action /S /JavaScript /JS (alert('xss')) >>\nendobj\n%%EOF\n";
        let dest = store_bead_attachment(ws, &bead_id, "doc.pdf", pdf, &limits).unwrap();

        // The sanitized PDF should not contain JavaScript or alert
        let stored = std::fs::read(&dest).unwrap();
        let stored_str = String::from_utf8_lossy(&stored);
        assert!(!stored_str.contains("/JavaScript"), "JS action should be neutralised");
        assert!(!stored_str.contains("alert"), "JS code should be stripped");

        // Sidecar metadata should record the sanitization
        let meta_path = dest.parent().unwrap().join("doc.pdf.meta.json");
        let meta: AttachmentMetadata =
            serde_json::from_str(&std::fs::read_to_string(&meta_path).unwrap()).unwrap();
        let rec = meta.pdf_sanitize.expect("PDF sanitize record should be present");
        assert!(!rec.removed_threats.is_empty(), "should record removed threats");
        assert!(!rec.unsafe_filename.is_empty());

        // The unsafe original should contain the original JS
        let unsafe_path = dest.parent().unwrap().join(&rec.unsafe_filename);
        assert!(unsafe_path.exists());
        let unsafe_content = std::fs::read_to_string(&unsafe_path).unwrap();
        assert!(unsafe_content.contains("/JavaScript"), "unsafe copy should preserve JS");
    }
}
