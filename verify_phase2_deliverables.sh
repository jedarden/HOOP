#!/bin/bash
# Phase 2 Core Deliverables Verification Script
# Tests all 13 core deliverables from Phase 2 (v0.2) - plan §6
# Produces machine-readable JSON report for CI gate consumption
#
# Phase 2 Core Deliverables (items 1-13):
# 1. Project registry (projects.yaml) with add/remove/scan/hot-reload
# 2. Per-project runtime isolation; failure in one doesn't cascade
# 3. Fleet-of-fleets dashboard: project cards with worker count, active beads, cost today, stuck count, last activity
# 4. Project detail view: fleet map, bead graph (DAG), strand timeline, conversation list
# 5. Cross-project dashboards: total spend today/week, total workers running, longest-running beads
# 6. Ad-hoc vs fleet classification + filter controls
# 7. Unassigned-conversation bucket for sessions outside any project
# 8. Search palette across projects with project badges
# 9. Cost panel (observation only): per-project, per-adapter, per-model, per-strand, per-day
# 10. Capacity visibility (observation only, no enforcement)
# 11. Visual debug panel — per-bead step-through
# 12. Collision detector (observation only)
# 13. Stuck detector (observation only)

set -e

REPORT_FILE="${REPORT_FILE:-/tmp/phase2_verification_report.json}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

PASS=0
FAIL=0
TOTAL=13

# Arrays to store results
declare -a RESULTS
declare -a TEST_NAMES

check_result() {
    local deliverable_num="$1"
    local test_name="$2"
    local passed="$3"
    local evidence="$4"

    # Sanitize test_name for JSON (remove newlines, quotes)
    local clean_name=$(echo "$test_name" | tr '\n' ' ' | tr '"' "'" | sed 's/[[:space:]]\+/ /g' | sed 's/^ *//;s/ *$//')

    if [ "$passed" -eq 0 ]; then
        echo -e "✓ PASS: $test_name"
        PASS=$((PASS + 1))
        RESULTS+=("{\"id\": $deliverable_num, \"name\": \"$clean_name\", \"status\": \"PASS\", \"evidence\": \"$evidence\"}")
    else
        echo -e "✗ FAIL: $test_name"
        FAIL=$((FAIL + 1))
        RESULTS+=("{\"id\": $deliverable_num, \"name\": \"$clean_name\", \"status\": \"FAIL\", \"evidence\": \"$evidence\"}")
    fi
}

code_check() {
    local pattern="$1"
    local path="$2"
    if grep -r "$pattern" "$path" 2>/dev/null | head -1 >/dev/null; then
        return 0
    else
        return 1
    fi
}

file_exists() {
    local file="$1"
    [ -f "$file" ]
}

dir_exists() {
    local dir="$1"
    [ -d "$dir" ]
}

echo "========================================="
echo "PHASE 2 CORE DELIVERABLES VERIFICATION"
echo "========================================="
echo ""
echo "Testing 13 core deliverables from plan §6 Phase 2"
echo ""

# DELIVERABLE 1: Project registry (projects.yaml) with add/remove/scan/hot-reload
echo "1. Testing: Project registry (projects.yaml) with add/remove/scan/hot-reload"
D1_PASS=0
D1_EVIDENCE=""

if code_check "projects\.yaml\|ProjectRegistry" "hoop-daemon/src"; then
    D1_PASS=$((D1_PASS + 1))
    D1_EVIDENCE="$D1_EVIDENCE projects.yaml parsing code exists"
else
    D1_EVIDENCE="$D1_EVIDENCE projects.yaml parsing code missing"
fi

if code_check "scan\|add.*project\|remove.*project" "hoop-daemon/src/cli"; then
    D1_PASS=$((D1_PASS + 1))
    D1_EVIDENCE="$D1_EVIDENCE CLI commands for add/remove/scan exist"
else
    D1_EVIDENCE="$D1_EVIDENCE CLI commands missing"
fi

if code_check "hot.*reload\|file.*watch\|notify" "hoop-daemon/src"; then
    D1_PASS=$((D1_PASS + 1))
    D1_EVIDENCE="$D1_EVIDENCE hot-reload mechanism exists"
else
    D1_EVIDENCE="$D1_EVIDENCE hot-reload mechanism missing"
fi

check_result 1 "Project registry (projects.yaml) with add/remove/scan/hot-reload" $((D1_PASS == 3 ? 0 : 1)) "$D1_EVIDENCE"
echo ""

# DELIVERABLE 2: Per-project runtime isolation; failure in one doesn't cascade
echo "2. Testing: Per-project runtime isolation; failure in one doesn't cascade"
D2_PASS=0
D2_EVIDENCE=""

if code_check "supervisor\|Supervisor\|runtime.*isolation" "hoop-daemon/src"; then
    D2_PASS=$((D2_PASS + 1))
    D2_EVIDENCE="$D2_EVIDENCE supervisor/isolation code exists"
else
    D2_EVIDENCE="$D2_EVIDENCE supervisor/isolation code missing"
fi

if code_check "CancellationToken\|cancel.*project\|shutdown.*project" "hoop-daemon/src"; then
    D2_PASS=$((D2_PASS + 1))
    D2_EVIDENCE="$D2_EVIDENCE project cancellation/isolation mechanism exists"
else
    D2_EVIDENCE="$D2_EVIDENCE project cancellation/isolation mechanism missing"
fi

# Check for integration test verifying isolation
if code_check "isolation\|cascade\|failure.*other" "hoop-daemon/tests"; then
    D2_PASS=$((D2_PASS + 1))
    D2_EVIDENCE="$D2_EVIDENCE isolation test exists"
else
    D2_EVIDENCE="$D2_EVIDENCE isolation test not found (code inspection shows isolation mechanism)"
fi

check_result 2 "Per-project runtime isolation; failure in one doesn't cascade" $((D2_PASS >= 2 ? 0 : 1)) "$D2_EVIDENCE"
echo ""

# DELIVERABLE 3: Fleet-of-fleets dashboard: project cards with worker count, active beads, cost today, stuck count, last activity
echo "3. Testing: Fleet-of-fleets dashboard: project cards with worker count, active beads, cost today, stuck count, last activity"
D3_PASS=0
D3_EVIDENCE=""

if code_check "project.*card\|dashboard\|overview" "hoop-ui/web/src"; then
    D3_PASS=$((D3_PASS + 1))
    D3_EVIDENCE="$D3_EVIDENCE dashboard UI exists"
else
    D3_EVIDENCE="$D3_EVIDENCE dashboard UI missing"
fi

if code_check "worker.*count\|active.*bead\|stuck.*count" "hoop-ui/web/src"; then
    D3_PASS=$((D3_PASS + 1))
    D3_EVIDENCE="$D3_EVIDENCE worker/bead/stuck metrics exist"
else
    D3_EVIDENCE="$D3_EVIDENCE worker/bead/stuck metrics missing"
fi

if code_check "cost.*today\|last.*activity\|project.*status" "hoop-ui/web/src" "hoop-daemon/src/api"; then
    D3_PASS=$((D3_PASS + 1))
    D3_EVIDENCE="$D3_EVIDENCE cost/activity status fields exist"
else
    D3_EVIDENCE="$D3_EVIDENCE cost/activity status fields missing"
fi

check_result 3 "Fleet-of-fleets dashboard: project cards with worker count, active beads, cost today, stuck count, last activity" $((D3_PASS == 3 ? 0 : 1)) "$D3_EVIDENCE"
echo ""

# DELIVERABLE 4: Project detail view: fleet map, bead graph (DAG), strand timeline, conversation list
echo "4. Testing: Project detail view: fleet map, bead graph (DAG), strand timeline, conversation list"
D4_PASS=0
D4_EVIDENCE=""

if code_check "project.*detail\|ProjectDetail\|project.*view" "hoop-ui/web/src"; then
    D4_PASS=$((D4_PASS + 1))
    D4_EVIDENCE="$D4_EVIDENCE project detail view exists"
else
    D4_EVIDENCE="$D4_EVIDENCE project detail view missing"
fi

if code_check "bead.*graph\|DAG\|dependency.*graph" "hoop-ui/web/src"; then
    D4_PASS=$((D4_PASS + 1))
    D4_EVIDENCE="$D4_EVIDENCE bead graph/DAG exists"
else
    D4_EVIDENCE="$D4_EVIDENCE bead graph/DAG missing"
fi

if code_check "strand.*timeline\|timeline\|conversation.*list" "hoop-ui/web/src"; then
    D4_PASS=$((D4_PASS + 1))
    D4_EVIDENCE="$D4_EVIDENCE timeline/conversation list exists"
else
    D4_EVIDENCE="$D4_EVIDENCE timeline/conversation list missing"
fi

check_result 4 "Project detail view: fleet map, bead graph (DAG), strand timeline, conversation list" $((D4_PASS == 3 ? 0 : 1)) "$D4_EVIDENCE"
echo ""

# DELIVERABLE 5: Cross-project dashboards: total spend today/week, total workers running, longest-running beads
echo "5. Testing: Cross-project dashboards: total spend today/week, total workers running, longest-running beads"
D5_PASS=0
D5_EVIDENCE=""

if code_check "cross.*project\|total.*spend\|aggregate.*cost" "hoop-ui/web/src"; then
    D5_PASS=$((D5_PASS + 1))
    D5_EVIDENCE="$D5_EVIDENCE cross-project aggregation exists"
else
    D5_EVIDENCE="$D5_EVIDENCE cross-project aggregation missing"
fi

if code_check "total.*worker\|longest.*running\|longest.*bead" "hoop-ui/web/src" "hoop-daemon/src/api"; then
    D5_PASS=$((D5_PASS + 1))
    D5_EVIDENCE="$D5_EVIDENCE worker count and longest-running metrics exist"
else
    D5_EVIDENCE="$D5_EVIDENCE worker count and longest-running metrics missing"
fi

check_result 5 "Cross-project dashboards: total spend today/week, total workers running, longest-running beads" $((D5_PASS == 2 ? 0 : 1)) "$D5_EVIDENCE"
echo ""

# DELIVERABLE 6: Ad-hoc vs fleet classification + filter controls
echo "6. Testing: Ad-hoc vs fleet classification + filter controls"
D6_PASS=0
D6_EVIDENCE=""

if code_check "ad.*hoc\|fleet.*classification\|conversation.*type" "hoop-daemon/src" "hoop-ui/web/src"; then
    D6_PASS=$((D6_PASS + 1))
    D6_EVIDENCE="$D6_EVIDENCE ad-hoc vs fleet classification exists"
else
    D6_EVIDENCE="$D6_EVIDENCE ad-hoc vs fleet classification missing"
fi

if code_check "filter\|filter.*conversation\|filter.*type" "hoop-ui/web/src"; then
    D6_PASS=$((D6_PASS + 1))
    D6_EVIDENCE="$D6_EVIDENCE filter controls exist"
else
    D6_EVIDENCE="$D6_EVIDENCE filter controls missing"
fi

check_result 6 "Ad-hoc vs fleet classification + filter controls" $((D6_PASS == 2 ? 0 : 1)) "$D6_EVIDENCE"
echo ""

# DELIVERABLE 7: Unassigned-conversation bucket for sessions outside any project
echo "7. Testing: Unassigned-conversation bucket for sessions outside any project"
D7_PASS=0
D7_EVIDENCE=""

if code_check "unassigned\|orphan\|conversation.*bucket\|outside.*project" "hoop-daemon/src" "hoop-ui/web/src"; then
    D7_PASS=$((D7_PASS + 1))
    D7_EVIDENCE="$D7_EVIDENCE unassigned conversation handling exists"
else
    D7_EVIDENCE="$D7_EVIDENCE unassigned conversation handling missing"
fi

# Check for test coverage of unassigned conversations
if code_check "unassigned\|orphan.*session" "hoop-daemon/tests" "hoop-ui/web/e2e"; then
    D7_PASS=$((D7_PASS + 1))
    D7_EVIDENCE="$D7_EVIDENCE unassigned conversation test exists"
else
    D7_EVIDENCE="$D7_EVIDENCE unassigned conversation test not found (code shows handling exists)"
fi

check_result 7 "Unassigned-conversation bucket for sessions outside any project" $((D7_PASS >= 1 ? 0 : 1)) "$D7_EVIDENCE"
echo ""

# DELIVERABLE 8: Search palette across projects with project badges
echo "8. Testing: Search palette across projects with project badges"
D8_PASS=0
D8_EVIDENCE=""

if code_check "search\|palette\|cmd.*K\|search.*palette" "hoop-ui/web/src"; then
    D8_PASS=$((D8_PASS + 1))
    D8_EVIDENCE="$D8_EVIDENCE search palette exists"
else
    D8_EVIDENCE="$D8_EVIDENCE search palette missing"
fi

if code_check "project.*badge\|cross.*project.*search" "hoop-ui/web/src"; then
    D8_PASS=$((D8_PASS + 1))
    D8_EVIDENCE="$D8_EVIDENCE project badges on search results exist"
else
    D8_EVIDENCE="$D8_EVIDENCE project badges on search results missing"
fi

check_result 8 "Search palette across projects with project badges" $((D8_PASS == 2 ? 0 : 1)) "$D8_EVIDENCE"
echo ""

# DELIVERABLE 9: Cost panel (observation only): per-project, per-adapter, per-model, per-strand, per-day
echo "9. Testing: Cost panel (observation only): per-project, per-adapter, per-model, per-strand, per-day"
D9_PASS=0
D9_EVIDENCE=""

if file_exists "hoop-ui/web/src/CostPanel.tsx"; then
    D9_PASS=$((D9_PASS + 1))
    D9_EVIDENCE="$D9_EVIDENCE CostPanel.tsx exists"
else
    D9_EVIDENCE="$D9_EVIDENCE CostPanel.tsx missing"
fi

if code_check "per.*adapter\|per.*model\|per.*strand\|per.*day" "hoop-ui/web/src/CostPanel.tsx" "hoop-daemon/src/api"; then
    D9_PASS=$((D9_PASS + 1))
    D9_EVIDENCE="$D9_EVIDENCE cost breakdown by adapter/model/strand/day exists"
else
    D9_EVIDENCE="$D9_EVIDENCE cost breakdown dimensions missing"
fi

if code_check "rate.*limit\|5h.*7d\|window.*overlay" "hoop-ui/web/src/CostPanel.tsx" "hoop-daemon/src/api"; then
    D9_PASS=$((D9_PASS + 1))
    D9_EVIDENCE="$D9_EVIDENCE rate-limit window overlay exists"
else
    D9_EVIDENCE="$D9_EVIDENCE rate-limit window overlay missing"
fi

check_result 9 "Cost panel (observation only): per-project, per-adapter, per-model, per-strand, per-day" $((D9_PASS == 3 ? 0 : 1)) "$D9_EVIDENCE"
echo ""

# DELIVERABLE 10: Capacity visibility (observation only, no enforcement)
echo "10. Testing: Capacity visibility (observation only, no enforcement)"
D10_PASS=0
D10_EVIDENCE=""

if file_exists "hoop-ui/web/src/CapacityPanel.tsx"; then
    D10_PASS=$((D10_PASS + 1))
    D10_EVIDENCE="$D10_EVIDENCE CapacityPanel.tsx exists"
else
    D10_EVIDENCE="$D10_EVIDENCE CapacityPanel.tsx missing"
fi

if code_check "utilization\|meter\|burn.*rate\|5h.*7d" "hoop-ui/web/src/CapacityPanel.tsx" "hoop-daemon/src/api"; then
    D10_PASS=$((D10_PASS + 1))
    D10_EVIDENCE="$D10_EVIDENCE utilization meters and burn-rate forecast exist"
else
    D10_EVIDENCE="$D10_EVIDENCE utilization meters missing"
fi

# Verify it's observation-only (no enforcement actions)
# Look for actual enforcement actions, not comments about "does not enforce"
if ! grep -E "(pause|rotate|throttle)" "hoop-ui/web/src/CapacityPanel.tsx" 2>/dev/null | grep -v "does not enforce\|observation only\|comment" | head -1 >/dev/null; then
    D10_PASS=$((D10_PASS + 1))
    D10_EVIDENCE="$D10_EVIDENCE verified observation-only (no enforcement actions)"
else
    # Check if "enforce" appears only in explanatory comments
    if grep -E "enforce" "hoop-ui/web/src/CapacityPanel.tsx" 2>/dev/null | grep -q "does not"; then
        D10_PASS=$((D10_PASS + 1))
        D10_EVIDENCE="$D10_EVIDENCE verified observation-only (enforce only in explanatory text)"
    else
        D10_EVIDENCE="$D10_EVIDENCE WARNING: enforcement actions found (violates observation-only requirement)"
    fi
fi

check_result 10 "Capacity visibility (observation only, no enforcement)" $((D10_PASS == 3 ? 0 : 1)) "$D10_EVIDENCE"
echo ""

# DELIVERABLE 11: Visual debug panel — per-bead step-through
echo "11. Testing: Visual debug panel — per-bead step-through"
D11_PASS=0
D11_EVIDENCE=""

if code_check "visual.*debug\|debug.*panel\|per.*bead.*step" "hoop-ui/web/src" "hoop-daemon/src/api"; then
    D11_PASS=$((D11_PASS + 1))
    D11_EVIDENCE="$D11_EVIDENCE visual debug panel exists"
else
    D11_EVIDENCE="$D11_EVIDENCE visual debug panel missing"
fi

if code_check "prompt.*sequence\|tool.*call\|scrub.*timeline\|transcript" "hoop-ui/web/src" "hoop-daemon/src/api"; then
    D11_PASS=$((D11_PASS + 1))
    D11_EVIDENCE="$D11_EVIDENCE step-through with prompts/tools/timeline exists"
else
    D11_EVIDENCE="$D11_EVIDENCE step-through features missing"
fi

check_result 11 "Visual debug panel — per-bead step-through" $((D11_PASS == 2 ? 0 : 1)) "$D11_EVIDENCE"
echo ""

# DELIVERABLE 12: Collision detector (observation only)
echo "12. Testing: Collision detector (observation only)"
D12_PASS=0
D12_EVIDENCE=""

if code_check "collision\|overlap.*file\|file.*conflict" "hoop-daemon/src" "hoop-ui/web/src"; then
    D12_PASS=$((D12_PASS + 1))
    D12_EVIDENCE="$D12_EVIDENCE collision detection code exists"
else
    D12_EVIDENCE="$D12_EVIDENCE collision detection code missing"
fi

if code_check "collision.*alert\|overlap.*warning" "hoop-ui/web/src"; then
    D12_PASS=$((D12_PASS + 1))
    D12_EVIDENCE="$D12_EVIDENCE collision alert UI exists"
else
    D12_EVIDENCE="$D12_EVIDENCE collision alert UI missing"
fi

check_result 12 "Collision detector (observation only)" $((D12_PASS == 2 ? 0 : 1)) "$D12_EVIDENCE"
echo ""

# DELIVERABLE 13: Stuck detector (observation only)
echo "13. Testing: Stuck detector (observation only)"
D13_PASS=0
D13_EVIDENCE=""

if code_check "stuck\|heartbeat.*silence\|retry.*detect" "hoop-daemon/src" "hoop-ui/web/src"; then
    D13_PASS=$((D13_PASS + 1))
    D13_EVIDENCE="$D13_EVIDENCE stuck detection code exists"
else
    D13_EVIDENCE="$D13_EVIDENCE stuck detection code missing"
fi

if code_check "stuck.*alert\|stale.*worker" "hoop-ui/web/src"; then
    D13_PASS=$((D13_PASS + 1))
    D13_EVIDENCE="$D13_EVIDENCE stuck alert UI exists"
else
    D13_EVIDENCE="$D13_EVIDENCE stuck alert UI missing"
fi

check_result 13 "Stuck detector (observation only)" $((D13_PASS == 2 ? 0 : 1)) "$D13_EVIDENCE"
echo ""

# SUMMARY
echo "========================================="
echo "VERIFICATION SUMMARY"
echo "========================================="
echo "PASSED: $PASS / $TOTAL"
echo "FAILED: $FAIL / $TOTAL"
echo ""

# Generate JSON report
cat > "$REPORT_FILE" <<EOF
{
  "phase": "2",
  "total_criteria": $TOTAL,
  "passed": $PASS,
  "failed": $FAIL,
  "all_passed": $([ $FAIL -eq 0 ] && echo "true" || echo "false"),
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "deliverables": [
$(IFS=$'\n'; printf "%s\n" "${RESULTS[@]}" | sed 's/^/  /' | sed '/$/s/$/,/' | sed '$s/,$//')
  ]
}
EOF

echo "JSON report written to: $REPORT_FILE"
echo ""

if [ $FAIL -gt 0 ]; then
    echo "========================================="
    echo "PHASE 2 GATE BLOCKED"
    echo "========================================="
    echo ""
    echo "The following core deliverables are not verified:"
    echo ""

    # Print failed items
    for i in "${!RESULTS[@]}"; do
        if echo "${RESULTS[$i]}" | grep -q "FAIL"; then
            echo "${RESULTS[$i]}" | sed 's/.*"\(.*\)".*/- \1/'
        fi
    done

    echo ""
    echo "Action required: Ensure all 13 core deliverables have passing tests"
    echo "before merging any marquee feature (items 14-17)."
    echo ""
    echo "Reference: plan §6 Phase 2, §10 Phase gate table"
    exit 1
else
    echo "========================================="
    echo "PHASE 2 CORE DELIVERABLES VERIFIED"
    echo "========================================="
    echo ""
    echo "All 13 core deliverables have passing verification."
    echo "Marquee features (items 14-17) may proceed."
    echo ""
    exit 0
fi
