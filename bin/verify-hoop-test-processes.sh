#!/usr/bin/env bash
# HOOP Test Process Verification Script
# Confirms no HOOP test processes remain after cleanup
# Source: docs/test-process-cleanup-patterns.md
#
# Usage:
#   ./bin/verify-hoop-test-processes.sh          # Run verification
#   ./bin/verify-hoop-test-processes.sh --verbose # Show detailed output
#
# Exit codes:
#   0 - No HOOP test processes found (clean)
#   1 - HOOP test processes found (unclean)
#   2 - Zombie/uninterruptible processes found (warning)
#
# This script can be run standalone or sourced by other cleanup scripts

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Verbose flag
VERBOSE=false
if [[ "${1:-}" == "--verbose" ]]; then
    VERBOSE=true
fi

# Count of issues found
TOTAL_ISSUES=0
ZOMBIE_COUNT=0
UNINTERRUPTIBLE_COUNT=0
ORPHANED_COUNT=0

# Array to store found processes
declare -a FOUND_PROCESSES
declare -a ZOMBIE_PROCESSES
declare -a UNINTERRUPTIBLE_PROCESSES
declare -a ORPHANED_PROCESSES

echo -e "${YELLOW}HOOP Test Process Verification${NC}"
echo "=================================="
echo ""

# Function to check for processes matching a pattern
# Only counts processes that are ACTUALLY related to HOOP tests
check_pattern() {
    local pattern="$1"
    local description="$2"
    local pids
    local filtered_pids=""

    # Use pgrep to find matching PIDs (more reliable than ps aux | grep)
    pids=$(pgrep -f "$pattern" 2>/dev/null || true)

    # Filter out false positives
    if [[ -n "$pids" ]]; then
        for pid in $pids; do
            # Get the full command line
            local cmd=$(ps -p "$pid" -o args= 2>/dev/null || echo "unknown")

            # Skip if it's the needle/claude agent running this task
            if [[ "$cmd" =~ "needle.*bf-16sku" ]] || [[ "$cmd" =~ "claude.*bf-16sku" ]]; then
                continue
            fi

            # Only count if it's actually HOOP-related
            if [[ "$cmd" =~ HOOP ]] || [[ "$cmd" =~ testrepo ]] || [[ "$cmd" =~ hoop-[a-f0-9] ]] || [[ "$cmd" =~ hoop_daemon-[a-f0-9] ]]; then
                filtered_pids="$filtered_pids $pid"
            fi
        done
    fi

    if [[ -n "$filtered_pids" ]]; then
        local count=$(echo "$filtered_pids" | wc -w)
        TOTAL_ISSUES=$((TOTAL_ISSUES + count))

        if [[ "$VERBOSE" == true ]]; then
            echo -e "${RED}✗ Found $count process(es) matching: $description${NC}"
            for pid in $filtered_pids; do
                local cmd=$(ps -p "$pid" -o args= 2>/dev/null || echo "unknown")
                echo -e "  ${RED}PID $pid:${NC} $cmd"
                FOUND_PROCESSES+=("PID $pid: $cmd")
            done
        else
            echo -e "${RED}✗ Found $count process(es) matching: $description${NC}"
            FOUND_PROCESSES+=("$description: $count process(es)")
        fi
    else
        echo -e "${GREEN}✓ No processes matching: $description${NC}"
    fi
}

# Function to check for zombie processes (defunct but not reaped)
check_zombies() {
    echo ""
    echo -e "${BLUE}Checking for zombie processes...${NC}"

    # Look for defunct (zombie) processes related to HOOP
    local zombies
    zombies=$(ps aux | grep -E 'hoop|testrepo' | grep -E 'Z$|<defunct>' | grep -v grep || true)

    if [[ -n "$zombies" ]]; then
        ZOMBIE_COUNT=$(echo "$zombies" | wc -l)
        TOTAL_ISSUES=$((TOTAL_ISSUES + ZOMBIE_COUNT))
        echo -e "${RED}✗ Found $ZOMBIE_COUNT zombie process(es)${NC}"
        ZOMBIE_PROCESSES+=("$zombies")
    else
        echo -e "${GREEN}✓ No zombie processes${NC}"
    fi
}

# Function to check for uninterruptible sleep (D state) processes
check_uninterruptible() {
    echo ""
    echo -e "${BLUE}Checking for uninterruptible processes (D state)...${NC}"

    # Look for processes in uninterruptible sleep related to HOOP
    local uninterruptible
    uninterruptible=$(ps aux | grep -E 'hoop|testrepo' | grep -E ' D ' | grep -v grep || true)

    if [[ -n "$uninterruptible" ]]; then
        UNINTERRUPTIBLE_COUNT=$(echo "$uninterruptible" | wc -l)
        TOTAL_ISSUES=$((TOTAL_ISSUES + UNINTERRUPTIBLE_COUNT))
        echo -e "${RED}✗ Found $UNINTERRUPTIBLE_COUNT uninterruptible process(es)${NC}"
        UNINTERRUPTIBLE_PROCESSES+=("$uninterruptible")
    else
        echo -e "${GREEN}✓ No uninterruptible processes${NC}"
    fi
}

# Function to check for orphaned processes (PPID = 1)
check_orphaned() {
    echo ""
    echo -e "${BLUE}Checking for orphaned HOOP subprocesses...${NC}"

    # Look for processes with PPID=1 that match HOOP patterns
    # Exclude system processes and needle agent processes
    local orphaned
    orphaned=$(ps ao pid,ppid,comm,args | awk -v hoop_path="$PWD/HOOP" '
        $2 == 1 && ($3 ~ /^(br|git|rg|tailscale|age|ffmpeg|aider|claude|codex|gemini|opencode|gcloud|systemctl|tmux|df)$/ || $4 ~ hoop_path || $4 ~ "testrepo") \
        && $3 !~ /^(agetty|systemd|needled|needle|claude)$/ \
        && ($4 ~ hoop_path || $4 ~ "testrepo" || $4 ~ "hoop-[a-f0-9]" || $4 ~ "hoop_daemon-[a-f0-9]") \
        {print $0}
    ' || true)

    if [[ -n "$orphaned" ]]; then
        ORPHANED_COUNT=$(echo "$orphaned" | wc -l)
        TOTAL_ISSUES=$((TOTAL_ISSUES + ORPHANED_COUNT))
        echo -e "${RED}✗ Found $ORPHANED_COUNT orphaned process(es)${NC}"
        ORPHANED_PROCESSES+=("$orphaned")
    else
        echo -e "${GREEN}✓ No orphaned processes${NC}"
    fi
}

# ============================================================================
# PRIMARY PATTERNS: HOOP test binaries
# ============================================================================

echo -e "${BLUE}Checking HOOP test binaries...${NC}"

# Pattern 1: hoop-* with 16+ hex chars (e.g., hoop-0964aabb985f3f32)
check_pattern 'hoop-[a-f0-9]\{16,\}$' 'HOOP test binaries (hoop-*)'

# Pattern 2: hoop_daemon-* with 16+ hex chars
check_pattern 'hoop_daemon-[a-f0-9]\{16,\}$' 'HOOP daemon test binaries (hoop_daemon-*)'

# Pattern 3: General HOOP target directory pattern
check_pattern 'HOOP/target/debug/deps' 'HOOP target/debug/deps processes'

# Pattern 4: Any HOOP-related process
check_pattern 'HOOP/target' 'HOOP target directory processes'

# ============================================================================
# TESTREPO PATTERNS
# ============================================================================

echo ""
echo -e "${BLUE}Checking testrepo processes...${NC}"

# Pattern 5: Testrepo stub binaries
check_pattern 'testrepo/bin/br' 'Testrepo br stub binary'

# Pattern 6: Testrepo scripts
check_pattern 'testrepo/scripts/' 'Testrepo script processes'

# Pattern 7: Any testrepo process
check_pattern 'testrepo/(bin|scripts)/' 'Testrepo bin/scripts processes'

# ============================================================================
# BUILD SCRIPT PATTERNS
# ============================================================================

echo ""
echo -e "${BLUE}Checking build script processes...${NC}"

# Pattern 8: Cargo build scripts
check_pattern 'build-script-build$' 'Cargo build scripts'

# Pattern 9: Any build script process
check_pattern 'target/debug/build.*build-script' 'Build script processes'

# ============================================================================
# SUBPROCESS PATTERNS (br, git, rg, etc.)
# ============================================================================

echo ""
echo -e "${BLUE}Checking HOOP subprocess patterns...${NC}"

# Pattern 10: br (beads CLI)
check_pattern '^br\s' 'br subprocesses'

# Pattern 11: git
check_pattern '^git\s' 'git subprocesses'

# Pattern 12: ripgrep (rg)
check_pattern '^rg\s' 'ripgrep subprocesses'

# Pattern 13: tailscale
check_pattern '^tailscale\s' 'tailscale subprocesses'

# Pattern 14: age (encryption)
check_pattern '^age\s' 'age subprocesses'

# Pattern 15: ffmpeg
check_pattern '^ffmpeg\s' 'ffmpeg subprocesses'

# Pattern 16: aider (agent adapter)
check_pattern '^aider\s' 'aider subprocesses'

# Pattern 17: claude (agent adapter)
check_pattern '^claude\s' 'claude subprocesses'

# Pattern 18: codex (agent adapter)
check_pattern '^codex\s' 'codex subprocesses'

# Pattern 19: gemini (agent adapter)
check_pattern '^gemini\s' 'gemini subprocesses'

# Pattern 20: opencode (agent adapter)
check_pattern '^opencode\s' 'opencode subprocesses'

# Pattern 21: gcloud
check_pattern '^gcloud\s' 'gcloud subprocesses'

# Pattern 22: systemctl
check_pattern '^systemctl\s' 'systemctl subprocesses'

# Note: tmux is not checked because it's a common system utility
# and not specific to HOOP test processes

# Pattern 23: df (disk free)
check_pattern '^df\s' 'df subprocesses'

# ============================================================================
# EDGE CASE CHECKS
# ============================================================================

check_zombies
check_uninterruptible
check_orphaned

# ============================================================================
# SUMMARY
# ============================================================================

echo ""
echo "=================================="
if [[ $TOTAL_ISSUES -eq 0 ]]; then
    echo -e "${GREEN}✓ VERIFICATION PASSED: No HOOP test processes found${NC}"
    echo ""
    echo "Environment is clean. Safe to proceed with tests."
    exit 0
else
    echo -e "${RED}✗ VERIFICATION FAILED: Found $TOTAL_ISSUES issue(s)${NC}"
    echo ""

    if [[ ${#FOUND_PROCESSES[@]} -gt 0 ]]; then
        echo -e "${RED}HOOP test processes found:${NC}"
        for proc in "${FOUND_PROCESSES[@]}"; do
            echo "  - $proc"
        done
    fi

    if [[ $ZOMBIE_COUNT -gt 0 ]]; then
        echo -e "${RED}Zombie processes found:${NC}"
        for zombie in "${ZOMBIE_PROCESSES[@]}"; do
            echo "  - $zombie"
        done
        echo ""
        echo "  Zombie processes are dead but not reaped. They usually"
        echo "  disappear automatically, but may require killing the parent."
    fi

    if [[ $UNINTERRUPTIBLE_COUNT -gt 0 ]]; then
        echo -e "${RED}Uninterruptible processes found:${NC}"
        for proc in "${UNINTERRUPTIBLE_PROCESSES[@]}"; do
            echo "  - $proc"
        done
        echo ""
        echo "  Processes in D state are waiting for I/O. They may"
        echo "  need SIGKILL or system intervention to terminate."
    fi

    if [[ $ORPHANED_COUNT -gt 0 ]]; then
        echo -e "${RED}Orphaned processes found:${NC}"
        for orphan in "${ORPHANED_PROCESSES[@]}"; do
            echo "  - $orphan"
        done
        echo ""
        echo "  Orphaned processes (PPID=1) need to be killed by PID."
    fi

    echo ""
    echo "Run cleanup script before proceeding:"
    echo "  ./bin/cleanup-hoop-test-processes.sh"
    echo "  ./bin/kill-hoop-test-processes"

    if [[ $ZOMBIE_COUNT -gt 0 || $UNINTERRUPTIBLE_COUNT -gt 0 ]]; then
        exit 2  # Warning exit code for zombie/uninterruptible
    else
        exit 1  # Normal failure exit code
    fi
fi
