#!/bin/bash

# run-with-log.sh - Redirect command output to log file while preserving exit code
#
# Usage: run-with-log.sh <log_file> <command> [args...]
#
# Arguments:
#   log_file  - Path to the log file where output will be written
#   command   - Command to execute
#   args...   - Additional arguments to pass to the command
#
# Example:
#   run-with-log.sh /tmp/test.log cargo test --workspace

set -euo pipefail

# Check arguments
if [ $# -lt 2 ]; then
    echo "Usage: $0 <log_file> <command> [args...]" >&2
    exit 1
fi

LOG_FILE="$1"
shift  # Remove log file from arguments, leaving only the command and its args

# Run the command, redirecting both stdout and stderr to the log file
# Preserve the exit code
"$@" > "$LOG_FILE" 2>&1
EXIT_CODE=$?

# Exit with the original command's exit code
exit $EXIT_CODE
