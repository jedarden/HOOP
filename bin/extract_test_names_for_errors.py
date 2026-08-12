#!/usr/bin/env python3
"""
Extract test function names and map them to line numbers for error inventory enhancement.
This script parses test files to find #[test] functions and their line ranges.
"""

import re
import json
import sys
from pathlib import Path
from typing import Dict, List, Tuple

def extract_test_functions(file_path: str) -> List[Dict[str, any]]:
    """Extract test function names and their line ranges from a Rust test file."""
    tests = []

    try:
        with open(file_path, 'r') as f:
            lines = f.readlines()

        current_test = None
        brace_count = 0
        test_start_line = 0

        for i, line in enumerate(lines, 1):
            # Look for #[test] or #[tokio::test] markers
            if re.search(r'#\[\[?(?:tokio::)?test\]?\]', line):
                if current_test:
                    # Save previous test if it exists
                    current_test['end_line'] = i - 1
                    if current_test['end_line'] >= current_test['start_line']:
                        tests.append(current_test)

                # Start new test
                current_test = {'start_line': i, 'end_line': len(lines), 'name': None, 'file': file_path}
                brace_count = 0

            # Look for function name after #[test]
            if current_test and current_test['name'] is None:
                match = re.search(r'fn\s+(\w+)\s*\(', line)
                if match:
                    current_test['name'] = match.group(1)
                    current_test['start_line'] = i

            # Track braces to determine function end
            if current_test and current_test['name']:
                brace_count += line.count('{') - line.count('}')
                if brace_count == 0 and '{' in line:
                    # Function closing
                    current_test['end_line'] = i
                    tests.append(current_test)
                    current_test = None

        # Handle last test if file ends without closing brace context
        if current_test and current_test['name']:
            tests.append(current_test)

    except Exception as e:
        print(f"Error processing {file_path}: {e}", file=sys.stderr)

    return tests

def find_test_for_line(line: int, tests: List[Dict]) -> str:
    """Find which test function contains a given line number."""
    for test in tests:
        if test['start_line'] <= line <= test['end_line']:
            return test['name']
    return "unknown"

def main():
    # Load the main error inventory
    inventory_path = Path('/home/coding/HOOP/error_messages_inventory.json')

    with open(inventory_path, 'r') as f:
        inventory = json.load(f)

    # Process each file and enhance with test names
    enhanced_inventory = {
        'generated': inventory['generated'],
        'summary': inventory['summary'],
        'error_type_counts': inventory['error_type_counts'],
        'files_processed': 0,
        'tests_extracted': 0,
        'errors_with_test_names': 0,
        'errors_by_file_and_test': {}
    }

    for file_path, errors in inventory['errors_by_file'].items():
        full_path = Path(f'/home/coding/HOOP/{file_path}')

        # Extract test functions from this file
        test_functions = extract_test_functions(str(full_path))

        if test_functions:
            enhanced_inventory['files_processed'] += 1
            enhanced_inventory['tests_extracted'] += len(test_functions)

        # Organize errors by test function
        file_structure = {}

        for error in errors:
            line = error['line']
            test_name = find_test_for_line(line, test_functions)

            if test_name != "unknown":
                enhanced_inventory['errors_with_test_names'] += 1

            if test_name not in file_structure:
                file_structure[test_name] = []

            enhanced_error = error.copy()
            enhanced_error['test_name'] = test_name
            file_structure[test_name].append(enhanced_error)

        enhanced_inventory['errors_by_file_and_test'][file_path] = file_structure

    # Output the enhanced inventory
    print(json.dumps(enhanced_inventory, indent=2))

if __name__ == '__main__':
    main()
