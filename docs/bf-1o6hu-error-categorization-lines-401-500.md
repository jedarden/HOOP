# Error Message Categorization: Lines 401-500

**Bead:** bf-1o6hu  
**Source:** `hoop-cli/tests/cli_test_helpers.rs` lines 401-500  
**Generated:** 2026-08-12

## Categorization Results

### Messages Found

All 5 "messages" from this range are assert statements in documentation comments (`//!`), not actual runtime error messages:

```
403: scan (assert_eq! in doc comment) -> other
410: scan (assert! in doc comment) -> other  
437: scan (assert_eq! in doc comment) -> other
442: scan (assert_eq! in doc comment) -> other
445: scan (assert! in doc comment) -> other
```

### Category Breakdown

| Category | Count | Percentage |
|----------|-------|------------|
| other | 5 | 100% |
| failed to | 0 | 0% |
| could not | 0 | 0% |
| unable to | 0 | 0% |
| error | 0 | 0% |
| invalid | 0 | 0% |
| not found | 0 | 0% |

## Notes

- **Lines 401-500 are entirely module-level documentation** (comments with `//!`)
- No actual runtime error messages exist in this range
- All extracted patterns are code examples showing how test utilities work
- For comparison, lines 1-100 and 101-200 contain actual test code with real error messages

## Context

This range documents:
- Flag position consistency utilities
- Test helper functions for parsing clap command structures  
- Documentation examples showing `assert_eq!` and `assert!` usage patterns
- No executable test code or error handling logic
