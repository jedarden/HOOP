#!/bin/bash
# Check that schema changes have corresponding CHANGELOG entries (§20)

set -e

SCHEMA_DIR="hoop-schema/schemas"
CHANGELOG_FILE="CHANGELOG.md"

# Get the list of changed schema files in this PR
# Uses git diff against the target branch (main for PRs)
if [[ "$GITHUB_EVENT_NAME" == "pull_request" ]]; then
  BASE_REF="origin/main"
else
  # For non-PR contexts, compare against HEAD~1
  BASE_REF="HEAD~1"
fi

# Get list of modified/added schema JSON files
CHANGED_SCHEMAS=$(git diff --name-only "$BASE_REF" -- "$SCHEMA_DIR"/*.json 2>/dev/null || true)

if [[ -z "$CHANGED_SCHEMAS" ]]; then
  echo "✓ No schema changes detected"
  exit 0
fi

echo "Schema changes detected:"
echo "$CHANGED_SCHEMAS"
echo ""

# Check if CHANGELOG.md was also modified
CHANGELOG_CHANGED=$(git diff --name-only "$BASE_REF" -- "$CHANGELOG_FILE" 2>/dev/null || true)

if [[ -z "$CHANGELOG_CHANGED" ]]; then
  echo "✗ CHANGELOG.md was not updated"
  echo ""
  echo "Schema changes require a CHANGELOG entry (§20)."
  echo "Please add an entry to the [Unreleased] section documenting:"
  echo "  - Version kind (MAJOR/MINOR/PATCH)"
  echo "  - Affected schemas"
  echo "  - Migration notes"
  exit 1
fi

echo "✓ CHANGELOG.md was updated"
exit 0
