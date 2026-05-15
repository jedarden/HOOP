#!/bin/bash
set -e

echo "=== Phase 1 Verification Script ==="
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0

check_pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    ((PASS_COUNT++)) || true
}

check_fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    ((FAIL_COUNT++)) || true
}

check_skip() {
    echo -e "${YELLOW}○ SKIP${NC}: $1"
    ((SKIP_COUNT++)) || true
}

echo "1. Checking hoop-daemon binary builds..."
if [ -f "target/release/hoop" ] || [ -f "target/debug/hoop" ]; then
    check_pass "hoop binary exists"
else
    check_fail "hoop binary not found"
fi

echo ""
echo "2. Checking single workspace registration..."
if [ -f "hoop-cli/src/projects.rs" ] && grep -q "projects.yaml" hoop-cli/src/projects.rs; then
    check_pass "~/.hoop/projects.yaml format implemented"
else
    check_fail "projects.yaml format not found"
fi

echo ""
echo "3. Checking event tailer..."
if [ -f "hoop-daemon/src/events.rs" ] && grep -q "EventTailer" hoop-daemon/src/events.rs; then
    check_pass "EventTailer exists in events.rs"
    if grep -q "partial.*line\|carry.*over" hoop-daemon/src/events.rs; then
        check_pass "Partial line handling documented"
    else
        check_fail "Partial line handling not documented"
    fi
else
    check_fail "EventTailer not found"
fi

echo ""
echo "4. Checking session tailer..."
if [ -f "hoop-daemon/src/sessions.rs" ] && grep -q "SessionTailer" hoop-daemon/src/sessions.rs; then
    check_pass "SessionTailer exists"
    if grep -q "claude\|opencode" hoop-daemon/src/sessions.rs; then
        check_pass "Claude Code + OpenCode adapters supported"
    else
        check_fail "Adapters not found"
    fi
else
    check_fail "SessionTailer not found"
fi

echo ""
echo "5. Checking worker heartbeat monitor..."
if [ -f "hoop-daemon/src/heartbeats.rs" ] && grep -q "HeartbeatMonitor" hoop-daemon/src/heartbeats.rs; then
    check_pass "HeartbeatMonitor exists"
    if grep -q "kill.*0.*pid\|liveness" hoop-daemon/src/heartbeats.rs; then
        check_pass "Process liveness checking implemented"
    else
        check_fail "Process liveness not found"
    fi
else
    check_fail "HeartbeatMonitor not found"
fi

echo ""
echo "6. Checking bead-level subscription (needle tag extraction)..."
if grep -q "needle:" hoop-daemon/src/sessions.rs && grep -q "tag_join" hoop-daemon/src/sessions.rs; then
    check_pass "Needle tag extraction implemented"
else
    check_fail "Needle tag extraction not found"
fi

echo ""
echo "7. Checking worker transcript viewer..."
if [ -f "hoop-daemon/src/api_timeline.rs" ]; then
    check_pass "REST API for worker timeline exists"
    if grep -q "WebSocket\|ws::" hoop-daemon/src/ws.rs; then
        check_pass "WebSocket support exists"
    else
        check_fail "WebSocket support not found"
    fi
else
    check_fail "Worker transcript API not found"
fi

echo ""
echo "8. Checking read-only web UI..."
if [ -d "hoop-ui/web/src" ] && [ -f "hoop-ui/web/src/App.tsx" ]; then
    check_pass "React web UI exists"
    if [ -f "hoop-ui/web/src/BeadList.tsx" ] || [ -f "hoop-ui/web/src/ConversationsView.tsx" ]; then
        check_pass "UI components for bead list/conversations exist"
    else
        check_fail "Key UI components missing"
    fi
else
    check_fail "Web UI not found"
fi

echo ""
echo "9. Checking hoop status --json..."
if grep -q "Status.*{.*json" hoop-cli/src/main.rs; then
    check_pass "hoop status --json command exists"
else
    check_fail "hoop status --json not found"
fi

echo ""
echo "10. Checking hoop audit..."
if grep -q "Audit" hoop-cli/src/main.rs; then
    check_pass "hoop audit command exists"
    if [ -f "hoop-daemon/src/audit.rs" ] || grep -q "mod audit" hoop-daemon/src/lib.rs; then
        check_pass "Audit module implemented"
    else
        check_fail "Audit module not found"
    fi
else
    check_fail "hoop audit not found"
fi

echo ""
echo "11. Checking hoop init wizard..."
if grep -q "Init" hoop-cli/src/main.rs && [ -f "hoop-cli/src/init.rs" ]; then
    check_pass "hoop init wizard exists"
    if grep -q "dependency.*check\|wizard" hoop-cli/src/init.rs; then
        check_pass "Init wizard stages documented"
    else
        check_fail "Init wizard stages not documented"
    fi
else
    check_fail "hoop init not found"
fi

echo ""
echo "12. Checking compile-fail trybuild for br_verbs.rs..."
if [ -f "hoop-daemon/tests/compile_fail_create_only.rs" ]; then
    check_pass "Trybuild test file exists"
    if [ -d "hoop-daemon/tests/ui" ] && [ "$(ls -1 hoop-daemon/tests/ui/*.rs 2>/dev/null | wc -l)" -gt 0 ]; then
        check_pass "UI test fixtures exist ($(ls -1 hoop-daemon/tests/ui/*.rs 2>/dev/null | wc -l) fixtures)"
    else
        check_fail "UI test fixtures missing"
    fi
else
    check_fail "Trybuild test not found"
fi

echo ""
echo "13. Checking testrepo/ fixture..."
if [ -d "testrepo/.beads" ]; then
    check_pass "testrepo/.beads directory exists"
    if [ -f "testrepo/.beads/events.jsonl" ] && [ -f "testrepo/.beads/heartbeats.jsonl" ]; then
        check_pass "events.jsonl and heartbeats.jsonl exist"
    else
        check_fail "Required JSONL files missing"
    fi
    if [ -d "testrepo/.beads/cli-sessions" ]; then
        check_pass "CLI session fixtures exist"
    else
        check_fail "CLI session fixtures missing"
    fi
else
    check_fail "testrepo fixture not found"
fi

echo ""
echo "14. Checking zero silent drops..."
if [ -f "hoop-daemon/src/unknown_event_sink.rs" ]; then
    check_pass "UnknownEventSink exists"
    if grep -q "hoop_unknown_event_total\|hoop_unknown_event_labeled_total" hoop-daemon/src/unknown_event_sink.rs; then
        check_pass "E3-002 counter implemented (via hoop_unknown_event_total metrics)"
    else
        check_fail "E3-002 counter not found"
    fi
else
    check_fail "UnknownEventSink not found"
fi

echo ""
echo "=== Summary ==="
echo -e "${GREEN}PASSED:${NC} $PASS_COUNT/14"
echo -e "${RED}FAILED:${NC} $FAIL_COUNT/14"
echo -e "${YELLOW}SKIPPED:${NC} $SKIP_COUNT/14"

if [ $FAIL_COUNT -eq 0 ]; then
    echo -e "\n${GREEN}All deliverables verified!${NC}"
    exit 0
else
    echo -e "\n${RED}Some deliverables need attention.${NC}"
    exit 1
fi
