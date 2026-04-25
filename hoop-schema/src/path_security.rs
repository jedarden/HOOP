//! Path-traversal hardening for all wire-derived filesystem paths (§13, §K2).
//!
//! Every filesystem path constructed from a wire-provided ID must:
//! 1. Pass ID regex validation before any path construction (enforced by
//!    `ValidBeadId`, `ValidStitchId`, `ValidWorkerName`, and friends in
//!    `hoop_schema::id_validators`).
//! 2. Be resolved to a real path with `canonicalize()` (realpath equivalent).
//! 3. Have its canonical form prefix-matched against a [`PathAllowlist`].
//! 4. Return a 400 rejection with a safe error message that never echoes
//!    the raw user input or the filesystem path.
//!
//! This module lives in `hoop-schema` so that both `hoop-daemon` and `hoop-mcp`
//! share the same canonical implementation instead of each maintaining inline copies.

use std::path::{Path, PathBuf};

// ── PathAllowlist ─────────────────────────────────────────────────────────────

/// Pre-computed set of canonical root directories for wire-derived path checks.
///
/// A path derived from a wire ID is accepted if (and only if) its canonical
/// form is a descendant of — or equal to — at least one root in this list.
#[derive(Debug, Clone)]
pub struct PathAllowlist {
    /// Canonical (symlink-resolved) root paths.  Must be absolute and real.
    roots: Vec<PathBuf>,
}

impl PathAllowlist {
    /// Construct an allowlist from an explicit set of pre-canonicalized roots.
    ///
    /// The caller is responsible for ensuring every entry in `roots` has
    /// already been passed through `canonicalize()`.
    pub fn from_roots(roots: Vec<PathBuf>) -> Self {
        Self { roots }
    }

    /// Build the standard allowlist for a bead workspace.
    ///
    /// Accepted roots:
    /// - `<workspace>/`
    /// - `<workspace>/.beads/`
    /// - `<workspace>/.beads/attachments/`
    ///
    /// The `.beads/attachments/` directory is created lazily when absent.
    pub fn for_workspace(workspace: &Path) -> std::io::Result<Self> {
        let beads = workspace.join(".beads");
        let attachments = beads.join("attachments");
        std::fs::create_dir_all(&attachments)?;

        let workspace_canon = workspace.canonicalize()?;
        let beads_canon = beads.canonicalize()?;
        let attachments_canon = attachments.canonicalize()?;

        Ok(Self::from_roots(vec![
            workspace_canon,
            beads_canon,
            attachments_canon,
        ]))
    }

    /// Build the standard allowlist for stitch and note attachments.
    ///
    /// Accepted root: `~/.hoop/attachments/`
    pub fn for_stitch_attachments() -> std::io::Result<Self> {
        let home =
            dirs::home_dir().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "home directory not found"))?;
        let dir = home.join(".hoop").join("attachments");
        std::fs::create_dir_all(&dir)?;
        let canon = dir.canonicalize()?;
        Ok(Self::from_roots(vec![canon]))
    }

    /// Build the standard allowlist for resumable upload temporary storage.
    ///
    /// Accepted root: `uploads_dir` (typically `~/.hoop/uploads/`).
    pub fn for_uploads(uploads_dir: &Path) -> std::io::Result<Self> {
        std::fs::create_dir_all(uploads_dir)?;
        let canon = uploads_dir.canonicalize()?;
        Ok(Self::from_roots(vec![canon]))
    }

    /// Return `true` if `candidate` is a descendant of (or equal to) any root.
    pub fn contains(&self, candidate: &Path) -> bool {
        self.roots.iter().any(|root| candidate.starts_with(root))
    }
}

// ── PathTraversalError ────────────────────────────────────────────────────────

/// Error returned when a wire-derived path fails the security checks.
///
/// Do **not** surface the internal variant details (or the path value) in HTTP
/// responses.
#[derive(Debug)]
pub enum PathTraversalError {
    /// `canonicalize()` failed (path does not exist, permission denied, …).
    Io(std::io::Error),
    /// The resolved path is not within any root in the allowlist.
    OutsideAllowlist,
}

impl std::fmt::Display for PathTraversalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "path resolution failed: {e}"),
            Self::OutsideAllowlist => write!(f, "path not within allowed roots"),
        }
    }
}

impl std::error::Error for PathTraversalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::OutsideAllowlist => None,
        }
    }
}

// ── Core function ─────────────────────────────────────────────────────────────

/// Canonicalize `path` and verify it descends from an entry in `allowlist`.
///
/// Returns the canonical (symlink-resolved) path on success.
///
/// **Important:** `path` must already exist on disk before this call because
/// `canonicalize()` is a realpath syscall that fails on missing paths.  When
/// creating a new file, canonicalize and check the *parent directory* first,
/// then re-join the sanitized filename.
pub fn canonicalize_and_check(
    path: &Path,
    allowlist: &PathAllowlist,
) -> Result<PathBuf, PathTraversalError> {
    let canonical = path.canonicalize().map_err(PathTraversalError::Io)?;
    if allowlist.contains(&canonical) {
        Ok(canonical)
    } else {
        Err(PathTraversalError::OutsideAllowlist)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_workspace() -> TempDir {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".beads").join("attachments")).unwrap();
        tmp
    }

    #[test]
    fn allowlist_from_roots_contains_root() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let al = PathAllowlist::from_roots(vec![root.clone()]);
        assert!(al.contains(&root));
        assert!(al.contains(&root.join("subdir")));
    }

    #[test]
    fn allowlist_from_roots_rejects_outside() {
        let tmp1 = TempDir::new().unwrap();
        let tmp2 = TempDir::new().unwrap();
        let root = tmp1.path().canonicalize().unwrap();
        let al = PathAllowlist::from_roots(vec![root]);
        let outside = tmp2.path().canonicalize().unwrap();
        assert!(!al.contains(&outside));
    }

    #[test]
    fn for_workspace_contains_expected_roots() {
        let tmp = setup_workspace();
        let al = PathAllowlist::for_workspace(tmp.path()).unwrap();
        let workspace_canon = tmp.path().canonicalize().unwrap();
        assert!(al.contains(&workspace_canon));
        assert!(al.contains(&workspace_canon.join(".beads")));
        assert!(al.contains(&workspace_canon.join(".beads").join("attachments")));
    }

    #[test]
    fn canonicalize_and_check_accepts_existing_path_in_allowlist() {
        let tmp = setup_workspace();
        let al = PathAllowlist::for_workspace(tmp.path()).unwrap();
        let dir = tmp.path().join(".beads").join("attachments");
        let result = canonicalize_and_check(&dir, &al);
        assert!(result.is_ok(), "existing path in allowlist should pass");
    }

    #[test]
    fn canonicalize_and_check_rejects_path_outside_allowlist() {
        let tmp = setup_workspace();
        let al = PathAllowlist::for_workspace(tmp.path()).unwrap();
        let outside = std::path::Path::new("/tmp");
        if outside.exists() {
            let result = canonicalize_and_check(outside, &al);
            assert!(result.is_err(), "path outside allowlist must be rejected");
        }
    }

    #[test]
    fn canonicalize_and_check_rejects_symlink_escape() {
        let tmp = setup_workspace();
        let al = PathAllowlist::for_workspace(tmp.path()).unwrap();

        // Create a symlink inside .beads/attachments/ pointing to /tmp (outside)
        let link = tmp.path().join(".beads").join("attachments").join("evil");
        let _ = std::os::unix::fs::symlink("/tmp", &link);

        if link.exists() {
            let result = canonicalize_and_check(&link, &al);
            assert!(result.is_err(), "symlink escaping allowlist must be rejected");
        }
    }
}
