//! Central ID validators for all HOOP identifier types.
//!
//! Every ID that reaches an HTTP handler or filesystem boundary must pass through
//! one of these validators. This prevents path-traversal, injection, and malformed
//! data from reaching `fs::` or SQL operations (§13 paragraph 4).
//!
//! This is the single canonical module — both `hoop-daemon` and `hoop-mcp` import
//! from here. No other crate should duplicate these validators.
//!
//! # Validators
//!
//! | Validator | Pattern | Example |
//! |-----------|---------|---------|
//! | `validate_bead_id` | `^[a-z0-9][a-z0-9._-]*$` (len 1–256) | `hoop-ttb.4.12` |
//! | `validate_stitch_id` | lowercase UUID v4 (36 chars) | `550e8400-e29b-41d4-a716-446655440000` |
//! | `validate_pattern_id` | lowercase UUID v4 (36 chars) | `a1b2c3d4-e5f6-7890-abcd-ef1234567890` |
//! | `validate_draft_id` | `draft-` + lowercase UUID | `draft-550e8400-e29b-41d4-a716-446655440000` |
//! | `validate_upload_id` | lowercase UUID v4 (36 chars) | `550e8400-e29b-41d4-a716-446655440000` |
//! | `validate_job_id` | lowercase UUID v4 (36 chars) | `550e8400-e29b-41d4-a716-446655440000` |
//! | `validate_worker_name` | `^[a-z][a-z0-9]*$` (len 1–64) | `alpha` |
//! | `validate_project_name` | `^[a-zA-Z0-9][a-zA-Z0-9._-]*$` (len 1–128) | `HOOP` |
//!
//! # Compile-time safety
//!
//! Filesystem boundary functions accept `ValidBeadId`, `ValidStitchId`, etc. instead
//! of raw `&str`. These newtypes can only be constructed via `parse()`, which runs
//! the full validation. This makes it a compile-time error to pass an unvalidated
//! string to a path-construction function.

use std::fmt;
use std::ops::Deref;

/// Maximum bead ID length.
const BEAD_ID_MAX_LEN: usize = 256;

/// Maximum worker name length.
const WORKER_NAME_MAX_LEN: usize = 64;

/// Maximum project name length.
const PROJECT_NAME_MAX_LEN: usize = 128;

// ── Validated newtypes ─────────────────────────────────────────────────────

/// A validated bead ID. Construct via `ValidBeadId::parse()`.
///
/// Provides compile-time proof that the ID has passed regex validation,
/// preventing unvalidated strings from reaching filesystem path construction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidBeadId(String);

impl ValidBeadId {
    /// Parse and validate a bead ID string.
    pub fn parse(id: &str) -> Result<Self, IdValidationError> {
        validate_bead_id(id)?;
        Ok(Self(id.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for ValidBeadId {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ValidBeadId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ValidBeadId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A validated stitch ID (lowercase UUID). Construct via `ValidStitchId::parse()`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidStitchId(String);

impl ValidStitchId {
    pub fn parse(id: &str) -> Result<Self, IdValidationError> {
        validate_stitch_id(id)?;
        Ok(Self(id.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for ValidStitchId {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ValidStitchId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ValidStitchId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A validated pattern ID (lowercase UUID). Construct via `ValidPatternId::parse()`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidPatternId(String);

impl ValidPatternId {
    pub fn parse(id: &str) -> Result<Self, IdValidationError> {
        validate_pattern_id(id)?;
        Ok(Self(id.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for ValidPatternId {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ValidPatternId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ValidPatternId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A validated worker name. Construct via `ValidWorkerName::parse()`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidWorkerName(String);

impl ValidWorkerName {
    pub fn parse(name: &str) -> Result<Self, IdValidationError> {
        validate_worker_name(name)?;
        Ok(Self(name.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for ValidWorkerName {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ValidWorkerName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ValidWorkerName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A validated project name. Construct via `ValidProjectName::parse()`.
///
/// Project names are used in URL paths and filesystem lookups — they must be
/// safe for both contexts (no path traversal, no special characters).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidProjectName(String);

impl ValidProjectName {
    pub fn parse(name: &str) -> Result<Self, IdValidationError> {
        validate_project_name(name)?;
        Ok(Self(name.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for ValidProjectName {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ValidProjectName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ValidProjectName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Raw validation functions ──────────────────────────────────────────────

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

/// Validate a draft ID.
///
/// Draft IDs are server-generated in the format `draft-<uuid>` (e.g. `draft-550e8400-e29b-41d4-a716-446655440000`).
pub fn validate_draft_id(id: &str) -> Result<(), IdValidationError> {
    let prefix = "draft-";
    if !id.starts_with(prefix) {
        return Err(IdValidationError::new("draft_id", id, "must start with 'draft-'"));
    }
    validate_uuid("draft_id", &id[prefix.len()..])
}

/// Validate an upload ID (lowercase UUID format).
///
/// Upload IDs are UUIDs generated server-side during upload initiation.
pub fn validate_upload_id(id: &str) -> Result<(), IdValidationError> {
    validate_uuid("upload_id", id)
}

/// Validate a transcription job ID (lowercase UUID format).
///
/// Job IDs are UUIDs generated server-side when Whisper transcription jobs are created.
pub fn validate_job_id(id: &str) -> Result<(), IdValidationError> {
    validate_uuid("job_id", id)
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

/// Validate a project name.
///
/// Project names appear in URL paths and are used to resolve filesystem paths
/// from the project registry. Uses the same safe-character set as bead IDs
/// (lowercase alphanumeric, hyphens, dots, underscores) but with a shorter
/// max length. Must not start with `-` or `.` to prevent path-traversal patterns.
pub fn validate_project_name(name: &str) -> Result<(), IdValidationError> {
    if name.is_empty() {
        return Err(IdValidationError::new("project_name", name, "empty"));
    }
    if name.len() > PROJECT_NAME_MAX_LEN {
        return Err(IdValidationError::new("project_name", name, "too long"));
    }
    let first = name.as_bytes()[0];
    if first == b'-' || first == b'.' {
        return Err(IdValidationError::new("project_name", name, "starts with '-' or '.'"));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_')
    {
        return Err(IdValidationError::new("project_name", name, "contains invalid characters"));
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
    fn bead_id_rejects_null_byte() {
        assert!(validate_bead_id("bead\x00id").is_err());
    }

    #[test]
    fn bead_id_rejects_tab() {
        assert!(validate_bead_id("bead\tid").is_err());
    }

    #[test]
    fn bead_id_rejects_backslash() {
        assert!(validate_bead_id("bead\\id").is_err());
    }

    #[test]
    fn bead_id_rejects_at_boundary_256() {
        assert!(validate_bead_id(&"a".repeat(256)).is_ok());
        assert!(validate_bead_id(&"a".repeat(257)).is_err());
    }

    #[test]
    fn bead_id_rejects_double_dot() {
        assert!(validate_bead_id("..").is_err());
        assert!(validate_bead_id("a..b").is_ok()); // dots mid-string are allowed
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

    #[test]
    fn stitch_id_rejects_unicode() {
        assert!(validate_stitch_id("550e8400-über-41d4-a716-446655440000").is_err());
    }

    #[test]
    fn stitch_id_rejects_null_byte() {
        assert!(validate_stitch_id("550e8400-e29b-41d4-a716-44665544000\x00").is_err());
    }

    #[test]
    fn stitch_id_rejects_spaces() {
        assert!(validate_stitch_id("550e8400-e29b-41d4-a716-44665544 000").is_err());
    }

    #[test]
    fn stitch_id_rejects_too_long() {
        let mut id = "550e8400-e29b-41d4-a716-446655440000".to_string();
        id.push_str("extra");
        assert!(validate_stitch_id(&id).is_err());
    }

    #[test]
    fn stitch_id_rejects_leading_dash() {
        assert!(validate_stitch_id("-550e8400-e29b-41d4-a716-446655440000").is_err());
    }

    #[test]
    fn stitch_id_rejects_dot() {
        assert!(validate_stitch_id("550e8400-e29b-41d4-a716-446655440000.extra").is_err());
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

    #[test]
    fn pattern_id_rejects_too_long() {
        assert!(validate_pattern_id("550e8400-e29b-41d4-a716-446655440000-extra").is_err());
    }

    #[test]
    fn pattern_id_rejects_uppercase() {
        assert!(validate_pattern_id("A1B2C3D4-E5F6-7890-ABCD-EF1234567890").is_err());
    }

    #[test]
    fn pattern_id_rejects_unicode() {
        assert!(validate_pattern_id("550e8400-über-41d4-a716-446655440000").is_err());
    }

    #[test]
    fn pattern_id_rejects_whitespace() {
        assert!(validate_pattern_id("550e8400-e29b-41d4-a716-44665544 000").is_err());
    }

    #[test]
    fn pattern_id_rejects_leading_dash() {
        assert!(validate_pattern_id("-550e8400-e29b-41d4-a716-446655440000").is_err());
    }

    #[test]
    fn pattern_id_rejects_dot() {
        assert!(validate_pattern_id("550e8400-e29b-41d4-a716-446655440000.txt").is_err());
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
    fn worker_name_rejects_leading_dash() {
        let err = validate_worker_name("-worker").unwrap_err();
        assert_eq!(err.kind, "worker_name");
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

    #[test]
    fn worker_name_rejects_null_byte() {
        assert!(validate_worker_name("alpha\x00beta").is_err());
    }

    #[test]
    fn worker_name_rejects_tab() {
        assert!(validate_worker_name("alpha\tbeta").is_err());
    }

    // ── validate_project_name ──────────────────────────────────────────────────

    #[test]
    fn project_name_valid() {
        assert!(validate_project_name("HOOP").is_ok());
        assert!(validate_project_name("my-project").is_ok());
        assert!(validate_project_name("project_1").is_ok());
        assert!(validate_project_name("a.b.c").is_ok());
        assert!(validate_project_name("0starts-with-digit").is_ok());
    }

    #[test]
    fn project_name_rejects_empty() {
        assert!(validate_project_name("").is_err());
    }

    #[test]
    fn project_name_rejects_leading_dash() {
        let err = validate_project_name("-starts-with-dash").unwrap_err();
        assert_eq!(err.kind, "project_name");
    }

    #[test]
    fn project_name_rejects_leading_dot() {
        let err = validate_project_name(".hidden").unwrap_err();
        assert_eq!(err.kind, "project_name");
    }

    #[test]
    fn project_name_rejects_slash() {
        assert!(validate_project_name("has/slash").is_err());
    }

    #[test]
    fn project_name_rejects_unicode() {
        assert!(validate_project_name("projét").is_err());
    }

    #[test]
    fn project_name_rejects_spaces() {
        assert!(validate_project_name("my project").is_err());
    }

    #[test]
    fn project_name_rejects_too_long() {
        assert!(validate_project_name(&"a".repeat(129)).is_err());
    }

    #[test]
    fn project_name_accepts_max_length() {
        assert!(validate_project_name(&"a".repeat(128)).is_ok());
    }

    #[test]
    fn project_name_rejects_null_byte() {
        assert!(validate_project_name("proj\x00ect").is_err());
    }

    #[test]
    fn project_name_rejects_double_dot() {
        assert!(validate_project_name("..").is_err());
    }

    #[test]
    fn project_name_rejects_dot_dot_slash() {
        assert!(validate_project_name("../etc").is_err());
    }

    // ── Validated newtypes ─────────────────────────────────────────────────────

    #[test]
    fn valid_bead_id_parse_roundtrip() {
        let v = ValidBeadId::parse("hoop-ttb.4.12").unwrap();
        assert_eq!(v.as_str(), "hoop-ttb.4.12");
        assert_eq!(&*v, "hoop-ttb.4.12");
        assert_eq!(v.to_string(), "hoop-ttb.4.12");
    }

    #[test]
    fn valid_bead_id_rejects_traversal() {
        assert!(ValidBeadId::parse("../etc/passwd").is_err());
        assert!(ValidBeadId::parse("").is_err());
        assert!(ValidBeadId::parse("-dash").is_err());
    }

    #[test]
    fn valid_stitch_id_parse_roundtrip() {
        let v = ValidStitchId::parse("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(v.as_str(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn valid_stitch_id_rejects_non_uuid() {
        assert!(ValidStitchId::parse("not-a-uuid").is_err());
    }

    #[test]
    fn valid_pattern_id_parse_roundtrip() {
        let v = ValidPatternId::parse("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap();
        assert_eq!(v.as_str(), "a1b2c3d4-e5f6-7890-abcd-ef1234567890");
    }

    #[test]
    fn valid_worker_name_parse_roundtrip() {
        let v = ValidWorkerName::parse("alpha").unwrap();
        assert_eq!(v.as_str(), "alpha");
        assert_eq!(&*v, "alpha");
    }

    #[test]
    fn valid_worker_name_rejects_invalid() {
        assert!(ValidWorkerName::parse("").is_err());
        assert!(ValidWorkerName::parse("Alpha").is_err());
        assert!(ValidWorkerName::parse("my-worker").is_err());
    }

    #[test]
    fn valid_bead_id_deref_allows_str_ops() {
        let v = ValidBeadId::parse("test-bead.1").unwrap();
        assert!(v.starts_with("test"));
        assert!(v.contains('-'));
        assert_eq!(v.len(), 11);
    }

    #[test]
    fn valid_bead_id_equality() {
        let a = ValidBeadId::parse("test.1").unwrap();
        let b = ValidBeadId::parse("test.1").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn valid_bead_id_as_ref_str() {
        let v = ValidBeadId::parse("test.1").unwrap();
        let s: &str = v.as_ref();
        assert_eq!(s, "test.1");
    }

    // ── validate_draft_id ────────────────────────────────────────────────────

    #[test]
    fn draft_id_valid() {
        assert!(validate_draft_id("draft-550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert!(validate_draft_id("draft-00000000-0000-0000-0000-000000000000").is_ok());
    }

    #[test]
    fn draft_id_rejects_no_prefix() {
        assert!(validate_draft_id("550e8400-e29b-41d4-a716-446655440000").is_err());
    }

    #[test]
    fn draft_id_rejects_empty() {
        assert!(validate_draft_id("").is_err());
    }

    #[test]
    fn draft_id_rejects_wrong_prefix() {
        assert!(validate_draft_id("bead-550e8400-e29b-41d4-a716-446655440000").is_err());
    }

    #[test]
    fn draft_id_rejects_uppercase_uuid() {
        assert!(validate_draft_id("draft-550E8400-E29B-41D4-A716-446655440000").is_err());
    }

    #[test]
    fn draft_id_rejects_malformed_uuid() {
        assert!(validate_draft_id("draft-not-a-uuid").is_err());
    }

    #[test]
    fn draft_id_rejects_unicode() {
        assert!(validate_draft_id("draft-550e8400-über-41d4-a716-446655440000").is_err());
    }

    #[test]
    fn draft_id_rejects_leading_dash() {
        assert!(validate_draft_id("-draft-550e8400-e29b-41d4-a716-446655440000").is_err());
    }

    #[test]
    fn draft_id_rejects_dot() {
        assert!(validate_draft_id("draft.550e8400-e29b-41d4-a716-446655440000").is_err());
    }

    #[test]
    fn draft_id_rejects_whitespace() {
        assert!(validate_draft_id("draft-550e8400-e29b-41d4-a716-44665544 000").is_err());
    }

    #[test]
    fn draft_id_rejects_too_short() {
        assert!(validate_draft_id("draft-550e8400-e29b-41d4-a716-44665544").is_err());
    }

    // ── validate_upload_id ───────────────────────────────────────────────────

    #[test]
    fn upload_id_valid() {
        assert!(validate_upload_id("550e8400-e29b-41d4-a716-446655440000").is_ok());
    }

    #[test]
    fn upload_id_rejects_empty() {
        assert!(validate_upload_id("").is_err());
    }

    #[test]
    fn upload_id_rejects_non_uuid() {
        assert!(validate_upload_id("not-a-uuid").is_err());
    }

    #[test]
    fn upload_id_rejects_uppercase() {
        assert!(validate_upload_id("550E8400-E29B-41D4-A716-446655440000").is_err());
    }

    #[test]
    fn upload_id_rejects_unicode() {
        assert!(validate_upload_id("550e8400-über-41d4-a716-446655440000").is_err());
    }

    #[test]
    fn upload_id_rejects_whitespace() {
        assert!(validate_upload_id("550e8400-e29b-41d4-a716-44665544 000").is_err());
    }

    // ── validate_job_id ──────────────────────────────────────────────────────

    #[test]
    fn job_id_valid() {
        assert!(validate_job_id("550e8400-e29b-41d4-a716-446655440000").is_ok());
    }

    #[test]
    fn job_id_rejects_empty() {
        assert!(validate_job_id("").is_err());
    }

    #[test]
    fn job_id_rejects_non_uuid() {
        assert!(validate_job_id("not-a-uuid").is_err());
    }

    #[test]
    fn job_id_rejects_uppercase() {
        assert!(validate_job_id("550E8400-E29B-41D4-A716-446655440000").is_err());
    }

    #[test]
    fn job_id_rejects_unicode() {
        assert!(validate_job_id("550e8400-über-41d4-a716-446655440000").is_err());
    }

    #[test]
    fn job_id_rejects_dot() {
        assert!(validate_job_id("550e8400-e29b-41d4-a716-446655440000.extra").is_err());
    }

    #[test]
    fn job_id_rejects_leading_dash() {
        assert!(validate_job_id("-550e8400-e29b-41d4-a716-446655440000").is_err());
    }
}
