# Test Log File Naming Convention

This document describes the standardized naming convention for test output log files in the HOOP project.

## Overview

Test log files use a descriptive, timestamp-based naming convention that makes it easy to:
- Identify which test or test suite produced the log
- Distinguish between multiple test runs
- Find logs chronologically
- Maintain safe filesystem names

## Naming Format

```
<test_name>_<timestamp>.log
```

### Components

1. **test_name** - Derived from the test command
   - For `cargo test --test beads_deletion_http`: `beads_deletion_http`
   - For `cargo test --lib`: `lib_test`
   - For `cargo test <specific_test>`: `<specific_test>`
   - For load tests: the test function name (e.g., `test_medium_scale_load_test`)

2. **timestamp** - ISO 8601 format (UTC)
   - Format: `YYYYMMDDTHHMMSSZ`
   - Example: `20260802T091933Z` (August 2, 2026, 09:19:33 UTC)

3. **.log** - Standard file extension

### Examples

```
beads_deletion_http_20260802T091933Z.log
lib_test_20260802T091933Z.log
test_medium_scale_load_test_20260802T091933Z.log
reflection_detector_integration_20260802T091933Z.log
```

## Special Character Handling

The naming function automatically sanitizes test names to ensure filesystem safety:

| Character | Replacement | Example |
|-----------|-------------|---------|
| Space | `_` | `test name` → `test_name` |
| `/` | `_` | `test/name` → `test_name` |
| `:` | `_` | `test:name` → `test_name` |
| `*` | `_` | `test*name` → `test_name` |
| `?` | (removed) | `test?name` → `testname` |
| `"` | (removed) | `test"name` → `testname` |
| `<` | `_` | `test<name` → `test_name` |
| `>` | `_` | `test>name` → `test_name` |
| `|` | (removed) | `test\|name` → `testname` |

Additional safeguards:
- Consecutive underscores are collapsed to single underscores
- Leading/trailing dots, hyphens, or underscores are removed
- Names longer than 200 characters are truncated (to stay under filesystem limits)

## Usage

### Auto-Generate Log Names (Recommended)

Use the `--auto` flag with `run-with-log.sh`:

```bash
# Auto-generates: beads_deletion_http_20260802T091933Z.log
./bin/run-with-log.sh --auto cargo test --test beads_deletion_http

# Auto-generates: lib_test_20260802T091933Z.log
./bin/run-with-log.sh --auto cargo test --lib
```

The script will:
1. Analyze the test command
2. Extract the test name
3. Generate a UTC timestamp
4. Create the log file in `logs/` directory if it exists, otherwise current directory
5. Print the log file path to stderr

### Manual Log Names

Specify a custom log file path if needed:

```bash
./bin/run-with-log.sh /tmp/my-custom-test.log cargo test --lib
```

### Using generate-test-log-name.sh Directly

Generate log names without running tests:

```bash
# Get just the log file name
./bin/generate-test-log-name.sh cargo test --test beads_deletion_http
# Output: beads_deletion_http_20260802T091933Z.log
```

## Integration with Makefile

The Makefile test targets can be updated to use auto-naming:

```makefile
test:
	@echo "=== Running tests with auto-generated log ==="
	@./bin/run-with-log.sh --auto cargo test --lib --features testing --verbose
	@echo "=== Verifying no processes remain after tests ==="
	@./bin/verify-hoop-test-processes.sh || echo "Warning: Some processes may remain"
```

## Log Storage

- **Preferred location:** `logs/` directory (auto-created if exists)
- **Fallback:** Repository root when `logs/` doesn't exist
- **Git status:** Test output logs are ignored by `.gitignore`

## Implementation

The naming convention is implemented in two scripts:

1. **`bin/generate-test-log-name.sh`**
   - Extracts test names from cargo test commands
   - Sanitizes names for filesystem safety
   - Generates UTC timestamps
   - Outputs the final log file name

2. **`bin/run-with-log.sh`**
   - Wrapper script for running commands with log output
   - Supports `--auto` flag for automatic log naming
   - Preserves exit codes
   - Can be used with any command, not just cargo test

## Testing the Naming Function

Verify the naming function works correctly:

```bash
# Test various patterns
./bin/generate-test-log-name.sh cargo test --lib
./bin/generate-test-log-name.sh cargo test --test beads_deletion_http
./bin/generate-test-log-name.sh cargo test --test load_test test_medium_scale_load_test

# Test special character handling
./bin/generate-test-log-name.sh cargo test --test 'test:with/special*chars?'
# Output: test_with_special_chars_20260802T091937Z.log

# Test long names (truncation)
./bin/generate-test-log-name.sh cargo test --test 'verylongnamethatexceedsfilesystemlimits...'
# Output: verylongnamethatexceedsfilesystemlimits_20260802T091938Z.log
```

## Rationale

### Why This Convention?

1. **Descriptive:** The test name immediately identifies what test produced the log
2. **Sortable:** ISO 8601 timestamps sort chronologically in filesystem listings
3. **Unique:** Timestamps prevent name collisions for the same test run multiple times
4. **Safe:** Sanitization prevents filesystem errors on different platforms
5. **Parseable:** Consistent format allows programmatic extraction of test name and timestamp

### Alternatives Considered

1. **Sequential numbering** (`test_001.log`, `test_002.log`)
   - Rejected: Doesn't identify test type, requires state management

2. **Git commit-based naming** (`test_<commit>.log`)
   - Rejected: Doesn't work for uncommitted changes, less readable

3. **Random IDs** (`test_<uuid>.log`)
   - Rejected: Not sortable, not descriptive, harder to correlate

## Future Enhancements

Potential improvements to consider:

1. **Log rotation:** Automatic cleanup of old logs beyond a certain age
2. **Structured metadata:** JSON sidecar files with test duration, exit code, etc.
3. **Compression:** Automatic gzip compression for large logs
4. **Indexing:** SQLite index of all logs for fast searching

## References

- Implementation: `bin/generate-test-log-name.sh`
- Wrapper script: `bin/run-with-log.sh`
- Makefile integration: `Makefile` test targets
- Git ignore patterns: `.gitignore` (lines 63-70)
