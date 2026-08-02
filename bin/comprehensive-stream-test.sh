#!/bin/bash

# comprehensive-stream-test.sh - Standalone test for stdout/stderr stream capture verification
#
# This script generates comprehensive output to both stdout and stderr streams
# to verify that all output is captured in log files without loss.
#
# Usage: comprehensive-stream-test.sh [--verify-only | --help]

set -euo pipefail

# Configuration
TOTAL_EXACT_STDOUT=100
TOTAL_EXACT_STDERR=100
TOTAL_INTERLEAVED=50
TOTAL_BURST=200

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Help message
if [ "${1:-}" = "--help" ]; then
    echo "Usage: $0 [--verify-only | --help]"
    echo ""
    echo "Options:"
    echo "  --verify-only  Skip running tests, only verify existing logs"
    echo "  --help         Show this help message"
    echo ""
    echo "This script tests stdout/stderr capture by generating:"
    echo "  - $TOTAL_EXACT_STDOUT exact stdout messages"
    echo "  - $TOTAL_EXACT_STDERR exact stderr messages"
    echo "  - $TOTAL_INTERLEAVED interleaved stdout/stderr pairs"
    echo "  - $TOTAL_BURST burst messages on each stream"
    echo "  - Mixed content lines with both streams"
    echo ""
    echo "Run this script via run-with-log.sh to verify stream capture."
    exit 0
fi

echo -e "${BLUE}=== COMPREHENSIVE STREAM CAPTURE TEST ===${NC}"
echo "This test will generate:"
echo "  - $TOTAL_EXACT_STDOUT exact stdout messages"
echo "  - $TOTAL_EXACT_STDERR exact stderr messages"
echo "  - $TOTAL_INTERLEAVED interleaved stdout/stderr pairs"
echo "  - $TOTAL_BURST burst messages on each stream"
echo "  - Mixed content lines"
echo
echo "Starting test execution..."
echo

# Phase 1: Exact sequential output
echo -e "${YELLOW}Phase 1: Exact sequential output...${NC}"
for i in $(seq 0 $TOTAL_EXACT_STDOUT); do
    echo "EXACT_STDOUT_${i}"
done

for i in $(seq 0 $TOTAL_EXACT_STDERR); do
    >&2 echo "EXACT_STDERR_${i}"
done

echo -e "${GREEN}✓ Phase 1 completed${NC}"
echo

# Phase 2: Interleaved output (alternating between streams)
echo -e "${YELLOW}Phase 2: Interleaved output...${NC}"
for i in $(seq 0 $TOTAL_INTERLEAVED); do
    echo "INTERLEAVED_STDOUT_${i}"
    >&2 echo "INTERLEAVED_STDERR_${i}"
done

echo -e "${GREEN}✓ Phase 2 completed${NC}"
echo

# Phase 3: Burst output (rapid consecutive writes)
echo -e "${YELLOW}Phase 3: Burst output...${NC}"

# Burst on stdout
for i in $(seq 0 $TOTAL_BURST); do
    echo "BURST_STDOUT_${i}"
done

# Burst on stderr
for i in $(seq 0 $TOTAL_BURST); do
    >&2 echo "BURST_STDERR_${i}"
done

echo -e "${GREEN}✓ Phase 3 completed${NC}"
echo

# Phase 4: Mixed content (lines with special characters and formatting)
echo -e "${YELLOW}Phase 4: Mixed content output...${NC}"

# Mixed stderr line with special data
>&2 echo "MIXED_LINE_STDERR_with_data_TIMESTAMP_$(date +%s)_USER_${USER}_PID_$$"

# Mixed stdout line with special data
echo "MIXED_LINE_STDOUT_with_data_TIMESTAMP_$(date +%s)_USER_${USER}_PID_$$"

# Lines with various special characters
echo "STDOUT: Special chars test - tabs\ttabs\ttabs - newlines"
>&2 echo "STDERR: Special chars test - backslash\\ quote\" dollar\$"

# Long lines
long_content="LONG_LINE_STDOUT_"
for i in {1..50}; do
    long_content="${long_content}DATA_${i}_"
done
echo "$long_content"

long_content="LONG_LINE_STDERR_"
for i in {1..50}; do
    long_content="${long_content}DATA_${i}_"
done
>&2 echo "$long_content"

echo -e "${GREEN}✓ Phase 4 completed${NC}"
echo

# Final marker
echo "TEST_COMPLETE"
echo
echo -e "${BLUE}=== TEST EXECUTION COMPLETED ===${NC}"
echo
echo "Verification Summary:"
echo "Expected output counts:"
echo "  - EXACT_STDOUT:   $((TOTAL_EXACT_STDOUT + 1)) messages"
echo "  - EXACT_STDERR:   $((TOTAL_EXACT_STDERR + 1)) messages"
echo "  - INTERLEAVED:     $((TOTAL_INTERLEAVED * 2)) messages ($TOTAL_INTERLEAVED stdout + $TOTAL_INTERLEAVED stderr)"
echo "  - BURST_STDOUT:  $((TOTAL_BURST + 1)) messages"
echo "  - BURST_STDERR:  $((TOTAL_BURST + 1)) messages"
echo "  - Mixed lines:     4 messages (2 stdout + 2 stderr)"
echo "  - Special chars:   2 messages (1 stdout + 1 stderr)"
echo "  - Long lines:      2 messages (1 stdout + 1 stderr)"
echo "  - Test markers:    1 message (TEST_COMPLETE)"
echo
echo "Total expected: ~$((TOTAL_EXACT_STDOUT + TOTAL_EXACT_STDERR + TOTAL_INTERLEAVED * 2 + TOTAL_BURST * 2 + 12)) messages across both streams"
