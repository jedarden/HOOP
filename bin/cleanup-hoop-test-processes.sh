#!/usr/bin/env bash
# HOOP Test Process Cleanup Script
# Kills all HOOP-related test processes and subprocesses reliably
# Source: docs/test-process-cleanup-patterns.md
#
# This script is TARGETED - it only kills processes that are actually
# related to HOOP tests, not all git/claude/etc processes on the system.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Count of killed processes
KILLED_COUNT=0

# Function to safely kill processes and count them
kill_and_count() {
    local pattern="$1"
    local signal="${2:--TERM}"  # Default to SIGTERM, can override with -9
    local description="$3"

    # Use pgrep to find matching PIDs
    local pids
    pids=$(pgrep -f "$pattern" 2>/dev/null || true)

    if [[ -n "$pids" ]]; then
        echo -e "${YELLOW}Killing $description...${NC}"
        # shellcheck disable=SC2086
        pkill "$signal" -f "$pattern" 2>/dev/null || true
        sleep 0.1  # Give processes a moment to terminate
        local count=$(echo "$pids" | wc -w)
        KILLED_COUNT=$((KILLED_COUNT + count))
        echo -e "${GREEN}  ✓ Killed $count process(es)${NC}"
    fi
}

echo -e "${YELLOW}HOOP Test Process Cleanup${NC}"
echo "=============================="
echo ""

# PRIMARY TARGET: HOOP test binaries in target/debug/deps/
# These are the main processes that leak
kill_and_count 'HOOP/target/debug/deps/hoop' '-TERM' 'HOOP test binaries'

# Testrepo stub binaries (used during integration tests)
kill_and_count 'testrepo/bin/br' '-TERM' 'Testrepo stub binaries'

# Build scripts that can be left running after interrupted builds
kill_and_count 'HOOP/target/debug/build.*build-script-build' '-TERM' 'HOOP build scripts'

# SUBPROCESS TARGETS: Only kill subprocesses that are CHILDREN of HOOP test processes
# This is the key - we don't want to kill git/claude/etc that are working on OTHER repos

echo -e "${YELLOW}Checking for HOOP subprocess children...${NC}"

# Find all HOOP test process PIDs
hoop_test_pids=$(pgrep -f 'HOOP/target/debug/deps/hoop' 2>/dev/null || true)

if [[ -n "$hoop_test_pids" ]]; then
    # For each HOOP test process, find its children and kill them
    for parent_pid in $hoop_test_pids; do
        # Get all child processes using pstree
        child_pids=$(pstree -p "$parent_pid" | grep -oP '(?<=\()[0-9]+(?=\))' || true)

        if [[ -n "$child_pids" ]]; then
            for child_pid in $child_pids; do
                # Get the command name
                cmd=$(ps -p "$child_pid" -o comm= 2>/dev/null || true)

                # Check if it's one of our known subprocess types
                if [[ "$cmd" =~ ^(br|git|rg|tailscale|age|ffmpeg|aider|claude|codex|gemini|opencode|gcloud|systemctl|tmux|df)$ ]]; then
                    echo -e "${YELLOW}Killing subprocess child: $cmd (PID: $child_pid)${NC}"
                    kill -9 "$child_pid" 2>/dev/null || true
                    KILLED_COUNT=$((KILLED_COUNT + 1))
                    echo -e "${GREEN}  ✓ Killed $cmd${NC}"
                fi
            done
        fi
    done

    # Now kill the parent HOOP test processes themselves
    echo -e "${YELLOW}Killing HOOP test parent processes...${NC}"
    pkill -9 -f 'HOOP/target/debug/deps/hoop' 2>/dev/null || true
    parent_count=$(echo "$hoop_test_pids" | wc -w)
    KILLED_COUNT=$((KILLED_COUNT + parent_count))
    echo -e "${GREEN}  ✓ Killed $parent_count HOOP test process(es)${NC}"
fi

# ORPHAN CHECK: Look for processes with PPID=1 that match HOOP patterns
# These are processes whose parent (the HOOP test) already died
echo -e "${YELLOW}Checking for orphaned HOOP subprocesses...${NC}"
orphaned_pids=$(ps ao pid,ppid,comm,args | awk -v hoop_path="$PWD/HOOP" '$2 == 1 && ($3 ~ /^(br|git|rg|tailscale|age|ffmpeg)$/ || $4 ~ hoop_path) {print $1}' || true)

if [[ -n "$orphaned_pids" ]]; then
    for pid in $orphaned_pids; do
        cmd=$(ps -p "$pid" -o comm= 2>/dev/null || true)
        args=$(ps -p "$pid" -o args= 2>/dev/null || true)

        # Only kill if it's actually related to HOOP
        if [[ "$args" =~ "$PWD/HOOP" || "$args" =~ "testrepo" ]]; then
            echo -e "${YELLOW}Killing orphaned HOOP subprocess: $cmd (PID: $pid)${NC}"
            kill -9 "$pid" 2>/dev/null || true
            KILLED_COUNT=$((KILLED_COUNT + 1))
            echo -e "${GREEN}  ✓ Killed orphaned $cmd${NC}"
        fi
    done
fi

echo ""
if [[ $KILLED_COUNT -gt 0 ]]; then
    echo -e "${GREEN}✓ Cleanup complete: killed $KILLED_COUNT HOOP-related process(es)${NC}"
else
    echo -e "${GREEN}✓ No HOOP test processes found - already clean${NC}"
fi

# Verification - use the dedicated verification script
echo ""
echo "Running verification..."
if ./bin/verify-hoop-test-processes.sh; then
    exit 0
else
    exit 1
fi
