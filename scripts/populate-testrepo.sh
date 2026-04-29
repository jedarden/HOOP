#!/bin/bash
# Populate testrepo with load test data
#
# This script creates synthetic bead data in the testrepo directory
# for performance budget verification.
#
# Usage:
#   ./populate-testrepo.sh [scale]
#
# Scale options:
#   small   - 2 projects × 2 workers × 10 beads (fast)
#   medium  - 5 projects × 2 workers × 50 beads (default)
#   full    - 20 projects × 5 workers × 300 beads (CI target)

set -e

SCALE="${1:-medium}"

case "$SCALE" in
  small)
    PROJECTS=2
    WORKERS=2
    BEADS=10
    ;;
  medium)
    PROJECTS=5
    WORKERS=2
    BEADS=50
    ;;
  full)
    PROJECTS=20
    WORKERS=5
    BEADS=300
    ;;
  *)
    echo "Unknown scale: $SCALE"
    echo "Usage: $0 [small|medium|full]"
    exit 1
    ;;
esac

TOTAL_BEADS=$((PROJECTS * WORKERS * BEADS))

echo "=== Populating testrepo with load test data ==="
echo "Scale: $SCALE"
echo "Projects: $PROJECTS"
echo "Workers per project: $WORKERS"
echo "Beads per worker: $BEADS"
echo "Total beads: $TOTAL_BEADS"
echo ""

# Set environment variables for the load test
export HOOP_LOAD_PROJECTS="$PROJECTS"
export HOOP_LOAD_WORKERS="$WORKERS"
export HOOP_LOAD_BEADS="$BEADS"

# Run the test to populate testrepo
cargo test --test load_test test_populate_testrepo -- --nocapture --ignored

echo ""
echo "=== testrepo populated successfully ==="
echo "Data location: testrepo/load-test-data/"
echo ""
echo "To run load tests against this data:"
echo "  cargo test --test load_test test_load_test_with_daemon -- --nocapture"
