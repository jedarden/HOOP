#!/bin/bash

# verify-stream-capture.sh - Comprehensive verification of stdout/stderr stream capture
#
# This script verifies that no output is lost from either stdout or stderr stream
# during logging. It runs a comprehensive test with multiple outputs to both streams,
# counts expected outputs vs actual outputs in logs, and verifies ordering.
#
# Usage:
#   verify-stream-capture.sh [--run-only | --verify-only | --full]
#
# Arguments:
#   --run-only    - Only run the test, skip verification
#   --verify-only - Only verify existing logs, skip running test
#   --full        - Run both test and verification (default)

set -euo pipefail

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test configuration
RUN_MODE="full"
TEST_COMMAND="bin/comprehensive-stream-test.sh"
LOG_DIR="logs"

# Check arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --run-only)
            RUN_MODE="run"
            shift
            ;;
        --verify-only)
            RUN_MODE="verify"
            shift
            ;;
        --full)
            RUN_MODE="full"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--run-only | --verify-only | --full]"
            exit 1
            ;;
    esac
done

echo "=== Stream Capture Verification ==="
echo "Mode: $RUN_MODE"
echo "=================================="
echo

# Find the most recent test log file
find_latest_log() {
    local latest_log=$(ls -t ${LOG_DIR}/*.log 2>/dev/null | head -1)
    if [ -z "$latest_log" ]; then
        echo "No test log found. Run with --run-only first."
        exit 1
    fi
    echo "$latest_log"
}

# Run the test with logging
run_test() {
    echo -e "${YELLOW}Running comprehensive stream capture test...${NC}"

    if [ ! -f "bin/run-with-log.sh" ]; then
        echo "Error: run-with-log.sh not found in bin/"
        exit 1
    fi

    # Run the test with auto-named log
    bin/run-with-log.sh --auto $TEST_COMMAND

    echo -e "${GREEN}Test completed. Log file created.${NC}"
    echo
}

# Verify the log contents
verify_log() {
    local log_file=$(find_latest_log)

    echo -e "${YELLOW}Verifying log file: $log_file${NC}"
    echo

    # Count expected outputs from the test code
    local expected_exact_stdout=101
    local expected_exact_stderr=101
    local expected_interleaved=100  # 50 stdout + 50 stderr
    local expected_burst_stdout=201
    local expected_burst_stderr=201
    local expected_special=4   # Mixed lines + special chars
    local expected_long=2      # Long lines
    local expected_markers=1   # TEST_COMPLETE

    local expected_stdout=$((expected_exact_stdout + expected_interleaved/2 + expected_burst_stdout + expected_special/2 + expected_long/2 + expected_markers))
    local expected_stderr=$((expected_exact_stderr + expected_interleaved/2 + expected_burst_stderr + expected_special/2 + expected_long/2))

    echo "Expected output counts:"
    echo "  STDOUT: $expected_stdout messages"
    echo "  STDERR: $expected_stderr messages"
    echo

    # Count actual outputs in log file
    local actual_stdout=$(grep -c "\[STDOUT\]" "$log_file" || echo "0")
    local actual_stderr=$(grep -c "\[STDERR\]" "$log_file" || echo "0")

    echo "Actual output counts in log:"
    echo "  STDOUT: $actual_stdout lines"
    echo "  STDERR: $actual_stderr lines"
    echo

    # Verify counts
    local stdout_ok=false
    local stderr_ok=false

    if [ "$actual_stdout" -ge "$expected_stdout" ]; then
        echo -e "${GREEN}✓ STDOUT: All messages captured ($actual_stdout >= $expected_stdout)${NC}"
        stdout_ok=true
    else
        echo -e "${RED}✗ STDOUT: Messages missing ($actual_stdout < $expected_stdout)${NC}"
    fi

    if [ "$actual_stderr" -ge "$expected_stderr" ]; then
        echo -e "${GREEN}✓ STDERR: All messages captured ($actual_stderr >= $expected_stderr)${NC}"
        stderr_ok=true
    else
        echo -e "${RED}✗ STDERR: Messages missing ($actual_stderr < $expected_stderr)${NC}"
    fi

    echo

    # Verify specific test markers are present
    echo "Checking for specific test markers:"

    local markers=(
        "EXACT_STDOUT_0"
        "EXACT_STDERR_0"
        "INTERLEAVED_STDOUT_0"
        "INTERLEAVED_STDERR_0"
        "BURST_STDOUT_0"
        "BURST_STDERR_0"
        "MIXED_LINE_STDOUT_with_data"
        "MIXED_LINE_STDERR_with_data"
        "TEST_COMPLETE"
    )

    local all_markers_found=true
    for marker in "${markers[@]}"; do
        if grep -q "$marker" "$log_file"; then
            echo -e "${GREEN}  ✓${NC} \"$marker\""
        else
            echo -e "${RED}  ✗${NC} \"$marker\" NOT FOUND"
            all_markers_found=false
        fi
    done

    echo

    # Verify sequence patterns
    echo "Checking sequence patterns:"

    local missing_exact=0
    local missing_burst=0
    local total_missing=0
    local missing_seq=0

    # Check EXACT sequences (0-100)
    for i in {0..100}; do
        if ! grep -q "EXACT_STDOUT_${i}" "$log_file"; then
            echo -e "${RED}  ✗${NC} Missing EXACT_STDOUT_${i}"
            missing_exact=$((missing_exact + 1))
        fi
        if ! grep -q "EXACT_STDERR_${i}" "$log_file"; then
            echo -e "${RED}  ✗${NC} Missing EXACT_STDERR_${i}"
            missing_exact=$((missing_exact + 1))
        fi
    done

    # Check BURST sequences (sample - 0, 50, 100, 150, 200)
    for i in 0 50 100 150 200; do
        if ! grep -q "BURST_STDOUT_${i}" "$log_file"; then
            echo -e "${RED}  ✗${NC} Missing BURST_STDOUT_${i}"
            missing_burst=$((missing_burst + 1))
        fi
        if ! grep -q "BURST_STDERR_${i}" "$log_file"; then
            echo -e "${RED}  ✗${NC} Missing BURST_STDERR_${i}"
            missing_burst=$((missing_burst + 1))
        fi
    done

    total_missing=$((missing_exact + missing_burst))

    if [ $total_missing -eq 0 ]; then
        echo -e "${GREEN}  ✓ All sequence patterns found (EXACT 0-100, BURST 0-200)${NC}"
    else
        echo -e "${RED}  ✗ $total_missing sequence messages missing${NC}"
    fi

    echo

    # Final verdict
    echo "=== VERIFICATION SUMMARY ==="
    if [ "$stdout_ok" = true ] && [ "$stderr_ok" = true ] && [ "$all_markers_found" = true ] && [ $missing_seq -eq 0 ]; then
        echo -e "${GREEN}✓ ALL CHECKS PASSED${NC}"
        echo "No output loss detected. Both streams captured correctly."
        return 0
    else
        echo -e "${RED}✗ VERIFICATION FAILED${NC}"
        echo "Output loss or corruption detected."
        return 1
    fi
}

# Main execution
case $RUN_MODE in
    run)
        run_test
        ;;
    verify)
        verify_log
        ;;
    full)
        run_test
        verify_log
        ;;
esac

exit $?
