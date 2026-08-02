#!/bin/bash

# run-with-log.sh - Redirect command output to log file while preserving exit code
#
# This script runs a command with its output redirected to a log file, while
# preserving the original command's exit code. It can automatically generate
# descriptive log file names from test commands.
#
# Usage:
#   run-with-log.sh <log_file> <command> [args...]
#   run-with-log.sh --auto <command> [args...]    # Auto-generate log name
#
# Arguments:
#   --auto       - Automatically generate descriptive log file name from command
#   log_file     - Path to the log file where output will be written
#   command      - Command to execute
#   args...      - Additional arguments to pass to the command
#
# Log Naming Convention (when using --auto):
#   <test_name>_<timestamp>.log
#   - test_name: Derived from test command (e.g., beads_deletion_http)
#   - timestamp: ISO 8601 format (YYYYMMDDTHHMMSSZ) in UTC
#   - .log: Standard extension
#
# Examples:
#   run-with-log.sh /tmp/test.log cargo test --workspace
#   run-with-log.sh --auto cargo test --lib
#   run-with-log.sh --auto cargo test --test beads_deletion_http
#
# Output:
#   - Command output is written to the log file
#   - If --auto is used, prints the generated log file path to stderr
#   - Exit code matches the original command's exit code

set -euo pipefail

# Find the script directory to locate companion scripts
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check arguments
if [ $# -lt 1 ]; then
    echo "Usage: $0 <log_file>|--auto <command> [args...]" >&2
    echo "" >&2
    echo "Options:" >&2
    echo "  --auto    Auto-generate descriptive log file name from command" >&2
    echo "" >&2
    echo "Examples:" >&2
    echo "  $0 /tmp/test.log cargo test --workspace" >&2
    echo "  $0 --auto cargo test --lib" >&2
    exit 1
fi

LOG_FILE=""

# Check if --auto flag is used
if [ "$1" = "--auto" ]; then
    shift  # Remove --auto from arguments

    # Generate log name using companion script
    if [ ! -f "${SCRIPT_DIR}/generate-test-log-name.sh" ]; then
        echo "Error: generate-test-log-name.sh not found in ${SCRIPT_DIR}" >&2
        exit 1
    fi

    LOG_NAME=$("${SCRIPT_DIR}/generate-test-log-name.sh" "$@")

    if [ -z "$LOG_NAME" ]; then
        echo "Error: Failed to generate log file name" >&2
        exit 1
    fi

    # Default to logs/ directory if it exists, otherwise current directory
    if [ -d "logs" ]; then
        LOG_FILE="logs/${LOG_NAME}"
    else
        LOG_FILE="${LOG_NAME}"
    fi

    echo "Auto-generated log file: ${LOG_FILE}" >&2
else
    # Use provided log file path
    LOG_FILE="$1"
    shift  # Remove log file from arguments, leaving only the command and its args
fi

# Run the command, redirecting both stdout and stderr to the log file
# Preserve the exit code
"$@" > "$LOG_FILE" 2>&1
EXIT_CODE=$?

# Exit with the original command's exit code
exit $EXIT_CODE
