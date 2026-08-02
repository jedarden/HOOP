# Bead bf-3hk6e: Stream Distinction Analysis

## Task
Verify that stdout and stderr streams are clearly distinguishable in the log format.

## Test Methodology
1. Examined existing `hoop-daemon/tests/stderr_stdout_capture.rs` test file
2. Created simple test script `test_streams.sh` that outputs to both streams
3. Ran script through `bin/run-with-log.sh` logging mechanism
4. Analyzed resulting log file for stream distinction

## Test Execution

### Test 1: Using existing test suite
```bash
./bin/verify-stdout-stderr-capture.sh
```

### Test 2: Direct log examination
Examined actual log file: `logs/stderr_stdout_capture_test_20260802T195413Z.log`

## Results Analysis

### Actual Log File Contents (excerpt from `logs/stderr_stdout_capture_test_20260802T195413Z.log`)
```
This is a message to STDOUT from test_stdout_stderr_output
STDOUT: Mixed output message 1
STDOUT: Mixed output message 2
This is a message to STDERR from test_stdout_stderr_output
STDERR: Mixed output message 1
STDERR: Mixed output message 2
STDERR_COUNT_000
STDERR_COUNT_001
...
STDERR_COUNT_010
STDERR_COUNT_0STDOUT_COUNT_000  ← STREAMS INTERLEAVED WITH NO MARKER
STDOUT_COUNT_001
```

### Critical Finding: NOT DISTINGUISHABLE
**The log format does NOT distinguish between stdout and stderr streams.**

#### Evidence from Real Test:
1. **No stream markers**: Lines have no prefix like `[STDOUT]` or `[STDERR]`
2. **Visible interleaving**: Line 147 shows `STDERR_COUNT_0STDOUT_COUNT_000` - streams are concatenated without separation
3. **Content-based guessing only**: You can only tell streams apart by recognizing test patterns (e.g., "STDERR_COUNT_" vs "STDOUT_COUNT_")
4. **No structural distinction**: Both streams write to the same log file in the order received

### Critical Finding: NOT DISTINGUISHABLE
**The log format does NOT distinguish between stdout and stderr streams.**

#### Evidence:
1. **No stream markers**: Lines have no prefix like `[STDOUT]` or `[STDERR]`
2. **No color coding**: No ANSI codes or other visual distinction
3. **No separate files**: Both streams write to the same log file
4. **Ambiguous interleaving**: Lines 6-8 (STDERR) appear mixed with stdout lines without any indication

#### Root Cause
Looking at `bin/run-with-log.sh` line 105:
```bash
"$@" > >(tee -a "$LOG_FILE" > "$CAPTURED_STDOUT") 2> >(tee -a "$LOG_FILE" > "$CAPTURED_STDERR")
```

Both process substitutions use `tee -a` to append to the same log file, with NO stream markers added.

#### What DOES work:
- Script captures streams separately in memory (`$CAPTURED_STDOUT`, `$CAPTURED_STDERR`)
- Exit summary shows stream sizes separately
- Environment variables export captured content

#### What DOES NOT work:
- Log file itself has no stream distinction
- Reader cannot determine stream origin for any given line
- No timestamps, no stream prefixes, no structural distinction

## Acceptance Criteria Status

✅ **Run test that outputs to both streams** - COMPLETED  
   - Ran `./bin/verify-stdout-stderr-capture.sh` which executes `hoop-daemon/tests/stderr_stdout_capture.rs`
   - Test outputs 100+ lines to each stream
   
❌ **Verify stdout and stderr are marked differently in logs** - FAILED (not marked)  
   - No automatic stream prefixes in log format
   - No timestamps, no ANSI codes, no structural markers
   
❌ **Confirm you can identify which stream each log line came from** - CANNOT CONFIRM  
   - Can only identify streams by recognizing test content patterns
   - Real-world output would be completely ambiguous
   
❌ **Check that there is no ambiguity between the two streams** - AMBIGUITY EXISTS  
   - Example: Line 147 shows `STDERR_COUNT_0STDOUT_COUNT_000` - two stream outputs concatenated without separation
   - Without test-specific patterns, impossible to determine stream origin

## Additional Evidence from Verification Script

The `verify-stdout-stderr-capture.sh` script itself is misleading. It reports:
```
✓ Streams ARE distinguishable in log
  - Stdout lines have no prefix
  - Stderr lines are prefixed with 'STDERR: '
```

But this is **incorrect** - the script is checking for content patterns that were manually written by the test code, NOT actual stream markers added by the logging system. The test code uses patterns like:
- `println!("STDOUT: Mixed output message 1");`
- `eprintln!("STDERR: Mixed output message 1");`

These are part of the message content, not logging system prefixes.

## Recommendation

The current logging mechanism does NOT meet the requirement of distinguishable streams in log format. To fix this, `run-with-log.sh` would need to:

1. **Add stream prefixes** when writing to log: Prefix stdout lines with `[STDOUT]` and stderr lines with `[STDERR]`
2. **OR use separate log files**: `log.stdout.log` and `log.stderr.log`
3. **OR add structured metadata**: Timestamps and stream indicators per line
4. **OR use structured logging**: JSON format with explicit `stream` field

Example fix for `run-with-log.sh`:
```bash
# Current (line 105):
"$@" > >(tee -a "$LOG_FILE" > "$CAPTURED_STDOUT") 2> >(tee -a "$LOG_FILE" > "$CAPTURED_STDERR")

# Fixed (with prefixes):
"$@" > >(sed 's/^/[STDOUT] /' | tee -a "$LOG_FILE" > "$CAPTURED_STDOUT") \
      2> >(sed 's/^/[STDERR] /' | tee -a "$LOG_FILE" > "$CAPTURED_STDERR")
```

## Conclusion

**❌ Stream distinction is NOT implemented in the current log format.**

**Evidence:**
- Real log file shows streams interleaved without markers
- Only test content patterns allow distinguishing streams
- No systematic stream identification mechanism
- Verification script gives false positive due to content-based detection

This is a **gap** that needs to be addressed if distinguishable streams are a requirement. The logging infrastructure captures both streams correctly but fails to mark them in the output file.
