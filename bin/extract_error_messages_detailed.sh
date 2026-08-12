#!/usr/bin/env python3
"""
Extract assertion error messages from HOOP test files.
Outputs structured JSON with file path, line number, pattern type, and error message.
"""

import os
import re
import json
from pathlib import Path
from datetime import datetime

# Regular expressions for different assertion patterns
PATTERNS = {
    'panic': re.compile(r'panic!\s*\(\s*"([^"]+)"(?:,\s*[^)]+)?\)'),
    'panic_format': re.compile(r'panic!\s*\(\s*format!\s*\(\s*"([^"]+)"[^)]*\)\s*(?:,\s*[^)]+)*\)?'),
    'assert': re.compile(r'assert!\s*\([^,]+,\s*"([^"]+)"(?:,\s*[^)]+)?\)'),
    'assert_eq': re.compile(r'assert_eq!\s*\([^,]+,\s*[^,]+,\s*"([^"]+)"(?:,\s*[^)]+)?\)'),
    'assert_ne': re.compile(r'assert_ne!\s*\([^,]+,\s*[^,]+,\s*"([^"]+)"(?:,\s*[^)]+)?\)'),
    'expect': re.compile(r'expect!\s*\([^,]+,\s*"([^"]+)"(?:,\s*[^)]+)?\)'),
    'expect_eq': re.compile(r'expect_eq!\s*\([^,]+,\s*[^,]+,\s*"([^"]+)"(?:,\s*[^)]+)?\)'),
    'expect_ne': re.compile(r'expect_ne!\s*\([^,]+,\s*[^,]+,\s*"([^"]+)"(?:,\s*[^)]+)?\)'),
    'expect_err': re.compile(r'expect_err!\s*\([^,]+,\s*"([^"]+)"(?:,\s*[^)]+)?\)'),
    'unwrap': re.compile(r'\.unwrap\(\)'),
    'unwrap_err': re.compile(r'\.unwrap_err\(\)'),
}

TEST_DIRS = [
    'hoop-daemon/tests/',
    'hoop-cli/tests/',
    'hoop-mcp/tests/',
    'hoop-schema/tests/',
    'tests/',
    'testrepo/tests/',
]

def extract_messages_from_file(file_path):
    """Extract all assertion messages from a single file."""
    findings = []

    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()

        for line_num, line in enumerate(lines, start=1):
            line_stripped = line.strip()

            # Check each pattern type
            for pattern_type, pattern in PATTERNS.items():
                matches = pattern.finditer(line_stripped)

                for match in matches:
                    finding = {
                        'file_path': str(file_path.relative_to('/home/coding/HOOP')),
                        'line_number': line_num,
                        'pattern_type': pattern_type,
                        'line_content': line_stripped,
                    }

                    # Extract the actual error message for patterns that have them
                    if pattern_type in ['panic', 'panic_format', 'assert', 'assert_eq', 'assert_ne',
                                       'expect', 'expect_eq', 'expect_ne', 'expect_err']:
                        if match.groups():
                            finding['error_message'] = match.group(1)

                    findings.append(finding)

    except Exception as e:
        print(f"Error processing {file_path}: {e}", file=__import__('sys').stderr)

    return findings

def main():
    """Main extraction function."""
    all_findings = []
    base_path = Path('/home/coding/HOOP')

    print("Extracting assertion error messages from HOOP test files...")
    print(f"Base path: {base_path}")
    print(f"Test directories: {TEST_DIRS}")

    files_processed = 0
    total_findings = 0

    for test_dir in TEST_DIRS:
        dir_path = base_path / test_dir
        if not dir_path.exists():
            print(f"Directory not found: {dir_path}")
            continue

        print(f"\nProcessing {test_dir}...")

        # Find all .rs files
        rs_files = list(dir_path.rglob('*.rs'))
        print(f"  Found {len(rs_files)} Rust files")

        for rs_file in rs_files:
            findings = extract_messages_from_file(rs_file)
            all_findings.extend(findings)
            files_processed += 1
            total_findings += len(findings)

            if files_processed % 20 == 0:
                print(f"  Processed {files_processed} files, {total_findings} findings so far...")

    print(f"\nExtraction complete!")
    print(f"Files processed: {files_processed}")
    print(f"Total findings: {total_findings}")

    # Categorize findings by pattern type
    by_type = {}
    for finding in all_findings:
        pattern_type = finding['pattern_type']
        if pattern_type not in by_type:
            by_type[pattern_type] = []
        by_type[pattern_type].append(finding)

    print(f"\nBreakdown by pattern type:")
    for pattern_type, findings in sorted(by_type.items()):
        print(f"  {pattern_type}: {len(findings)} occurrences")

    # Save to JSON
    output_file = base_path / 'test-assertion-messages.json'
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump({
            'metadata': {
                'extraction_date': datetime.utcnow().isoformat() + 'Z',
                'files_processed': files_processed,
                'total_findings': total_findings,
                'breakdown_by_type': {k: len(v) for k, v in by_type.items()}
            },
            'findings': all_findings
        }, f, indent=2)

    print(f"\nSaved to {output_file}")

    # Also create a human-readable markdown version
    md_file = base_path / 'test-assertion-messages-detailed.md'
    with open(md_file, 'w', encoding='utf-8') as f:
        f.write(f"""# HOOP Test Assertion Error Messages

**Extracted:** {datetime.utcnow().isoformat()}Z
**Files processed:** {files_processed}
**Total findings:** {total_findings}

This is a comprehensive inventory of assertion error messages in HOOP test files.

---

## Summary by Pattern Type

""")
        for pattern_type, count in sorted([(k, len(v)) for k, v in by_type.items()], key=lambda x: -x[1]):
            f.write(f"- **{pattern_type}**: {count} occurrences\n")

        f.write("\n---\n\n")

        # Show samples for each pattern type
        for pattern_type, findings in sorted(by_type.items(), key=lambda x: -len(x[1])):
            f.write(f"## {pattern_type} patterns ({len(findings)} occurrences)\n\n")

            # Show first 20 examples
            for i, finding in enumerate(findings[:20], 1):
                f.write(f"### {i}. {finding['file_path']}:{finding['line_number']}\n\n")
                f.write(f"**Pattern:** `{finding['pattern_type']}`\n\n")

                if 'error_message' in finding:
                    f.write(f"**Error message:** `{finding['error_message']}`\n\n")

                f.write(f"**Line:**\n```rust\n{finding['line_content']}\n```\n\n")

            if len(findings) > 20:
                f.write(f"... and {len(findings) - 20} more\n\n")

            f.write("---\n\n")

    print(f"Saved markdown version to {md_file}")

if __name__ == '__main__':
    main()
