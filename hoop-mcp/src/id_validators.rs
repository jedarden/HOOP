//! Lightweight ID validators for the MCP server.
//!
//! These mirror `hoop-daemon::id_validators` so the MCP server can reject
//! malformed IDs before they reach HTTP or DB boundaries. The canonical
//! module lives in `hoop-daemon/src/id_validators.rs`; this copy exists
//! because `hoop-mcp` does not depend on `hoop-daemon`.

/// Validate a stitch ID (lowercase UUID v4 format, 36 chars).
pub fn validate_stitch_id(id: &str) -> Result<(), String> {
    let b = id.as_bytes();
    if b.len() != 36 {
        return Err(format!("invalid stitch_id: must be 36 characters (UUID format)"));
    }
    let dashes = [8, 13, 18, 23];
    for (i, &byte) in b.iter().enumerate() {
        if dashes.contains(&i) {
            if byte != b'-' {
                return Err("invalid stitch_id: invalid UUID format".into());
            }
        } else if !byte.is_ascii_hexdigit() {
            return Err("invalid stitch_id: contains non-hex characters".into());
        } else if byte.is_ascii_uppercase() {
            return Err("invalid stitch_id: must be lowercase".into());
        }
    }
    Ok(())
}

/// Validate a bead ID.
///
/// Bead IDs are lowercase alphanumeric with hyphens, dots, underscores.
/// Must not start with `-` or `.`. Max 256 chars.
pub fn validate_bead_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("invalid bead_id: empty".into());
    }
    if id.len() > 256 {
        return Err("invalid bead_id: too long".into());
    }
    let first = id.as_bytes()[0];
    if first == b'-' || first == b'.' {
        return Err("invalid bead_id: starts with '-' or '.'".into());
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.' || c == '_')
    {
        return Err("invalid bead_id: contains invalid characters".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stitch_id_valid() {
        assert!(validate_stitch_id("550e8400-e29b-41d4-a716-446655440000").is_ok());
    }

    #[test]
    fn stitch_id_rejects_garbage() {
        assert!(validate_stitch_id("not-a-uuid").is_err());
        assert!(validate_stitch_id("").is_err());
    }

    #[test]
    fn bead_id_valid() {
        assert!(validate_bead_id("hoop-ttb.4.12").is_ok());
        assert!(validate_bead_id("bd-abc123").is_ok());
    }

    #[test]
    fn bead_id_rejects_traversal() {
        assert!(validate_bead_id("").is_err());
        assert!(validate_bead_id("-dash").is_err());
        assert!(validate_bead_id(".dot").is_err());
        assert!(validate_bead_id("has/slash").is_err());
        assert!(validate_bead_id("has space").is_err());
    }
}
