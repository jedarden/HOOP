#!/bin/bash

# verify-stdout-stderr-capture.sh - Verify stdout and stderr are captured correctly
#
# This script runs a test that outputs to both stdout and stderr, then verifies
# that both streams appear in the log file and checks if they are distinguishable.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Create logs directory
mkdir -p logs

# Generate timestamp for log file
TIMESTAMP=$(date -u +"%Y%m%dT%H%M%SZ")
LOG_FILE="logs/stderr_stdout_capture_test_${TIMESTAMP}.log"

echo "=== Running stdout/stderr capture test ==="
echo "Log file: ${LOG_FILE}"
echo ""

# Run the test with log capture
echo "Step 1: Running test with run-with-log.sh..."
"${SCRIPT_DIR}/run-with-log.sh" "${LOG_FILE}" cargo test -p hoop-daemon --test stderr_stdout_capture -- --nocapture

EXIT_CODE=${HOOP_CAPTURED_EXIT_CODE:-$?}
echo ""
echo "Test exit code: ${EXIT_CODE}"
echo ""

# Verify log file was created
echo "Step 2: Verifying log file was created..."
if [ ! -f "${LOG_FILE}" ]; then
    echo "ERROR: Log file not created at ${LOG_FILE}"
    exit 1
fi
echo "✓ Log file created: ${LOG_FILE}"
echo ""

# Verify stdout content
echo "Step 3: Verifying stdout content..."
if grep -q "This is a message to STDOUT" "${LOG_FILE}"; then
    echo "✓ Found stdout message in log"
else
    echo "ERROR: Stdout message not found in log"
    exit 1
fi
echo ""

# Verify stderr content
echo "Step 4: Verifying stderr content..."
if grep -q "This is a message to STDERR" "${LOG_FILE}"; then
    echo "✓ Found stderr message in log"
else
    echo "ERROR: Stderr message not found in log"
    exit 1
fi
echo ""

# Verify specific markers
echo "Step 5: Verifying stream-specific markers..."
if grep -q "STDOUT_MARKER" "${LOG_FILE}"; then
    echo "✓ Found STDOUT_MARKER in log"
else
    echo "ERROR: STDOUT_MARKER not found in log"
    exit 1
fi

if grep -q "STDERR_MARKER" "${LOG_FILE}"; then
    echo "✓ Found STDERR_MARKER in log"
else
    echo "ERROR: STDERR_MARKER not found in log"
    exit 1
fi
echo ""

# Verify high-volume output
echo "Step 6: Verifying high-volume output was captured..."
STDOUT_COUNT=$(grep -c "STDOUT_COUNT_" "${LOG_FILE}" || true)
STDERR_COUNT=$(grep -c "STDERR_COUNT_" "${LOG_FILE}" || true)

echo "Found ${STDOUT_COUNT} stdout count markers"
echo "Found ${STDERR_COUNT} stderr count markers"

if [ "${STDOUT_COUNT}" -ge 100 ] && [ "${STDERR_COUNT}" -ge 100 ]; then
    echo "✓ High-volume output captured (both streams have 100+ lines)"
else
    echo "ERROR: Some output appears to be missing"
    echo "Expected at least 100 lines per stream, got ${STDOUT_COUNT} stdout and ${STDERR_COUNT} stderr"
    exit 1
fi
echo ""

# Check if streams are distinguishable
echo "Step 7: Checking if streams are distinguishable in log..."

# Check for stream prefixes
STDOUT_PREFIX_COUNT=$(grep -c "^\\[STDOUT\\] " "${LOG_FILE}" || true)
STDERR_PREFIX_COUNT=$(grep -c "^\\[STDERR\\] " "${LOG_FILE}" || true)

echo "Found ${STDOUT_PREFIX_COUNT} lines with [STDOUT] prefix"
echo "Found ${STDERR_PREFIX_COUNT} lines with [STDERR] prefix"

# Check for interleaved lines (indicative of the bug)
INTERLEAVED_COUNT=$(grep -c "STDOUT.*STDERR\\|STDERR.*STDOUT" "${LOG_FILE}" || true)

if [ "${INTERLEAVED_COUNT}" -gt 0 ]; then
    echo "⚠ WARNING: Found ${INTERLEAVED_COUNT} lines with interleaved stdout/stderr content"
    echo "  This indicates streams are not properly separated"
else
    echo "✓ No interleaved content detected"
fi

# Verify distinguishability using the test-specific markers
echo ""
echo "Checking test-specific messages with stream prefixes:"
if grep -q "\\[STDOUT\\].*This is a message to STDOUT" "${LOG_FILE}"; then
    echo "✓ Found stdout message with proper prefix"
else
    echo "⚠ Stdout message not found with [STDOUT] prefix"
fi

if grep -q "\\[STDERR\\].*This is a message to STDERR" "${LOG_FILE}"; then
    echo "✓ Found stderr message with proper prefix"
else
    echo "⚠ Stderr message not found with [STDERR] prefix"
fi

# Final verdict
if [ "${STDOUT_PREFIX_COUNT}" -gt 0 ] && [ "${STDERR_PREFIX_COUNT}" -gt 0 ] && [ "${INTERLEAVED_COUNT}" -eq 0 ]; then
    echo ""
    echo "✓ Streams ARE clearly distinguishable in log"
    echo "  - Stdout lines are prefixed with '[STDOUT] '"
    echo "  - Stderr lines are prefixed with '[STDERR] '"
    echo "  - No interleaved content detected"

    # Show sample output with distinction
    echo ""
    echo "Sample output from log file showing stream distinction:"
    echo "---"
    grep -E "\\[(STDOUT|STDERR)\\].*(This is a message to (STDOUT|STDERR)|MARKER|SEQ_0)" "${LOG_FILE}" | head -10
    echo "---"
else
    echo ""
    echo "⚠ Streams are NOT clearly distinguishable in log"
    if [ "${INTERLEAVED_COUNT}" -gt 0 ]; then
        echo "  - Found ${INTERLEAVED_COUNT} interleaved lines"
    fi
    if [ "${STDOUT_PREFIX_COUNT}" -eq 0 ]; then
        echo "  - No stdout prefix markers found"
    fi
    if [ "${STDERR_PREFIX_COUNT}" -eq 0 ]; then
        echo "  - No stderr prefix markers found"
    fi
fi

echo ""

# Provide summary
echo "=== Verification Summary ==="
echo "✓ Log file created successfully"
echo "✓ Stdout content captured"
echo "✓ Stderr content captured"
echo "✓ Both streams present in same log file"

# Check distinguishability with new prefixes
STDOUT_PREFIX_COUNT=$(grep -c "^\\[STDOUT\\] " "${LOG_FILE}" || true)
STDERR_PREFIX_COUNT=$(grep -c "^\\[STDERR\\] " "${LOG_FILE}" || true)
INTERLEAVED_COUNT=$(grep -c "STDOUT.*STDERR\\|STDERR.*STDOUT" "${LOG_FILE}" || true)

if [ "${STDOUT_PREFIX_COUNT}" -gt 0 ] && [ "${STDERR_PREFIX_COUNT}" -gt 0 ] && [ "${INTERLEAVED_COUNT}" -eq 0 ]; then
    echo "✓ Streams are clearly distinguishable in log"
    echo "  - Stdout lines prefixed with '[STDOUT]' (${STDOUT_PREFIX_COUNT} lines)"
    echo "  - Stderr lines prefixed with '[STDERR]' (${STDERR_PREFIX_COUNT} lines)"
    echo "  - No interleaved content detected"
else
    echo "⚠ Streams distinguishability check failed:"
    [ "${STDOUT_PREFIX_COUNT}" -eq 0 ] && echo "  - No stdout prefix markers found"
    [ "${STDERR_PREFIX_COUNT}" -eq 0 ] && echo "  - No stderr prefix markers found"
    [ "${INTERLEAVED_COUNT}" -gt 0 ] && echo "  - Found ${INTERLEAVED_COUNT} interleaved lines"
fi
echo ""
echo "Full log available at: ${LOG_FILE}"
echo ""

exit 0
