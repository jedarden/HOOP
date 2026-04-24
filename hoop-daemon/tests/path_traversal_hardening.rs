//! Path-traversal hardening — 10 attack vector integration tests (§13, §K2).
//!
//! Each test exercises a distinct class of traversal attack against the
//! two-layer defence:
//!   Layer 1 — ID regex validation (ValidBeadId, ValidWorkerName, …)
//!   Layer 2 — realpath canonicalization + PathAllowlist prefix-check
//!
//! All 10 vectors must be rejected before any filesystem mutation can occur.
//!
//! Run with:
//!   cargo test --test path_traversal_hardening

use hoop_daemon::path_security::{canonicalize_and_check, PathAllowlist};
use hoop_schema::id_validators::{ValidBeadId, ValidStitchId, ValidWorkerName};
use tempfile::TempDir;

// ── Test fixture ──────────────────────────────────────────────────────────────

fn workspace() -> TempDir {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join(".beads").join("attachments")).unwrap();
    tmp
}

// ── Layer-1 attacks: rejected at regex/ID-validation before path construction ─

/// Attack vector 1 — classic `../etc/passwd` as a bead ID.
///
/// The slash alone is enough to fail `validate_bead_id`, which rejects any
/// character outside `[a-z0-9._-]`.
#[test]
fn attack_1_dotdot_slash_in_bead_id() {
    assert!(
        ValidBeadId::parse("../etc/passwd").is_err(),
        "dotdot-slash traversal must be rejected at the ID validator"
    );
}

/// Attack vector 2 — deeply-nested double-dot traversal.
///
/// Multiple `../` segments are equally rejected: slashes are not in the
/// bead-ID alphabet.
#[test]
fn attack_2_multi_dotdot_in_bead_id() {
    assert!(
        ValidBeadId::parse("../../etc/shadow").is_err(),
        "multi-segment dotdot traversal must be rejected at the ID validator"
    );
}

/// Attack vector 3 — null-byte injection.
///
/// A null byte embedded in the ID is rejected by the regex check (only
/// printable ASCII in `[a-z0-9._-]` is allowed).
#[test]
fn attack_3_null_byte_in_bead_id() {
    assert!(
        ValidBeadId::parse("bead\x00../../etc/passwd").is_err(),
        "null-byte injection must be rejected at the ID validator"
    );
}

/// Attack vector 4 — absolute path passed as a bead ID.
///
/// Leading `/` is not in the bead-ID alphabet.
#[test]
fn attack_4_absolute_path_as_bead_id() {
    assert!(
        ValidBeadId::parse("/etc/passwd").is_err(),
        "absolute-path bead ID must be rejected at the ID validator"
    );
}

/// Attack vector 5 — home-directory tilde expansion attempt.
///
/// `~` is not in the bead-ID alphabet.
#[test]
fn attack_5_tilde_expansion_in_bead_id() {
    assert!(
        ValidBeadId::parse("~/.ssh/id_rsa").is_err(),
        "tilde-expansion attempt must be rejected at the ID validator"
    );
}

/// Attack vector 6 — URL/percent-encoded traversal sequence.
///
/// `%2e` (`.`) and `%2f` (`/`) are not in the bead-ID alphabet; `%` itself
/// is rejected.
#[test]
fn attack_6_percent_encoded_traversal_in_bead_id() {
    assert!(
        ValidBeadId::parse("%2e%2e%2fetc%2fpasswd").is_err(),
        "percent-encoded traversal must be rejected at the ID validator"
    );
    assert!(
        ValidBeadId::parse("%2e%2e/etc/passwd").is_err(),
        "mixed percent/slash traversal must be rejected at the ID validator"
    );
}

/// Attack vector 7 — Unicode lookalike for dot (FULLSTOP U+FF0E).
///
/// Multi-byte Unicode is not in the bead-ID alphabet (only ASCII).
#[test]
fn attack_7_unicode_fullstop_lookalike_in_bead_id() {
    // U+FF0E FULLSTOP — visual lookalike for ASCII dot
    assert!(
        ValidBeadId::parse("\u{FF0E}\u{FF0E}/etc/passwd").is_err(),
        "Unicode lookalike dot must be rejected at the ID validator"
    );
}

/// Attack vector 8 — Windows-style backslash path separator.
///
/// `\` is not in the bead-ID alphabet.
#[test]
fn attack_8_backslash_separator_in_bead_id() {
    assert!(
        ValidBeadId::parse("..\\etc\\passwd").is_err(),
        "backslash separator must be rejected at the ID validator"
    );
}

/// Attack vector 9 — valid-looking prefix followed by traversal.
///
/// `valid-bead/../../../etc` still contains `/` and is rejected.
#[test]
fn attack_9_valid_prefix_then_traversal_in_bead_id() {
    assert!(
        ValidBeadId::parse("valid-bead/../../../etc/passwd").is_err(),
        "traversal after valid-looking prefix must be rejected at the ID validator"
    );
}

// ── Layer-2 attack: symlink escape caught by canonicalize_and_check ───────────

/// Attack vector 10 — symlink inside the allowlist that resolves outside it.
///
/// An attacker who can create a symlink inside `<workspace>/.beads/attachments/`
/// pointing to a path outside the workspace (e.g., `/etc/passwd`) must be
/// stopped by `canonicalize_and_check`.  The function resolves the symlink via
/// `realpath` and checks the result against the pre-computed `PathAllowlist`.
#[test]
fn attack_10_symlink_escape_via_canonicalize_and_check() {
    let ws = workspace();
    let allowlist = PathAllowlist::for_workspace(ws.path())
        .expect("allowlist construction must succeed");

    // Place a symlink inside .beads/attachments/ that points to /etc (outside).
    let link = ws
        .path()
        .join(".beads")
        .join("attachments")
        .join("evil-link");
    let _ = std::os::unix::fs::symlink("/etc", &link);

    // The symlink exists on disk, but canonicalize_and_check must reject it
    // because /etc is outside the workspace allowlist.
    if link.exists() {
        // exists() follows symlinks, so /etc must be present for this branch
        let result = canonicalize_and_check(&link, &allowlist);
        assert!(
            result.is_err(),
            "symlink escaping the allowlist must be rejected by canonicalize_and_check"
        );
    }
    // If /etc doesn't exist on this system, the symlink is dangling and
    // canonicalize() returns Io(NotFound) — still an error.  Both outcomes
    // satisfy the requirement.
    if !link.exists() {
        let result = canonicalize_and_check(&link, &allowlist);
        assert!(
            result.is_err(),
            "dangling symlink must also be rejected (canonicalize fails)"
        );
    }
}

// ── Bonus: worker name and stitch ID validators block traversal too ───────────

/// Worker names used as path components (e.g., session directories) must be
/// `^[a-z][a-z0-9]*$` — no separators, dots, or special characters.
#[test]
fn worker_name_rejects_traversal_characters() {
    assert!(ValidWorkerName::parse("../escape").is_err());
    assert!(ValidWorkerName::parse("/etc/passwd").is_err());
    assert!(ValidWorkerName::parse("worker\x00evil").is_err());
    assert!(ValidWorkerName::parse("worker.name").is_err()); // dots not allowed
}

/// Stitch IDs are UUIDs; anything that looks like a path is instantly rejected
/// because it cannot match the strict UUID format.
#[test]
fn stitch_id_rejects_traversal_attempts() {
    assert!(ValidStitchId::parse("../etc/passwd").is_err());
    assert!(ValidStitchId::parse("/etc/passwd").is_err());
    assert!(ValidStitchId::parse("../../etc/shadow").is_err());
}

// ── HTTP rejection sanity check ───────────────────────────────────────────────

/// The `safe_rejection` helper must return 400 and must not echo any path
/// component or raw user input in the response body.
#[test]
fn safe_rejection_body_contains_no_path_information() {
    use hoop_daemon::path_security::safe_rejection;

    let attack_inputs = [
        "../etc/passwd",
        "../../etc/shadow",
        "/absolute/path",
        "~/.ssh/id_rsa",
        "bead\x00evil",
    ];

    let (status, body) = safe_rejection("bead_id");
    assert_eq!(
        status,
        axum::http::StatusCode::BAD_REQUEST,
        "safe_rejection must return 400"
    );
    for fragment in &attack_inputs {
        assert!(
            !body.contains(fragment),
            "response body must not echo attack input {:?}: body was {:?}",
            fragment,
            body
        );
    }
    // The response must not contain filesystem path separators
    assert!(
        !body.contains('/'),
        "response body must not contain path separators"
    );
}

// ── Upload-specific Layer-2 tests ─────────────────────────────────────────────

/// The uploads allowlist must accept paths inside the uploads directory.
#[test]
fn uploads_allowlist_accepts_paths_inside_uploads_dir() {
    let tmp = TempDir::new().unwrap();
    let uploads_dir = tmp.path().join("uploads");
    let al = PathAllowlist::for_uploads(&uploads_dir).unwrap();

    // Create a fake upload directory inside uploads
    let fake_upload = uploads_dir.join("550e8400-e29b-41d4-a716-446655440000");
    std::fs::create_dir_all(&fake_upload).unwrap();
    let canon = canonicalize_and_check(&fake_upload, &al);
    assert!(canon.is_ok(), "upload dir inside allowlist must be accepted");
}

/// A symlink inside the uploads directory pointing outside must be rejected.
#[test]
fn uploads_allowlist_rejects_symlink_escape() {
    let tmp = TempDir::new().unwrap();
    let uploads_dir = tmp.path().join("uploads");
    let al = PathAllowlist::for_uploads(&uploads_dir).unwrap();

    // Create a symlink inside uploads/ that points to /etc (outside)
    let link = uploads_dir.join("evil-symlink");
    let _ = std::os::unix::fs::symlink("/etc", &link);

    if link.exists() {
        let result = canonicalize_and_check(&link, &al);
        assert!(
            result.is_err(),
            "symlink escaping uploads dir must be rejected"
        );
    }
}

/// The UploadRegistry must reject accessing an upload whose directory is a
/// symlink pointing outside the uploads root.
#[test]
fn upload_registry_rejects_symlink_escape() {
    use hoop_daemon::uploads::{UploadConfig, UploadRegistry};
    use hoop_schema::id_validators::ValidUploadId;

    let tmp = TempDir::new().unwrap();
    let uploads_dir = tmp.path().join("uploads");

    let config = UploadConfig {
        chunk_size: 1024,
        max_file_size: 10 * 1024,
        upload_ttl_hours: 24,
        uploads_dir: uploads_dir.clone(),
    };
    let registry = UploadRegistry::new(config).unwrap();

    // Create a symlink that looks like a valid UUID but points to /tmp
    let fake_uuid = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
    let link = uploads_dir.join(fake_uuid);
    let _ = std::os::unix::fs::symlink("/tmp", &link);

    // Even though the UUID is valid, canonicalize_and_check should reject
    if link.exists() {
        let valid_id = ValidUploadId::parse(fake_uuid).unwrap();
        let result = registry.get_progress(&valid_id);
        assert!(
            result.is_err(),
            "symlink-escaped upload directory must be rejected"
        );
    }
}
