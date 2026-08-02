#!/bin/bash

# test-log-naming-demo.sh - Demonstration of test log naming functionality
#
# This script demonstrates the auto-generated test log naming feature
# and can be used as a reference for integrating it into test workflows.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "================================"
echo "Test Log Naming Demonstration"
echo "================================"
echo ""

# Ensure we're in the project root
cd "$PROJECT_ROOT"

echo "1. Generating log names for various test patterns"
echo "---------------------------------------------------"
echo ""

echo "   Pattern: cargo test --lib"
echo "   Output: $(./bin/generate-test-log-name.sh cargo test --lib)"
echo ""

echo "   Pattern: cargo test --test beads_deletion_http"
echo "   Output: $(./bin/generate-test-log-name.sh cargo test --test beads_deletion_http)"
echo ""

echo "   Pattern: cargo test --test load_test test_medium_scale_load_test"
echo "   Output: $(./bin/generate-test-log-name.sh cargo test --test load_test test_medium_scale_load_test)"
echo ""

echo "2. Special character handling"
echo "------------------------------"
echo ""

echo "   Input: test:with/colons*and?slashes"
echo "   Output: $(./bin/generate-test-log-name.sh cargo test --test 'test:with/colons*and?slashes')"
echo ""

echo "3. Running command with auto-generated log"
echo "--------------------------------------------"
echo ""

# Create a simple test command
echo "Running: echo 'Sample test output' with auto-naming..."
echo ""

# Use run-with-log.sh with --auto
# Run it and capture stderr to extract the log file path
./bin/run-with-log.sh --auto bash -c 'echo "=== Sample Test Output ===" && echo "This is a demonstration of auto-generated log naming" && echo "Timestamp: $(date -u +"%Y-%m-%d %H:%M:%S UTC")"' 2>&1 | tee /tmp/demo-output.txt > /dev/null

# Extract log file path from output
# The output format is: "Auto-generated log file: <path>" so we extract from field 4 onwards
LOG_FILE=$(grep 'Auto-generated log file:' /tmp/demo-output.txt | awk '{print substr($0, index($0, $4))}')
rm -f /tmp/demo-output.txt

if [ -f "$LOG_FILE" ]; then
    echo "✓ Log file created: $LOG_FILE"
    echo ""
    echo "   Contents:"
    echo "   --------"
    sed 's/^/   /' "$LOG_FILE"
    echo ""
    echo "✓ Cleaning up demo log file..."
    rm "$LOG_FILE"
else
    echo "✗ Log file was not created (expected: $LOG_FILE)"
    echo ""
    echo "   Available log files:"
    ls -la logs/*.log 2>/dev/null | tail -3 || echo "   No logs in logs/"
    exit 1
fi

echo ""
echo "4. Integration examples"
echo "------------------------"
echo ""

echo "   In Makefile:"
echo "   ------------"
cat <<'MAKE_EXAMPLE'
test:
	@echo "=== Running tests with auto-generated log ==="
	@./bin/run-with-log.sh --auto cargo test --lib --features testing --verbose
	@./bin/verify-hoop-test-processes.sh || echo "Warning: Some processes may remain"
MAKE_EXAMPLE

echo ""
echo "   In shell scripts:"
echo "   -----------------"
cat <<'SHELL_EXAMPLE'
#!/bin/bash
# Run specific integration test with logging
./bin/run-with-log.sh --auto cargo test --test backup_restore_cycle
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
    echo "Test failed, check log file: $LOG_FILE"
    exit $EXIT_CODE
fi
SHELL_EXAMPLE

echo ""
echo "================================"
echo "✓ Demonstration complete"
echo "================================"
echo ""
echo "For full documentation, see:"
echo "  docs/test-log-naming-convention.md"
echo ""
echo "Scripts:"
echo "  bin/generate-test-log-name.sh - Generate log names from commands"
echo "  bin/run-with-log.sh                - Run commands with log output"
