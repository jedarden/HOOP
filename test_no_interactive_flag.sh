#!/bin/bash
# Test script to verify --no-interactive flag works with all HOOP subcommands
# This tests flag parsing (not execution behavior)

set -e

HOOP_BIN="./target/debug/hoop"
FAILED_TESTS=()
PASSED_TESTS=()
SKIPPED_TESTS=()

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test a single command with the flag
test_command() {
    local cmd="$1"
    local desc="$2"

    echo -n "Testing: $desc ... "

    # Test flag before command
    if $HOOP_BIN --no-interactive $cmd --help 2>&1 | grep -q "Usage:"; then
        # Test flag after command (reconstruct args to insert flag)
        local cmd_parts=($cmd)
        local subcommand="${cmd_parts[0]}"
        local args="${cmd_parts[@]:1}"

        if [ -n "$args" ]; then
            if $HOOP_BIN $subcommand $args --no-interactive --help 2>&1 | grep -q "Usage:"; then
                # Test short form -y
                if $HOOP_BIN -y $cmd --help 2>&1 | grep -q "Usage:"; then
                    echo -e "${GREEN}✓ PASSED${NC}"
                    PASSED_TESTS+=("$desc")
                    return 0
                else
                    echo -e "${RED}✗ FAILED (short form -y)${NC}"
                    FAILED_TESTS+=("$desc (short form -y)")
                    return 1
                fi
            else
                echo -e "${RED}✗ FAILED (flag after command)${NC}"
                FAILED_TESTS+=("$desc (flag after command)")
                return 1
            fi
        else
            echo -e "${YELLOW}~ SKIPPED (no args)${NC}"
            SKIPPED_TESTS+=("$desc")
            return 0
        fi
    else
        echo -e "${RED}✗ FAILED (flag before command)${NC}"
        FAILED_TESTS+=("$desc (flag before command)")
        return 1
    fi
}

echo "=========================================="
echo "Testing --no-interactive flag parsing"
echo "=========================================="
echo

# Test top-level commands
echo "Top-level commands:"
test_command "serve" "serve"
test_command "add /tmp" "add"
test_command "scan /tmp" "scan"
test_command "list" "list"
test_command "remove test-project" "remove (with --confirm needed for execution)"
test_command "status" "status"
test_command "agent" "agent"
test_command "new test-project" "new"
test_command "stitch" "stitch"
test_command "install-systemd" "install-systemd"
test_command "restore --from s3://bucket/key" "restore (with --confirm needed for execution)"
test_command "init" "init"

echo
echo "Projects subcommands:"
test_command "projects add /tmp" "projects add"
test_command "projects scan /tmp" "projects scan"
test_command "projects list" "projects list"
test_command "projects remove test" "projects remove (with --confirm needed)"
test_command "projects show test" "projects show"

echo
echo "Audit subcommands:"
test_command "audit check" "audit check"
test_command "audit verify" "audit verify"

echo
echo "Migrate subcommands:"
test_command "migrate status" "migrate status"
test_command "migrate run" "migrate run (with --confirm needed)"
test_command "migrate major-upgrade" "migrate major-upgrade (with --confirm needed)"
test_command "migrate rollback 1.0.0" "migrate rollback (with --confirm needed)"
test_command "migrate rebuild-percentile-index" "migrate rebuild-percentile-index"

echo
echo "Config subcommands:"
test_command "config diff" "config diff"

echo
echo "Backup subcommands:"
test_command "backup create" "backup create"
test_command "backup list" "backup list"

echo
echo "Script subcommands:"
test_command "script list" "script list"
test_command "script show test" "script show"

echo
echo "Risk patterns subcommands:"
test_command "risk-patterns list" "risk-patterns list"
test_command "risk-patterns show test" "risk-patterns show"

echo
echo "Skills subcommands:"
test_command "skills list" "skills list"
test_command "skills show test" "skills show"

echo
echo "Pattern subcommands:"
test_command "pattern list" "pattern list"
test_command "pattern show test" "pattern show"

echo
echo "Reflection subcommands:"
test_command "reflection list" "reflection list"
test_command "reflection show test" "reflection show"

echo
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo -e "${GREEN}Passed: ${#PASSED_TESTS[@]}${NC}"
echo -e "${RED}Failed: ${#FAILED_TESTS[@]}${NC}"
echo -e "${YELLOW}Skipped: ${#SKIPPED_TESTS[@]}${NC}"

if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
    echo
    echo -e "${RED}Failed tests:${NC}"
    for test in "${FAILED_TESTS[@]}"; do
        echo "  - $test"
    done
    exit 1
else
    echo
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
fi
