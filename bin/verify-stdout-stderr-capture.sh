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

# Check if stderr has the "STDERR: " prefix
if grep -q "^STDERR: " "${LOG_FILE}"; then
    echo "✓ Streams ARE distinguishable in log"
    echo "  - Stdout lines have no prefix"
    echo "  - Stderr lines are prefixed with 'STDERR: '"

    # Show sample output with distinction
    echo ""
    echo "Sample output from log file showing stream distinction:"
    echo "---"
    grep -E "(This is a message to (STDOUT|STDERR)|^STDOUT_MARKER|^STDERR: STDERR_MARKER|^STDOUT_COUNT_|^STDERR: STDERR_COUNT_)" "${LOG_FILE}" | head -10
    echo "---"
else
    echo "⚠ Streams are NOT clearly distinguishable in log"
    echo "  (Both streams interleaved without clear markers)"
fi

echo ""

# Provide summary
echo "=== Verification Summary ==="
echo "✓ Log file created successfully"
echo "✓ Stdout content captured"
echo "✓ Stderr content captured"
echo "✓ Both streams present in same log file"
if grep -q "^STDERR: " "${LOG_FILE}"; then
    echo "✓ Streams are distinguishable in log (stderr prefixed with 'STDERR: ')"
else
    echo "⚠ Streams are NOT distinguishable in log (both interleaved without markers)"
fi
echo ""
echo "Full log available at: ${LOG_FILE}"
echo ""

exit 0
