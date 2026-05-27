#!/bin/bash
# Playwright UI Test Runner
#
# Runs the Playwright UI test suite for Phase 2 gate validation.
# Tests run on desktop (1280px) and phone (375px) viewports.
#
# Usage:
#   ./run-playwright-tests.sh              # Run all smoke tests
#   ./run-playwright-tests.sh smoke        # Run smoke tests only
#   ./run-playwright-tests.sh visual       # Run visual regression tests
#   ./run-playwright-tests.sh mobile       # Run mobile responsiveness tests
#
# Phase 2 gate requirement (§10):
#   "UI Playwright tests green on desktop + phone viewport"
#
# Environment variables:
#   BASE_URL    - URL of the hoop-daemon (default: starts dev server)
#   CI          - Set to "true" for CI mode (no retries, single worker)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
WEB_DIR="$REPO_ROOT/hoop-ui/web"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test spec: all, smoke, visual, mobile
TEST_SPEC="${1:-all}"

echo "=== HOOP Playwright UI Test Suite ==="
echo "Test spec: $TEST_SPEC"
echo "Phase 2 gate: UI tests on desktop + phone viewport"
echo ""

cd "$WEB_DIR"

# Check if pnpm is available
if ! command -v pnpm &> /dev/null; then
    echo -e "${RED}✗ pnpm not found${NC}"
    echo "Install with: npm install -g pnpm"
    exit 1
fi

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    pnpm install --frozen-lockfile
fi

# Install Playwright browsers if needed
if ! npx playwright --version &> /dev/null; then
    echo "Installing Playwright browsers..."
    npx playwright install --with-deps chromium
fi

# Determine which tests to run
case "$TEST_SPEC" in
    smoke)
        echo "Running smoke tests on desktop and phone viewports..."
        npx playwright test --project=chromium --project=desktop-1280 --project=mobile-375 smoke-tests.spec.ts
        ;;
    visual)
        echo "Running visual regression tests on desktop and phone viewports..."
        npx playwright test --project=chromium --project=desktop-1280 --project=mobile-375 visual-regression.spec.ts
        ;;
    mobile)
        echo "Running mobile responsiveness tests..."
        npx playwright test --project=mobile-375 --project=mobile-700 --project=tablet-768 mobile-responsiveness.spec.ts
        ;;
    all|"")
        echo "Running full smoke test suite on desktop and mobile viewports..."
        npx playwright test --project=chromium --project=desktop-1280 --project=mobile-375 smoke-tests.spec.ts
        ;;
    *)
        echo -e "${RED}Unknown test spec: $TEST_SPEC${NC}"
        echo "Valid options: all, smoke, visual, mobile"
        exit 1
        ;;
esac

TEST_RESULT=$?

echo ""
if [ $TEST_RESULT -eq 0 ]; then
    echo -e "${GREEN}✓ Playwright tests passed${NC}"
    echo ""
    echo "Phase 2 gate requirement satisfied:"
    echo "  ✓ UI tests green on desktop viewport"
    echo "  ✓ UI tests green on phone viewport"
    echo ""
    echo "View HTML report: npx playwright show-report"
    exit 0
else
    echo -e "${RED}✗ Playwright tests failed${NC}"
    echo ""
    echo "Phase 2 gate blocked: Fix failing tests before proceeding"
    echo ""
    echo "View HTML report: npx playwright show-report"
    exit 1
fi
