//! Path-traversal hardening for all wire-derived filesystem paths (§13, §K2).
//!
//! Re-exports the canonical implementation from `hoop_schema::path_security`
//! and adds axum-specific HTTP helpers.

pub use hoop_schema::path_security::{
    canonicalize_and_check, PathAllowlist, PathTraversalError,
};

// ── HTTP helpers ──────────────────────────────────────────────────────────────

/// Build a safe HTTP 400 rejection that never echoes raw user input.
///
/// The message tells the caller *what kind* of parameter was invalid,
/// not its value or any filesystem detail.
pub fn safe_rejection(kind: &'static str) -> (axum::http::StatusCode, String) {
    (
        axum::http::StatusCode::BAD_REQUEST,
        format!("Invalid {} parameter", kind),
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn safe_rejection_returns_400() {
        let (status, body) = safe_rejection("bead_id");
        assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
        assert!(body.contains("bead_id"));
        assert!(!body.contains('/'));   // no filesystem path in the message
        assert!(!body.contains(".."));
    }

    #[test]
    fn re_exported_canonicalize_and_check_works() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".beads").join("attachments")).unwrap();
        let al = PathAllowlist::for_workspace(tmp.path()).unwrap();
        let dir = tmp.path().join(".beads").join("attachments");
        let result = canonicalize_and_check(&dir, &al);
        assert!(result.is_ok(), "re-exported function should work");
    }
}
