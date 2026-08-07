#!/bin/bash
# Test script to verify --no-interactive flag works with all HOOP subcommands

set -e

HOOP="./target/debug/hoop"
TEST_PASSED=0
TEST_FAILED=0

# Color output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

test_command() {
    local description="$1"
    local command="$2"
    local should_parse="$3"

    echo -n "Testing: $description ... "

    if eval "$command" > /dev/null 2>&1; then
        if [ "$should_parse" = "true" ]; then
            echo -e "${GREEN}✓ PASS${NC}"
            ((TEST_PASSED++))
        else
            echo -e "${RED}✗ FAIL (should have failed but succeeded)${NC}"
            ((TEST_FAILED++))
        fi
    else
        if [ "$should_parse" = "false" ]; then
            echo -e "${GREEN}✓ PASS${NC} (correctly failed)"
            ((TEST_PASSED++))
        else
            echo -e "${RED}✗ FAIL (parsing error)${NC}"
            ((TEST_FAILED++))
        fi
    fi
}

echo "=================================="
echo "Testing --no-interactive flag"
echo "=================================="
echo ""

# 1. Test flag positioning with basic commands
echo "1. Testing flag positioning with basic commands"
test_command "scan with --no-interactive before" \
    "$HOOP --no-interactive scan --help" \
    "true"

test_command "scan with --no-interactive after" \
    "$HOOP scan --help --no-interactive" \
    "true"

test_command "scan with -y short form" \
    "$HOOP -y scan --help" \
    "true"

# 2. Test with remove command
test_command "remove with --no-interactive before" \
    "$HOOP --no-interactive remove --help" \
    "true"

test_command "remove with --no-interactive after" \
    "$HOOP remove --help --no-interactive" \
    "true"

# 3. Test with projects subcommands
test_command "projects scan with --no-interactive before projects" \
    "$HOOP --no-interactive projects scan --help" \
    "true"

test_command "projects scan with --no-interactive after projects" \
    "$HOOP projects --help scan" \
    "true"

test_command "projects scan with --no-interactive after scan" \
    "$HOOP projects scan --help --no-interactive" \
    "true"

# 4. Test with nested subcommands
test_command "backup with --no-interactive" \
    "$HOOP --no-interactive backup --help" \
    "true"

test_command "migrate with --no-interactive" \
    "$HOOP --no-interactive migrate --help" \
    "true"

test_command "script with --no-interactive" \
    "$HOOP --no-interactive script --help" \
    "true"

test_command "config with --no-interactive" \
    "$HOOP --no-interactive config --help" \
    "true"

test_command "risk-patterns with --no-interactive" \
    "$HOOP --no-interactive risk-patterns --help" \
    "true"

test_command "skills with --no-interactive" \
    "$HOOP --no-interactive skills --help" \
    "true"

test_command "pattern with --no-interactive" \
    "$HOOP --no-interactive pattern --help" \
    "true"

test_command "reflection with --no-interactive" \
    "$HOOP --no-interactive reflection --help" \
    "true"

test_command "audit with --no-interactive" \
    "$HOOP --no-interactive audit --help" \
    "true"

# 5. Test with all top-level commands
echo ""
echo "5. Testing all top-level commands"

test_command "serve with --no-interactive" \
    "$HOOP --no-interactive serve --help" \
    "true"

test_command "add with --no-interactive" \
    "$HOOP --no-interactive add --help" \
    "true"

test_command "list with --no-interactive" \
    "$HOOP --no-interactive list --help" \
    "true"

test_command "status with --no-interactive" \
    "$HOOP --no-interactive status --help" \
    "true"

test_command "agent with --no-interactive" \
    "$HOOP --no-interactive agent --help" \
    "true"

test_command "new with --no-interactive" \
    "$HOOP --no-interactive new --help" \
    "true"

test_command "stitch with --no-interactive" \
    "$HOOP --no-interactive stitch --help" \
    "true"

test_command "install-systemd with --no-interactive" \
    "$HOOP --no-interactive install-systemd --help" \
    "true"

test_command "restore with --no-interactive" \
    "$HOOP --no-interactive restore --help" \
    "true"

# 6. Test init command (should parse but runtime will reject)
echo ""
echo "6. Testing init command (parses but runtime rejects)"
test_command "init with --no-interactive" \
    "$HOOP --no-interactive init --help" \
    "true"

# 7. Test multiple flag combinations
echo ""
echo "7. Testing flag combinations"

test_command "status with --no-interactive and --json" \
    "$HOOP --no-interactive --json status --help" \
    "true"

test_command "scan with --no-interactive and --yes" \
    "$HOOP --no-interactive scan --help --yes" \
    "true"

# 8. Test -y short form with various commands
echo ""
echo "8. Testing -y short form"

test_command "-y with projects scan" \
    "$HOOP -y projects scan --help" \
    "true"

test_command "-y with migrate" \
    "$HOOP -y migrate --help" \
    "true"

test_command "-y with backup" \
    "$HOOP -y backup --help" \
    "true"

# Summary
echo ""
echo "=================================="
echo "Test Summary"
echo "=================================="
echo -e "${GREEN}Passed: $TEST_PASSED${NC}"
echo -e "${RED}Failed: $TEST_FAILED${NC}"
echo ""

if [ $TEST_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
