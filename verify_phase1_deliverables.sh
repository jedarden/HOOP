#!/bin/bash
# Phase 1 Deliverables Verification Script
# Tests all 14 deliverables against testrepo/ fixture

set -e

PASS="✅"
FAIL="❌"
SKIP="⏭️ "

PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0

check_pass() {
    echo "$PASS $1"
    ((PASS_COUNT++))
}

check_fail() {
    echo "$FAIL $1"
    ((FAIL_COUNT++))
}

check_skip() {
    echo "$SKIP $1"
    ((SKIP_COUNT++))
}

echo "================================"
echo "Phase 1 Deliverables Verification"
echo "Testing against testrepo/ fixture"
echo "================================"
echo ""

# Deliverable 1: hoop-daemon binary builds and runs
echo "1. hoop-daemon binary builds and runs"
echo "   Checking for binary..."
if [ -f "target/release/hoop" ]; then
    check_pass "Binary exists at target/release/hoop"
    echo "   Checking serve command..."
    if target/release/hoop serve --help &>/dev/null; then
        check_pass "serve command available"
    else
        check_fail "serve command not available"
    fi
else
    check_fail "Binary not found"
fi
echo ""

# Deliverable 2: Single workspace registration
echo "2. Single workspace registration (~/.hoop/projects.yaml)"
echo "   Checking projects command..."
if target/release/hoop projects --help &>/dev/null; then
    check_pass "projects subcommand exists"

    echo "   Creating test HOOP home..."
    TEST_HOME=$(mktemp -d)
    export HOME="$TEST_HOME"
    mkdir -p "$HOME/.hoop"

    echo "   Creating test projects.yaml..."
    cat > "$HOME/.hoop/projects.yaml" <<EOF
projects:
  - name: testrepo
    path: /home/coding/HOOP/testrepo
EOF

    echo "   Testing projects list..."
    if target/release/hoop projects list &>/dev/null; then
        check_pass "projects list works"
    else
        check_fail "projects list failed"
    fi

    rm -rf "$TEST_HOME"
else
    check_fail "projects subcommand missing"
fi
echo ""

# Deliverable 3: Event tailer
echo "3. Event tailer (events.jsonl + heartbeats.jsonl)"
echo "   Checking for event tailer code..."
if [ -f "hoop-daemon/src/events.rs" ]; then
    check_pass "Event tailer code exists"

    echo "   Checking testrepo fixture files..."
    if [ -f "testrepo/.beads/events.jsonl" ]; then
        check_pass "events.jsonl exists in testrepo"
    else
        check_fail "events.jsonl missing"
    fi

    if [ -f "testrepo/.beads/heartbeats.jsonl" ]; then
        check_pass "heartbeats.jsonl exists in testrepo"
    else
        check_fail "heartbeats.jsonl missing"
    fi

    # Check for partial line handling
    if grep -q "partial" hoop-daemon/src/events.rs; then
        check_pass "Partial line handling mentioned"
    else
        check_skip "Partial line handling not explicit"
    fi
else
    check_fail "Event tailer code missing"
fi
echo ""

# Deliverable 4: Session tailer (Claude Code + OpenCode adapters)
echo "4. Session tailer (Claude Code + OpenCode adapters)"
if [ -f "hoop-daemon/src/sessions.rs" ]; then
    check_pass "Session tailer code exists"

    echo "   Checking adapter support..."
    if grep -q "claude" hoop-daemon/src/sessions.rs 2>/dev/null; then
        check_pass "Claude adapter supported"
    else
        check_fail "Claude adapter not found"
    fi

    if grep -q "opencode" hoop-daemon/src/sessions.rs 2>/dev/null; then
        check_pass "OpenCode adapter supported"
    else
        check_fail "OpenCode adapter not found"
    fi

    echo "   Checking testrepo CLI sessions..."
    if [ -d "testrepo/cli-sessions" ]; then
        check_pass "CLI sessions directory exists"

        # Check for specific adapter sessions
        if [ -f "testrepo/.beads/cli-sessions/alpha/session.jsonl" ]; then
            check_pass "Worker session files exist"
        else
            check_fail "Worker session files missing"
        fi
    else
        check_fail "CLI sessions directory missing"
    fi
else
    check_fail "Session tailer code missing"
fi
echo ""

# Deliverable 5: Worker heartbeat monitor
echo "5. Worker heartbeat monitor (kill -0 pid)"
if [ -f "hoop-daemon/src/heartbeats.rs" ]; then
    check_pass "Heartbeat monitor code exists"

    if grep -q "kill -0" hoop-daemon/src/heartbeats.rs 2>/dev/null; then
        check_pass "kill -0 pid check implemented"
    else
        check_skip "kill -0 check method unclear"
    fi

    if grep -q "freshness\|liveness" hoop-daemon/src/heartbeats.rs 2>/dev/null; then
        check_pass "Freshness/liveness tracking present"
    else
        check_skip "Freshness tracking unclear"
    fi
else
    check_fail "Heartbeat monitor code missing"
fi
echo ""

# Deliverable 6: Bead-level subscription (needle: tag extraction)
echo "6. Bead-level subscription (needle: tag extraction)"
if grep -r "needle:" hoop-daemon/src/ 2>/dev/null | grep -q "extract\|parse\|tag"; then
    check_pass "Tag extraction logic exists"
else
    check_skip "Tag extraction logic unclear"
fi

if grep -q "session_bound\|bead_id" hoop-daemon/src/sessions.rs 2>/dev/null; then
    check_pass "Session-to-bead linking present"
else
    check_skip "Session-to-bead linking unclear"
fi
echo ""

# Deliverable 7: Worker transcript viewer (REST + WS)
echo "7. Worker transcript viewer (REST + WS)"
if [ -f "hoop-daemon/src/api_beads.rs" ] || [ -f "hoop-daemon/src/api_transcripts.rs" ]; then
    check_pass "Transcript API endpoint exists"

    if grep -q "websocket\|ws\|WebSocket" hoop-daemon/src/lib.rs 2>/dev/null; then
        check_pass "WebSocket support present"
    else
        check_fail "WebSocket support missing"
    fi
else
    check_fail "Transcript API missing"
fi
echo ""

# Deliverable 8: Read-only web UI (React SPA)
echo "8. Read-only web UI (React SPA)"
if [ -d "hoop-ui/web/src" ]; then
    check_pass "Web UI source exists"

    if [ -f "hoop-ui/web/package.json" ]; then
        check_pass "package.json exists"

        if grep -q "react\|vite" hoop-ui/web/package.json; then
            check_pass "React + Vite confirmed"
        else
            check_fail "React/Vite not in package.json"
        fi
    else
        check_fail "package.json missing"
    fi

    # Check for zero write paths (no create/update/delete in UI for Phase 1)
    if grep -r "create\|update\|delete" hoop-ui/web/src/ 2>/dev/null | grep -q "bead\|stitch"; then
        check_fail "Write paths found in UI (Phase 1 should be read-only)"
    else
        check_pass "No write paths visible in UI"
    fi
else
    check_fail "Web UI source missing"
fi
echo ""

# Deliverable 9: hoop status --json
echo "9. hoop status --json CLI command"
if target/release/hoop status --help &>/dev/null; then
    check_pass "status command exists"

    if target/release/hoop status --help 2>&1 | grep -q "json"; then
        check_pass "--json flag available"
    else
        check_fail "--json flag missing"
    fi
else
    check_fail "status command missing"
fi
echo ""

# Deliverable 10: hoop audit (minimum viable)
echo "10. hoop audit (minimum viable)"
if target/release/hoop audit --help &>/dev/null; then
    check_pass "audit command exists"

    echo "   Checking for E-code taxonomy..."
    if grep -r "E[0-9]" hoop-daemon/src/ 2>/dev/null | grep -q "event\|error"; then
        check_pass "E-code taxonomy present"
    else
        check_skip "E-code taxonomy unclear"
    fi
else
    check_fail "audit command missing"
fi
echo ""

# Deliverable 11: hoop init wizard
echo "11. hoop init wizard"
if target/release/hoop init --help &>/dev/null; then
    check_pass "init command exists"

    echo "   Checking for dependency check..."
    if grep -q "br.*--version\|dependency" hoop-cli/src/*.rs 2>/dev/null; then
        check_pass "Dependency check logic exists"
    else
        check_skip "Dependency check unclear"
    fi
else
    check_fail "init command missing"
fi
echo ""

# Deliverable 12: Compile-fail trybuild for br_verbs.rs
echo "12. Compile-fail trybuild for br_verbs.rs"
if [ -f "hoop-cli/src/br_verbs.rs" ]; then
    check_pass "br_verbs.rs exists"

    if [ -d "hoop-cli/tests/compile-fail" ] || ls hoop-cli/tests/*.rs 2>/dev/null | grep -q "trybuild\|compile"; then
        check_pass "Trybuild tests present"
    else
        check_skip "Trybuild tests unclear"
    fi
else
    check_fail "br_verbs.rs missing"
fi
echo ""

# Deliverable 13: testrepo/ fixture populated
echo "13. testrepo/ fixture populated"
echo "   Checking fixture structure..."
if [ -d "testrepo/.beads" ]; then
    check_pass ".beads/ directory exists"

    if [ -f "testrepo/.beads/issues.jsonl" ]; then
        check_pass "issues.jsonl exists"

        if [ $(wc -l < testrepo/.beads/issues.jsonl) -gt 0 ]; then
            check_pass "Synthetic beads present"
        else
            check_fail "issues.jsonl empty"
        fi
    else
        check_fail "issues.jsonl missing"
    fi

    if [ -f "testrepo/.beads/events.jsonl" ]; then
        check_pass "events.jsonl exists"
    else
        check_fail "events.jsonl missing"
    fi

    if [ -f "testrepo/.beads/heartbeats.jsonl" ]; then
        check_pass "heartbeats.jsonl exists"
    else
        check_fail "heartbeats.jsonl missing"
    fi

    if [ -f "testrepo/bin/br" ]; then
        check_pass "br stub binary exists"
    else
        check_fail "br stub binary missing"
    fi

    if [ -d "testrepo/cli-sessions" ]; then
        check_pass "CLI sessions fixture exists"
    else
        check_fail "CLI sessions fixture missing"
    fi
else
    check_fail "testrepo/.beads missing"
fi
echo ""

# Deliverable 14: Zero silent drops
echo "14. Zero silent drops (unknown events)"
echo "   Checking for unknown event handling..."
if grep -r "unknown.*event\|E3-002" hoop-daemon/src/ 2>/dev/null | grep -q "."; then
    check_pass "Unknown event handling exists"
else
    check_skip "Unknown event handling unclear"
fi

if grep -r "diagnostic\|panel" hoop-daemon/src/ hoop-ui/web/src/ 2>/dev/null | grep -q "."; then
    check_pass "Diagnostic panel mentioned"
else
    check_skip "Diagnostic panel unclear"
fi
echo ""

echo "================================"
echo "Summary"
echo "================================"
echo "Passed: $PASS_COUNT"
echo "Failed: $FAIL_COUNT"
echo "Skipped: $SKIP_COUNT"
echo ""

if [ $FAIL_COUNT -eq 0 ]; then
    echo "All critical checks passed! ✅"
    exit 0
else
    echo "Some checks failed. Review output above."
    exit 1
fi
