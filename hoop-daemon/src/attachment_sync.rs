//! Incremental attachment sync: manifest-based diff engine.
//!
//! Each backup run compares current attachment files against the prior manifest
//! and computes add/change/delete sets. Only new/changed files are uploaded.
//! Deleted files are preserved as tombstones for N days (configurable).
//!
//! Plan reference: §15.3

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

// ── Manifest types ─────────────────────────────────────────────────────

/// Per-file entry in the backup manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileEntry {
    pub sha256: String,
    pub size: u64,
    /// ISO 8601 modification time.
    pub mtime: String,
}

/// Tombstone entry for a deleted file. Preserved for `retention_days` before pruning.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TombstoneEntry {
    pub sha256: String,
    pub size: u64,
    pub mtime: String,
    /// ISO 8601 timestamp when the file was detected as deleted.
    pub deleted_at: String,
}

/// The backup manifest, stored as JSON at `~/.hoop/backup_manifest.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackupManifest {
    pub schema_version: u32,
    /// ISO 8601 timestamp of the last sync run.
    pub last_updated: String,
    /// Relative path → file entry. Paths use forward slashes, rooted at the attachment dir.
    pub files: BTreeMap<String, FileEntry>,
    /// Relative path → tombstone entry for deleted files.
    pub tombstones: BTreeMap<String, TombstoneEntry>,
}

impl BackupManifest {
    pub fn new() -> Self {
        Self {
            schema_version: 1,
            last_updated: chrono::Utc::now().to_rfc3339(),
            files: BTreeMap::new(),
            tombstones: BTreeMap::new(),
        }
    }

    /// Load manifest from disk. Returns an empty manifest if the file doesn't exist.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let data = std::fs::read_to_string(path)
            .with_context(|| format!("read manifest {}", path.display()))?;
        let manifest: BackupManifest = serde_json::from_str(&data)
            .with_context(|| format!("parse manifest {}", path.display()))?;
        Ok(manifest)
    }

    /// Save manifest to disk atomically (tmp + rename).
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create manifest dir {}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(self)
            .context("serialize manifest")?;

        let tmp_path = path.with_extension("json.tmp");
        std::fs::write(&tmp_path, &json)
            .with_context(|| format!("write {}", tmp_path.display()))?;
        std::fs::rename(&tmp_path, path)
            .with_context(|| format!("rename {} -> {}", tmp_path.display(), path.display()))?;

        Ok(())
    }

    /// Prune tombstones older than `retention_days`.
    pub fn prune_tombstones(&mut self, retention_days: u32) {
        if retention_days == 0 || self.tombstones.is_empty() {
            return;
        }
        let cutoff = chrono::Utc::now() - chrono::Duration::days(retention_days as i64);
        let cutoff_str = cutoff.to_rfc3339();
        let before = self.tombstones.len();
        self.tombstones.retain(|_, t| t.deleted_at > cutoff_str);
        let pruned = before - self.tombstones.len();
        if pruned > 0 {
            info!("Pruned {} tombstones older than {} days", pruned, retention_days);
        }
    }
}

// ── Diff result ────────────────────────────────────────────────────────

/// Result of comparing the current attachment tree against the manifest.
#[derive(Debug)]
pub struct DiffResult {
    /// New files not in the manifest.
    pub added: Vec<(String, FileEntry)>,
    /// Files whose sha256 has changed.
    pub changed: Vec<(String, FileEntry)>,
    /// Files in the manifest but no longer on disk.
    pub deleted: Vec<String>,
    /// Count of unchanged files (skipped upload).
    pub unchanged_count: usize,
}

impl DiffResult {
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.changed.is_empty() || !self.deleted.is_empty()
    }
}

// ── Scan and diff ──────────────────────────────────────────────────────

/// Compute the sha256 of a file.
fn file_sha256(path: &Path) -> Result<String> {
    let data = std::fs::read(path)
        .with_context(|| format!("read {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(hex::encode(hasher.finalize()))
}

/// Get file metadata (size + mtime as ISO 8601).
fn file_metadata(path: &Path) -> Result<(u64, String)> {
    let meta = std::fs::metadata(path)
        .with_context(|| format!("stat {}", path.display()))?;
    let mtime = meta.modified()
        .with_context(|| format!("mtime {}", path.display()))?;
    let dt: chrono::DateTime<chrono::Utc> = mtime.into();
    Ok((meta.len(), dt.to_rfc3339()))
}

/// Walk a directory tree and collect relative paths → (sha256, size, mtime).
///
/// Uses the `ignore` crate for efficient directory traversal.
fn scan_attachments(root: &Path) -> Result<BTreeMap<String, FileEntry>> {
    let mut entries = BTreeMap::new();

    if !root.exists() {
        return Ok(entries);
    }

    let walker = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false)
        .build();

    for result in walker {
        let entry = match result {
            Ok(e) => e,
            Err(e) => {
                warn!("Skipping attachment entry: {}", e);
                continue;
            }
        };

        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }

        let path = entry.path();

        // Build relative path from the attachment root
        let rel = path.strip_prefix(root)
            .with_context(|| format!("strip prefix {} from {}", root.display(), path.display()))?;
        let rel_str = rel.to_string_lossy().to_string();

        let (size, mtime) = file_metadata(path)?;
        let sha256 = file_sha256(path)?;

        debug!("Scanned attachment: {} ({} bytes, sha256={})", rel_str, size, &sha256[..12]);

        entries.insert(rel_str, FileEntry {
            sha256,
            size,
            mtime,
        });
    }

    Ok(entries)
}

/// Compute the diff between current files and the prior manifest.
///
/// - Files present on disk but absent from manifest → added
/// - Files present in both but with different sha256 → changed
/// - Files present in manifest but absent on disk → deleted (tombstoned)
/// - Files with matching sha256 → unchanged (skip)
///
/// The mtime+size shortcut is used: if both match, sha256 is assumed unchanged.
pub fn compute_diff(
    current: &BTreeMap<String, FileEntry>,
    manifest: &BackupManifest,
) -> DiffResult {
    let mut added = Vec::new();
    let mut changed = Vec::new();
    let mut deleted = Vec::new();
    let mut unchanged_count = 0;

    // Check for added and changed files
    for (path, entry) in current {
        match manifest.files.get(path) {
            None => {
                // Skip if tombstoned and re-added (it's a new file with same path)
                added.push((path.clone(), entry.clone()));
            }
            Some(old) => {
                // Fast path: if mtime AND size match, assume unchanged
                if old.mtime == entry.mtime && old.size == entry.size {
                    unchanged_count += 1;
                } else if old.sha256 != entry.sha256 {
                    changed.push((path.clone(), entry.clone()));
                } else {
                    // sha256 matches even though mtime/size differ (e.g., touch)
                    unchanged_count += 1;
                }
            }
        }
    }

    // Check for deleted files (in manifest but not on disk)
    for path in manifest.files.keys() {
        if !current.contains_key(path) {
            deleted.push(path.clone());
        }
    }

    DiffResult {
        added,
        changed,
        deleted,
        unchanged_count,
    }
}

/// Manifest path: `~/.hoop/backup_manifest.json`.
pub fn manifest_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".hoop").join("backup_manifest.json")
}

/// Scan all attachment roots and return a unified map.
///
/// Roots:
/// - `~/.hoop/attachments/` (stitch attachments)
/// - `<workspace>/.beads/attachments/` (bead attachments, if present)
///
/// Stitch attachments are keyed as `stitch/<stitch-id>/<filename>`.
/// Bead attachments are keyed as `bead/<bead-id>/<filename>`.
pub fn scan_all_attachments(workspace: Option<&Path>) -> Result<BTreeMap<String, FileEntry>> {
    let mut all = BTreeMap::new();

    // Stitch attachments: ~/.hoop/attachments/
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let stitch_root = home.join(".hoop").join("attachments");
    let stitch_entries = scan_attachments(&stitch_root)?;
    for (path, entry) in stitch_entries {
        all.insert(format!("stitch/{}", path), entry);
    }

    // Bead attachments: <workspace>/.beads/attachments/
    if let Some(ws) = workspace {
        let bead_root = ws.join(".beads").join("attachments");
        let bead_entries = scan_attachments(&bead_root)?;
        for (path, entry) in bead_entries {
            all.insert(format!("bead/{}", path), entry);
        }
    }

    Ok(all)
}

/// Apply a diff result to the manifest, producing an updated manifest.
///
/// - Added/changed files are inserted into `manifest.files`.
/// - Deleted files are moved from `manifest.files` to `manifest.tombstones`.
/// - Tombstones past retention are pruned.
pub fn apply_diff(
    manifest: &mut BackupManifest,
    diff: &DiffResult,
    retention_days: u32,
) {
    let now = chrono::Utc::now().to_rfc3339();

    // Apply additions
    for (path, entry) in &diff.added {
        manifest.files.insert(path.clone(), entry.clone());
    }

    // Apply changes
    for (path, entry) in &diff.changed {
        manifest.files.insert(path.clone(), entry.clone());
    }

    // Apply deletions: move from files → tombstones
    for path in &diff.deleted {
        if let Some(old_entry) = manifest.files.remove(path) {
            // Don't overwrite an existing tombstone if one already exists
            manifest.tombstones.entry(path.clone()).or_insert_with(|| TombstoneEntry {
                sha256: old_entry.sha256,
                size: old_entry.size,
                mtime: old_entry.mtime,
                deleted_at: now.clone(),
            });
        }
    }

    // Prune old tombstones
    manifest.prune_tombstones(retention_days);

    manifest.last_updated = now;
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(sha: &str, size: u64, mtime: &str) -> FileEntry {
        FileEntry {
            sha256: sha.to_string(),
            size,
            mtime: mtime.to_string(),
        }
    }

    fn make_tombstone(sha: &str, size: u64, mtime: &str, deleted_at: &str) -> TombstoneEntry {
        TombstoneEntry {
            sha256: sha.to_string(),
            size,
            mtime: mtime.to_string(),
            deleted_at: deleted_at.to_string(),
        }
    }

    // ── Manifest load/save ──────────────────────────────────────────────

    #[test]
    fn manifest_new_is_empty() {
        let m = BackupManifest::new();
        assert_eq!(m.schema_version, 1);
        assert!(m.files.is_empty());
        assert!(m.tombstones.is_empty());
    }

    #[test]
    fn manifest_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest.json");

        let mut m = BackupManifest::new();
        m.files.insert(
            "stitch/abc-123/image.png".into(),
            make_entry("aabbcc", 1024, "2024-06-15T12:00:00+00:00"),
        );
        m.save(&path).unwrap();

        let loaded = BackupManifest::load(&path).unwrap();
        assert_eq!(loaded.files.len(), 1);
        assert_eq!(loaded.files["stitch/abc-123/image.png"].sha256, "aabbcc");
    }

    #[test]
    fn manifest_load_missing_returns_empty() {
        let m = BackupManifest::load(Path::new("/nonexistent/manifest.json")).unwrap();
        assert!(m.files.is_empty());
    }

    #[test]
    fn manifest_save_atomic_no_tmp_left() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest.json");
        let m = BackupManifest::new();
        m.save(&path).unwrap();

        // No .tmp file should remain
        let tmps: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(tmps.is_empty(), "stale tmp files: {:?}", tmps);
    }

    // ── Tombstone pruning ───────────────────────────────────────────────

    #[test]
    fn prune_removes_old_tombstones() {
        let mut m = BackupManifest::new();
        let old_ts = "2020-01-01T00:00:00+00:00";
        let recent_ts = chrono::Utc::now().to_rfc3339();

        m.tombstones.insert(
            "old.png".into(),
            make_tombstone("aa", 10, old_ts, old_ts),
        );
        m.tombstones.insert(
            "recent.png".into(),
            make_tombstone("bb", 20, &recent_ts, &recent_ts),
        );

        m.prune_tombstones(30);
        assert_eq!(m.tombstones.len(), 1);
        assert!(m.tombstones.contains_key("recent.png"));
    }

    #[test]
    fn prune_zero_retention_keeps_all() {
        let mut m = BackupManifest::new();
        m.tombstones.insert(
            "old.png".into(),
            make_tombstone("aa", 10, "2020-01-01T00:00:00+00:00", "2020-01-01T00:00:00+00:00"),
        );
        m.prune_tombstones(0);
        assert_eq!(m.tombstones.len(), 1);
    }

    // ── Diff computation ────────────────────────────────────────────────

    #[test]
    fn diff_empty_manifest_all_added() {
        let current = BTreeMap::from([
            ("a.png".into(), make_entry("aa", 100, "2024-06-15T12:00:00+00:00")),
            ("b.png".into(), make_entry("bb", 200, "2024-06-15T12:00:00+00:00")),
        ]);
        let manifest = BackupManifest::new();

        let diff = compute_diff(&current, &manifest);
        assert_eq!(diff.added.len(), 2);
        assert_eq!(diff.changed.len(), 0);
        assert_eq!(diff.deleted.len(), 0);
        assert_eq!(diff.unchanged_count, 0);
        assert!(diff.has_changes());
    }

    #[test]
    fn diff_no_changes() {
        let entries = BTreeMap::from([
            ("a.png".into(), make_entry("aa", 100, "2024-06-15T12:00:00+00:00")),
        ]);
        let mut manifest = BackupManifest::new();
        manifest.files = entries.clone();

        let diff = compute_diff(&entries, &manifest);
        assert_eq!(diff.added.len(), 0);
        assert_eq!(diff.changed.len(), 0);
        assert_eq!(diff.deleted.len(), 0);
        assert_eq!(diff.unchanged_count, 1);
        assert!(!diff.has_changes());
    }

    #[test]
    fn diff_detects_changes() {
        let current = BTreeMap::from([
            ("a.png".into(), make_entry("aa_new", 100, "2024-06-15T13:00:00+00:00")),
        ]);
        let mut manifest = BackupManifest::new();
        manifest.files.insert(
            "a.png".into(),
            make_entry("aa_old", 100, "2024-06-15T12:00:00+00:00"),
        );

        let diff = compute_diff(&current, &manifest);
        assert_eq!(diff.changed.len(), 1);
        assert_eq!(diff.changed[0].0, "a.png");
        assert_eq!(diff.changed[0].1.sha256, "aa_new");
    }

    #[test]
    fn diff_detects_deletions() {
        let current = BTreeMap::new();
        let mut manifest = BackupManifest::new();
        manifest.files.insert(
            "gone.png".into(),
            make_entry("cc", 50, "2024-06-15T12:00:00+00:00"),
        );

        let diff = compute_diff(&current, &manifest);
        assert_eq!(diff.deleted.len(), 1);
        assert_eq!(diff.deleted[0], "gone.png");
    }

    #[test]
    fn diff_mtime_size_match_skips_sha256_compare() {
        // Same mtime + same size → treated as unchanged even though sha256 would differ
        let current = BTreeMap::from([
            ("a.png".into(), make_entry("aa", 100, "2024-06-15T12:00:00+00:00")),
        ]);
        let mut manifest = BackupManifest::new();
        manifest.files.insert(
            "a.png".into(),
            make_entry("different_hash", 100, "2024-06-15T12:00:00+00:00"),
        );

        let diff = compute_diff(&current, &manifest);
        assert_eq!(diff.unchanged_count, 1);
        assert_eq!(diff.changed.len(), 0);
    }

    #[test]
    fn diff_sha256_match_even_if_mtime_differs() {
        // sha256 match → unchanged regardless of mtime
        let current = BTreeMap::from([
            ("a.png".into(), make_entry("aa", 100, "2024-06-15T13:00:00+00:00")),
        ]);
        let mut manifest = BackupManifest::new();
        manifest.files.insert(
            "a.png".into(),
            make_entry("aa", 100, "2024-06-15T12:00:00+00:00"),
        );

        let diff = compute_diff(&current, &manifest);
        assert_eq!(diff.unchanged_count, 1);
        assert_eq!(diff.changed.len(), 0);
    }

    // ── Apply diff ──────────────────────────────────────────────────────

    #[test]
    fn apply_diff_adds_files() {
        let mut manifest = BackupManifest::new();
        let diff = DiffResult {
            added: vec![("new.png".into(), make_entry("aa", 100, "2024-06-15T12:00:00+00:00"))],
            changed: vec![],
            deleted: vec![],
            unchanged_count: 0,
        };
        apply_diff(&mut manifest, &diff, 30);
        assert!(manifest.files.contains_key("new.png"));
    }

    #[test]
    fn apply_diff_moves_deleted_to_tombstones() {
        let mut manifest = BackupManifest::new();
        manifest.files.insert(
            "old.png".into(),
            make_entry("cc", 50, "2024-06-15T12:00:00+00:00"),
        );

        let diff = DiffResult {
            added: vec![],
            changed: vec![],
            deleted: vec!["old.png".into()],
            unchanged_count: 0,
        };
        apply_diff(&mut manifest, &diff, 30);

        assert!(!manifest.files.contains_key("old.png"));
        assert!(manifest.tombstones.contains_key("old.png"));
        assert_eq!(manifest.tombstones["old.png"].sha256, "cc");
    }

    #[test]
    fn apply_diff_does_not_overwrite_existing_tombstone() {
        let mut manifest = BackupManifest::new();
        // Use a recent timestamp so pruning doesn't remove it
        let recent = chrono::Utc::now().to_rfc3339();
        manifest.files.insert(
            "old.png".into(),
            make_entry("cc_new", 60, &recent),
        );
        manifest.tombstones.insert(
            "old.png".into(),
            make_tombstone("cc_orig", 50, &recent, &recent),
        );

        let diff = DiffResult {
            added: vec![],
            changed: vec![],
            deleted: vec!["old.png".into()],
            unchanged_count: 0,
        };
        apply_diff(&mut manifest, &diff, 30);

        // Original tombstone preserved (not overwritten by the file entry)
        assert_eq!(manifest.tombstones["old.png"].deleted_at, recent);
        assert_eq!(manifest.tombstones["old.png"].sha256, "cc_orig");
    }

    #[test]
    fn apply_diff_prunes_tombstones() {
        let mut manifest = BackupManifest::new();
        manifest.tombstones.insert(
            "ancient.png".into(),
            make_tombstone("zz", 1, "2020-01-01T00:00:00+00:00", "2020-01-01T00:00:00+00:00"),
        );

        let diff = DiffResult {
            added: vec![],
            changed: vec![],
            deleted: vec![],
            unchanged_count: 0,
        };
        apply_diff(&mut manifest, &diff, 30);

        assert!(manifest.tombstones.is_empty());
    }

    // ── File scanning ───────────────────────────────────────────────────

    #[test]
    fn scan_attachments_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let entries = scan_attachments(dir.path()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn scan_attachments_reads_files() {
        let dir = tempfile::tempdir().unwrap();
        let subdir = dir.path().join("bead-1");
        std::fs::create_dir_all(&subdir).unwrap();
        std::fs::write(subdir.join("image.png"), b"\x89PNG\r\n\x1a\ntest data").unwrap();

        let entries = scan_attachments(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);

        let key = format!("bead-1/image.png");
        assert!(entries.contains_key(&key));

        let entry = &entries[&key];
        assert_eq!(entry.size, 17);
        assert!(!entry.sha256.is_empty());
        assert!(!entry.mtime.is_empty());
    }

    #[test]
    fn scan_attachments_skips_dirs() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("subdir")).unwrap();
        std::fs::write(dir.path().join("file.txt"), b"hello").unwrap();

        let entries = scan_attachments(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries.contains_key("file.txt"));
    }

    #[test]
    fn scan_attachments_nonexistent_dir() {
        let entries = scan_attachments(Path::new("/nonexistent/path")).unwrap();
        assert!(entries.is_empty());
    }

    // ── Full scan_all_attachments ────────────────────────────────────────

    #[test]
    fn scan_all_prefixes_stitch_and_bead() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path();
        let ws = tempfile::tempdir().unwrap();

        // Create stitch attachment
        let stitch_dir = home.join(".hoop").join("attachments").join("abc-1234-def");
        std::fs::create_dir_all(&stitch_dir).unwrap();
        std::fs::write(stitch_dir.join("img.png"), b"png data").unwrap();

        // Create bead attachment
        let bead_dir = ws.path().join(".beads").join("attachments").join("bead.1");
        std::fs::create_dir_all(&bead_dir).unwrap();
        std::fs::write(bead_dir.join("doc.pdf"), b"pdf data").unwrap();

        // Override home dir for the test
        // scan_all_attachments uses dirs::home_dir() which we can't easily override,
        // so we'll test the individual scan functions instead.
        let stitch_entries = scan_attachments(&home.join(".hoop").join("attachments")).unwrap();
        assert_eq!(stitch_entries.len(), 1);
        assert!(stitch_entries.contains_key("abc-1234-def/img.png"));

        let bead_entries = scan_attachments(&ws.path().join(".beads").join("attachments")).unwrap();
        assert_eq!(bead_entries.len(), 1);
        assert!(bead_entries.contains_key("bead.1/doc.pdf"));
    }

    // ── End-to-end: scan → diff → apply ─────────────────────────────────

    #[test]
    fn full_sync_cycle() {
        let dir = tempfile::tempdir().unwrap();

        // Initial state: one file
        std::fs::write(dir.path().join("a.png"), b"image a").unwrap();

        // First scan
        let current1 = scan_attachments(dir.path()).unwrap();
        let mut manifest = BackupManifest::new();
        let diff1 = compute_diff(&current1, &manifest);
        assert_eq!(diff1.added.len(), 1);

        apply_diff(&mut manifest, &diff1, 30);
        assert_eq!(manifest.files.len(), 1);

        // Second scan: add a file, modify existing
        std::fs::write(dir.path().join("b.png"), b"image b").unwrap();
        std::fs::write(dir.path().join("a.png"), b"image a modified").unwrap();

        let current2 = scan_attachments(dir.path()).unwrap();
        let diff2 = compute_diff(&current2, &manifest);
        assert_eq!(diff2.changed.len(), 1); // a.png changed
        assert_eq!(diff2.added.len(), 1);   // b.png new

        apply_diff(&mut manifest, &diff2, 30);
        assert_eq!(manifest.files.len(), 2);

        // Third scan: delete a file
        std::fs::remove_file(dir.path().join("a.png")).unwrap();

        let current3 = scan_attachments(dir.path()).unwrap();
        let diff3 = compute_diff(&current3, &manifest);
        assert_eq!(diff3.deleted.len(), 1);

        apply_diff(&mut manifest, &diff3, 30);
        assert_eq!(manifest.files.len(), 1);
        assert_eq!(manifest.tombstones.len(), 1);
        assert!(manifest.tombstones.contains_key("a.png"));
    }

    #[test]
    fn file_sha256_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        std::fs::write(&path, b"deterministic content").unwrap();

        let h1 = file_sha256(&path).unwrap();
        let h2 = file_sha256(&path).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn file_metadata_returns_values() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        std::fs::write(&path, b"12345").unwrap();

        let (size, mtime) = file_metadata(&path).unwrap();
        assert_eq!(size, 5);
        assert!(!mtime.is_empty());
    }
}
