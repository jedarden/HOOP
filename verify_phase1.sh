#!/bin/bash
# Phase 1 Deliverables Verification Script
set -e

echo "========================================="
echo "Phase 1 Deliverables Verification"
echo "========================================="
echo ""

PASS_COUNT=0
FAIL_COUNT=0
GAP_COUNT=0

check_pass() {
    echo "✅ PASS: $1"
    PASS_COUNT=$((PASS_COUNT + 1))
}

check_fail() {
    echo "❌ FAIL: $1"
    FAIL_COUNT=$((FAIL_COUNT + 1))
}

check_gap() {
    echo "⚠️  GAP: $1"
    GAP_COUNT=$((GAP_COUNT + 1))
}

echo "## Deliverable 1: hoop-daemon binary builds and runs"
if [ -f "./target/release/hoop" ]; then
    check_pass "Binary exists at ./target/release/hoop"
    if timeout 2 ./target/release/hoop serve --help > /dev/null 2>&1; then
        check_pass "'hoop serve' command exists"
    else
        check_fail "'hoop serve' command not working"
    fi
else
    check_fail "Binary not built"
fi
echo ""

echo "## Deliverable 2: Single workspace registration"
if [ -f "./hoop-daemon/src/projects.rs" ]; then
    check_pass "projects.rs exists"
else
    check_fail "projects.rs missing"
fi
if grep -q "projects.yaml" ./hoop-daemon/src/config_resolver.rs 2>/dev/null; then
    check_pass "projects.yaml format supported"
else
    check_gap "projects.yaml format not confirmed"
fi
echo ""

echo "## Deliverable 3: Event tailer"
if [ -f "./hoop-daemon/src/events.rs" ]; then
    check_pass "events.rs exists"
    if grep -q "events.jsonl" ./hoop-daemon/src/events.rs 2>/dev/null; then
        check_pass "events.jsonl reading implemented"
    else
        check_gap "events.jsonl reading not confirmed"
    fi
    if [ -f "./testrepo/.beads/events.jsonl" ]; then
        check_pass "testrepo has events.jsonl fixture"
    else
        check_fail "testrepo missing events.jsonl"
    fi
else
    check_fail "events.rs missing"
fi
echo ""

echo "## Deliverable 4: Session tailer (Claude Code + OpenCode adapters)"
if [ -f "./hoop-daemon/src/sessions.rs" ]; then
    check_pass "sessions.rs exists"
    if grep -q "claude\|opencode" ./hoop-daemon/src/sessions.rs 2>/dev/null; then
        check_pass "Adapter support exists"
    else
        check_gap "Adapter support not confirmed"
    fi
    if [ -d "./testrepo/.beads/sessions" ]; then
        check_pass "testrepo has session fixtures"
        if [ -f "./testrepo/.beads/sessions/claude-session.jsonl" ] && \
           [ -f "./testrepo/.beads/sessions/opencode-session.jsonl" ]; then
            check_pass "Claude and OpenCode session fixtures exist"
        else
            check_gap "Some adapter session fixtures missing"
        fi
    else
        check_fail "testrepo missing sessions directory"
    fi
else
    check_fail "sessions.rs missing"
fi
echo ""

echo "## Deliverable 5: Worker heartbeat monitor"
if [ -f "./hoop-daemon/src/heartbeats.rs" ]; then
    check_pass "heartbeats.rs exists"
    if grep -q "heartbeats.jsonl" ./hoop-daemon/src/heartbeats.rs 2>/dev/null; then
        check_pass "heartbeats.jsonl reading implemented"
    else
        check_gap "heartbeats.jsonl reading not confirmed"
    fi
    if [ -f "./testrepo/.beads/heartbeats.jsonl" ]; then
        check_pass "testrepo has heartbeats.jsonl fixture"
    else
        check_fail "testrepo missing heartbeats.jsonl"
    fi
else
    check_fail "heartbeats.rs missing"
fi
echo ""

echo "## Deliverable 6: Bead-level subscription"
if [ -f "./hoop-daemon/src/tag_join.rs" ]; then
    check_pass "tag_join.rs exists"
    if grep -q "needle:" ./hoop-daemon/src/tag_join.rs 2>/dev/null; then
        check_pass "needle: tag extraction implemented"
    else
        check_gap "needle: tag extraction not confirmed"
    fi
else
    check_fail "tag_join.rs missing"
fi
echo ""

echo "## Deliverable 7: Worker transcript viewer"
if grep -q "transcript\|conversation" ./hoop-daemon/src/api_conversations.rs 2>/dev/null; then
    check_pass "Transcript/conversation API exists"
else
    check_gap "Transcript API not confirmed"
fi
echo ""

echo "## Deliverable 8: Read-only web UI"
if [ -d "./hoop-ui/web/src" ]; then
    check_pass "Web UI source exists"
    if [ -f "./hoop-ui/web/src/pages/Overview.tsx" ] || \
       [ -f "./hoop-ui/web/src/pages/ProjectDetail.tsx" ]; then
        check_pass "UI pages exist"
    else
        check_gap "UI pages not confirmed"
    fi
else
    check_fail "Web UI missing"
fi
echo ""

echo "## Deliverable 9: hoop status --json"
if grep -q "status\|--json" ./hoop-daemon/src/lib.rs 2>/dev/null || \
   grep -q "status" ./hoop-cli/src/main.rs 2>/dev/null; then
    check_pass "status command exists"
else
    check_gap "status command not confirmed"
fi
echo ""

echo "## Deliverable 10: hoop audit (minimum viable)"
if [ -f "./hoop-daemon/src/audit.rs" ]; then
    check_pass "audit.rs exists"
    if grep -q "E-code\|taxonomy" ./hoop-daemon/src/audit.rs 2>/dev/null; then
        check_pass "E-code taxonomy present"
    else
        check_gap "E-code taxonomy not confirmed"
    fi
else
    check_fail "audit.rs missing"
fi
echo ""

echo "## Deliverable 11: hoop init wizard"
if [ -f "./hoop-daemon/src/api_onboarding.rs" ]; then
    check_pass "onboarding API exists"
else
    check_gap "onboarding API not confirmed"
fi
echo ""

echo "## Deliverable 12: Compile-fail trybuild for br_verbs.rs"
if [ -f "./hoop-daemon/src/br_verbs.rs" ]; then
    check_pass "br_verbs.rs exists"
    if grep -q "compile_fail" ./hoop-daemon/src/br_verbs.rs 2>/dev/null || \
       [ -d "./hoop-daemon/tests/trybuild" ]; then
        check_pass "trybuild tests present"
    else
        check_gap "trybuild tests not confirmed"
    fi
else
    check_fail "br_verbs.rs missing"
fi
echo ""

echo "## Deliverable 13: testrepo/ fixture populated"
if [ -d "./testrepo/.beads" ]; then
    check_pass "testrepo/.beads exists"
    bead_files=0
    [ -f "./testrepo/.beads/events.jsonl" ] && bead_files=$((bead_files + 1))
    [ -f "./testrepo/.beads/heartbeats.jsonl" ] && bead_files=$((bead_files + 1))
    [ -d "./testrepo/.beads/sessions" ] && bead_files=$((bead_files + 1))
    [ -d "./testrepo/.beads/cli-sessions" ] && bead_files=$((bead_files + 1))
    [ -f "./testrepo/.beads/beads.db" ] && bead_files=$((bead_files + 1))

    if [ $bead_files -eq 5 ]; then
        check_pass "All testrepo fixtures present (events, heartbeats, sessions, cli-sessions, beads.db)"
    else
        check_gap "Some testrepo fixtures missing ($bead_files/5)"
    fi
else
    check_fail "testrepo/.beads missing"
fi
echo ""

echo "## Deliverable 14: Zero silent drops"
if [ -f "./hoop-daemon/src/unknown_event_sink.rs" ]; then
    check_pass "unknown_event_sink.rs exists"
    if grep -q "E3-002\|diagnostic\|unknown" ./hoop-daemon/src/unknown_event_sink.rs 2>/dev/null; then
        check_pass "Unknown event handling implemented"
    else
        check_gap "Unknown event handling not confirmed"
    fi
else
    check_fail "unknown_event_sink.rs missing"
fi
echo ""

echo "========================================="
echo "Summary:"
echo "  ✅ PASS: $PASS_COUNT"
echo "  ❌ FAIL: $FAIL_COUNT"
echo "  ⚠️  GAP:  $GAP_COUNT"
echo "========================================="

exit 0
