#!/usr/bin/env python3
"""
Extract assertion error messages from HOOP test files.
Processes assert!, assert_eq!, assert_ne!, expect!, expect_eq!, expect_ne!,
expect_err!, panic!, unwrap(), unwrap_err() patterns.
"""

import os
import re
import json
from pathlib import Path
from typing import List, Dict, Any

# Patterns to search for
ASSERTION_PATTERNS = {
    'assert!': r'assert!\s*\(\s*([^,)]+)(?:,\s*["\']([^"\']+)["\'])?\s*\)',
    'assert_eq!': r'assert_eq!\s*\(\s*([^,]+),\s*([^,)]+)(?:,\s*["\']([^"\']+)["\'])?\s*\)',
    'assert_ne!': r'assert_ne!\s*\(\s*([^,]+),\s*([^,)]+)(?:,\s*["\']([^"\']+)["\'])?\s*\)',
    'expect!': r'expect!\s*\(\s*([^,)]+)(?:,\s*["\']([^"\']+)["\'])?\s*\)\s*[.;]',
    'expect_eq!': r'expect_eq!\s*\(\s*([^,]+),\s*([^,)]+)(?:,\s*["\']([^"\']+)["\'])?\s*\)\s*[.;]',
    'expect_ne!': r'expect_ne!\s*\(\s*([^,]+),\s*([^,)]+)(?:,\s*["\']([^"\']+)["\'])?\s*\)\s*[.;]',
    'expect_err!': r'expect_err!\s*\(\s*([^,)]+)(?:,\s*["\']([^"\']+)["\'])?\s*\)\s*[.;]',
    'panic!': r'panic!\s*\(\s*["\']([^"\']+)["\']',
    'unwrap()': r'\.unwrap\(\)(?:\s*\.expect\(\s*["\']([^"\']+)["\']\))?',
    'unwrap_err()': r'\.unwrap_err\(\)(?:\s*\.expect\(\s*["\']([^"\']+)["\']\))?',
}

def extract_from_file(file_path: str) -> List[Dict[str, Any]]:
    """Extract assertion error messages from a single file."""
    findings = []

    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
            lines = content.split('\n')
    except Exception as e:
        print(f"Warning: Could not read {file_path}: {e}")
        return findings

    for line_num, line in enumerate(lines, start=1):
        for pattern_name, pattern_regex in ASSERTION_PATTERNS.items():
            matches = re.finditer(pattern_regex, line)
            for match in matches:
                error_msg = None
                # Extract error message based on pattern type
                if pattern_name in ['assert!', 'expect!', 'expect_err!', 'panic!']:
                    # These have error message as second capture group
                    groups = match.groups()
                    if len(groups) >= 2 and groups[1]:
                        error_msg = groups[1]
                elif pattern_name in ['assert_eq!', 'assert_ne!', 'expect_eq!', 'expect_ne!']:
                    # These have error message as third capture group
                    groups = match.groups()
                    if len(groups) >= 3 and groups[2]:
                        error_msg = groups[2]
                elif pattern_name in ['unwrap()', 'unwrap_err()']:
                    # These might have expect() chained
                    groups = match.groups()
                    if len(groups) >= 1 and groups[0]:
                        error_msg = groups[0]

                if error_msg:
                    findings.append({
                        'file_path': str(file_path),
                        'line_number': line_num,
                        'pattern_type': pattern_name,
                        'error_message': error_msg,
                        'line_content': line.strip(),
                        'match_text': match.group(0),
                    })

    return findings

def find_test_files(root_dir: str) -> List[str]:
    """Find all Rust test files in the repository."""
    test_files = []
    root_path = Path(root_dir)

    # Common test patterns
    test_patterns = [
        'tests/**/*.rs',
        '**/*test*.rs',
        '**/tests/**/*.rs',
        'hoop-*/tests/**/*.rs',
        'hoop-*/tests_phase*/**/*.rs',
        'testrepo/tests/**/*.rs',
        'testrepo/benches/**/*.rs',
    ]

    for pattern in test_patterns:
        for file_path in root_path.glob(pattern):
            if 'target' not in str(file_path) and '.git' not in str(file_path):
                test_files.append(str(file_path))

    # Also check for #[cfg(test)] and #[test] in source files
    for file_path in root_path.rglob('*.rs'):
        if 'target' not in str(file_path) and '.git' not in str(file_path):
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                    if '#[test]' in content or '#[cfg(test)]' in content:
                        if str(file_path) not in test_files:
                            test_files.append(str(file_path))
            except:
                pass

    return sorted(set(test_files))

def main():
    """Main extraction function."""
    root_dir = '/home/coding/HOOP'

    print("Finding test files...")
    test_files = find_test_files(root_dir)
    print(f"Found {len(test_files)} test files")

    print("Extracting assertion error messages...")
    all_findings = []
    for i, file_path in enumerate(test_files, 1):
        if i % 50 == 0:
            print(f"Processing {i}/{len(test_files)} files...")
        findings = extract_from_file(file_path)
        all_findings.extend(findings)

    print(f"\nExtracted {len(all_findings)} assertion error messages")

    # Save to JSON
    output_file = '/home/coding/HOOP/assertion_error_messages.json'
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(all_findings, f, indent=2)

    print(f"Results saved to {output_file}")

    # Print summary
    print("\n=== SUMMARY ===")
    pattern_counts = {}
    for finding in all_findings:
        pattern = finding['pattern_type']
        pattern_counts[pattern] = pattern_counts.get(pattern, 0) + 1

    print("\nPattern distribution:")
    for pattern, count in sorted(pattern_counts.items()):
        print(f"  {pattern}: {count}")

    print(f"\nTotal unique error messages: {len(set(f['error_message'] for f in all_findings))}")

    # Show some examples
    print("\n=== SAMPLE ERROR MESSAGES ===")
    for i, finding in enumerate(all_findings[:20], 1):
        print(f"\n{i}. {finding['file_path']}:{finding['line_number']}")
        print(f"   Pattern: {finding['pattern_type']}")
        print(f"   Message: {finding['error_message']}")

if __name__ == '__main__':
    main()
