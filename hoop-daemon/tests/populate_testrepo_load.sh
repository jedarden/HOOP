#!/bin/bash
# Populate testrepo/ with load test data (20 projects × 5 workers × 300 beads)
#
# This script creates synthetic bead data for performance budget verification.
# It's designed to be run in CI or locally for testing.
#
# Plan reference: §6 Phase 6 deliverable 9
# Feeds into hoop-ttb.7.11 performance budget verification

set -euo pipefail

# Configuration
NUM_PROJECTS=${HOOP_LOAD_PROJECTS:-20}
WORKERS_PER_PROJECT=${HOOP_LOAD_WORKERS:-5}
BEADS_PER_WORKER=${HOOP_LOAD_BEADS:-300}
EVENT_CADENCE_MS=${HOOP_LOAD_CADENCE_MS:-10}

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TESTREPO_DIR="$REPO_ROOT/testrepo"

echo "=== Populating testrepo with Load Test Data ==="
echo "Projects: $NUM_PROJECTS"
echo "Workers per project: $WORKERS_PER_PROJECT"
echo "Beads per worker: $BEADS_PER_WORKER"
echo "Total beads: $((NUM_PROJECTS * WORKERS_PER_PROJECT * BEADS_PER_WORKER))"
echo ""

# Check if testrepo exists
if [ ! -d "$TESTREPO_DIR" ]; then
  echo "Error: testrepo directory not found at $TESTREPO_DIR"
  echo "Please create testrepo/ first"
  exit 1
fi

# Create load-test projects directory
LOAD_TEST_DIR="$TESTREPO_DIR/load-test-data"
mkdir -p "$LOAD_TEST_DIR"

echo "Creating load test projects..."

# Create projects
for proj_idx in $(seq 0 $((NUM_PROJECTS - 1))); do
  project_name=$(printf "load-test-project-%03d" "$proj_idx")
  project_dir="$LOAD_TEST_DIR/$project_name"
  beads_dir="$project_dir/.beads"

  mkdir -p "$beads_dir"

  echo "  Creating $project_name..."

  # Generate events.jsonl
  events_file="$beads_dir/events.jsonl"
  > "$events_file"

  current_time=$(date -u +"%Y-%m-%dT%H:%M:%S.%3NZ")
  current_ts=$(date -d "$current_time" +%s)000  # milliseconds

  for worker_idx in $(seq 0 $((WORKERS_PER_PROJECT - 1))); do
    worker_name="$project_name-worker-$(printf "%02d" "$worker_idx")"

    for bead_idx in $(seq 0 $((BEADS_PER_WORKER - 1))); do
      bead_id="$project_name-bd-$(printf "%06d" "$bead_idx")"
      strand_id="strand-$(printf "%04d" "$((bead_idx % 100)))"

      # Determine success/failure (70% success)
      if [ $((bead_idx % 10)) -lt 7 ]; then
        # Success path
        claim_ts=$(date -u -d @$((current_ts / 1000)) +"%Y-%m-%dT%H:%M:%S.%3NZ")
        current_ts=$((current_ts + EVENT_CADENCE_MS))
        dispatch_ts=$(date -u -d @$((current_ts / 1000)) +"%Y-%m-%dT%H:%M:%S.%3NZ")
        current_ts=$((current_ts + EVENT_CADENCE_MS))
        duration_ms=$((1000 + (bead_idx % 5000)))
        complete_ts=$(date -u -d @$((current_ts / 1000)) +"%Y-%m-%dT%H:%M:%S.%3NZ")
        current_ts=$((current_ts + EVENT_CADENCE_MS))
        close_ts=$(date -u -d @$((current_ts / 1000)) +"%Y-%m-%dT%H:%M:%S.%3NZ")
        current_ts=$((current_ts + EVENT_CADENCE_MS * 2))

        cat >> "$events_file" <<EOF
{"ts":"$claim_ts","type":"claim","worker":"$worker_name","bead":"$bead_id","strand":"$strand_id"}
{"ts":"$dispatch_ts","type":"dispatch","worker":"$worker_name","bead":"$bead_id","adapter":"claude","model":"claude-sonnet-4-6"}
{"ts":"$complete_ts","type":"complete","worker":"$worker_name","bead":"$bead_id","outcome":"success","duration_ms":$duration_ms,"exit_code":0}
{"ts":"$close_ts","type":"close","worker":"$worker_name","bead":"$bead_id"}
EOF
      else
        # Failure path
        claim_ts=$(date -u -d @$((current_ts / 1000)) +"%Y-%m-%dT%H:%M:%S.%3NZ")
        current_ts=$((current_ts + EVENT_CADENCE_MS))
        dispatch_ts=$(date -u -d @$((current_ts / 1000)) +"%Y-%m-%dT%H:%M:%S.%3NZ")
        current_ts=$((current_ts + EVENT_CADENCE_MS))
        duration_ms=$((500 + (bead_idx % 2000)))
        fail_ts=$(date -u -d @$((current_ts / 1000)) +"%Y-%m-%dT%H:%M:%S.%3NZ")
        current_ts=$((current_ts + EVENT_CADENCE_MS))
        release_ts=$(date -u -d @$((current_ts / 1000)) +"%Y-%m-%dT%H:%M:%S.%3NZ")
        current_ts=$((current_ts + EVENT_CADENCE_MS * 2))

        cat >> "$events_file" <<EOF
{"ts":"$claim_ts","type":"claim","worker":"$worker_name","bead":"$bead_id","strand":"$strand_id"}
{"ts":"$dispatch_ts","type":"dispatch","worker":"$worker_name","bead":"$bead_id","adapter":"claude","model":"claude-sonnet-4-6"}
{"ts":"$fail_ts","type":"fail","worker":"$worker_name","bead":"$bead_id","error":"simulated failure","duration_ms":$duration_ms}
{"ts":"$release_ts","type":"release","worker":"$worker_name","bead":"$bead_id"}
EOF
      fi
    done
  done

  # Generate heartbeats.jsonl
  heartbeats_file="$beads_dir/heartbeats.jsonl"
  > "$heartbeats_file"

  heartbeat_time=$(date -u +"%Y-%m-%dT%H:%M:%S.%3NZ")

  for worker_idx in $(seq 0 $((WORKERS_PER_PROJECT - 1))); do
    worker_name="$project_name-worker-$(printf "%02d" "$worker_idx")"

    if [ $((worker_idx % 3)) -eq 0 ]; then
      # Executing state
      bead_id="$project_name-bd-$(printf "%06d" "$((worker_idx * 10))")"
      pid=$((12345 + worker_idx))
      cat >> "$heartbeats_file" <<EOF
{"worker":"$worker_name","ts":"$heartbeat_time","state":"executing","bead":"$bead_id","pid":$pid,"adapter":"claude"}
EOF
    else
      # Idle state
      cat >> "$heartbeats_file" <<EOF
{"worker":"$worker_name","ts":"$heartbeat_time","state":"idle","last_strand":"pluck"}
EOF
    fi
  done

  # Generate beads.jsonl
  beads_file="$beads_dir/beads.jsonl"
  > "$beads_file"

  bead_time=$(date -u +"%Y-%m-%dT%H:%M:%S.%3NZ")

  for worker_idx in $(seq 0 $((WORKERS_PER_PROJECT - 1))); do
    for bead_idx in $(seq 0 $((BEADS_PER_WORKER - 1))); do
      bead_id="$project_name-bd-$(printf "%06d" "$bead_idx")"
      is_open=$((bead_idx % 10 < 3))  # 30% open

      status="closed"
      if [ "$is_open" -eq 1 ]; then
        status="open"
      fi

      cat >> "$beads_file" <<EOF
{"id":"$bead_id","title":"Load test bead $bead_idx","description":"Synthetic bead for load testing","status":"$status","priority":0,"issue_type":"task","created_at":"$bead_time","updated_at":"$bead_time","created_by":"load-test-generator","dependencies":[],"project":"$project_name"}
EOF
    done
  done

  # Count lines in events file
  event_count=$(wc -l < "$events_file")
  echo "    Generated $event_count events"
done

echo ""
echo "=== Load test data generation complete ==="
echo "Location: $LOAD_TEST_DIR"
echo ""
echo "To use this data with the test daemon, update your projects.yaml to include:"
echo "  - name: load-test-project-000"
echo "    path: $LOAD_TEST_DIR/load-test-project-000"
echo ""
echo "Performance budgets:"
echo "  - API Latency: < 500ms"
echo "  - WS Fan-out Lag: < 100ms"
echo "  - Memory: < 4GB"
