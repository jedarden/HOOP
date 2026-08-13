# Error Messages Table: Lines 401-500

**Source:** `hoop-cli/tests/cli_test_helpers.rs`  
**Range:** Lines 401-500  
**Bead:** bf-b0l85  
**Generated:** 2026-08-13

## Complete Message Catalog

| Line | Error Message | Category | Notes |
|------|---------------|----------|-------|
| 403 | `assert_eq!(before.subcommand, after.subcommand);` | other | Assert statement in doc comment example |
| 410 | `assert!(verify_flag_position_consistency(&["scan", "/tmp"]).is_ok());` | other | Assert statement in doc comment example |
| 437 | `assert_eq!(before.subcommand, Some("scan".to_string()));` | other | Assert statement in doc comment example |
| 442 | `assert_eq!(after.subcommand, Some("scan".to_string()));` | other | Assert statement in doc comment example |
| 445 | `assert!(verify_flag_position_consistency(&["scan", "/tmp"]).is_ok());` | other | Assert statement in doc comment example |

## Category Summary

| Category | Count | Percentage |
|----------|-------|------------|
| other | 5 | 100% |
| **Total** | **5** | **100%** |

## Key Findings

- **Lines 401-500 contain only documentation** (`//!` doc comments)
- **No runtime error messages** exist in this range
- All extracted messages are **code examples** showing test utility usage
- These are **assert statements** embedded in documentation, not actual error handling

## Cross-Reference

For comparison with other line ranges:
- Lines 1-100: Contains runtime error messages with "failed to", "could not" patterns
- Lines 101-200: Contains runtime error messages with "failed to", "could not" patterns  
- Lines 401-500: **Only documentation examples (this file)**

## Related Documentation

- Detailed categorization: `docs/bf-1o6hu-error-categorization-lines-401-500.md`
- Source extraction: `error_messages_lines_401_500.md`
- Test coverage: `test_coverage_report_401_500.md`
