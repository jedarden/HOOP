#!/usr/bin/env python3
"""
Populate testrepo with load test data for performance budget verification.

Generates synthetic bead data for:
- 20 projects (configurable)
- 5 workers per project (configurable)
- 300 beads per worker (configurable)

Usage:
  python scripts/populate-load-test-data.py

Environment variables:
  HOOP_LOAD_PROJECTS    - number of projects (default: 20)
  HOOP_LOAD_WORKERS     - workers per project (default: 5)
  HOOP_LOAD_BEADS       - beads per worker (default: 300)
"""

import json
import os
import sys
from datetime import datetime, timedelta
from pathlib import Path

# Configuration
NUM_PROJECTS = int(os.environ.get("HOOP_LOAD_PROJECTS", "20"))
WORKERS_PER_PROJECT = int(os.environ.get("HOOP_LOAD_WORKERS", "5"))
BEADS_PER_WORKER = int(os.environ.get("HOOP_LOAD_BEADS", "300"))

# Paths
SCRIPT_DIR = Path(__file__).parent
REPO_ROOT = SCRIPT_DIR.parent
TESTREPO_PATH = REPO_ROOT / "testrepo"
LOAD_TEST_DATA_PATH = TESTREPO_PATH / "load-test-data"


def generate_events(project_name, worker_name, beads_per_worker):
    """Generate synthetic NeedleEvent stream for a worker."""
    events = []
    current_ts = datetime.utcnow()

    for bead_idx in range(beads_per_worker):
        bead_id = f"{project_name}-bd-{bead_idx:06d}"
        is_success = bead_idx % 10 < 7  # 70% success rate

        # Claim event
        events.append({
            "Claim": {
                "ts": current_ts.isoformat() + "Z",
                "worker": worker_name,
                "bead": bead_id,
                "strand": f"strand-{bead_idx % 100:04d}"
            }
        })
        current_ts += timedelta(milliseconds=10)

        # Dispatch event
        events.append({
            "Dispatch": {
                "ts": current_ts.isoformat() + "Z",
                "worker": worker_name,
                "bead": bead_id,
                "adapter": "claude",
                "model": "claude-sonnet-4-6"
            }
        })
        current_ts += timedelta(milliseconds=10)

        if is_success:
            # Complete event
            events.append({
                "Complete": {
                    "ts": current_ts.isoformat() + "Z",
                    "worker": worker_name,
                    "bead": bead_id,
                    "outcome": "success",
                    "duration_ms": 1000 + (bead_idx % 5000),
                    "exit_code": 0
                }
            })

            # Close event
            events.append({
                "Close": {
                    "ts": current_ts.isoformat() + "Z",
                    "worker": worker_name,
                    "bead": bead_id
                }
            })
        else:
            # Fail event
            events.append({
                "Fail": {
                    "ts": current_ts.isoformat() + "Z",
                    "worker": worker_name,
                    "bead": bead_id,
                    "error": "simulated failure",
                    "duration_ms": 500 + (bead_idx % 2000)
                }
            })

            # Release event
            events.append({
                "Release": {
                    "ts": current_ts.isoformat() + "Z",
                    "worker": worker_name,
                    "bead": bead_id
                }
            })

        current_ts += timedelta(milliseconds=20)

    return events


def generate_heartbeats(project_name, workers_per_project):
    """Generate synthetic worker heartbeats for a project."""
    heartbeats = []
    ts = datetime.utcnow()

    for worker_idx in range(workers_per_project):
        worker_name = f"{project_name}-worker-{worker_idx:02d}"

        # Mix of idle and executing states
        if worker_idx % 3 == 0:
            state = {
                "Executing": {
                    "bead": f"{project_name}-bd-{worker_idx * 10:06d}",
                    "pid": 12345 + worker_idx,
                    "adapter": "claude"
                }
            }
        else:
            state = {
                "Idle": {
                    "last_strand": "pluck"
                }
            }

        heartbeats.append({
            "worker": worker_name,
            "ts": ts.isoformat() + "Z",
            "state": state
        })

    return heartbeats


def generate_beads(project_name, beads_per_worker):
    """Generate synthetic bead records for a project."""
    beads = []
    ts = datetime.utcnow()

    total_beads = beads_per_worker  # Simplified - just use beads_per_worker directly

    for bead_idx in range(total_beads):
        bead_id = f"{project_name}-bd-{bead_idx:06d}"
        is_open = bead_idx % 10 < 3  # 30% open

        beads.append({
            "id": bead_id,
            "title": f"Load test bead {bead_idx}",
            "description": "Synthetic bead for load testing",
            "status": "open" if is_open else "closed",
            "priority": 0,
            "issue_type": "task",
            "created_at": ts.isoformat() + "Z",
            "updated_at": ts.isoformat() + "Z",
            "created_by": "load-test-generator",
            "dependencies": [],
            "project": project_name
        })

    return beads


def populate_project(project_idx):
    """Populate a single project with load test data."""
    project_name = f"load-test-project-{project_idx:03d}"
    project_dir = LOAD_TEST_DATA_PATH / project_name / ".beads"

    # Create directory structure
    project_dir.mkdir(parents=True, exist_ok=True)

    # Generate events for all workers
    all_events = []
    for worker_idx in range(WORKERS_PER_PROJECT):
        worker_name = f"{project_name}-worker-{worker_idx:02d}"
        events = generate_events(project_name, worker_name, BEADS_PER_WORKER)
        all_events.extend(events)

    # Write events.jsonl
    events_path = project_dir / "events.jsonl"
    with open(events_path, "w") as f:
        for event in all_events:
            f.write(json.dumps(event) + "\n")

    # Write heartbeats.jsonl
    heartbeats = generate_heartbeats(project_name, WORKERS_PER_PROJECT)
    heartbeats_path = project_dir / "heartbeats.jsonl"
    with open(heartbeats_path, "w") as f:
        for heartbeat in heartbeats:
            f.write(json.dumps(heartbeat) + "\n")

    # Write beads.jsonl
    beads = generate_beads(project_name, BEADS_PER_WORKER)
    beads_path = project_dir / "beads.jsonl"
    with open(beads_path, "w") as f:
        for bead in beads:
            f.write(json.dumps(bead) + "\n")

    return project_name, len(all_events)


def main():
    """Main entry point."""
    print("=== Populate testrepo with Load Test Data ===")
    print(f"Target: {LOAD_TEST_DATA_PATH}")
    print()
    print("Configuration:")
    print(f"  Projects: {NUM_PROJECTS}")
    print(f"  Workers per project: {WORKERS_PER_PROJECT}")
    print(f"  Beads per worker: {BEADS_PER_WORKER}")
    print(f"  Total beads: {NUM_PROJECTS * WORKERS_PER_PROJECT * BEADS_PER_WORKER}")
    print()

    # Create load-test-data directory
    LOAD_TEST_DATA_PATH.mkdir(parents=True, exist_ok=True)

    # Populate each project
    total_events = 0
    for project_idx in range(NUM_PROJECTS):
        project_name, event_count = populate_project(project_idx)
        total_events += event_count
        print(f"  Created {project_name} ({event_count} events)")

    print()
    print(f"✓ testrepo populated successfully")
    print()
    print(f"Generated data location:")
    print(f"  {LOAD_TEST_DATA_PATH}/")
    print()
    print(f"Total events: {total_events}")
    print()
    print("Run load test with:")
    print("  cargo test --package hoop-daemon --test load_test_integration load_test_ci_performance_budgets -- --nocapture")
    print()
    print("Or run the CI script:")
    print("  .github/scripts/run-performance-budget-test.sh")

    return 0


if __name__ == "__main__":
    sys.exit(main())
