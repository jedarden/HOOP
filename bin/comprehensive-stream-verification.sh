#!/bin/bash

# comprehensive-stream-verification.sh - Comprehensive test for stdout/stderr capture
#
# This script performs detailed verification that no output is lost from either stream
# by generating known quantities of output and verifying exact counts in logs.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Create logs directory
mkdir -p logs

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Helper functions
pass_test() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    ((PASSED_TESTS++))
    ((TOTAL_TESTS++))
}

fail_test() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    ((FAILED_TESTS++))
    ((TOTAL_TESTS++))
}

warn_test() {
    echo -e "${YELLOW}⚠ WARN${NC}: $1"
    ((TOTAL_TESTS++))
}

echo "========================================"
echo "Comprehensive Stream Verification Test"
echo "========================================"
echo ""

# Test 1: Exact count verification
echo "Test 1: Running exact count verification test..."
TIMESTAMP=$(date -u +"%Y%m%dT%H%M%SZ")
LOG_FILE="logs/comprehensive_test_${TIMESTAMP}.log"

# Create a test script that outputs exact known quantities
TEST_SCRIPT=$(mktemp)
cat > "$TEST_SCRIPT" << 'EOF'
#!/bin/bash

# Test 1: Exact 50 stdout, 50 stderr sequentially
for i in $(seq 0 49); do
    echo "EXACT_STDOUT_${i}"
done

for i in $(seq 0 49); do
    echo "EXACT_STDERR_${i}" >&2
done

# Test 2: 100 interleaved pairs
for i in $(seq 0 99); do
    echo "INTERLEAVED_STDOUT_${i}"
    echo "INTERLEAVED_STDERR_${i}" >&2
done

# Test 3: Rapid burst (stress test)
for i in $(seq 0 199); do
    echo "BURST_STDOUT_${i}"
    echo "BURST_STDERR_${i}" >&2
done

# Test 4: Mixed content in same line
echo "MIXED_LINE_STDOUT_with_data"
echo "MIXED_LINE_STDERR_with_data" >&2

echo "TEST_COMPLETE"
EOF

chmod +x "$TEST_SCRIPT"

echo "Running test script with log capture..."
"$SCRIPT_DIR/run-with-log.sh" "$LOG_FILE" "$TEST_SCRIPT"

# Clean up temp file
rm -f "$TEST_SCRIPT"

if [ ! -f "$LOG_FILE" ]; then
    fail_test "Log file not created"
    exit 1
fi

echo "Analyzing log file: $LOG_FILE"
echo ""

# Test 2: Verify exact counts
echo "Test 2: Verifying exact output counts..."

# Count EXACT_STDOUT_ messages (expected: 50)
EXACT_STDOUT_COUNT=$(grep -c "\[STDOUT\] EXACT_STDOUT_" "$LOG_FILE" || true)
if [ "$EXACT_STDOUT_COUNT" -eq 50 ]; then
    pass_test "Exact stdout count: $EXACT_STDOUT_COUNT/50"
else
    fail_test "Exact stdout count: expected 50, got $EXACT_STDOUT_COUNT"
fi

# Count EXACT_STDERR_ messages (expected: 50)
EXACT_STDERR_COUNT=$(grep -c "\[STDERR\] EXACT_STDERR_" "$LOG_FILE" || true)
if [ "$EXACT_STDERR_COUNT" -eq 50 ]; then
    pass_test "Exact stderr count: $EXACT_STDERR_COUNT/50"
else
    fail_test "Exact stderr count: expected 50, got $EXACT_STDERR_COUNT"
fi

# Count INTERLEAVED_STDOUT_ messages (expected: 100)
INTERLEAVED_STDOUT_COUNT=$(grep -c "\[STDOUT\] INTERLEAVED_STDOUT_" "$LOG_FILE" || true)
if [ "$INTERLEAVED_STDOUT_COUNT" -eq 100 ]; then
    pass_test "Interleaved stdout count: $INTERLEAVED_STDOUT_COUNT/100"
else
    fail_test "Interleaved stdout count: expected 100, got $INTERLEAVED_STDOUT_COUNT"
fi

# Count INTERLEAVED_STDERR_ messages (expected: 100)
INTERLEAVED_STDERR_COUNT=$(grep -c "\[STDERR\] INTERLEAVED_STDERR_" "$LOG_FILE" || true)
if [ "$INTERLEAVED_STDERR_COUNT" -eq 100 ]; then
    pass_test "Interleaved stderr count: $INTERLEAVED_STDERR_COUNT/100"
else
    fail_test "Interleaved stderr count: expected 100, got $INTERLEAVED_STDERR_COUNT"
fi

# Count BURST_STDOUT_ messages (expected: 200)
BURST_STDOUT_COUNT=$(grep -c "\[STDOUT\] BURST_STDOUT_" "$LOG_FILE" || true)
if [ "$BURST_STDOUT_COUNT" -eq 200 ]; then
    pass_test "Burst stdout count: $BURST_STDOUT_COUNT/200"
else
    fail_test "Burst stdout count: expected 200, got $BURST_STDOUT_COUNT"
fi

# Count BURST_STDERR_ messages (expected: 200)
BURST_STDERR_COUNT=$(grep -c "\[STDERR\] BURST_STDERR_" "$LOG_FILE" || true)
if [ "$BURST_STDERR_COUNT" -eq 200 ]; then
    pass_test "Burst stderr count: $BURST_STDERR_COUNT/200"
else
    fail_test "Burst stderr count: expected 200, got $BURST_STDERR_COUNT"
fi

echo ""

# Test 3: Verify no missing sequential messages
echo "Test 3: Verifying no missing sequential messages..."

# Check for gaps in exact stdout (0-49)
MISSING_EXACT_STDOUT=0
for i in $(seq 0 49); do
    if ! grep -q "\[STDOUT\] EXACT_STDOUT_${i}" "$LOG_FILE"; then
        echo "  Missing EXACT_STDOUT_${i}"
        ((MISSING_EXACT_STDOUT++))
    fi
done

if [ "$MISSING_EXACT_STDOUT" -eq 0 ]; then
    pass_test "No missing exact stdout messages (0-49)"
else
    fail_test "Missing $MISSING_EXACT_STDOUT exact stdout messages"
fi

# Check for gaps in exact stderr (0-49)
MISSING_EXACT_STDERR=0
for i in $(seq 0 49); do
    if ! grep -q "\[STDERR\] EXACT_STDERR_${i}" "$LOG_FILE"; then
        echo "  Missing EXACT_STDERR_${i}"
        ((MISSING_EXACT_STDERR++))
    fi
done

if [ "$MISSING_EXACT_STDERR" -eq 0 ]; then
    pass_test "No missing exact stderr messages (0-49)"
else
    fail_test "Missing $MISSING_EXACT_STDERR exact stderr messages"
fi

# Sample check for interleaved (check every 10th to save time)
MISSING_INTERLEAVED_STDOUT=0
for i in $(seq 0 10 99); do
    if ! grep -q "\[STDOUT\] INTERLEAVED_STDOUT_${i}" "$LOG_FILE"; then
        ((MISSING_INTERLEAVED_STDOUT++))
    fi
done

if [ "$MISSING_INTERLEAVED_STDOUT" -eq 0 ]; then
    pass_test "Sample check: No missing interleaved stdout messages"
else
    fail_test "Missing $MISSING_INTERLEAVED_STDOUT interleaved stdout messages (sample)"
fi

# Sample check for interleaved stderr (check every 10th)
MISSING_INTERLEAVED_STDERR=0
for i in $(seq 0 10 99); do
    if ! grep -q "\[STDERR\] INTERLEAVED_STDERR_${i}" "$LOG_FILE"; then
        ((MISSING_INTERLEAVED_STDERR++))
    fi
done

if [ "$MISSING_INTERLEAVED_STDERR" -eq 0 ]; then
    pass_test "Sample check: No missing interleaved stderr messages"
else
    fail_test "Missing $MISSING_INTERLEAVED_STDERR interleaved stderr messages (sample)"
fi

echo ""

# Test 4: Verify stream prefixes are correct
echo "Test 4: Verifying stream prefixes..."

# Check that no stdout message appears without [STDOUT] prefix
MISPREFIXED_STDOUT=$(grep -E "EXACT_STDOUT_[0-9]|INTERLEAVED_STDOUT_[0-9]|BURST_STDOUT_[0-9]" "$LOG_FILE" | grep -v "^\[STDOUT\]" | wc -l)
if [ "$MISPREFIXED_STDOUT" -eq 0 ]; then
    pass_test "All stdout messages have [STDOUT] prefix"
else
    fail_test "Found $MISPREFIXED_STDOUT stdout messages without [STDOUT] prefix"
fi

# Check that no stderr message appears without [STDERR] prefix
MISPREFIXED_STDERR=$(grep -E "EXACT_STDERR_[0-9]|INTERLEAVED_STDERR_[0-9]|BURST_STDERR_[0-9]" "$LOG_FILE" | grep -v "^\[STDERR\]" | wc -l)
if [ "$MISPREFIXED_STDERR" -eq 0 ]; then
    pass_test "All stderr messages have [STDERR] prefix"
else
    fail_test "Found $MISPREFIXED_STDERR stderr messages without [STDERR] prefix"
fi

echo ""

# Test 5: Verify no content mixing within lines
echo "Test 5: Verifying no content mixing within lines..."

# Check for lines that contain both patterns (would indicate buffer corruption)
MIXED_LINES=$(grep -E "STDOUT.*STDERR|STDERR.*STDOUT" "$LOG_FILE" | grep -v "^\\[\\(STDOUT\\|STDERR\\)\\] " | wc -l)
if [ "$MIXED_LINES" -eq 0 ]; then
    pass_test "No content mixing detected within lines"
else
    fail_test "Found $MIXED_LINES lines with mixed content (possible buffer corruption)"
fi

echo ""

# Test 6: Summary statistics
echo "Test 6: Summary statistics..."

TOTAL_STDOUT_LINES=$(grep -c "^\[STDOUT\] " "$LOG_FILE" || true)
TOTAL_STDERR_LINES=$(grep -c "^\[STDERR\] " "$LOG_FILE" || true)
TOTAL_LINES=$(wc -l < "$LOG_FILE")

echo "  Total lines in log: $TOTAL_LINES"
echo "  Stdout lines: $TOTAL_STDOUT_LINES"
echo "  Stderr lines: $TOTAL_STDERR_LINES"
echo "  Expected stdout: 350 (50 + 100 + 200)"
echo "  Expected stderr: 350 (50 + 100 + 200)"

EXPECTED_STDOUT=350
EXPECTED_STDERR=350

if [ "$TOTAL_STDOUT_LINES" -eq "$EXPECTED_STDOUT" ]; then
    pass_test "Total stdout lines match expected count"
else
    warn_test "Total stdout lines: expected $EXPECTED_STDOUT, got $TOTAL_STDOUT_LINES"
fi

if [ "$TOTAL_STDERR_LINES" -eq "$EXPECTED_STDERR" ]; then
    pass_test "Total stderr lines match expected count"
else
    warn_test "Total stderr lines: expected $EXPECTED_STDERR, got $TOTAL_STDERR_LINES"
fi

echo ""

# Test 7: Verify test completion marker
echo "Test 7: Verifying test completion..."

if grep -q "TEST_COMPLETE" "$LOG_FILE"; then
    pass_test "Test completion marker found"
else
    fail_test "Test completion marker not found (test may have failed)"
fi

echo ""

# Final summary
echo "========================================"
echo "Final Summary"
echo "========================================"
echo "Total tests: $TOTAL_TESTS"
echo "Passed: $PASSED_TESTS"
echo "Failed: $FAILED_TESTS"
echo ""

if [ "$FAILED_TESTS" -eq 0 ]; then
    echo -e "${GREEN}✓ ALL TESTS PASSED${NC}"
    echo "No output loss detected from either stream"
    echo ""
    echo "Log file saved at: $LOG_FILE"
    exit 0
else
    echo -e "${RED}✗ SOME TESTS FAILED${NC}"
    echo "Output loss or corruption detected"
    echo ""
    echo "Log file available at: $LOG_FILE"
    echo "Review the log file for details"
    exit 1
fi
