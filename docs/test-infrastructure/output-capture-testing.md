# Output Capture Test Infrastructure

This document describes the test infrastructure for verifying stdout/stderr capture in HOOP integration tests.

## Overview

The output capture testing infrastructure allows verification that both stdout and stderr streams are properly captured and distinguishable in log files when tests are run via wrapper scripts.

## Components

### 1. Helper Module (`hoop-daemon/tests/output_capture_helpers.rs`)

The helper module provides reusable utilities for output generation and verification:

#### Core Types

```rust
pub enum OutputStream {
    Stdout,
    Stderr,
}
```

Represents the two output streams that can be tested. Each stream:
- Can write messages via `.write(message)`
- Has a log prefix (`.log_prefix()`) used in log files
- Derives `Hash` for use as HashMap keys

#### Output Generation Functions

- `generate_stdout_message(message)` - Write to stdout with flush
- `generate_stderr_message(message)` - Write to stderr with flush
- `generate_output(stream, message)` - Write to specific stream with flush
- `generate_sequence(stream, prefix, count)` - Generate numbered sequence
- `generate_mixed_output(messages)` - Alternate between streams
- `generate_interleaved_output(prefix, count)` - Interleave stdout/stderr
- `generate_high_volume_output(stdout_count, stderr_count)` - Stress test
- `generate_stream_markers()` - Add identifiable markers
- `generate_configured_output(config)` - Structured generation

#### Log File Analysis

```rust
pub struct LogFileParser {
    path: String,
    content: HashMap<OutputStream, Vec<String>>,
}
```

The `LogFileParser` provides:
- `new(path)` - Create parser for log file
- `parse()` - Parse log file and separate stdout/stderr lines
- `get_lines(stream)` - Get all lines from a stream
- `line_count(stream)` - Count lines per stream
- `contains_pattern(stream, pattern)` - Check for pattern presence
- `count_pattern(stream, pattern)` - Count pattern occurrences
- `verify_patterns(stream, patterns)` - Verify multiple patterns

#### Verification

```rust
pub struct VerificationResult {
    pub stdout_passed: bool,
    pub stderr_passed: bool,
    pub missing_stdout: Vec<String>,
    pub missing_stderr: Vec<String>,
    pub extra_stdout: Vec<String>,
    pub extra_stderr: Vec<String>,
}
```

Provides structured verification results with `.passed()` and `.summary()` methods.

#### Utility Functions

- `verify_output_patterns(log_path, expected_stdout, expected_stderr)` - Verify patterns in log
- `find_latest_log(base_dir, pattern)` - Find most recent log file matching pattern

### 2. Test File (`hoop-daemon/tests/stderr_stdout_capture.rs`)

Integration tests using the helper module:

#### Available Tests

- `test_stdout_stderr_output` - Basic output generation
- `test_stream_distinction` - Verify streams are distinguishable
- `test_no_output_loss` - High-volume output test (no data loss)
- `test_interleaved_output_preservation` - Verify ordering preserved
- `test_configured_output_generation` - Test structured generation
- `test_mixed_stream_sequences` - Test various output patterns
- `test_output_flush_behavior` - Verify explicit flushing works

## Usage Pattern

### For Writing Tests

```rust
use output_capture_helpers::*;

#[test]
fn test_my_feature() {
    // Generate test output
    generate_stdout_message("Feature started");
    
    // Do some work
    // ...
    
    generate_stderr_message("Feature warning");
    generate_stdout_message("Feature completed");
}
```

### For Verification

Tests are run via wrapper scripts that capture output to log files. The log format uses prefixes:
- `[STDOUT]` lines are stdout output
- `[STDERR]` lines are stderr output

After running tests, verify captured output:

```rust
let result = verify_output_patterns(
    &Path::new("/path/to/test.log"),
    &["Feature started", "Feature completed"],  // expected stdout
    &["Feature warning"],                      // expected stderr
);

assert!(result.passed(), "{}", result.summary());
```

## Reusability

The helper module is designed to be reusable across all integration tests that need to verify output capture:

1. **Stream-specific output** - All functions accept `OutputStream` parameter
2. **Flexible generation** - Multiple patterns (sequence, interleaved, configured)
3. **Log analysis** - Parser separates streams automatically
4. **Verification helpers** - Pattern matching and detailed results

## Implementation Notes

### Hash Derivation

The `OutputStream` enum derives `Hash` to enable use as `HashMap` keys in the `LogFileParser`.

### Flush Behavior

All output generation functions explicitly flush after writing to ensure immediate capture. This prevents buffered output from being lost if tests fail or crash.

### Log Prefix Convention

The helper uses `[STDOUT]` and `[STDERR]` prefixes. This convention must be matched by any wrapper script that runs the tests.

## Acceptance Criteria Met

✅ Create test helper functions for output generation  
✅ Set up test file structure for output capture tests  
✅ Add helper to read and parse log files  
✅ Add helper to compare generated output vs log content  
✅ Framework must support both stdout and stderr testing  
✅ All helpers must be reusable across subsequent tests

## Future Enhancements

Potential improvements to the infrastructure:

1. **Automated log detection** - Auto-discover log files from test metadata
2. **Timestamp verification** - Verify ordering when timestamps are available
3. **Binary output support** - Extend helpers for binary stdout/stderr
4. **Performance metrics** - Track capture latency and buffer efficiency
5. **Integration with cargo** - Custom test runner for automatic capture
