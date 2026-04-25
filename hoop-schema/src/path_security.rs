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

    // ── Path-traversal attack vectors (§13, §K2) ──────────────────────────────────
    //
    // 10 attack vectors covering ID-level rejection, path-level rejection, and
    // symlink-based escapes.  Every vector must be rejected — either by the ID
    // validator (before path construction) or by canonicalize_and_check.

    use crate::id_validators::{
        validate_bead_id, validate_stitch_id, validate_worker_name, validate_project_name,
    };

    /// Vector 1: Basic directory traversal via `../` in bead ID.
    ///
    /// An attacker sends `../etc/passwd` as a bead_id hoping the server will
    /// construct a path like `<workspace>/.beads/attachments/../../etc/passwd`.
    /// Rejected at ID validation (contains `/` and leading `.`).
    #[test]
    fn attack_vector_1_basic_traversal_in_bead_id() {
        assert!(validate_bead_id("../etc/passwd").is_err());
        assert!(validate_bead_id("../../tmp/evil").is_err());
    }

    /// Vector 2: Absolute path injection as stitch ID.
    ///
    /// An attacker sends `/etc/shadow` or an absolute path as a stitch_id.
    /// Rejected at ID validation (not a valid UUID format).
    #[test]
    fn attack_vector_2_absolute_path_as_stitch_id() {
        assert!(validate_stitch_id("/etc/shadow").is_err());
        assert!(validate_stitch_id("/tmp/malicious").is_err());
    }

    /// Vector 3: Null byte injection in worker name.
    ///
    /// Null bytes can truncate strings in C library calls or confuse path
    /// parsers.  Rejected at ID validation (contains `\0`).
    #[test]
    fn attack_vector_3_null_byte_in_worker_name() {
        assert!(validate_worker_name("alpha\x00beta").is_err());
        assert!(validate_worker_name("valid\x00/../../etc").is_err());
    }

    /// Vector 4: URL-encoded traversal in bead ID.
    ///
    /// An attacker sends `%2e%2e%2f%2e%2e%2f` (URL-encoded `../../`) hoping
    /// it will be decoded after validation.  Rejected at ID validation
    /// (contains `%` which is not in the allowed character set).
    #[test]
    fn attack_vector_4_url_encoded_traversal_in_bead_id() {
        assert!(validate_bead_id("%2e%2e%2f%2e%2e%2fetc%2fpasswd").is_err());
        assert!(validate_bead_id("..%2F..%2Fetc").is_err());
    }

    /// Vector 5: Backslash traversal in project name.
    ///
    /// On mixed-OS environments, backslashes might be interpreted as path
    /// separators.  Rejected at ID validation (`\` not in allowed chars).
    #[test]
    fn attack_vector_5_backslash_traversal_in_project_name() {
        assert!(validate_project_name("..\\..\\etc").is_err());
        assert!(validate_project_name("project\\..\\..\\etc").is_err());
    }

    /// Vector 6: Symlink escape from within workspace via bead attachment path.
    ///
    /// An attacker creates a symlink inside the workspace pointing outside,
    /// then requests a path through it.  canonicalize resolves the symlink
    /// and the allowlist prefix-check rejects the resolved target.
    #[test]
    fn attack_vector_6_symlink_escape_from_workspace() {
        let tmp = setup_workspace();
        let al = PathAllowlist::for_workspace(tmp.path()).unwrap();

        // Symlink: workspace/.beads/attachments/escape -> /tmp
        let link = tmp.path().join(".beads").join("attachments").join("escape");
        let _ = std::os::unix::fs::symlink("/tmp", &link);

        if link.exists() {
            // The symlink itself resolves to /tmp which is outside the workspace
            let result = canonicalize_and_check(&link, &al);
            assert!(result.is_err(), "symlink to /tmp must be rejected");

            // A file "inside" the symlink directory is also outside
            let file_through_link = link.join("evil.txt");
            // The file may not exist, so canonicalize would fail — either way rejected
            let result = canonicalize_and_check(&file_through_link, &al);
            assert!(result.is_err(), "path through escaping symlink must be rejected");
        }
    }

    /// Vector 7: Deeply nested symlink chain.
    ///
    /// An attacker creates a chain of symlinks: a -> b -> c -> /tmp.
    /// canonicalize follows the entire chain and the allowlist rejects the
    /// final target.
    #[test]
    fn attack_vector_7_nested_symlink_chain() {
        let tmp = setup_workspace();
        let al = PathAllowlist::for_workspace(tmp.path()).unwrap();

        let attach = tmp.path().join(".beads").join("attachments");

        // chain: link3 -> link2 -> link1 -> /tmp
        let link1 = attach.join("link1");
        let link2 = attach.join("link2");
        let link3 = attach.join("link3");

        let _ = std::os::unix::fs::symlink("/tmp", &link1);
        let _ = std::os::unix::fs::symlink(&link1, &link2);
        let _ = std::os::unix::fs::symlink(&link2, &link3);

        if link3.exists() {
            let result = canonicalize_and_check(&link3, &al);
            assert!(result.is_err(), "nested symlink chain escaping allowlist must be rejected");
        }
    }

    /// Vector 8: Leading dash and dot tricks in bead/project IDs.
    ///
    /// Leading `-` could cause argument injection in subprocess calls.
    /// Leading `.` could create hidden files or reference parent dirs.
    /// Rejected at ID validation.
    #[test]
    fn attack_vector_8_leading_dash_and_dot() {
        assert!(validate_bead_id("-rf").is_err());
        assert!(validate_bead_id(".hidden").is_err());
        assert!(validate_project_name("-evil").is_err());
        assert!(validate_project_name(".env").is_err());
        assert!(validate_bead_id("..").is_err());
        assert!(validate_project_name("..").is_err());
    }

    /// Vector 9: Overlong ID to overflow buffers or bypass checks.
    ///
    /// Sending a bead_id longer than 256 chars or worker_name longer than
    /// 64 chars.  Rejected at ID validation (length limits).
    #[test]
    fn attack_vector_9_overlong_id() {
        assert!(validate_bead_id(&"a".repeat(257)).is_err());
        assert!(validate_worker_name(&"a".repeat(65)).is_err());
        assert!(validate_project_name(&"a".repeat(129)).is_err());
    }

    /// Vector 10: Path that resolves outside allowlist after join + canonicalize.
    ///
    /// Simulates an attacker controlling part of a path (e.g. a filename)
    /// that contains `..` components.  The ID-level validator catches the
    /// slash, but we also verify the allowlist catches it if a path somehow
    /// reaches canonicalize_and_check with `..` components.
    #[test]
    fn attack_vector_10_path_resolves_outside_via_dotdot() {
        let tmp = setup_workspace();
        let al = PathAllowlist::for_workspace(tmp.path()).unwrap();

        // Even though the ID validators prevent `..` in IDs, defense-in-depth:
        // verify that canonicalize_and_check rejects a hand-crafted path with
        // `..` components that would escape the workspace.
        let escape_path = tmp.path().join(".beads").join("attachments")
            .join("..").join("..").join("..").join("tmp");

        // This canonicalizes to /tmp which is outside the workspace
        if std::path::Path::new("/tmp").exists() {
            let result = canonicalize_and_check(&escape_path, &al);
            assert!(
                result.is_err(),
                "path with .. components escaping workspace must be rejected"
            );
        }
    }

    /// Meta-test: all 10 attack vectors are rejected by the combined
    /// ID validation + canonicalize pipeline.
    #[test]
    fn all_attack_vectors_rejected() {
        // Vector 1: basic traversal
        assert!(validate_bead_id("../etc/passwd").is_err());
        // Vector 2: absolute path as UUID
        assert!(validate_stitch_id("/etc/shadow").is_err());
        // Vector 3: null byte
        assert!(validate_worker_name("alpha\x00beta").is_err());
        // Vector 4: URL-encoded traversal
        assert!(validate_bead_id("%2e%2e%2f").is_err());
        // Vector 5: backslash
        assert!(validate_project_name("a\\..\\b").is_err());
        // Vector 6-7: symlink escapes tested above (require filesystem)
        // Vector 8: leading dash/dot
        assert!(validate_bead_id("-rf").is_err());
        // Vector 9: overlong
        assert!(validate_bead_id(&"x".repeat(300)).is_err());
        // Vector 10: dot-dot components (path-level, tested above)
    }
}
