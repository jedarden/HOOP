//! Per-adapter reasoning effort validation.
//!
//! Validates that a given reasoning effort value is valid for a specific adapter.
//! Reference: §13 of reference-feature-inventory.md

/// Check if a reasoning effort value is valid for the given adapter.
///
/// # Arguments
///
/// * `adapter` - The adapter name (e.g., "claude", "codex", "opencode", "gemini", "zai")
/// * `effort` - The reasoning effort value to validate
///
/// # Returns
///
/// * `Ok(())` if the effort is valid for the adapter
/// * `Err(String)` with a descriptive error message if invalid
///
/// # Valid effort levels per adapter
///
/// - **Claude**: `low`, `medium`, `high`, `xhigh`, `max`
/// - **Codex**: `minimal`, `low`, `medium`, `high`, `xhigh`
/// - **Others (opencode, gemini, zai)**: No effort validation (pass-through)
///
/// # Example
///
/// ```ignore
/// use hoop_schema::effort::is_effort_valid_for_provider;
///
/// assert!(is_effort_valid_for_provider("claude", "high").is_ok());
/// assert!(is_effort_valid_for_provider("claude", "minimal").is_err());
/// assert!(is_effort_valid_for_provider("codex", "minimal").is_ok());
/// ```
pub fn is_effort_valid_for_provider(adapter: &str, effort: &str) -> Result<(), String> {
    const CLAUDE_EFFORTS: &[&str] = &["low", "medium", "high", "xhigh", "max"];
    const CODEX_EFFORTS: &[&str] = &["minimal", "low", "medium", "high", "xhigh"];

    match adapter {
        "claude" => {
            if CLAUDE_EFFORTS.contains(&effort) {
                Ok(())
            } else {
                Err(format!(
                    "Invalid reasoning_effort '{effort}' for adapter '{adapter}'. Valid options: {}",
                    CLAUDE_EFFORTS.join(", ")
                ))
            }
        }
        "codex" => {
            if CODEX_EFFORTS.contains(&effort) {
                Ok(())
            } else {
                Err(format!(
                    "Invalid reasoning_effort '{effort}' for adapter '{adapter}'. Valid options: {}",
                    CODEX_EFFORTS.join(", ")
                ))
            }
        }
        _ => {
            // For other adapters (opencode, gemini, zai), we allow any effort value
            // as pass-through. The adapter implementation will handle validation.
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_valid_efforts() {
        for effort in ["low", "medium", "high", "xhigh", "max"] {
            assert!(is_effort_valid_for_provider("claude", effort).is_ok());
        }
    }

    #[test]
    fn test_claude_invalid_efforts() {
        for effort in ["minimal", "invalid", "", "extra_high"] {
            assert!(is_effort_valid_for_provider("claude", effort).is_err());
        }
    }

    #[test]
    fn test_codex_valid_efforts() {
        for effort in ["minimal", "low", "medium", "high", "xhigh"] {
            assert!(is_effort_valid_for_provider("codex", effort).is_ok());
        }
    }

    #[test]
    fn test_codex_invalid_efforts() {
        for effort in ["max", "invalid", "", "extra_high"] {
            assert!(is_effort_valid_for_provider("codex", effort).is_err());
        }
    }

    #[test]
    fn test_other_adapters_pass_through() {
        // Other adapters allow any effort value (pass-through)
        for adapter in ["opencode", "gemini", "zai"] {
            for effort in ["low", "high", "custom", "any_value"] {
                assert!(is_effort_valid_for_provider(adapter, effort).is_ok());
            }
        }
    }

    #[test]
    fn test_error_messages_include_valid_options() {
        let err = is_effort_valid_for_provider("claude", "minimal").unwrap_err();
        assert!(err.contains("Invalid reasoning_effort"));
        assert!(err.contains("'minimal'"));
        assert!(err.contains("low, medium, high, xhigh, max"));
    }
}
