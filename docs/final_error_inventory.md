# HOOP Test Suite Error Message Inventory

**Generated:** 2026-08-12T12:58:17Z  
**Bead:** bf-yucsh (Final deliverable)  
**Previous Data Source:** bf-5z50g (comprehensive error extraction)

## Summary

This document provides a comprehensive inventory of all error messages across the HOOP test suite, organized for both programmatic access and human review.

| Metric | Count |
|--------|-------|
| **Total Test Files** | 120 |
| **Files with Errors** | 97 |
| **Total Error Messages** | 2,944 |
| **Test Functions Extracted** | 1,264 |
| **Errors Mapped to Tests** | 2,749 (93.4%) |
| **Errors Outside Tests** | 195 (6.6%) |

## Error Type Distribution

| Error Type | Count | Percentage |
|------------|-------|------------|
| `expect()` | 1,904 | 64.7% |
| `assert!` | 495 | 16.8% |
| `assert_eq!` | 377 | 12.8% |
| `panic!` | 94 | 3.2% |
| `unwrap()` | 35 | 1.2% |
| `bail!` | 21 | 0.7% |
| `error return` | 17 | 0.6% |
| `assert_ne!` | 1 | 0.03% |

## File Organization

Files are organized by crate and function:

- **hoop-cli/**: Command-line interface tests (68 files, 1,847 errors)
- **hoop-daemon/**: Daemon process tests (42 files, 876 errors)
- **hoop-mcp/**: MCP server tests (8 files, 134 errors)
- **hoop-ui/**: Web interface tests (2 files, 87 errors)

## Usage

### Programmatic Access

The companion JSON file (`final_error_inventory.json`) contains structured data:

```json
{
  "generated": "2026-08-12T12:58:17Z",
  "summary": {...},
  "error_type_counts": {...},
  "errors_by_file_and_test": {
    "hoop-cli/tests/clap_test_utils.rs": {
      "test_all_commands_no_interactive": [
        {
          "type": "assert_eq",
          "message": "Some tests failed: {:?}",
          "line": 1450,
          "raw_line": "//     assert_eq!(failures.len(), 0, \"Some tests failed: {:?}\", failures);",
          "test_name": "test_all_commands_no_interactive"
        }
      ]
    }
  }
}
```

### Query Examples

**Find all panic messages:**
```bash
jq '.errors_by_file_and_test[][] | select(.type == "panic")' docs/final_error_inventory.json
```

**Get errors from a specific test:**
```bash
jq '.errors_by_file_and_test["hoop-cli/tests/cli_test_helpers.rs"]["test_verify_flag_propagation"]' docs/final_error_inventory.json
```

**Count errors by type across all tests:**
```bash
jq '.errors_by_file_and_test[][] | .type' docs/final_error_inventory.json | sort | uniq -c
```

## Error Message Patterns

### Common Error Messages

#### 1. File System Operations
- `"Failed to read {}"` - File read failures
- `"Failed to create {}"` - Directory/file creation failures  
- `"Failed to write {}"` - Write operation failures

#### 2. Parsing Errors
- `"Parse error: {}"` - General parse failures
- `"Failed to parse {}"` - Specific parse context failures
- `"Invalid position: {}"` - Invalid command positions

#### 3. Assertion Messages
- `"Should parse with {}"` - Expected parsing behavior
- `"Should parse successfully"` - General success expectations
- `"Must show {}"` - Required error message verification

#### 4. Test Infrastructure
- `"Some tests failed: {:?}"` - Test aggregation failures
- `"Flag propagation failed: {}"` - Flag behavior verification
- `"Default flag assertion failed"` - Default value checks

## Test Files with Highest Error Counts

| File | Error Count | Primary Error Types |
|------|-------------|-------------------|
| `hoop-cli/tests/cli_test_helpers.rs` | 287 | expect, assert, error_string |
| `hoop-cli/tests/clap_test_utils.rs` | 156 | expect, assert_eq, panic |
| `hoop-daemon/tests/capacity_tests.rs` | 98 | assert, panic, unwrap |
| `hoop-cli/tests/cli_test_utils.rs` | 87 | expect, error_string |
| `hoop-mcp/tests/mcp_integration_tests.rs` | 76 | expect, assert |

## Data Quality Notes

### Coverage Statistics
- **93.4%** of error messages are mapped to specific test functions
- **6.6%** occur in helper functions, setup code, or module-level contexts
- **100%** of files with errors were successfully processed

### Extraction Methodology
1. Test functions identified via `#[test]` and `#[tokio::test]` markers
2. Function boundaries determined by brace counting
3. Error messages mapped to containing test by line number range
4. Helper errors (outside test functions) labeled as `"test_name": null`

### Validation
- Line numbers validated against source files
- Error types classified by Rust macro pattern matching
- Raw source lines preserved for context verification

## Maintenance

This inventory should be regenerated when:
- New test files are added
- Major test refactoring occurs
- Error message patterns change significantly
- Before releases requiring comprehensive error validation

**Regeneration command:**
```bash
python3 bin/extract_test_names_for_errors.py > docs/final_error_inventory.json
# Then update this markdown document to match
```

## Related Documentation

- `error_messages_inventory.json` - Original extraction from bf-5z50g
- `test_error_messages_catalog.json` - Alternative catalog format
- `hoop_error_catalog_comprehensive.json` - Detailed per-line catalog
- `bf-rilp7-error-pattern-mapping.md` - Pattern discovery analysis

## Appendix: Error Type Classifications

### `expect()` (1,904 occurrences)
Intentional test failures with descriptive messages. Used for:
- Validating expected parse results
- Checking success conditions
- Verifying state after operations

### `assert!` (495 occurrences)
Boolean condition assertions. Used for:
- General condition checking
- State validation
- Simple true/false expectations

### `assert_eq!` (377 occurrences)
Equality assertions. Used for:
- Value comparisons
- Structure validation
- Result verification

### `panic!` (94 occurrences)
Fatal test failures. Used for:
- Impossible state detection
- Critical invariant violations
- Early failure on invalid conditions

### `unwrap()` (35 occurrences)
Direct unwrapping with panic on failure. Used for:
- Test setup that must succeed
- Configuration loading
- Critical resource acquisition

### `bail!` (21 occurrences)
Early error return (via anyhow). Used for:
- Validation failures
- Early exit from error conditions
- Propagating errors with context

### Error Returns (17 occurrences)
Explicit `Err()` returns. Used for:
- Helper function error propagation
- Expected error conditions
- Error state signaling

---

**This inventory supports comprehensive error message validation and consistency checking across the HOOP test suite.**
