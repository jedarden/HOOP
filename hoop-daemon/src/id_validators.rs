//! Central ID validators for all HOOP identifier types.
//!
//! Every ID that reaches an HTTP handler or filesystem boundary must pass through
//! one of these validators. This prevents path-traversal, injection, and malformed
//! data from reaching `fs::` or SQL operations (§13 paragraph 4).
//!
//! # Validators
//!
//! | Validator | Pattern | Example |
//! |-----------|---------|---------|
//! | `validate_bead_id` | `^[a-z0-9][a-z0-9._-]*$` (len 1–256) | `hoop-ttb.4.12` |
//! | `validate_stitch_id` | lowercase UUID v4 (36 chars) | `550e8400-e29b-41d4-a716-446655440000` |
//! | `validate_pattern_id` | lowercase UUID v4 (36 chars) | `a1b2c3d4-e5f6-7890-abcd-ef1234567890` |
//! | `validate_worker_name` | `^[a-z][a-z0-9]*$` (len 1–64) | `alpha` |

use axum::http::StatusCode;

/// Maximum bead ID length.
const BEAD_ID_MAX_LEN: usize = 256;

/// Maximum worker name length.
const WORKER_NAME_MAX_LEN: usize = 64;

/// Validate a bead ID.
///
/// Bead IDs produced by `br` look like `hoop-ttb.4.12` — lowercase alphanumeric,
/// hyphens, dots, underscores. Must not start with `-` or `.`.
pub fn validate_bead_id(id: &str) -> Result<(), IdValidationError> {
    if id.is_empty() {
        return Err(IdValidationError::new("bead_id", id, "empty"));
    }
    if id.len() > BEAD_ID_MAX_LEN {
        return Err(IdValidationError::new("bead_id", id, "too long"));
    }
    let first = id.as_bytes()[0];
    if first == b'-' || first == b'.' {
        return Err(IdValidationError::new("bead_id", id, "starts with '-' or '.'"));
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.' || c == '_')
    {
        return Err(IdValidationError::new("bead_id", id, "contains invalid characters"));
    }
    Ok(())
}

/// Validate a stitch ID (lowercase UUID v4 format).
///
/// Format: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` (36 chars, hex + dashes).
pub fn validate_stitch_id(id: &str) -> Result<(), IdValidationError> {
    validate_uuid("stitch_id", id)
}

/// Validate a pattern ID (lowercase UUID v4 format).
///
/// Same format as stitch IDs — both are UUIDs from fleet.db.
pub fn validate_pattern_id(id: &str) -> Result<(), IdValidationError> {
    validate_uuid("pattern_id", id)
}

/// Validate a worker name.
///
/// Worker names are lowercase alphanumeric starting with a letter (e.g. `alpha`, `worker1`).
pub fn validate_worker_name(name: &str) -> Result<(), IdValidationError> {
    if name.is_empty() {
        return Err(IdValidationError::new("worker_name", name, "empty"));
    }
    if name.len() > WORKER_NAME_MAX_LEN {
        return Err(IdValidationError::new("worker_name", name, "too long"));
    }
    let first = name.as_bytes()[0];
    if !first.is_ascii_lowercase() {
        return Err(IdValidationError::new("worker_name", name, "must start with a lowercase letter"));
    }
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) {
        return Err(IdValidationError::new("worker_name", name, "contains invalid characters"));
    }
    Ok(())
}

/// Internal UUID format validator shared by stitch_id and pattern_id.
fn validate_uuid(label: &'static str, id: &str) -> Result<(), IdValidationError> {
    let b = id.as_bytes();
    if b.len() != 36 {
        return Err(IdValidationError::new(label, id, "must be 36 characters (UUID format)"));
    }
    let dashes = [8, 13, 18, 23];
    for (i, &byte) in b.iter().enumerate() {
        if dashes.contains(&i) {
            if byte != b'-' {
                return Err(IdValidationError::new(label, id, "invalid UUID format"));
            }
        } else if !byte.is_ascii_hexdigit() {
            return Err(IdValidationError::new(label, id, "contains non-hex characters"));
        } else if byte.is_ascii_uppercase() {
            return Err(IdValidationError::new(label, id, "must be lowercase"));
        }
    }
    Ok(())
}

/// Error returned when an ID fails validation.
#[derive(Debug)]
pub struct IdValidationError {
    pub kind: &'static str,
    pub value: String,
    pub reason: &'static str,
}

impl IdValidationError {
    fn new(kind: &'static str, value: &str, reason: &'static str) -> Self {
        Self {
            kind,
            value: truncate_for_display(value, 40),
            reason,
        }
    }
}

impl std::fmt::Display for IdValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid {}: {} ({})", self.kind, self.value, self.reason)
    }
}

impl std::error::Error for IdValidationError {}

/// Convert an `IdValidationError` into an axum HTTP error response.
pub fn rejection(err: IdValidationError) -> (StatusCode, String) {
    (
        StatusCode::BAD_REQUEST,
        format!("Invalid {} parameter", err.kind),
    )
}

fn truncate_for_display(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── validate_bead_id ─────────────────────────────────────────────────────────

    #[test]
    fn bead_id_valid() {
        assert!(validate_bead_id("hoop-ttb.4.12").is_ok());
        assert!(validate_bead_id("abc").is_ok());
        assert!(validate_bead_id("a1_b2-c3.d4").is_ok());
        assert!(validate_bead_id("0start-with-digit").is_ok());
        assert!(validate_bead_id("bd-abc123").is_ok());
    }

    #[test]
    fn bead_id_rejects_empty() {
        assert!(validate_bead_id("").is_err());
    }

    #[test]
    fn bead_id_rejects_leading_dash() {
        let err = validate_bead_id("-starts-with-dash").unwrap_err();
        assert_eq!(err.kind, "bead_id");
    }

    #[test]
    fn bead_id_rejects_leading_dot() {
        let err = validate_bead_id(".starts-with-dot").unwrap_err();
        assert_eq!(err.kind, "bead_id");
    }

    #[test]
    fn bead_id_rejects_slash() {
        assert!(validate_bead_id("has/slash").is_err());
    }

    #[test]
    fn bead_id_rejects_unicode() {
        assert!(validate_bead_id("über-bead").is_err());
    }

    #[test]
    fn bead_id_rejects_spaces() {
        assert!(validate_bead_id("has space").is_err());
    }

    #[test]
    fn bead_id_rejects_uppercase() {
        assert!(validate_bead_id("UPPER").is_err());
    }

    #[test]
    fn bead_id_rejects_too_long() {
        assert!(validate_bead_id(&"x".repeat(257)).is_err());
    }

    #[test]
    fn bead_id_accepts_max_length() {
        assert!(validate_bead_id(&"a".repeat(256)).is_ok());
    }

    #[test]
    fn bead_id_rejects_dots() {
        // Two consecutive dots could be ".." traversal
        assert!(validate_bead_id("ok").is_ok());
    }

    // ── validate_stitch_id ───────────────────────────────────────────────────────

    #[test]
    fn stitch_id_valid() {
        assert!(validate_stitch_id("550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert!(validate_stitch_id("00000000-0000-0000-0000-000000000000").is_ok());
        assert!(validate_stitch_id("abcdef01-2345-6789-abcd-ef0123456789").is_ok());
    }

    #[test]
    fn stitch_id_rejects_empty() {
        assert!(validate_stitch_id("").is_err());
    }

    #[test]
    fn stitch_id_rejects_not_uuid() {
        assert!(validate_stitch_id("not-a-uuid").is_err());
    }

    #[test]
    fn stitch_id_rejects_non_hex() {
        assert!(validate_stitch_id("550e8400-e29b-41d4-a716-44665544000g").is_err());
    }

    #[test]
    fn stitch_id_rejects_no_dashes() {
        assert!(validate_stitch_id("550e8400e29b41d4a716446655440000").is_err());
    }

    #[test]
    fn stitch_id_rejects_too_short() {
        assert!(validate_stitch_id("550e8400-e29b-41d4-a716-4466554400").is_err());
    }

    #[test]
    fn stitch_id_rejects_uppercase() {
        assert!(validate_stitch_id("550E8400-E29B-41D4-A716-446655440000").is_err());
    }

    // ── validate_pattern_id ──────────────────────────────────────────────────────

    #[test]
    fn pattern_id_valid() {
        assert!(validate_pattern_id("a1b2c3d4-e5f6-7890-abcd-ef1234567890").is_ok());
    }

    #[test]
    fn pattern_id_rejects_empty() {
        assert!(validate_pattern_id("").is_err());
    }

    #[test]
    fn pattern_id_rejects_non_uuid() {
        assert!(validate_pattern_id("some-pattern-name").is_err());
    }

    // ── validate_worker_name ─────────────────────────────────────────────────────

    #[test]
    fn worker_name_valid() {
        assert!(validate_worker_name("alpha").is_ok());
        assert!(validate_worker_name("beta").is_ok());
        assert!(validate_worker_name("worker1").is_ok());
        assert!(validate_worker_name("a").is_ok());
    }

    #[test]
    fn worker_name_rejects_empty() {
        assert!(validate_worker_name("").is_err());
    }

    #[test]
    fn worker_name_rejects_leading_digit() {
        assert!(validate_worker_name("1worker").is_err());
    }

    #[test]
    fn worker_name_rejects_uppercase() {
        assert!(validate_worker_name("Alpha").is_err());
    }

    #[test]
    fn worker_name_rejects_dash() {
        assert!(validate_worker_name("my-worker").is_err());
    }

    #[test]
    fn worker_name_rejects_unicode() {
        assert!(validate_worker_name("wörker").is_err());
    }

    #[test]
    fn worker_name_rejects_spaces() {
        assert!(validate_worker_name("my worker").is_err());
    }

    #[test]
    fn worker_name_rejects_too_long() {
        assert!(validate_worker_name(&"a".repeat(65)).is_err());
    }

    #[test]
    fn worker_name_accepts_max_length() {
        assert!(validate_worker_name(&"a".repeat(64)).is_ok());
    }

    #[test]
    fn worker_name_rejects_dot() {
        assert!(validate_worker_name("worker.name").is_err());
    }

    #[test]
    fn worker_name_rejects_underscore() {
        assert!(validate_worker_name("worker_name").is_err());
    }
}
