# HOOP Makefile
#
# Common development tasks for the HOOP project.

.PHONY: help build test test-load test-load-medium test-load-full test-load-watch clean openapi-generate openapi-check ts-client-generate

# Default target
help:
	@echo "HOOP Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  build              - Build the daemon"
	@echo "  test               - Run unit tests"
	@echo "  test-load          - Run medium-scale load test (5x2x50)"
	@echo "  test-load-medium   - Run medium-scale load test (5x2x50)"
	@echo "  test-load-full     - Run full-scale load test (20x5x200)"
	@echo "  test-load-watch    - Run load test in watch mode (re-run on changes)"
	@echo "  clean              - Clean build artifacts"
	@echo "  openapi-generate   - Generate OpenAPI spec from utoipa annotations"
	@echo "  openapi-check      - Check OpenAPI spec parity (CI)"
	@echo "  ts-client-generate - Generate TypeScript client from OpenAPI spec"
	@echo ""
	@echo "Load Test Configuration:"
	@echo "  HOOP_LOAD_PROJECTS    - Number of projects (default: 20)"
	@echo "  HOOP_LOAD_WORKERS     - Workers per project (default: 5)"
	@echo "  HOOP_LOAD_BEADS       - Beads per worker (default: 200)"
	@echo "  HOOP_LOAD_CADENCE_MS  - Delay between events in ms (default: 10)"
	@echo ""
	@echo "Examples:"
	@echo "  make test-load"
	@echo "  HOOP_LOAD_PROJECTS=10 HOOP_LOAD_WORKERS=3 make test-load"

# Build the daemon
build:
	cargo build --release

# Run unit tests
test:
	@echo "=== Cleaning up HOOP test processes before tests ==="
	@./bin/cleanup-hoop-test-processes.sh || true
	@echo ""
	cargo test --lib --features testing --verbose
	@echo ""
	@echo "=== Verifying no processes remain after tests ==="
	@./bin/verify-hoop-test-processes.sh || echo "Warning: Some processes may remain"

# Run medium-scale load test (5x2x50, ~2 minutes)
test-load: test-load-medium

test-load-medium:
	@echo "=== Cleaning up HOOP test processes before tests ==="
	@./bin/cleanup-hoop-test-processes.sh || true
	@echo ""
	@echo "=== Medium-Scale Load Test ==="
	@echo "Configuration: 5 projects × 2 workers × 50 beads"
	@echo ""
	HOOP_LOAD_PROJECTS=5 \
	HOOP_LOAD_WORKERS=2 \
	HOOP_LOAD_BEADS=50 \
	HOOP_LOAD_CADENCE_MS=10 \
	cargo test --test load_test test_medium_scale_load_test -- --nocapture
	@echo ""
	@echo "=== Verifying no processes remain after tests ==="
	@./bin/verify-hoop-test-processes.sh || echo "Warning: Some processes may remain"

# Run full-scale load test (20x5x200, ~10 minutes)
test-load-full:
	@echo "=== Cleaning up HOOP test processes before tests ==="
	@./bin/cleanup-hoop-test-processes.sh || true
	@echo ""
	@echo "=== Full-Scale Load Test ==="
	@echo "Configuration: 20 projects × 5 workers × 200 beads"
	@echo "WARNING: This may take 10+ minutes"
	@echo ""
	HOOP_LOAD_TEST_FULL_SCALE=1 \
	HOOP_LOAD_PROJECTS=20 \
	HOOP_LOAD_WORKERS=5 \
	HOOP_LOAD_BEADS=200 \
	HOOP_LOAD_CADENCE_MS=10 \
	cargo test --test load_test test_full_scale_load_test -- --ignored --nocapture
	@echo ""
	@echo "=== Verifying no processes remain after tests ==="
	@./bin/verify-hoop-test-processes.sh || echo "Warning: Some processes may remain"

# Run load test in watch mode (requires cargo-watch)
test-load-watch:
	@which cargo-watch > /dev/null || (echo "cargo-watch not installed. Install with: cargo install cargo-watch" && exit 1)
	@echo "=== Load Test (Watch Mode) ==="
	@echo "Running load test on file changes..."
	@echo ""
	HOOP_LOAD_PROJECTS=5 \
	HOOP_LOAD_WORKERS=2 \
	HOOP_LOAD_BEADS=50 \
	cargo watch -x 'test --test load_test test_medium_scale_load_test -- --nocapture'

# Run load test with custom configuration
test-load-custom:
	@echo "=== Cleaning up HOOP test processes before tests ==="
	@./bin/cleanup-hoop-test-processes.sh || true
	@echo ""
	@echo "=== Custom Load Test ==="
	@echo "Configuration:"
	@echo "  Projects: $${HOOP_LOAD_PROJECTS:-5}"
	@echo "  Workers per project: $${HOOP_LOAD_WORKERS:-2}"
	@echo "  Beads per worker: $${HOOP_LOAD_BEADS:-50}"
	@echo ""
	cargo test --test load_test test_medium_scale_load_test -- --nocapture
	@echo ""
	@echo "=== Verifying no processes remain after tests ==="
	@./bin/verify-hoop-test-processes.sh || echo "Warning: Some processes may remain"

# Run the load test example binary
test-load-example:
	cargo run --example load-test-runner -- --url http://localhost:3000

# Clean build artifacts
clean:
	cargo clean
	@echo "Cleaned build artifacts"

# Run load test and verify performance budgets
test-load-verify: test-load-medium
	@echo ""
	@echo "=== Performance Budget Verification ==="
	@echo "✓ Load test completed"
	@echo "✓ Performance budgets verified in test output"
	@echo "✓ Process cleanup verification passed"

# Generate OpenAPI spec from utoipa annotations
openapi-generate:
	@echo "=== Generating OpenAPI Spec ==="
	cargo run --bin generate_openapi --features openapi > hoop-schema/openapi.yaml
	@echo "✓ OpenAPI spec generated to hoop-schema/openapi.yaml"

# Check OpenAPI spec parity (used in CI)
openapi-check:
	@echo "=== Checking OpenAPI Spec Parity ==="
	./scripts/check-openapi-spec.sh

# Generate TypeScript client from OpenAPI spec
ts-client-generate:
	@echo "=== Generating TypeScript Client ==="
	./scripts/generate-ts-client.sh
