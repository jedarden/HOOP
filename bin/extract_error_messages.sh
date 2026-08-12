#!/bin/bash
# Extract all error messages from HOOP test suite

echo "Extracting error messages from HOOP test suite..."
echo "========================================"
echo ""

OUTPUT_DIR="/tmp/hoop_error_messages"
mkdir -p "$OUTPUT_DIR"

# Search for different error patterns
echo "Searching for error patterns..."

# 1. assert! patterns
echo "=== assert! patterns ===" > "$OUTPUT_DIR/assert_messages.txt"
grep -rn "assert!" /home/coding/HOOP/tests/ 2>/dev/null >> "$OUTPUT_DIR/assert_messages.txt"

# 2. assert_eq! and assert_ne! patterns
echo "=== assert_eq! and assert_ne! patterns ===" > "$OUTPUT_DIR/assert_eq_ne_messages.txt"
grep -rn "assert_eq\|assert_ne" /home/coding/HOOP/tests/ 2>/dev/null >> "$OUTPUT_DIR/assert_eq_ne_messages.txt"

# 3. panic! patterns
echo "=== panic! patterns ===" > "$OUTPUT_DIR/panic_messages.txt"
grep -rn "panic!" /home/coding/HOOP/tests/ 2>/dev/null >> "$OUTPUT_DIR/panic_messages.txt"

# 4. unwrap() and expect() patterns
echo "=== unwrap() and expect() patterns ===" > "$OUTPUT_DIR/unwrap_expect_messages.txt"
grep -rn "\.unwrap()\|\.expect(" /home/coding/HOOP/tests/ 2>/dev/null >> "$OUTPUT_DIR/unwrap_expect_messages.txt"

# 5. anyhow patterns
echo "=== anyhow patterns ===" > "$OUTPUT_DIR/anyhow_messages.txt"
grep -rn "anyhow::" /home/coding/HOOP/tests/ 2>/dev/null >> "$OUTPUT_DIR/anyhow_messages.txt"

# 6. Error and Result patterns
echo "=== Error and Result patterns ===" > "$OUTPUT_DIR/error_result_messages.txt"
grep -rn "Error::\|Result<.*,.*E>" /home/coding/HOOP/tests/ 2>/dev/null >> "$OUTPUT_DIR/error_result_messages.txt"

# 7. expect_err! patterns
echo "=== expect_err! patterns ===" > "$OUTPUT_DIR/expect_err_messages.txt"
grep -rn "expect_err!" /home/coding/HOOP/tests/ 2>/dev/null >> "$OUTPUT_DIR/expect_err_messages.txt"

# 8. should_panic patterns
echo "=== should_panic patterns ===" > "$OUTPUT_DIR/should_panic_messages.txt"
grep -rn "#\[should_panic" /home/coding/HOOP/tests/ 2>/dev/null >> "$OUTPUT_DIR/should_panic_messages.txt"

# 9. To assist with pattern matching, I'll expand the search to include additional error-related patterns
echo "=== Additional error patterns ===" > "$OUTPUT_DIR/additional_error_messages.txt"
grep -rn "bail!\|ensure!\|return Err\|map_err\|unwrap_err" /home/coding/HOOP/tests/ 2>/dev/null >> "$OUTPUT_DIR/additional_error_messages.txt"

echo "Error messages extracted to $OUTPUT_DIR"
echo ""
echo "Summary:"
wc -l "$OUTPUT_DIR"/*.txt
