#!/data/data/com.termux/files/usr/bin/bash
# termux-hoop-listener.sh — phone-side push-to-talk listener for HOOP
#
# Runs inside Termux on the Pixel 6. Listens for HOOP_DICTATE_START /
# HOOP_DICTATE_STOP broadcast intents via termux-broadcast-receiver,
# records audio, and uploads to HOOP via Tailscale.
#
# INSTALL
#   See: hoop-adb setup  (run on the coding host)
#
# REQUIREMENTS (inside Termux)
#   pkg install termux-api sox curl
#   Grant microphone permission to Termux (Android Settings → Apps → Termux)
#
# RUN
#   nohup ~/hoop-listener.sh > ~/.hoop-listener.log 2>&1 &

set -euo pipefail

# ── configuration ─────────────────────────────────────────────────────────────

# HOOP daemon Tailscale URL — update to your coding host's Tailscale IP
HOOP_URL="${HOOP_URL:-http://100.64.0.1:3000}"

# Project to file notes under (overridden per-recording by broadcast extras)
DEFAULT_PROJECT="${HOOP_DEFAULT_PROJECT:-}"

# Temporary recording file
RECORDING_FILE="/sdcard/hoop-recording-$$.m4a"

# ── helpers ───────────────────────────────────────────────────────────────────

log() { echo "[hoop-listener] $(date '+%H:%M:%S') $*"; }

cleanup() {
    rm -f "$RECORDING_FILE" "$RECORDING_FILE.wav"
    termux-microphone-record -q 2>/dev/null || true
}
trap cleanup EXIT

# ── main loop ─────────────────────────────────────────────────────────────────

log "HOOP listener started. Waiting for HOOP_DICTATE_START..."

while true; do
    # Block until we receive a broadcast intent
    # termux-broadcast-receiver blocks until an intent matching the filter arrives
    INTENT=$(termux-broadcast-receiver -a HOOP_DICTATE_START -t 300 2>/dev/null || echo "timeout")

    if [[ "$INTENT" == "timeout" ]]; then
        log "Heartbeat (no intent received in 5m)"
        continue
    fi

    log "HOOP_DICTATE_START received"

    # Extract project from intent extras (if present)
    PROJECT=""
    if echo "$INTENT" | grep -q '"project"'; then
        PROJECT=$(echo "$INTENT" | python3 -c \
            "import sys,json; d=json.load(sys.stdin); print(d.get('extras',{}).get('project',''))" \
            2>/dev/null || echo "")
    fi
    PROJECT="${PROJECT:-$DEFAULT_PROJECT}"

    # Build upload URL
    UPLOAD_URL="${HOOP_URL}/api/adb/dictate"
    if [[ -n "$PROJECT" ]]; then
        UPLOAD_URL="${UPLOAD_URL}?project=${PROJECT}"
    fi

    # Generate timestamped filename
    TIMESTAMP=$(date '+%Y%m%d-%H%M%S')
    FILENAME="adb-${TIMESTAMP}.m4a"
    RECORDING_FILE="/sdcard/hoop-recording-${TIMESTAMP}.m4a"

    # Start recording
    log "Recording to ${RECORDING_FILE}..."
    termux-microphone-record -f "$RECORDING_FILE" -e m4a &
    RECORD_PID=$!

    # Wait for HOOP_DICTATE_STOP
    log "Recording... send HOOP_DICTATE_STOP to finish"
    STOP_INTENT=$(termux-broadcast-receiver -a HOOP_DICTATE_STOP -t 300 2>/dev/null || echo "timeout")

    if [[ "$STOP_INTENT" == "timeout" ]]; then
        log "Warning: no STOP received in 5 minutes, stopping automatically"
    else
        log "HOOP_DICTATE_STOP received"
    fi

    # Stop recording
    termux-microphone-record -q
    wait "$RECORD_PID" 2>/dev/null || true

    if [[ ! -f "$RECORDING_FILE" ]] || [[ ! -s "$RECORDING_FILE" ]]; then
        log "Error: recording file missing or empty, skipping upload"
        continue
    fi

    FILE_SIZE=$(stat -c%s "$RECORDING_FILE" 2>/dev/null || echo "0")
    log "Recording complete (${FILE_SIZE} bytes), uploading to HOOP..."

    # Upload to HOOP
    HTTP_STATUS=$(curl -sf -w "%{http_code}" -o /tmp/hoop-upload-response.json \
        -X POST "${UPLOAD_URL}&filename=${FILENAME}" \
        --data-binary @"$RECORDING_FILE" \
        -H "Content-Type: audio/mp4" \
        2>/dev/null || echo "000")

    if [[ "$HTTP_STATUS" == "201" ]]; then
        STITCH_ID=$(python3 -c \
            "import json; d=json.load(open('/tmp/hoop-upload-response.json')); print(d.get('stitch_id','?'))" \
            2>/dev/null || echo "?")
        log "Note created: stitch_id=${STITCH_ID} project=${PROJECT:-auto}"
    else
        log "Upload failed (HTTP ${HTTP_STATUS}). Response:"
        cat /tmp/hoop-upload-response.json 2>/dev/null || true
    fi

    rm -f "$RECORDING_FILE"
done
