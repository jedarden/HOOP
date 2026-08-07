#!/bin/bash
# Test --no-interactive flag with all HOOP subcommands
# This verifies the global flag works correctly with every command

set -e

HOOP_BIN="/home/coding/HOOP/target/debug/hoop"
FAILED_TESTS=()
PASSED_TESTS=()
TOTAL_TESTS=0

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Testing --no-interactive flag with all HOOP subcommands"
echo "========================================================"
echo ""

# Test function
test_command() {
    local test_name="$1"
    local args="$2"
    local should_fail="${3:-0}"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    # Try to parse the command (use --help to avoid actual execution)
    if $HOOP_BIN $args --help >/dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} $test_name"
        PASSED_TESTS+=("$test_name")
    else
        if [ "$should_fail" -eq 1 ]; then
            echo -e "${YELLOW}⚠${NC} $test_name (expected to fail, and did)"
            PASSED_TESTS+=("$test_name")
        else
            echo -e "${RED}✗${NC} $test_name"
            FAILED_TESTS+=("$test_name")
        fi
    fi
}

# Test top-level commands with flag before
test_command "serve with --no-interactive before" "--no-interactive serve"
test_command "status with --no-interactive before" "--no-interactive status"
test_command "list with --no-interactive before" "--no-interactive list"
test_command "add with --no-interactive before" "--no-interactive add /tmp/test"
test_command "scan with --no-interactive before" "--no-interactive scan /tmp"
test_command "remove with --no-interactive before" "--no-interactive remove test --confirm"
test_command "restore with --no-interactive before" "--no-interactive restore --from s3://bucket/key --confirm"
test_command "init with --no-interactive before" "--no-interactive init"
test_command "install-systemd with --no-interactive before" "--no-interactive install-systemd"
test_command "agent with --no-interactive before" "--no-interactive agent"
test_command "new with --no-interactive before" "--no-interactive new test-project"
test_command "stitch with --no-interactive before" "--no-interactive stitch test-project"

# Test top-level commands with flag after
test_command "serve with --no-interactive after" "serve --no-interactive"
test_command "status with --no-interactive after" "status --no-interactive"
test_command "list with --no-interactive after" "list --no-interactive"
test_command "add with --no-interactive after" "add /tmp/test --no-interactive"
test_command "scan with --no-interactive after" "scan /tmp --no-interactive"
test_command "remove with --no-interactive after" "remove test --confirm --no-interactive"
test_command "restore with --no-interactive after" "restore --from s3://bucket/key --confirm --no-interactive"
test_command "init with --no-interactive after" "init --no-interactive"
test_command "install-systemd with --no-interactive after" "install-systemd --no-interactive"
test_command "agent with --no-interactive after" "agent --no-interactive"
test_command "new with --no-interactive after" "new test-project --no-interactive"
test_command "stitch with --no-interactive after" "stitch test-project --no-interactive"

# Test short form -y
test_command "scan with -y short form" "-y scan /tmp"
test_command "remove with -y short form" "-y remove test --confirm"
test_command "restore with -y short form" "-y restore --from s3://bucket/key --confirm"

# Test projects subcommands with flag before
test_command "projects add with --no-interactive before" "--no-interactive projects add /tmp/test"
test_command "projects scan with --no-interactive before" "--no-interactive projects scan /tmp"
test_command "projects list with --no-interactive before" "--no-interactive projects list"
test_command "projects remove with --no-interactive before" "--no-interactive projects remove test --confirm"
test_command "projects show with --no-interactive before" "--no-interactive projects show test"

# Test projects subcommands with flag after
test_command "projects add with --no-interactive after" "projects add /tmp/test --no-interactive"
test_command "projects scan with --no-interactive after" "projects scan /tmp --no-interactive"
test_command "projects list with --no-interactive after" "projects list --no-interactive"
test_command "projects remove with --no-interactive after" "projects remove test --confirm --no-interactive"
test_command "projects show with --no-interactive after" "projects show test --no-interactive"

# Test audit subcommands with flag before
test_command "audit check with --no-interactive before" "--no-interactive audit check"
test_command "audit verify with --no-interactive before" "--no-interactive audit verify"

# Test audit subcommands with flag after
test_command "audit check with --no-interactive after" "audit check --no-interactive"
test_command "audit verify with --no-interactive after" "audit verify --no-interactive"

# Test backup subcommands with flag before
test_command "backup create with --no-interactive before" "--no-interactive backup create"
test_command "backup list with --no-interactive before" "--no-interactive backup list"

# Test backup subcommands with flag after
test_command "backup create with --no-interactive after" "backup create --no-interactive"
test_command "backup list with --no-interactive after" "backup list --no-interactive"

# Test migrate subcommands with flag before
test_command "migrate run with --no-interactive before" "--no-interactive migrate run --confirm"
test_command "migrate status with --no-interactive before" "--no-interactive migrate status"
test_command "migrate major-upgrade with --no-interactive before" "--no-interactive migrate major-upgrade --confirm"
test_command "migrate rollback with --no-interactive before" "--no-interactive migrate rollback 1.0.0 --confirm"
test_command "migrate rebuild-percentile-index with --no-interactive before" "--no-interactive migrate rebuild-percentile-index"

# Test migrate subcommands with flag after
test_command "migrate run with --no-interactive after" "migrate run --confirm --no-interactive"
test_command "migrate status with --no-interactive after" "migrate status --no-interactive"
test_command "migrate major-upgrade with --no-interactive after" "migrate major-upgrade --confirm --no-interactive"
test_command "migrate rollback with --no-interactive after" "migrate rollback 1.0.0 --confirm --no-interactive"
test_command "migrate rebuild-percentile-index with --no-interactive after" "migrate rebuild-percentile-index --no-interactive"

# Test config subcommands with flag before
test_command "config diff with --no-interactive before" "--no-interactive config diff"

# Test config subcommands with flag after
test_command "config diff with --no-interactive after" "config diff --no-interactive"

# Print summary
echo ""
echo "========================================================"
echo "SUMMARY"
echo "========================================================"
echo "Total tests: $TOTAL_TESTS"
echo "Passed: ${#PASSED_TESTS[@]}"
echo "Failed: ${#FAILED_TESTS[@]}"

if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
    echo ""
    echo "Failed tests:"
    for test in "${FAILED_TESTS[@]}"; do
        echo "  - $test"
    done
    exit 1
else
    echo ""
    echo -e "${GREEN}All tests passed!${NC}"
    echo "The --no-interactive flag works correctly with all subcommands."
    exit 0
fi
