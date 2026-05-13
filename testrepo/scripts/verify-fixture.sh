#!/usr/bin/env bash
# Quick verification script for testrepo fixture

set -euo pipefail

TESTREPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$TESTREPO_ROOT"

echo "=== testrepo fixture verification ==="
echo "Root: $TESTREPO_ROOT"
echo

# Track pass/fail
PASS=0
FAIL=0

check() {
    local name="$1"
    local test_cmd="$2"

    if eval "$test_cmd" > /dev/null 2>&1; then
        echo "✓ $name"
        PASS=$((PASS + 1))
        return 0
    else
        echo "✗ $name"
        FAIL=$((FAIL + 1))
        return 1
    fi
}

echo "Structure checks:"
check "testrepo/ exists" "[ -d '$TESTREPO_ROOT' ]"
check ".beads/ exists" "[ -d '.beads' ]"
check "bin/br exists and executable" "[ -x 'bin/br' ]"
check "cli-sessions/ exists" "[ -d 'cli-sessions' ]"
check "scripts/ exists" "[ -d 'scripts' ]"

echo
echo "Data file checks:"
check ".beads/issues.jsonl exists" "[ -f '.beads/issues.jsonl' ]"
check ".beads/events.jsonl exists" "[ -f '.beads/events.jsonl' ]"
check ".beads/heartbeats.jsonl exists" "[ -f '.beads/heartbeats.jsonl' ]"
check ".beads/config.yaml exists" "[ -f '.beads/config.yaml' ]"
check ".beads/beads.db exists" "[ -f '.beads/beads.db' ]"

echo
echo "CLI session checks:"
check "Claude session exists" "[ -f 'cli-sessions/claude/session.jsonl' ]"
check "Codex session exists" "[ -f 'cli-sessions/codex/session.jsonl' ]"
check "Gemini session exists" "[ -f 'cli-sessions/gemini/session.jsonl' ]"
check "OpenCode session exists" "[ -f 'cli-sessions/opencode/session.jsonl' ]"
check "Aider session exists" "[ -f 'cli-sessions/aider/session.jsonl' ]"

echo
echo "Attachment checks:"
check "Screenshot attachment exists" "[ -f '.beads/attachments/tr-open-001/screenshot.png' ]"
check "Audio attachment exists" "[ -f '.beads/attachments/tr-open-001/audio_message.wav' ]"
check "Video attachment exists" "[ -f '.beads/attachments/tr-open-001/demo_video.mp4' ]"

echo
echo "Content checks:"
check "issues.jsonl has entries" "[ \$(wc -l < .beads/issues.jsonl) -gt 0 ]"
check "events.jsonl has entries" "[ \$(wc -l < .beads/events.jsonl) -gt 0 ]"
check "heartbeats.jsonl has entries" "[ \$(wc -l < .beads/heartbeats.jsonl) -gt 0 ]"
check "Claude session has entries" "[ \$(wc -l < cli-sessions/claude/session.jsonl) -gt 0 ]"

echo
echo "br stub functionality check:"
if OUTPUT=$(bash bin/br list 2>&1); then
    if echo "$OUTPUT" | jq -e '. | type == "array"' > /dev/null 2>&1; then
        echo "✓ br stub returns valid JSON"
        PASS=$((PASS + 1))
    else
        echo "✗ br stub does not return valid JSON"
        FAIL=$((FAIL + 1))
    fi
else
    echo "✗ br stub failed to execute"
    FAIL=$((FAIL + 1))
fi

echo
echo "Size check:"
SIZE_BYTES=$(du -sb "$TESTREPO_ROOT" | cut -f1)
MAX_BYTES=52428800  # 50MB
if [ "$SIZE_BYTES" -lt "$MAX_BYTES" ]; then
    SIZE_HR=$(du -sh "$TESTREPO_ROOT" | cut -f1)
    echo "✓ Size bounded: $SIZE_HR (${SIZE_BYTES} bytes < 50MB)"
    PASS=$((PASS + 1))
else
    echo "✗ Size exceeds 50MB limit: ${SIZE_BYTES} bytes"
    FAIL=$((FAIL + 1))
fi

echo
echo "Regeneration scripts check:"
check "regenerate-fixtures.sh exists and executable" "[ -x 'scripts/regenerate-fixtures.sh' ]"
check "regenerate-cli-sessions.py exists" "[ -f 'scripts/regenerate-cli-sessions.py' ]"
check "regenerate-attachments.py exists" "[ -f 'scripts/regenerate-attachments.py' ]"

echo
echo "=== Summary ==="
echo "Passed: $PASS"
echo "Failed: $FAIL"

if [ $FAIL -eq 0 ]; then
    echo "✓ All checks passed!"
    exit 0
else
    echo "✗ Some checks failed"
    exit 1
fi
