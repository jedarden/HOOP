# HOOP Error Messages Catalog - Analysis Report

## Extraction Summary

**Date:** 2026-08-15
**Total Error Messages Found:** 1,192
**Test Files Analyzed:** 252
**Catalog Files:**
- `hoop_error_messages_catalog.json` - Summary with samples (29KB)
- `hoop_error_messages_complete.json` - Complete dataset (288KB)

## Methodology

### Search Patterns
Error messages were extracted using ripgrep with the following patterns:
- `assert` - assert!, assert_eq!, assert_ne! macros
- `bail` - bail!, ensure! macros
- `expect` - .expect() calls
- `unwrap_err` - .unwrap_err(), .expect_err() calls
- `Error` - Error:: variants, anyhow::Error
- `anyhow` - anyhow::anyhow!, anyhow::Error::msg
- `Err()` - Error return patterns

### File Coverage
Searched across all HOOP test directories:
- `tests/` - Root test directory
- `hoop-cli/tests/` - CLI integration tests
- `hoop-daemon/tests/` - Daemon integration tests
- `hoop-schema/tests/` - Schema tests
- `hoop-mcp/tests/` - MCP protocol tests
- `testrepo/tests/` - Test repository examples

## Pattern Distribution

| Pattern Type | Count | Percentage | Description |
|--------------|-------|------------|-------------|
| expect | 598 | 50.2% | Error messages from .expect() calls |
| assert | 500 | 42.0% | Error messages from assert macros |
| error_return | 39 | 3.3% | Error messages from Err() returns |
| unknown | 29 | 2.4% | Unclassified error patterns |
| bail | 19 | 1.6% | Error messages from bail! macro |
| anyhow | 7 | 0.6% | Error messages from anyhow crate |

## Top Files by Error Message Count

| File | Error Count | Primary Pattern |
|------|--------------|-----------------|
| hoop-daemon/tests_phase5/adapter_failover_test.rs | 119 | assert |
| hoop-daemon/tests/adapter_failover_test.rs | 119 | assert |
| hoop-cli/tests/cli_test_helpers.rs | 104 | expect |
| tests/cli_test_helpers.rs | 89 | expect |
| hoop-daemon/tests/testrepo_integration.rs | 78 | expect |
| hoop-daemon/tests/testrepo_harness_integration.rs | 73 | expect |
| hoop-cli/tests/cli_test_utils.rs | 70 | expect |
| hoop-cli/tests/init_handler_integration_tests.rs | 64 | expect |
| tests/acceptance/s1_morning_review.rs | 43 | mixed |
| tests/acceptance/s4_daemon_restart.rs | 41 | mixed |

## Common Error Message Patterns

### Infrastructure Errors
- "Failed to spawn daemon" (20+ occurrences)
- "Failed to fetch/parse X" (15+ occurrences)
- "Failed to create/read X" (10+ occurrences)

### Data Validation Errors
- "X must be a number" (8+ occurrences)
- "X should be present" (5+ occurrences)
- "X must be non-negative" (3+ occurrences)

### Endpoint Testing Errors
- "X endpoint should return 200" (12+ occurrences)
- "Dashboard endpoint" variations (6+ occurrences)

## Usage Examples

### Query the Catalog
```bash
# Find all error messages from a specific file
jq '.error_messages[] | select(.file == "tests/cli_test_helpers.rs")' \
  hoop_error_messages_complete.json

# Count error messages by pattern type
jq '.error_messages[].pattern_type' hoop_error_messages_complete.json | sort | uniq -c

# Find error messages containing specific text
jq '.error_messages[] | select(.error_message | contains("spawn"))' \
  hoop_error_messages_complete.json

# Get top 10 files with most errors
jq '.error_messages[].file' hoop_error_messages_complete.json | \
  sort | uniq -c | sort -rn | head -10
```

### Load in Python
```python
import json

with open('hoop_error_messages_complete.json') as f:
    errors = json.load(f)

# Filter by pattern type
expect_errors = [e for e in errors if e['pattern_type'] == 'expect']

# Group by file
from collections import defaultdict
by_file = defaultdict(list)
for error in errors:
    by_file[error['file']].append(error)

# Find all error messages containing 'daemon'
daemon_errors = [e for e in errors if 'daemon' in e['error_message'].lower()]
```

## Quality Assessment

### Strengths
- ✅ Comprehensive coverage across all test modules
- ✅ Machine-readable JSON format for programmatic analysis
- ✅ Includes context (line number and code snippet) for each error
- ✅ Pattern classification enables filtering by error type
- ✅ Both summary and complete datasets available

### Limitations
- ⚠️ Some error messages are short ("workspace root", "total_workers")
- ⚠️ 29 messages classified as "unknown" pattern type
- ⚠️ Some test helper files dominate the counts (may skew statistics)
- ⚠️ Does not capture runtime-generated error messages
- ⚠️ Static analysis only - no validation that messages are actually shown to users

## Next Steps for Analysis

1. **Message Quality Review**
   - Identify overly generic error messages
   - Find duplicate messages that could be consolidated
   - Spot inconsistencies in terminology

2. **Coverage Analysis**
   - Map error messages to code paths
   - Identify untested error scenarios
   - Validate error handling completeness

3. **Documentation Integration**
   - Link error messages to user-facing documentation
   - Create error message style guide
   - Build error code reference

4. **Internationalization Preparation**
   - Identify hardcoded user-facing messages
   - Tag messages requiring localization
   - Extract message templates for translation

## Catalog Maintenance

### Update Process
To regenerate the catalog after code changes:

```bash
# Run the extraction script
/tmp/extract_error_messages_fast.sh

# Update the catalog files (manual process or automated)
# Review changes and commit updates
```

### Frequency
- **After major test refactors** - Regenerate full catalog
- **Before releases** - Review new error messages
- **Quarterly** - Quality audit and cleanup

## File Structure

```
docs/test-coverage/
├── hoop_error_messages_catalog.json          # Summary + 100 samples
├── hoop_error_messages_complete.json         # Full 1,192 error dataset
└── error_message_catalog_analysis.md         # This analysis document
```

## Conclusion

This catalog provides a comprehensive view of error handling across the HOOP test suite. The 1,192 error messages reveal consistent patterns in testing infrastructure, with strong emphasis on daemon lifecycle management and API endpoint validation. The dataset is ready for downstream analysis including message quality improvements, coverage validation, and documentation integration.