#!/usr/bin/env python3
"""
Parse clippy output and categorize warnings by file and module.
"""
import re
import json
from collections import defaultdict
from pathlib import Path

def parse_clippy_output(file_path):
    """Parse clippy warning output and categorize by file."""
    with open(file_path, 'r') as f:
        content = f.read()

    # Pattern to match clippy warnings
    # Format: warning: <message>
    #   --> <file>:<line>:<col>
    warning_pattern = re.compile(
        r'warning: (.+?)\s*\n\s+--> ([^:]+:\d+:\d+)',
        re.MULTILINE
    )

    # Track warnings by file
    by_file = defaultdict(lambda: defaultdict(list))

    # Find all warnings
    for match in warning_pattern.finditer(content):
        message = match.group(1).strip()
        file_path = match.group(2).split(':')[0]

        # Extract line number
        line_match = re.search(r':(\d+):\d+', match.group(2))
        line_num = int(line_match.group(1)) if line_match else None

        # Determine warning category
        category = categorize_warning(message)

        # Determine module from file path
        module = extract_module(file_path)

        # Store warning
        by_file[file_path][category].append({
            'line': line_num,
            'message': message
        })

    return by_file

def categorize_warning(message):
    """Categorize warning by type."""
    categories = {
        'private_interfaces': 'type .* is more private than the item',
        'dead_code': 'is never used',
        'unnecessary_sort_by': 'consider using `sort_by_key`',
        'disallowed_methods': 'use of a disallowed method',
        'unnecessary_unwrap': 'called `unwrap` .* after checking',
        'explicit_counter_loop': 'the variable .* is used as a loop counter',
        'if_same_then_else': 'this `if` has identical blocks',
        'too_many_arguments': 'this function has too many arguments',
        'large_enum_variant': 'large size difference between variants',
        'len_without_is_empty': 'has a public `len` method, but no `is_empty`',
        'ptr_arg': 'writing `&mut Vec` instead of `&mut \\[_]`',
        'should_implement_trait': 'can be confused for the standard trait',
        'doc_overindented_list_items': 'doc list item overindented',
        'manual_strip': 'stripping a prefix manually',
        'non_snake_case': 'should have a snake case name',
    }

    for category, pattern in categories.items():
        if re.search(pattern, message):
            return category

    return 'other'

def extract_module(file_path):
    """Extract module name from file path."""
    # Extract from hoop-daemon/src/<module>.rs
    # or hoop-cli/src/<module>.rs
    if 'hoop-daemon/src/' in file_path:
        match = re.search(r'hoop-daemon/src/([^/]+)', file_path)
        if match:
            return f"hoop-daemon::{match.group(1).replace('.rs', '')}"
    elif 'hoop-cli/src/' in file_path:
        match = re.search(r'hoop-cli/src/([^/]+)', file_path)
        if match:
            return f"hoop-cli::{match.group(1).replace('.rs', '')}"
    elif 'hoop-mcp/src/' in file_path:
        match = re.search(r'hoop-mcp/src/([^/]+)', file_path)
        if match:
            return f"hoop-mcp::{match.group(1).replace('.rs', '')}"

    return 'unknown'

def generate_summary(by_file):
    """Generate summary statistics."""
    total_by_category = defaultdict(int)
    total_by_file = defaultdict(int)

    for file_path, categories in by_file.items():
        file_total = sum(len(warnings) for warnings in categories.values())
        total_by_file[file_path] = file_total

        for category, warnings in categories.items():
            total_by_category[category] += len(warnings)

    return {
        'total_by_category': dict(total_by_category),
        'total_by_file': dict(total_by_file),
        'grand_total': sum(total_by_file.values())
    }

def main():
    input_file = Path('/home/coding/HOOP/docs/redundant-pattern-raw-output.txt')
    output_file = Path('/home/coding/HOOP/docs/redundant-pattern-by-file.json')

    # Generate stats file too
    stats_file = Path('/home/coding/HOOP/docs/redundant-pattern-stats.json')

    # Parse clippy output
    by_file = parse_clippy_output(input_file)

    # Generate summary
    summary = generate_summary(by_file)

    # Build output structure
    output = {
        'summary': summary,
        'by_file': {}
    }

    # Organize by file
    for file_path in sorted(by_file.keys()):
        categories = {}
        for category in sorted(by_file[file_path].keys()):
            warnings = by_file[file_path][category]
            categories[category] = {
                'count': len(warnings),
                'line_numbers': [w['line'] for w in warnings if w['line']],
                'messages': [w['message'] for w in warnings]
            }

        output['by_file'][file_path] = {
            'module': extract_module(file_path),
            'categories': categories,
            'total': sum(len(w) for w in by_file[file_path].values())
        }

    # Write JSON output
    with open(output_file, 'w') as f:
        json.dump(output, f, indent=2)

    # Write stats summary
    stats = {
        'generated': '2026-08-13',
        'workspace': 'HOOP',
        'total_warnings': summary['grand_total'],
        'files_with_warnings': len(by_file),
        'categories': summary['total_by_category']
    }
    with open(stats_file, 'w') as f:
        json.dump(stats, f, indent=2)

    print(f"✓ Categorized {summary['grand_total']} warnings across {len(by_file)} files")
    print(f"✓ Output written to {output_file}")
    print(f"✓ Stats written to {stats_file}")

    # Print category breakdown
    print("\nCategory breakdown:")
    for category, count in sorted(summary['total_by_category'].items(), key=lambda x: -x[1]):
        print(f"  {category}: {count}")

if __name__ == '__main__':
    main()