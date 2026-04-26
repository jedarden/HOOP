#!/usr/bin/env bash
# Fixture regeneration script for testrepo
# This script helps regenerate synthetic test fixtures for the testrepo workspace

set -euo pipefail

TESTREPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$TESTREPO_ROOT"

echo "=== testrepo fixture regeneration ==="
echo "Root: $TESTREPO_ROOT"
echo

# Function to create a minimal PNG
create_minimal_png() {
    local path="$1"
    # 1x1 red PNG (minimal valid PNG)
    printf '\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\x0cIDATx\x9cc\x00\x01\x00\x00\x05\x00\x01\x0d\n-\xb4\x00\x00\x00\x00IEND\xaeB`\x82' > "$path"
}

# Function to create a minimal WAV
create_minimal_wav() {
    local path="$1"
    # Minimal WAV header + 1 sample of silence
    printf 'RIFF\x24\x00\x00\x00WAVEfmt \x10\x00\x00\x00\x01\x00\x01\x00\x44\xAC\x00\x00\x88\x58\x01\x00\x02\x00\x10\x00data\x00\x00\x00\x00' > "$path"
}

# Function to create a placeholder video file
create_placeholder_video() {
    local path="$1"
    echo "# Placeholder MP4 video for testing" > "$path"
}

# Function to create synthetic bead JSON
create_bead() {
    local id="$1"
    local title="$2"
    local status="$3"
    local priority="${4:-2}"
    local issue_type="${5:-bug}"
    local assignee="${6:-null}"

    cat <<EOF
{"id":"$id","content_hash":null,"title":"$title","description":"Synthetic test bead","design":"","acceptance_criteria":"","notes":"","status":"$status","priority":$priority,"issue_type":"$issue_type","assignee":$assignee,"owner":"","estimated_minutes":null,"created_at":"$(date -u +"%Y-%m-%dT%H:%M:%SZ")","created_by":"system","updated_at":"$(date -u +"%Y-%m-%dT%H:%M:%SZ")","closed_at":null,"close_reason":"","closed_by_session":"","due_at":null,"defer_until":null,"external_ref":null,"source_system":"","source_repo":".","deleted_at":null,"deleted_by":"","delete_reason":"","original_type":"","compaction_level":0,"compacted_at":null,"compacted_at_commit":null,"original_size":null,"sender":"","ephemeral":0,"pinned":0,"is_template":0}
EOF
}

# Function to create NEEDLE event
create_needle_event() {
    local event_type="$1"
    local worker="${2:-alpha}"
    local bead="${3:-bd-test001}"

    case "$event_type" in
        claim)
            echo "{\"event\":\"claim\",\"ts\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\",\"worker\":\"$worker\",\"bead\":\"$bead\",\"strand\":\"pluck\"}"
            ;;
        dispatch)
            echo "{\"event\":\"dispatch\",\"ts\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\",\"worker\":\"$worker\",\"bead\":\"$bead\",\"adapter\":\"claude\",\"model\":\"claude-opus-4-6\"}"
            ;;
        complete)
            echo "{\"event\":\"complete\",\"ts\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\",\"worker\":\"$worker\",\"bead\":\"$bead\",\"outcome\":\"success\",\"duration_ms\":287104,\"exit_code\":0}"
            ;;
        fail)
            echo "{\"event\":\"fail\",\"ts\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\",\"worker\":\"$worker\",\"bead\":\"$bead\",\"error\":\"context limit exceeded\",\"duration_ms\":90000}"
            ;;
        release)
            echo "{\"event\":\"release\",\"ts\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\",\"worker\":\"$worker\",\"bead\":\"$bead\"}"
            ;;
        timeout)
            echo "{\"event\":\"timeout\",\"ts\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\",\"worker\":\"$worker\",\"bead\":\"$bead\"}"
            ;;
        crash)
            echo "{\"event\":\"crash\",\"ts\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\",\"worker\":\"$worker\",\"bead\":\"$bead\",\"exit_code\":137}"
            ;;
        close)
            echo "{\"event\":\"close\",\"ts\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\",\"worker\":\"$worker\",\"bead\":\"$bead\"}"
            ;;
        update)
            echo "{\"event\":\"update\",\"ts\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\",\"worker\":\"$worker\",\"bead\":\"$bead\",\"changes\":[\"status\"]}"
            ;;
    esac
}

# Function to create heartbeat
create_heartbeat() {
    local state="$1"
    local worker="${2:-alpha}"
    local bead="${3:-bd-test001}"
    local pid="${4:-12345}"

    case "$state" in
        idle)
            echo "{\"ts\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\",\"worker\":\"$worker\",\"state\":\"idle\"}"
            ;;
        executing)
            echo "{\"ts\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\",\"worker\":\"$worker\",\"state\":\"executing\",\"bead\":\"$bead\",\"pid\":$pid,\"adapter\":\"claude\"}"
            ;;
        knot)
            echo "{\"ts\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\",\"worker\":\"$worker\",\"state\":\"knot\",\"reason\":\"adapter unavailable\"}"
            ;;
    esac
}

# Main regeneration tasks
main() {
    echo "Regenerating testrepo fixtures..."

    # 1. Regenerate asset files
    echo
    echo "[1/6] Regenerating asset files..."
    create_minimal_png "assets/images/example.png"
    create_minimal_wav "assets/audio/example.wav"
    create_placeholder_video "assets/video/example.mp4"
    echo "  - Created assets/images/example.png"
    echo "  - Created assets/audio/example.wav"
    echo "  - Created assets/video/example.mp4"

    # 2. Regenerate .beads/ issues.jsonl with synthetic beads in various states
    echo
    echo "[2/6] Regenerating .beads/issues.jsonl..."
    cat > .beads/issues.jsonl <<EOF
$(create_bead "tr-open-001" "Fix memory leak in parser" "open" 3 "bug")
$(create_bead "tr-open-002" "Add streaming support" "open" 2 "feature")
$(create_bead "tr-open-003" "Update documentation" "open" 1 "task")
$(create_bead "tr-claimed-001" "Implement retry logic" "in_progress" 3 "feature" '"alpha"')
$(create_bead "tr-claimed-002" "Refactor database layer" "in_progress" 2 "refactor" '"bravo"')
$(create_bead "tr-claimed-003" "Add telemetry hooks" "in_progress" 2 "feature" '"charlie"')
$(create_bead "tr-closed-001" "Initial scaffold" "closed" 3 "task" "null" "2026-04-15T12:00:00Z" "completed" "alpha-001")
$(create_bead "tr-closed-002" "Add test suite" "closed" 3 "task" "null" "2026-04-16T16:30:00Z" "completed" "alpha-002")
$(create_bead "tr-closed-003" "Implement parser" "closed" 3 "feature" "null" "2026-04-18T15:00:00Z" "completed" "alpha-003")
$(create_bead "tr-failed-001" "Complex migration script" "open" 2 "task" '"bravo"')
$(create_bead "tr-failed-002" "Deep code analysis" "open" 2 "investigation" '"charlie"')
$(create_bead "tr-failed-003" "Multi-file refactor" "open" 2 "refactor" '"delta"')
EOF
    echo "  - Created 12 synthetic beads (open, claimed, closed, failed)"

    # 3. Regenerate .beads/events.jsonl with all event types
    echo
    echo "[3/6] Regenerating .beads/events.jsonl..."
    cat > .beads/events.jsonl <<EOF
$(create_needle_event "claim" "alpha" "bd-abc123")
$(create_needle_event "dispatch" "alpha" "bd-abc123")
$(create_needle_event "complete" "alpha" "bd-abc123")
$(create_needle_event "fail" "bravo" "bd-def456")
$(create_needle_event "release" "alpha" "bd-abc123")
$(create_needle_event "timeout" "charlie" "bd-ghi789")
$(create_needle_event "crash" "delta" "bd-jkl012")
$(create_needle_event "close" "alpha" "bd-abc123")
$(create_needle_event "update" "alpha" "bd-abc123")
EOF
    echo "  - Created events covering all event types"

    # 4. Regenerate .beads/heartbeats.jsonl with all states
    echo
    echo "[4/6] Regenerating .beads/heartbeats.jsonl..."
    cat > .beads/heartbeats.jsonl <<EOF
$(create_heartbeat "idle" "alpha")
$(create_heartbeat "executing" "alpha" "bd-abc123" "12345")
$(create_heartbeat "knot" "alpha")
EOF
    echo "  - Created heartbeats covering all states"

    # 5. Regenerate br stub fixtures
    echo
    echo "[5/6] Regenerating br stub fixtures..."
    # Ensure bin/br is executable
    chmod +x bin/br
    echo "  - bin/br stub is ready"

    # 6. Report size
    echo
    echo "[6/6] Checking testrepo size..."
    local size=$(du -sh "$TESTREPO_ROOT" | cut -f1)
    local size_bytes=$(du -sb "$TESTREPO_ROOT" | cut -f1)
    local max_bytes=52428800  # 50MB
    echo "  - Current size: $size (${size_bytes} bytes)"
    if [ "$size_bytes" -lt "$max_bytes" ]; then
        echo "  ✓ Size is bounded (<50MB)"
    else
        echo "  ✗ Size exceeds 50MB limit!"
        return 1
    fi

    echo
    echo "=== Fixture regeneration complete ==="
    echo
    echo "Next steps:"
    echo "  1. Run integration tests: cargo test --test golden_transcripts_regression"
    echo "  2. Run event roundtrip tests: cargo test --test needle_events_roundtrip"
    echo "  3. Commit changes: git add testrepo/ && git commit -m 'feat(testrepo): regenerate fixtures'"
}

# Run main function
main "$@"
