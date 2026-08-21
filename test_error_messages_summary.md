# HOOP Test Error Messages Catalog

**Generated:** 2026-08-21  
**Total Error Messages:** 3,466  
**Test Directories Searched:** `hoop-daemon/tests/`, `hoop-cli/tests/`, `tests/`

## Summary Statistics

### Error Message Categories
| Category | Count | Percentage |
|----------|-------|------------|
| expect call with detailed message | 2,014 | 58.1% |
| expect with failure | 1,024 | 29.5% |
| assert_eq detailed message | 392 | 11.3% |
| bail error message | 17 | 0.5% |
| expect with invalid | 10 | 0.3% |
| anyhow error message | 6 | 0.2% |
| expect with error | 2 | 0.1% |
| assert_ne detailed message | 1 | 0.0% |

### Files with Most Error Messages
| File | Count |
|------|-------|
| `hoop-daemon/tests/integration_harness.rs` | 219 |
| `hoop-daemon/tests/adapter_failover_test.rs` | 160 |
| `hoop-daemon/tests/acceptance/s3_bead_creation_from_chat.rs` | 107 |
| `hoop-daemon/tests/testrepo_integration.rs` | 106 |
| `hoop-daemon/tests/s3_bead_creation_from_chat.rs` | 104 |
| `hoop-daemon/tests/testrepo_harness_integration.rs` | 95 |
| `hoop-daemon/tests/draft_queue_invariants.rs` | 93 |
| `hoop-daemon/tests/acceptance/s6_machine_mode.rs` | 90 |
| `hoop-daemon/tests/stitch_percentile_index_integration.rs` | 90 |
| `hoop-daemon/tests/state_projections.rs` | 85 |

### Most Common Error Keywords
| Keyword | Count |
|---------|-------|
| failed | 2,021 |
| should | 686 |
| spawn | 385 |
| daemon | 372 |
| parse | 368 |
| create | 299 |
| fetch | 216 |
| write | 152 |
| succeed | 119 |
| beads | 106 |

## Error Message Patterns

The following patterns were used to extract error messages:

1. **`unwrap_err().expect("message")`** - Error expectations from `Result::unwrap_err()`
2. **`expect_err!("message")`** - Expected error assertions
3. **`bail!("message")`** - Early return with error using anyhow
4. **`ensure!("message")`** - Condition-based error returns
5. **`anyhow!("message")`** - Generic error creation
6. **`.expect("Error...")`** - Expect calls with error descriptions
7. **`.expect("Fail...")`** - Expect calls with failure descriptions
8. **`.expect("Invalid...")`** - Expect calls with invalid descriptions
9. **`.expect("detailed message...")`** - Detailed expect calls (10+ chars)
10. **`assert_eq(..., "detailed message")`** - Equality assertions with messages
11. **`assert_ne(..., "detailed message")`** - Inequality assertions with messages
12. **`Error::EnumVariant`** - Error enum variants

## Common Error Message Types

### Daemon Lifecycle Errors
- "Daemon did not become ready"
- "Daemon failed to become ready within 10 seconds"
- "Daemon failed to start"

### File System Errors
- "testrepo/.beads/events.jsonl should exist"
- "testrepo/.beads/heartbeats.jsonl should exist"
- "events.jsonl should not be empty"
- "heartbeats.jsonl should not be empty"

### Configuration Errors
- "config.yml should be created"

### Test Fixture Errors
- "Events fixture should contain at least one claim event"
- "Heartbeats fixture should contain at least one idle state"
- "Events fixture should contain at least one fail event"

## Data Format

The complete error messages catalog is available in `test_error_messages.json` with the following structure:

```json
[
  {
    "file": "path/to/test/file.rs",
    "line": 123,
    "category": "error_category",
    "line_content": "actual line content from source"
  }
]
```

## Usage Examples

### Load and analyze with Python:
```python
import json
with open('test_error_messages.json') as f:
    errors = json.load(f)
    
# Filter by category
daemon_errors = [e for e in errors if 'daemon' in e['line_content'].lower()]

# Group by file
by_file = {}
for error in errors:
    by_file.setdefault(error['file'], []).append(error)
```

### Search for specific patterns:
```bash
# Find all "daemon" related errors
jq '.[] | select(.line_content | test("daemon"; "i"))' test_error_messages.json

# Get statistics by file
jq 'group_by(.file) | map({file: .[0].file, count: length}) | sort_by(-count)' test_error_messages.json
```

## Next Steps

This catalog can be used for:
1. **Error message consistency analysis** - Identify inconsistent error phrasing
2. **Documentation generation** - Auto-generate error handling documentation
3. **Test coverage analysis** - Ensure all error paths have tests
4. **Internationalization preparation** - Identify all user-facing error messages
5. **Error taxonomy development** - Build structured error classification system