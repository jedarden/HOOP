#!/bin/bash

# generate-test-log-name.sh - Generate descriptive log file names from test commands
#
# This script analyzes test commands and generates descriptive, safe log file names
# with timestamps for uniqueness. It handles various test command patterns.
#
# Usage:
#   generate-test-log-name.sh <test_command> [args...]
#
# Examples:
#   generate-test-log-name.sh cargo test --lib
#   generate-test-log-name.sh cargo test --test beads_deletion_http
#   generate-test-log-name.sh cargo test beads_deletion_http_test
#
# Output:
#   Prints a log file name to stdout, e.g.:
#   lib_test_20260802T064026Z.log
#   beads_deletion_http_test_20260802T064026Z.log
#
# Naming Convention:
#   <test_name>_<timestamp>.log
#   - test_name: Derived from test command, sanitized for filesystem safety
#   - timestamp: ISO 8601 format (YYYYMMDDTHHMMSSZ) in UTC
#   - .log: Standard extension

set -euo pipefail

# Generate UTC timestamp in ISO 8601 format (YYYYMMDDTHHMMSSZ)
# Example: 20260802T064026Z
generate_timestamp() {
    date -u +"%Y%m%dT%H%M%SZ"
}

# Sanitize a string for safe filesystem usage
# - Replace spaces with underscores
# - Remove or replace special characters (/, :, *, ?, ", <, >, |)
# - Limit length to 255 characters (filesystem limit)
# - Prevent leading/trailing dots or hyphens
sanitize_name() {
    local input="$1"

    # Replace spaces with underscores
    input="${input// /_}"

    # Remove unsafe filesystem characters
    # Replace with empty string or underscore
    input="${input//\//_}"      # Forward slash
    input="${input//\\/_}"      # Backslash
    input="${input//:/_}"       # Colon
    input="${input//\*/_}"      # Asterisk
    input="${input//\?/}"       # Question mark (remove)
    input="${input//\"/}"       # Double quote (remove)
    input="${input//</_}"       # Less than
    input="${input//>/_}"       # Greater than
    input="${input//|/}"        # Pipe (remove)

    # Remove consecutive underscores
    input="${input//__/_}"

    # Remove leading/trailing underscores, dots, hyphens
    input="${input#[._-]}"
    input="${input%[._-]}"

    # Limit to 200 characters (leave room for timestamp and extension)
    local max_length=200
    if [ ${#input} -gt $max_length ]; then
        input="${input:0:$max_length}"
        # Trim trailing underscores after truncation
        input="${input%_}"
    fi

    echo "$input"
}

# Extract test name from cargo test command
extract_test_name() {
    local cmd="$1"
    shift
    local args=("$@")

    local test_name=""

    # Check for --test flag: cargo test --test <test_name>
    for ((i=0; i<${#args[@]}; i++)); do
        if [ "${args[$i]}" = "--test" ] && [ $((i+1)) -lt ${#args[@]} ]; then
            test_name="${args[$((i+1))]}"
            break
        fi
    done

    # If no --test flag, look for test name as positional argument
    # Pattern: cargo test <test_name>
    if [ -z "$test_name" ]; then
        for arg in "${args[@]}"; do
            # Skip common cargo flags and options
            case "$arg" in
                --*) continue ;;
                -*) continue ;;
                *=*) continue ;;
                lib|test|doc) continue ;;  # Skip test profiles
                *)
                    # First non-flag argument is likely the test name
                    if [ -z "$test_name" ]; then
                        test_name="$arg"
                    fi
                    ;;
            esac
        done
    fi

    # If still no test name, derive from test type
    if [ -z "$test_name" ]; then
        # Check for --lib flag (library tests)
        for arg in "${args[@]}"; do
            if [ "$arg" = "--lib" ]; then
                test_name="lib_test"
                break
            fi
        done

        # Check for --doc flag (documentation tests)
        for arg in "${args[@]}"; do
            if [ "$arg" = "--doc" ]; then
                test_name="doc_test"
                break
            fi
        done

        # Default to "unit_test" if running just "cargo test"
        if [ -z "$test_name" ]; then
            test_name="unit_test"
        fi
    fi

    # Handle load tests specifically
    for arg in "${args[@]}"; do
        if [[ "$arg" =~ load.*test ]]; then
            test_name="${arg}"
            break
        fi
    done

    echo "$test_name"
}

# Main logic
main() {
    if [ $# -lt 1 ]; then
        echo "Usage: $0 <test_command> [args...]" >&2
        echo "Example: $0 cargo test --test beads_deletion_http" >&2
        exit 1
    fi

    # Extract components
    local cmd="$1"
    shift
    local args=("$@")

    # Extract test name from command
    local test_name
    test_name=$(extract_test_name "$cmd" "${args[@]}")

    # Sanitize the test name for filesystem safety
    local sanitized_name
    sanitized_name=$(sanitize_name "$test_name")

    # Generate timestamp
    local timestamp
    timestamp=$(generate_timestamp)

    # Combine: <sanitized_test_name>_<timestamp>.log
    local log_name="${sanitized_name}_${timestamp}.log"

    # Output the log file name
    echo "$log_name"
}

# Run main
main "$@"
