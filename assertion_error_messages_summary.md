# HOOP Test Suite Assertion Error Messages

## Overview
This report catalogs assertion error messages extracted from HOOP test files on 2026-08-12.

## Statistics
- **Total test files processed**: 452
- **Total assertion patterns found**: 673
- **Unique error messages**: 444
- **Files with assertions**: 89

## Pattern Distribution
| Pattern Type | Count | Percentage |
|--------------|-------|------------|
| assert_eq! | 465 | 69.1% |
| assert! | 205 | 30.5% |
| assert_ne! | 3 | 0.4% |
| expect! | 0 | 0% |
| expect_eq! | 0 | 0% |
| expect_ne! | 0 | 0% |
| expect_err! | 0 | 0% |
| panic! | 0 | 0% |
| unwrap() | 0 | 0% |
| unwrap_err() | 0 | 0% |

**Note**: expect!, panic!, unwrap(), and unwrap_err() patterns were searched but not found with explicit error messages in the test files.

## Files with Most Assertions
1. `hoop-daemon/tests/property_invariants.rs` - 38 assertions
2. `hoop-daemon/tests/backup_restore_cycle.rs` - 31 assertions
3. `hoop-daemon/tests/multi_operator_concurrency.rs` - 25 assertions
4. `tests/acceptance/s4_daemon_restart.rs` - 23 assertions
5. `hoop-daemon/tests/reflection_detector_integration.rs` - 22 assertions

## Common Error Message Categories

### State Validation
- "State should be deleted"
- "Config should have encryption enabled/disabled"
- "Status should be a JSON object"
- "Projects should be an array"

### Operational Flow
- "Worker should have written events"
- "Daemon should still be healthy"
- "Draft should appear in queue"
- "Should succeed when [condition]"

### Failure Conditions
- "Should return None when [condition]"
- "Should fail when [condition]"
- "Backup should fail when encryption enabled but age key missing"

### Configuration Validation
- "Flag should be true/false"
- "no_interactive should be true/false"
- "Flag should be present/absent"

## Sample Error Messages by Pattern Type

### assert_eq! Examples
- "Flag should be true when present before init"
- "no_interactive should be true"
- "no_interactive value must be consistent"
- "Each project should be an object"
- "Should be parseable by jq"

### assert! Examples
- "State should be deleted"
- "Should succeed when encryption disabled"
- "age_key should be None when encryption disabled"
- "Should succeed when age key provided"
- "Encrypted file should exist"

### assert_ne! Examples
- "Events should not be empty"
- "Counts should differ"
- "Values should not be equal"

## Files Covered
- hoop-daemon/tests/ (76 files with assertions)
- hoop-cli/src/ (test modules in source files)
- hoop-mcp/tests/ (3 files with assertions)
- tests/acceptance/ (6 files with assertions)
- testrepo/tests/ (integration and example tests)

## Methodology
1. **File Discovery**: Scanned for test files using patterns:
   - `tests/**/*.rs`
   - `**/*test*.rs`
   - `hoop-*/tests/**/*.rs`
   - Files containing `#[test]` or `#[cfg(test)]`

2. **Pattern Matching**: Used regex patterns for:
   - `assert!(condition, "message")`
   - `assert_eq!(left, right, "message")`
   - `assert_ne!(left, right, "message")`
   - `expect!(condition, "message")`
   - `expect_eq!(left, right, "message")`
   - `expect_ne!(left, right, "message")`
   - `expect_err!(expression, "message")`
   - `panic!("message")`
   - `.unwrap().expect("message")`
   - `.unwrap_err().expect("message")`

3. **Data Extracted**: For each assertion:
   - File path (relative to HOOP root)
   - Line number
   - Pattern type (assert!, assert_eq!, etc.)
   - Error message text
   - Full line content
   - Matched text

## Output Format
Full results saved as JSON: `assertion_error_messages.json`

Each entry contains:
```json
{
  "file_path": "path/to/file.rs",
  "line_number": 123,
  "pattern_type": "assert_eq!",
  "error_message": "Actual error message text",
  "line_content": "assert_eq!(a, b, \"Actual error message text\");",
  "match_text": "assert_eq!(a, b, \"Actual error message text\")"
}
```

## Next Steps
This extraction covers assertion macros only. Separate extractions needed for:
1. Error type variants (custom error enums)
2. anyhow! error messages
3. Eyre/snip-clip error messages
4. anyhow::anyhow!() calls
5. Error::custom() calls

## Generated
- Date: 2026-08-12
- Bead: bf-2tkio
- Script: extract_assertion_errors.py
