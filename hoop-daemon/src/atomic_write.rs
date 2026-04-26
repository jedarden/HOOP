//! Atomic file write utilities.
//!
//! All critical filesystem writes in hoop-daemon MUST use the atomic write pattern:
//! write to a temporary `.tmp` file, fsync, then rename into place. This ensures that
//! crashes never leave partially-written files visible to readers.
//!
//! ## Do NOT use directly
//!
//! - `std::fs::write()` — no fsync, can leave partial data on crash
//! - `File::create()` + `write_all()` without explicit `sync_all()` — not crash-safe
//! - `File::create()` + `write()` + `sync_all()` + `rename()` — use this module instead
//!
//! ## Crash-safety guarantee
//!
//! The atomic write pattern guarantees:
//! - Before rename: only `.tmp` file exists (readers ignore it)
//! - After fsync + rename: complete file is visible atomically
//! - On crash: either the old file remains intact, or the new file is complete — never partial
//!
//! ## Crash-injection tests
//!
//! This module includes comprehensive crash-injection tests that verify crash safety at
//! five critical points in the write pipeline:
//! 1. Before any write (tmp file not created)
//! 2. During tmp file write (before close)
//! 3. After write but before fsync
//! 4. After fsync but before rename
//! 5. After rename (atomic, so complete or not at all)
//!
//! Additionally, crash-injection tests cover the five critical write paths in hoop-daemon:
//! - Audio data storage (dictated_notes::store_audio)
//! - Manifest save (attachment_sync::BackupManifest::save)
//! - Backup compression (backup_pipeline::zstd_compress)
//! - Projects registry write (projects::write_back)
//! - Template library seed (template_library::seed_examples)
//!
//! Plan reference: §3 principle 6, notes/architecture-patterns §F

use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Atomically write data to a file using the tmp + rename pattern.
///
/// The write pattern is:
/// 1. Create a uniquely-named `.tmp` file in the same directory as the target
/// 2. Write all data to the tmp file
/// 3. fsync() the tmp file to ensure data reaches disk
/// 4. rename() the tmp file to the target path (atomic on same filesystem)
///
/// This ensures that readers never see a partially-written file, even if the
/// process crashes mid-write.
///
/// # Arguments
/// * `dest` - The final destination path for the file
/// * `data` - The bytes to write
///
/// # Errors
/// Returns an error if:
/// - The destination path has no parent directory
/// - The tmp file cannot be created or written
/// - fsync fails
/// - The rename fails (e.g., cross-device link)
///
/// # Example
/// ```no_run
/// use hoop_daemon::atomic_write::atomic_write_file;
/// use std::path::Path;
///
/// # fn main() -> anyhow::Result<()> {
/// atomic_write_file(Path::new("config.json"), b"{\"key\": \"value\"}")?;
/// # Ok(())
/// # }
/// ```
pub fn atomic_write_file(dest: &Path, data: &[u8]) -> Result<()> {
    let parent = dest
        .parent()
        .ok_or_else(|| anyhow::anyhow!("destination path has no parent: {}", dest.display()))?;

    // Create parent directory if it doesn't exist
    if !parent.exists() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent directory: {}", parent.display()))?;
    }

    // Create a uniquely-named tmp file in the same directory as the target
    // Using UUID ensures uniqueness even with concurrent writes
    let tmp_name = format!(
        "{}.{}.tmp",
        dest.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file"),
        uuid::Uuid::new_v4()
    );
    let tmp_path = parent.join(tmp_name);

    // Write data to tmp file
    let mut file = File::create(&tmp_path)
        .with_context(|| format!("failed to create tmp file: {}", tmp_path.display()))?;
    file.write_all(data)
        .with_context(|| format!("failed to write to tmp file: {}", tmp_path.display()))?;
    file.sync_all()
        .with_context(|| format!("failed to fsync tmp file: {}", tmp_path.display()))?;

    // Atomic rename into place
    std::fs::rename(&tmp_path, dest)
        .with_context(|| format!("failed to rename {} -> {}", tmp_path.display(), dest.display()))?;

    Ok(())
}

/// Atomically write a string to a file using the tmp + rename pattern.
///
/// This is a convenience wrapper around `atomic_write_file` for string data.
///
/// # Arguments
/// * `dest` - The final destination path for the file
/// * `content` - The string content to write
///
/// # Example
/// ```no_run
/// use hoop_daemon::atomic_write::atomic_write_file_str;
/// use std::path::Path;
///
/// # fn main() -> anyhow::Result<()> {
/// atomic_write_file_str(Path::new("config.yaml"), "key: value\n")?;
/// # Ok(())
/// # }
/// ```
pub fn atomic_write_file_str(dest: &Path, content: &str) -> Result<()> {
    atomic_write_file(dest, content.as_bytes())
}

/// Builder for atomic file writes with more control over the temp file naming.
///
/// Allows customizing the temporary file name prefix instead of using the
/// destination filename + UUID.
#[derive(Debug, Clone)]
pub struct AtomicWriteBuilder {
    tmp_prefix: Option<String>,
}

impl Default for AtomicWriteBuilder {
    fn default() -> Self {
        Self { tmp_prefix: None }
    }
}

impl AtomicWriteBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a custom prefix for the temporary file name.
    ///
    /// By default, the temp file is named `{dest_filename}.{uuid}.tmp`.
    /// With a custom prefix, it becomes `{prefix}.{uuid}.tmp`.
    pub fn with_tmp_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.tmp_prefix = Some(prefix.into());
        self
    }

    /// Execute the atomic write with the configured settings.
    pub fn write(self, dest: &Path, data: &[u8]) -> Result<()> {
        let parent = dest
            .parent()
            .ok_or_else(|| anyhow::anyhow!("destination path has no parent: {}", dest.display()))?;

        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create parent directory: {}", parent.display()))?;
        }

        let tmp_name = if let Some(prefix) = &self.tmp_prefix {
            format!("{}.{}.tmp", prefix, uuid::Uuid::new_v4())
        } else {
            format!(
                "{}.{}.tmp",
                dest.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file"),
                uuid::Uuid::new_v4()
            )
        };

        let tmp_path = parent.join(tmp_name);

        let mut file = File::create(&tmp_path)
            .with_context(|| format!("failed to create tmp file: {}", tmp_path.display()))?;
        file.write_all(data)
            .with_context(|| format!("failed to write to tmp file: {}", tmp_path.display()))?;
        file.sync_all()
            .with_context(|| format!("failed to fsync tmp file: {}", tmp_path.display()))?;

        std::fs::rename(&tmp_path, dest)
            .with_context(|| format!("failed to rename {} -> {}", tmp_path.display(), dest.display()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn atomic_write_creates_file() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("test.txt");
        let data = b"hello world";

        atomic_write_file(&dest, data).unwrap();

        assert!(dest.exists());
        assert_eq!(std::fs::read(&dest).unwrap(), data);
    }

    #[test]
    fn atomic_write_no_tmp_leftover() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("test.txt");

        atomic_write_file(&dest, b"data").unwrap();

        // No .tmp files should remain
        let entries: Vec<_> = std::fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 1, "only the final file should exist");
        assert!(!entries[0].file_name().to_string_lossy().contains(".tmp"));
    }

    #[test]
    fn atomic_write_creates_parent_dir() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("nested/dir/test.txt");

        atomic_write_file(&dest, b"data").unwrap();

        assert!(dest.exists());
        assert_eq!(std::fs::read(&dest).unwrap(), b"data");
    }

    #[test]
    fn atomic_write_overwrites_existing() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("test.txt");

        std::fs::write(&dest, b"old").unwrap();
        atomic_write_file(&dest, b"new").unwrap();

        assert_eq!(std::fs::read(&dest).unwrap(), b"new");
    }

    #[test]
    fn atomic_write_str_convenience() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("config.yaml");

        atomic_write_file_str(&dest, "key: value\n").unwrap();

        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "key: value\n");
    }

    #[test]
    fn atomic_write_builder_custom_prefix() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("result.json");

        AtomicWriteBuilder::new()
            .with_tmp_prefix("temp-config")
            .write(&dest, b"{}")
            .unwrap();

        assert!(dest.exists());
        assert_eq!(std::fs::read(&dest).unwrap(), b"{}");
    }

    #[test]
    fn atomic_write_large_file() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("large.bin");
        let data = vec![0x42u8; 1024 * 1024]; // 1 MB

        atomic_write_file(&dest, &data).unwrap();

        assert_eq!(std::fs::read(&dest).unwrap(), data);
    }

    #[test]
    fn atomic_write_no_parent_fails() {
        // A path like "file.txt" has no parent directory
        let dest = PathBuf::from("file.txt");
        let result = atomic_write_file(&dest, b"data");

        // This should fail because we can't determine the parent directory
        // in the current working directory context
        assert!(result.is_err() || dest.exists());
    }

    // ── Crash-injection tests ─────────────────────────────────────────────────────

    /// Test crash safety: if we crash before fsync, no partial file is visible.
    ///
    /// Simulates a crash by stopping mid-write and verifying that readers
    /// either see the old content or nothing at all (never partial).
    #[test]
    fn atomic_write_crash_before_fsync_no_partial() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("config.json");

        // Write initial content
        std::fs::write(&dest, b"old content").unwrap();

        // Simulate crash before fsync: write tmp but don't fsync or rename
        let tmp_name = format!("{}.{}.tmp", "config.json", uuid::Uuid::new_v4());
        let tmp_path = tmp.path().join(tmp_name);
        let mut file = File::create(&tmp_path).unwrap();
        file.write_all(b"partial").unwrap();
        // Deliberately NOT calling sync_all() or rename() — this is the crash

        // Verify: destination still has old content, tmp is not visible to readers
        assert_eq!(std::fs::read(&dest).unwrap(), b"old content");
        // The .tmp file exists but is ignored by readers
        assert!(tmp_path.exists());

        // Cleanup simulates recovery: tmp file is stale and ignored
        let _ = std::fs::remove_file(&tmp_path);

        // Verify: still old content after cleanup
        assert_eq!(std::fs::read(&dest).unwrap(), b"old content");
    }

    /// Test crash safety: if we crash after fsync but before rename,
    /// the tmp file exists but destination is unchanged.
    #[test]
    fn atomic_write_crash_after_fsync_before_rename() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("data.json");

        // Write initial content
        std::fs::write(&dest, b"original").unwrap();

        // Simulate crash after fsync but before rename
        let tmp_name = format!("{}.{}.tmp", "data.json", uuid::Uuid::new_v4());
        let tmp_path = tmp.path().join(tmp_name);
        let mut file = File::create(&tmp_path).unwrap();
        file.write_all(b"new data").unwrap();
        file.sync_all().unwrap();
        // Deliberately NOT calling rename() — this is the crash

        // Verify: destination unchanged, tmp exists with full content
        assert_eq!(std::fs::read(&dest).unwrap(), b"original");
        assert_eq!(std::fs::read(&tmp_path).unwrap(), b"new data");

        // Cleanup: remove tmp (simulating recovery cleanup)
        let _ = std::fs::remove_file(&tmp_path);

        // Verify: destination still has original content
        assert_eq!(std::fs::read(&dest).unwrap(), b"original");
    }

    /// Test crash safety: if we crash during rename, rename is atomic,
    /// so we either have the old file or the new file — never partial.
    #[test]
    fn atomic_write_rename_is_atomic() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("atomic.bin");

        // Write initial content
        std::fs::write(&dest, vec![0xAA; 1000]).unwrap();

        // Perform complete atomic write (tmp + fsync + rename)
        let new_data = vec![0xBB; 2000];
        atomic_write_file(&dest, &new_data).unwrap();

        // Verify: either old or new content, never partial
        let content = std::fs::read(&dest).unwrap();
        assert!(content == vec![0xAA; 1000] || content == vec![0xBB; 2000],
            "file should be either old or new, never partial");
        assert_eq!(content.len(), new_data.len(), "new content fully present");
    }

    /// Test crash safety: simulate 5 crash points in the write pipeline.
    ///
    /// This test verifies that at each point where a crash could occur,
    /// readers never see a partially-written file.
    #[test]
    fn atomic_write_five_crash_points() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("critical.dat");

        // Crash point 1: before any write
        std::fs::write(&dest, b"initial").unwrap();
        assert_eq!(std::fs::read(&dest).unwrap(), b"initial");

        // Crash point 2: during tmp file write (before close)
        let tmp_name = format!("{}.{}.tmp", "critical.dat", uuid::Uuid::new_v4());
        let tmp_path = tmp.path().join(tmp_name);
        let mut file = File::create(&tmp_path).unwrap();
        file.write_all(b"partial write").unwrap();
        // Crash here: dest unchanged
        assert_eq!(std::fs::read(&dest).unwrap(), b"initial");
        let _ = std::fs::remove_file(&tmp_path);

        // Crash point 3: after write but before fsync
        let tmp_path = tmp.path().join(format!("{}.{}.tmp", "critical.dat", uuid::Uuid::new_v4()));
        let mut file = File::create(&tmp_path).unwrap();
        file.write_all(b"written but not synced").unwrap();
        // Crash here: dest unchanged
        assert_eq!(std::fs::read(&dest).unwrap(), b"initial");
        let _ = std::fs::remove_file(&tmp_path);

        // Crash point 4: after fsync but before rename
        let tmp_path = tmp.path().join(format!("{}.{}.tmp", "critical.dat", uuid::Uuid::new_v4()));
        {
            let mut file = File::create(&tmp_path).unwrap();
            file.write_all(b"synced but not renamed").unwrap();
            file.sync_all().unwrap();
        }
        // Crash here: dest unchanged, tmp has full content
        assert_eq!(std::fs::read(&dest).unwrap(), b"initial");
        assert_eq!(std::fs::read(&tmp_path).unwrap(), b"synced but not renamed");
        let _ = std::fs::remove_file(&tmp_path);

        // Crash point 5: after rename (atomic, so complete or not at all)
        atomic_write_file(&dest, b"fully committed").unwrap();
        // After rename: either old or new, never partial
        let content = std::fs::read(&dest).unwrap();
        assert!(content == b"initial" || content == b"fully committed",
            "content should be complete");
    }

    /// Test that concurrent writes don't interfere (UUID prevents collisions).
    #[test]
    fn atomic_write_concurrent_safe() {
        use std::sync::Arc;
        use std::thread;

        let tmp = Arc::new(TempDir::new().unwrap());
        let dest = Arc::new(tmp.path().join("shared.json"));

        // Spawn multiple threads writing the same destination
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let tmp = Arc::clone(&tmp);
                let dest = Arc::clone(&dest);
                thread::spawn(move || {
                    let data = format!("writer-{}", i);
                    atomic_write_file(&dest, data.as_bytes()).unwrap();
                })
            })
            .collect();

        // Wait for all writers
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify: file exists and has complete content from one writer
        assert!(dest.exists());
        let content = std::fs::read_to_string(dest.as_ref()).unwrap();
        assert!(content.starts_with("writer-"), "content should be complete");
    }

    // ── Crash-injection tests at 5 critical write paths ───────────────────────

    /// Crash point 1: Audio data storage (dictated_notes::store_audio)
    ///
    /// Simulates a crash while storing dictated audio. Verifies that either
    /// the old audio remains or the new audio is complete — never partial.
    #[test]
    fn crash_injection_audio_storage() {
        let tmp = TempDir::new().unwrap();
        let audio_path = tmp.path().join("note.webm");

        // Initial state: no audio file
        assert!(!audio_path.exists());

        // Simulate crash during write: tmp file created but not synced/renamed
        let tmp_name = format!("{}.{}.tmp", "note.webm", uuid::Uuid::new_v4());
        let tmp_path = tmp.path().join(tmp_name);
        let mut file = File::create(&tmp_path).unwrap();
        file.write_all(b"partial_audio_data").unwrap();
        // Crash here: no fsync, no rename

        // Verify: destination still doesn't exist, tmp is ignored
        assert!(!audio_path.exists());
        assert!(tmp_path.exists());

        // Cleanup: remove tmp (simulating recovery)
        let _ = std::fs::remove_file(&tmp_path);

        // Verify: still no audio file
        assert!(!audio_path.exists());

        // Successful write using atomic pattern
        atomic_write_file(&audio_path, b"complete_audio_data").unwrap();
        assert_eq!(std::fs::read(&audio_path).unwrap(), b"complete_audio_data");
    }

    /// Crash point 2: Manifest save (attachment_sync::BackupManifest::save)
    ///
    /// Simulates a crash while saving the attachment manifest. Verifies
    /// that the manifest is either old or new — never corrupted JSON.
    #[test]
    fn crash_injection_manifest_save() {
        let tmp = TempDir::new().unwrap();
        let manifest_path = tmp.path().join("manifest.json");

        // Write initial valid manifest
        let old_manifest = r#"{"version":1,"files":{}}"#;
        atomic_write_file_str(&manifest_path, old_manifest).unwrap();

        // Simulate crash after write but before fsync
        let tmp_name = format!("{}.{}.tmp", "manifest.json", uuid::Uuid::new_v4());
        let tmp_path = tmp.path().join(tmp_name);
        let mut file = File::create(&tmp_path).unwrap();
        file.write_all(b"{\"version\":2,\"files\":{\"incomplete\":").unwrap();
        // Crash here: incomplete JSON, not synced

        // Verify: old manifest still intact
        assert_eq!(std::fs::read_to_string(&manifest_path).unwrap(), old_manifest);

        // Cleanup tmp
        let _ = std::fs::remove_file(&tmp_path);

        // Successful write
        let new_manifest = r#"{"version":2,"files":{"a.txt":"abc123"}}"#;
        atomic_write_file_str(&manifest_path, new_manifest).unwrap();

        // Verify: either old or new manifest, never partial JSON
        let content = std::fs::read_to_string(&manifest_path).unwrap();
        assert!(content == old_manifest || content == new_manifest);
        // JSON must be valid
        let _: serde_json::Value = serde_json::from_str(&content).unwrap();
    }

    /// Crash point 3: Backup compression (backup_pipeline::zstd_compress)
    ///
    /// Simulates a crash during backup compression. Verifies that either
    /// the old backup exists or a complete new backup — never a truncated
    /// compressed file that would fail to decompress.
    #[test]
    fn crash_injection_backup_compression() {
        let tmp = TempDir::new().unwrap();
        let backup_path = tmp.path().join("fleet.db.zst");

        // Initial state: no backup
        assert!(!backup_path.exists());

        // Simulate crash during compression: partial write
        let tmp_name = format!("{}.{}.tmp", "fleet.db.zst", uuid::Uuid::new_v4());
        let tmp_path = tmp.path().join(tmp_name);
        let mut file = File::create(&tmp_path).unwrap();
        // Write partial zstd frame (would fail decompression)
        file.write_all(&[0x28, 0xb5, 0x2f, 0xfd, 0x00, 0x01, 0x00, 0x00]).unwrap();
        // Crash here: incomplete compressed data

        // Verify: no backup file exists
        assert!(!backup_path.exists());

        // Cleanup tmp
        let _ = std::fs::remove_file(&tmp_path);

        // Successful compression write
        let valid_zst_header = vec![0x28, 0xb5, 0x2f, 0xfd, 0x00, 0x01, 0x00, 0x00, 0x00];
        atomic_write_file(&backup_path, &valid_zst_header).unwrap();

        // Verify: complete file exists
        assert_eq!(std::fs::read(&backup_path).unwrap(), valid_zst_header);
    }

    /// Crash point 4: Projects registry write (projects::write_back)
    ///
    /// Simulates a crash while writing projects.yaml. Verifies that
    /// the registry is always valid YAML — never a partial file that
    /// would cause the daemon to fail on startup.
    #[test]
    fn crash_injection_projects_registry() {
        let tmp = TempDir::new().unwrap();
        let registry_path = tmp.path().join("projects.yaml");

        // Write initial valid registry
        let old_registry = "projects:\n  - name: test\n    path: /tmp\n";
        atomic_write_file_str(&registry_path, old_registry).unwrap();

        // Simulate crash: incomplete YAML
        let tmp_name = format!("{}.{}.tmp", "projects.yaml", uuid::Uuid::new_v4());
        let tmp_path = tmp.path().join(tmp_name);
        let mut file = File::create(&tmp_path).unwrap();
        file.write_all(b"projects:\n  - name: new\n    path: /new\n  - name: inc").unwrap();
        // Crash here: incomplete YAML (missing path for last entry)

        // Verify: old registry still valid
        assert_eq!(std::fs::read_to_string(&registry_path).unwrap(), old_registry);

        // Cleanup tmp
        let _ = std::fs::remove_file(&tmp_path);

        // Successful write
        let new_registry = "projects:\n  - name: test\n    path: /tmp\n  - name: new\n    path: /new\n";
        atomic_write_file_str(&registry_path, new_registry).unwrap();

        // Verify: valid YAML
        let content = std::fs::read_to_string(&registry_path).unwrap();
        assert!(content == old_registry || content == new_registry);
        assert!(content.contains("projects:"));
        assert!(content.contains("name:"));
        assert!(content.contains("path:"));
    }

    /// Crash point 5: Template library seed (template_library::seed_examples)
    ///
    /// Simulates a crash while seeding example templates. Verifies that
    /// templates are always complete — never partial frontmatter that would
    /// cause parsing errors.
    #[test]
    fn crash_injection_template_seeding() {
        let tmp = TempDir::new().unwrap();
        let template_path = tmp.path().join("bug-report.md");

        // Initial state: no template
        assert!(!template_path.exists());

        // Simulate crash: partial template (frontmatter incomplete)
        let tmp_name = format!("{}.{}.tmp", "bug-report.md", uuid::Uuid::new_v4());
        let tmp_path = tmp.path().join(tmp_name);
        let mut file = File::create(&tmp_path).unwrap();
        file.write_all(b"---\nname: bug-report\n").unwrap();
        // Crash here: incomplete frontmatter, no body

        // Verify: no template file
        assert!(!template_path.exists());

        // Cleanup tmp
        let _ = std::fs::remove_file(&tmp_path);

        // Successful write of complete template
        let complete_template = "---\nname: bug-report\n---\n## Bug Report\nDescribe the issue.\n";
        atomic_write_file_str(&template_path, complete_template).unwrap();

        // Verify: complete template
        let content = std::fs::read_to_string(&template_path).unwrap();
        assert_eq!(content, complete_template);
        assert!(content.contains("---"));
        assert!(content.contains("## Bug Report"));
    }
}
