#!/bin/bash
# Extract Error type and anyhow error messages from HOOP tests

OUTPUT_FILE="/home/coding/HOOP/error_messages_catalog.md"
echo "# Error Messages Catalog - HOOP Tests" > "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "Generated: $(date)" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "## Summary" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# Count total patterns
TOTAL_EXPECT=$(grep -r "\.expect(" /home/coding/HOOP --include="*test*.rs" --include="*/tests/*.rs" | wc -l)
TOTAL_UNWRAP_ERR=$(grep -r "\.unwrap_err(" /home/coding/HOOP --include="*test*.rs" --include="*/tests/*.rs" | wc -l)
TOTAL_ANYHOW=$(grep -r "anyhow!(" /home/coding/HOOP --include="*test*.rs" --include="*/tests/*.rs" | wc -l)
TOTAL_BAIL=$(grep -r "anyhow::bail!" /home/coding/HOOP --include="*test*.rs" --include="*/tests/*.rs" | wc -l)
TOTAL_CONTEXT=$(grep -r "\.context(" /home/coding/HOOP --include="*test*.rs" --include="*/tests/*.rs" | wc -l)

echo "- \`.expect()\` patterns: $TOTAL_EXPECT" >> "$OUTPUT_FILE"
echo "- \`.unwrap_err()\` patterns: $TOTAL_UNWRAP_ERR" >> "$OUTPUT_FILE"
echo "- \`anyhow!()\` patterns: $TOTAL_ANYHOW" >> "$OUTPUT_FILE"
echo "- \`anyhow::bail!()\` patterns: $TOTAL_BAIL" >> "$OUTPUT_FILE"
echo "- \`.context()\` patterns: $TOTAL_CONTEXT" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

echo "## Error Type Patterns" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

### Extract .expect() patterns
echo "### \`.expect()\` patterns" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
grep -rn "\.expect(" /home/coding/HOOP --include="*test*.rs" --include="*/tests/*.rs" | while IFS=: read -r file line rest; do
    # Extract the error message from .expect("...")
    message=$(echo "$rest" | grep -oP '\.expect\(\K[^)]+\)' | head -1)
    if [ -n "$message" ]; then
        echo "- **File:** \`$file:$line\`" >> "$OUTPUT_FILE"
        echo "  - **Message:** \`.expect($message)\`" >> "$OUTPUT_FILE"
        echo "  - **Type:** Error type" >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
    fi
done

### Extract .unwrap_err() patterns
echo "### \`.unwrap_err()\` patterns" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
grep -rn "\.unwrap_err(" /home/coding/HOOP --include="*test*.rs" --include="*/tests/*.rs" | while IFS=: read -r file line rest; do
    echo "- **File:** \`$file:$line\`" >> "$OUTPUT_FILE"
    echo "  - **Message:** \`.unwrap_err()\`" >> "$OUTPUT_FILE"
    echo "  - **Type:** Error type" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
done

echo "## anyhow Error Patterns" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

### Extract anyhow!() patterns
echo "### \`anyhow!()\` patterns" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
grep -rn "anyhow!(" /home/coding/HOOP --include="*test*.rs" --include="*/tests/*.rs" | while IFS=: read -r file line rest; do
    # Extract the error message from anyhow!("...")
    message=$(echo "$rest" | grep -oP 'anyhow!\(\K[^)]+\)' | head -1)
    if [ -n "$message" ]; then
        echo "- **File:** \`$file:$line\`" >> "$OUTPUT_FILE"
        echo "  - **Message:** \`anyhow!($message)\`" >> "$OUTPUT_FILE"
        echo "  - **Type:** anyhow error" >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
    fi
done

### Extract anyhow::bail!() patterns
echo "### \`anyhow::bail!()\` patterns" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
grep -rn "anyhow::bail!" /home/coding/HOOP --include="*test*.rs" --include="*/tests/*.rs" | while IFS=: read -r file line rest; do
    # Extract the error message from anyhow::bail!("...")
    message=$(echo "$rest" | grep -oP 'anyhow::bail!\(\K[^)]+\)' | head -1)
    if [ -n "$message" ]; then
        echo "- **File:** \`$file:$line\`" >> "$OUTPUT_FILE"
        echo "  - **Message:** \`anyhow::bail!($message)\`" >> "$OUTPUT_FILE"
        echo "  - **Type:** anyhow bail" >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
    fi
done

### Extract .context() patterns
echo "### \`.context()\` patterns" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
grep -rn "\.context(" /home/coding/HOOP --include="*test*.rs" --include="*/tests/*.rs" | while IFS=: read -r file line rest; do
    # Extract the error message from .context("...")
    message=$(echo "$rest" | grep -oP '\.context\(\K[^)]+\)' | head -1)
    if [ -n "$message" ]; then
        echo "- **File:** \`$file:$line\`" >> "$OUTPUT_FILE"
        echo "  - **Message:** \`.context($message)\`" >> "$OUTPUT_FILE"
        echo "  - **Type:** anyhow context" >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
    fi
done

echo "Extraction complete: $OUTPUT_FILE"
