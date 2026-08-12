#!/usr/bin/env python3
"""
Extract and catalog all error messages from HOOP test suite.
Creates a comprehensive inventory in markdown format.
"""

import os
import re
from pathlib import Path
from collections import defaultdict
import json

# Error patterns to search for
ERROR_PATTERNS = {
    'assert': r'assert!\s*\([^,]+,\s*["\']([^"\']+)["\']',
    'assert_eq': r'assert_eq!\s*\([^,]+,\s*[^,]+,\s*["\']?([^"\']+)["\']?',
    'assert_ne': r'assert_ne!\s*\([^,]+,\s*[^,]+,\s*["\']?([^"\']+)["\']?',
    'panic': r'panic!\s*\(\s*["\']([^"\']+)["\']',
    'expect': r'\.expect\s*\(\s*["\']([^"\']+)["\']',
    'anyhow': r'anyhow::anyhow!\s*\(\s*["\']([^"\']+)["\']',
    'bail': r'bail!\s*\(\s*["\']([^"\']+)["\']',
    'ensure': r'ensure!\s*\([^,]+,\s*["\']([^"\']+)["\']',
    'unwrap': r'\.unwrap\(\)',
    'expect_err': r'expect_err!\s*\(\s*["\']([^"\']+)["\']',
    'should_panic': r'should_panic\s*\(\s*(expected\s*=\s*["\']([^"\']+)["\'])?',
}

def extract_error_messages_from_file(file_path):
    """Extract all error messages from a single file."""
    errors = []

    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()

        for line_num, line in enumerate(lines, 1):
            # Check each error pattern
            for error_type, pattern in ERROR_PATTERNS.items():
                matches = re.finditer(pattern, line)
                for match in matches:
                    error_msg = match.group(1) if match.groups() else pattern
                    errors.append({
                        'line': line_num,
                        'type': error_type,
                        'message': error_msg,
                        'context': line.strip()
                    })

    except Exception as e:
        print(f"Error reading {file_path}: {e}")

    return errors

def scan_test_files(root_dir):
    """Scan all test files in the HOOP repository."""
    test_files = []

    # Find all test files
    for path in Path(root_dir).rglob('*.rs'):
        if 'test' in path.name or 'tests' in str(path):
            test_files.append(path)

    return test_files

def main():
    root_dir = '/home/coding/HOOP'
    output_file = '/tmp/hoop_error_messages/complete_error_catalog.json'
    markdown_file = '/tmp/hoop_error_messages/error_catalog.md'

    print("Scanning for test files...")
    test_files = scan_test_files(root_dir)
    print(f"Found {len(test_files)} test files")

    # Collect all errors
    all_errors = defaultdict(list)
    total_errors = 0

    for test_file in test_files:
        rel_path = str(test_file.relative_to(root_dir))
        errors = extract_error_messages_from_file(test_file)

        if errors:
            all_errors[rel_path] = errors
            total_errors += len(errors)

    print(f"Extracted {total_errors} error messages from {len(all_errors)} files")

    # Save JSON catalog
    with open(output_file, 'w') as f:
        json.dump(dict(all_errors), f, indent=2)

    # Generate markdown catalog
    with open(markdown_file, 'w') as f:
        f.write("# HOOP Test Suite Error Message Catalog\n\n")
        f.write(f"Generated: {os.popen('date').read().strip()}\n\n")
        f.write(f"Total files with errors: {len(all_errors)}\n")
        f.write(f"Total error messages: {total_errors}\n\n")

        # Group by error type
        errors_by_type = defaultdict(list)
        for file_path, errors in all_errors.items():
            for error in errors:
                errors_by_type[error['type']].append({
                    'file': file_path,
                    'line': error['line'],
                    'message': error['message'],
                    'context': error['context']
                })

        f.write("## Summary by Error Type\n\n")
        for error_type, errors in sorted(errors_by_type.items()):
            f.write(f"- {error_type}: {len(errors)} occurrences\n")

        f.write("\n## Detailed Error Messages by File\n\n")

        # List errors by file
        for file_path in sorted(all_errors.keys()):
            errors = all_errors[file_path]
            f.write(f"### {file_path}\n\n")
            f.write(f"Total errors: {len(errors)}\n\n")

            # Group by error type within file
            by_type = defaultdict(list)
            for error in errors:
                by_type[error['type']].append(error)

            for error_type in sorted(by_type.keys()):
                f.write(f"#### {error_type} ({len(by_type[error_type])} occurrences)\n\n")

                for error in by_type[error_type]:
                    f.write(f"- Line {error['line']}: `{error['message']}`\n")
                    f.write(f"  ```rust\n")
                    f.write(f"  {error['context']}\n")
                    f.write(f"  ```\n\n")

    print(f"\nCatalog saved to:")
    print(f"  JSON: {output_file}")
    print(f"  Markdown: {markdown_file}")

if __name__ == '__main__':
    main()
