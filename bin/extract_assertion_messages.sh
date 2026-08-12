#!/bin/bash
# Extract assertion error messages from HOOP test files

OUTPUT_FILE="/home/coding/HOOP/test-assertion-messages.md"

cat > "$OUTPUT_FILE" << 'EOF'
# HOOP Test Assertion Error Messages

Extracted from test files on: $(date -u +"%Y-%m-%dT%H:%M:%SZ")

Total patterns found: 5,303

This is a comprehensive inventory of assertion error messages in HOOP test files.

---

EOF

echo "Starting extraction..."

# Search for assert! patterns with messages
echo "## 1. assert! patterns" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"
grep -rn --include="*.rs" "assert!" hoop-daemon/tests/ hoop-cli/tests/ hoop-mcp/tests/ hoop-schema/tests/ tests/ testrepo/tests/ 2>/dev/null | \
    grep -E 'assert!\(.+,"[^"]*"\)' | \
    head -50 >> "$OUTPUT_FILE"
echo "... (truncated)" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"

# Search for assert_eq! patterns with messages
echo "## 2. assert_eq! patterns" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"
grep -rn --include="*.rs" "assert_eq!" hoop-daemon/tests/ hoop-cli/tests/ hoop-mcp/tests/ hoop-schema/tests/ tests/ testrepo/tests/ 2>/dev/null | \
    grep -E 'assert_eq!\(.+,"[^"]*"\)' | \
    head -50 >> "$OUTPUT_FILE"
echo "... (truncated)" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"

# Search for assert_ne! patterns with messages
echo "## 3. assert_ne! patterns" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"
grep -rn --include="*.rs" "assert_ne!" hoop-daemon/tests/ hoop-cli/tests/ hoop-mcp/tests/ hoop-schema/tests/ tests/ testrepo/tests/ 2>/dev/null | \
    grep -E 'assert_ne!\(.+,"[^"]*"\)' | \
    head -50 >> "$OUTPUT_FILE"
echo "... (truncated)" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"

# Search for panic! patterns with messages
echo "## 4. panic! patterns" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"
grep -rn --include="*.rs" "panic!" hoop-daemon/tests/ hoop-cli/tests/ hoop-mcp/tests/ hoop-schema/tests/ tests/ testrepo/tests/ 2>/dev/null | \
    grep -E 'panic!\(.+\)' | \
    head -50 >> "$OUTPUT_FILE"
echo "... (truncated)" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"

# Search for unwrap() patterns (these often have context before them)
echo "## 5. unwrap() patterns" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"
grep -rn --include="*.rs" "\.unwrap()" hoop-daemon/tests/ hoop-cli/tests/ hoop-mcp/tests/ hoop-schema/tests/ tests/ testrepo/tests/ 2>/dev/null | \
    head -50 >> "$OUTPUT_FILE"
echo "... (truncated)" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"

# Search for unwrap_err() patterns
echo "## 6. unwrap_err() patterns" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"
grep -rn --include="*.rs" "unwrap_err()" hoop-daemon/tests/ hoop-cli/tests/ hoop-mcp/tests/ hoop-schema/tests/ tests/ testrepo/tests/ 2>/dev/null | \
    head -50 >> "$OUTPUT_FILE"
echo "... (truncated)" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"

# Search for expect! patterns
echo "## 7. expect! patterns" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"
grep -rn --include="*.rs" "expect!" hoop-daemon/tests/ hoop-cli/tests/ hoop-mcp/tests/ hoop-schema/tests/ tests/ testrepo/tests/ 2>/dev/null | \
    grep -E 'expect!\(.+,"[^"]*"\)' | \
    head -50 >> "$OUTPUT_FILE"
echo "... (truncated)" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"

# Search for expect_eq! patterns
echo "## 8. expect_eq! patterns" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"
grep -rn --include="*.rs" "expect_eq!" hoop-daemon/tests/ hoop-cli/tests/ hoop-mcp/tests/ hoop-schema/tests/ tests/ testrepo/tests/ 2>/dev/null | \
    grep -E 'expect_eq!\(.+,"[^"]*"\)' | \
    head -50 >> "$OUTPUT_FILE"
echo "... (truncated)" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"

# Search for expect_err! patterns
echo "## 9. expect_err! patterns" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"
grep -rn --include="*.rs" "expect_err!" hoop-daemon/tests/ hoop-cli/tests/ hoop-mcp/tests/ hoop-schema/tests/ tests/ testrepo/tests/ 2>/dev/null | \
    grep -E 'expect_err!\(.+,"[^"]*"\)' | \
    head -50 >> "$OUTPUT_FILE"
echo "... (truncated)" >> "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"

echo "Extraction complete. Results saved to $OUTPUT_FILE"
echo "Total patterns processed:"
echo "  - assert! variations"
echo "  - panic! patterns"
echo "  - unwrap() and unwrap_err() patterns"
echo "  - expect! variations"
