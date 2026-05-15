#!/bin/bash
# Phase 1 Deliverables Quick Check

echo "Phase 1 Deliverables Status Check"
echo "=================================="
echo ""

# Array to store results
declare -a results

check() {
    local num=$1
    local name=$2
    local cmd=$3

    printf "%2d. %s..." "$num" "$name"
    if eval "$cmd" >/dev/null 2>&1; then
        echo " PASS"
        results[$num]="PASS"
    else
        echo " FAIL"
        results[$num]="FAIL"
    fi
}

check 1 "hoop-daemon binary builds" "[ -f target/release/hoop ]"
check 2 "projects subcommand" "target/release/hoop projects --help"
check 3 "event tailer code" "[ -f hoop-daemon/src/events.rs ]"
check 4 "session tailer code" "[ -f hoop-daemon/src/sessions.rs ]"
check 5 "heartbeat monitor" "[ -f hoop-daemon/src/heartbeats.rs ]"
check 6 "testrepo events.jsonl" "[ -f testrepo/.beads/events.jsonl ]"
check 7 "testrepo heartbeats.jsonl" "[ -f testrepo/.beads/heartbeats.jsonl ]"
check 8 "testrepo issues.jsonl" "[ -f testrepo/.beads/issues.jsonl ]"
check 9 "testrepo CLI sessions" "[ -d testrepo/cli-sessions ]"
check 10 "status command" "target/release/hoop status --help"
check 11 "audit command" "target/release/hoop audit --help"
check 12 "init command" "target/release/hoop init --help"
check 13 "web UI source" "[ -d hoop-ui/web/src ]"
check 14 "WebSocket support" "grep -q websocket hoop-daemon/src/lib.rs"

echo ""
echo "Results Summary:"
echo "================"

pass=0
fail=0
for i in {1..14}; do
    if [ "${results[$i]}" = "PASS" ]; then
        ((pass++))
    else
        ((fail++))
        printf "%2d. FAIL\n" "$i"
    fi
done

echo "Passed: $pass/14"
echo "Failed: $fail/14"

if [ $fail -eq 0 ]; then
    echo ""
    echo "All deliverables verified!"
    exit 0
else
    echo ""
    echo "Some deliverables need attention."
    exit 1
fi
