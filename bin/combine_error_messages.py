#!/usr/bin/env python3
"""
Combine and deduplicate error messages from multiple extraction sources.
Sources:
1. error_messages_catalog.jsonl (Error/anyhow patterns)
2. assertion_messages_v2.json (assertion patterns)
"""

import json
import sys
from pathlib import Path
from collections import defaultdict
from typing import Dict, List, Any


def load_error_catalog(jsonl_path: Path) -> List[Dict[str, Any]]:
    """Load error messages from JSONL catalog."""
    messages = []
    with open(jsonl_path, 'r') as f:
        for line in f:
            if line.strip():
                msg = json.loads(line)
                # Normalize structure
                messages.append({
                    'file_path': msg['file'],
                    'line_number': msg['line'],
                    'pattern_type': msg['pattern'],
                    'error_category': msg['type'],  # 'Error' or 'anyhow'
                    'message': msg['message'],
                    'source': 'error_anyhow'
                })
    return messages


def load_assertion_catalog(json_path: Path) -> List[Dict[str, Any]]:
    """Load assertion messages from JSON catalog."""
    with open(json_path, 'r') as f:
        data = json.load(f)

    messages = []
    for finding in data.get('findings', []):
        messages.append({
            'file_path': finding['file_path'],
            'line_number': finding['line_number'],
            'pattern_type': finding['pattern_type'],
            'error_category': 'assertion',
            'message': finding['message'],
            'source': 'assertion',
            'context': finding.get('context', '')
        })
    return messages


def normalize_path(file_path: str) -> str:
    """Normalize file paths for deduplication."""
    # Convert absolute paths to relative if possible
    if file_path.startswith('/home/coding/HOOP/'):
        return file_path[len('/home/coding/HOOP/'):]
    return file_path


def deduplicate_messages(messages: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """Remove duplicate error messages."""
    seen = set()
    unique = []

    for msg in messages:
        # Create deduplication key
        normalized_path = normalize_path(msg['file_path'])
        key = (
            normalized_path,
            msg['line_number'],
            msg['pattern_type'],
            msg['message']
        )

        if key not in seen:
            seen.add(key)
            unique.append(msg)

    return unique


def generate_inventory_summary(messages: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Generate summary statistics."""
    by_category = defaultdict(int)
    by_pattern = defaultdict(int)
    by_file = defaultdict(int)

    for msg in messages:
        by_category[msg['error_category']] += 1
        by_pattern[msg['pattern_type']] += 1
        normalized_path = normalize_path(msg['file_path'])
        by_file[normalized_path] += 1

    # Get top 20 files by message count
    top_files = sorted(by_file.items(), key=lambda x: x[1], reverse=True)[:20]

    return {
        'total_messages': len(messages),
        'by_error_category': dict(by_category),
        'by_pattern_type': dict(by_pattern),
        'top_files_by_density': top_files,
        'total_files': len(by_file)
    }


def save_json_inventory(messages: List[Dict[str, Any]], summary: Dict[str, Any], output_path: Path):
    """Save combined inventory as JSON."""
    inventory = {
        'generated': '2026-08-12',
        'summary': summary,
        'messages': messages
    }

    with open(output_path, 'w') as f:
        json.dump(inventory, f, indent=2)


def save_markdown_inventory(messages: List[Dict[str, Any]], summary: Dict[str, Any], output_path: Path):
    """Save combined inventory as Markdown."""
    with open(output_path, 'w') as f:
        f.write("# HOOP Error Messages Complete Inventory\n\n")
        f.write("**Generated:** 2026-08-12\n")
        f.write(f"**Total Error Messages:** {summary['total_messages']:,}\n")
        f.write(f"**Total Files:** {summary['total_files']}\n\n")
        f.write("---\n\n")

        # Summary Statistics
        f.write("## Summary Statistics\n\n")

        f.write("### By Error Category\n")
        for category, count in sorted(summary['by_error_category'].items()):
            f.write(f"- **{category}**: {count:,} instances\n")
        f.write("\n")

        f.write("### By Pattern Type\n")
        for pattern, count in sorted(summary['by_pattern_type'].items(), key=lambda x: x[1], reverse=True):
            f.write(f"- **{pattern}**: {count:,} instances\n")
        f.write("\n")

        f.write("### Top 20 Files by Error Message Density\n")
        f.write("| File | Count |\n")
        f.write("|------|-------|\n")
        for file_path, count in summary['top_files_by_density']:
            # Shorten path for display
            display_path = file_path if len(file_path) <= 60 else '...' + file_path[-57:]
            f.write(f"| `{display_path}` | {count} |\n")
        f.write("\n")

        f.write("---\n\n")

        # Grouped by pattern type
        f.write("## Detailed Messages by Pattern Type\n\n")

        # Group messages by pattern type
        by_pattern = defaultdict(list)
        for msg in messages:
            by_pattern[msg['pattern_type']].append(msg)

        for pattern_type in sorted(by_pattern.keys()):
            pattern_msgs = by_pattern[pattern_type]
            f.write(f"### {pattern_type} ({len(pattern_msgs):,} instances)\n\n")

            # Group by file for better organization
            by_file = defaultdict(list)
            for msg in pattern_msgs:
                by_file[normalize_path(msg['file_path'])].append(msg)

            for file_path in sorted(by_file.keys()):
                f.write(f"#### {file_path}\n\n")
                for msg in sorted(by_file[file_path], key=lambda x: x['line_number']):
                    f.write(f"- **Line {msg['line_number']}**: {msg['message']}\n")
                f.write("\n")

        f.write("---\n\n")
        f.write("## Methodology\n\n")
        f.write("This inventory combines two extraction sources:\n\n")
        f.write("1. **Error/anyhow patterns** (extracted via `extract_error_messages.sh`)\n")
        f.write("   - `.expect()`, `.expect_err()`, `.unwrap_err()`\n")
        f.write("   - `anyhow!()`, `anyhow::bail!()`, `.context()`\n\n")
        f.write("2. **Assertion patterns** (extracted via `extract_assertion_messages_v2.py`)\n")
        f.write("   - `assert!()`, `assert_eq!()`, `assert_ne!()`\n")
        f.write("   - `panic!()`, `unwrap()`, `unwrap_err()`\n\n")
        f.write("All duplicates removed based on: file path + line number + pattern type + message text.\n\n")
        f.write("---\n\n")
        f.write("## Raw Data\n\n")
        f.write("Complete structured data available in JSON format at:\n")
        f.write("`docs/error_messages_complete_inventory.json`\n")


def main():
    """Main function."""
    hoop_root = Path('/home/coding/HOOP')

    # Load both catalogs
    error_catalog = hoop_root / 'error_messages_catalog.jsonl'
    assertion_catalog = hoop_root / 'assertion_messages_v2.json'

    print("Loading error messages from Error/anyhow catalog...")
    error_messages = load_error_catalog(error_catalog)
    print(f"  Loaded {len(error_messages):,} messages")

    print("Loading assertion messages from assertion catalog...")
    assertion_messages = load_assertion_catalog(assertion_catalog)
    print(f"  Loaded {len(assertion_messages):,} messages")

    # Combine
    print("\nCombining catalogs...")
    all_messages = error_messages + assertion_messages
    print(f"  Total before deduplication: {len(all_messages):,}")

    # Deduplicate
    print("Deduplicating...")
    unique_messages = deduplicate_messages(all_messages)
    print(f"  Total after deduplication: {len(unique_messages):,}")
    print(f"  Removed {len(all_messages) - len(unique_messages):,} duplicates")

    # Generate summary
    print("\nGenerating summary statistics...")
    summary = generate_inventory_summary(unique_messages)

    # Save outputs
    docs_dir = hoop_root / 'docs'
    docs_dir.mkdir(exist_ok=True)

    json_output = docs_dir / 'error_messages_complete_inventory.json'
    markdown_output = docs_dir / 'error_messages_complete_inventory.md'

    print(f"\nSaving JSON inventory to {json_output}...")
    save_json_inventory(unique_messages, summary, json_output)

    print(f"Saving Markdown inventory to {markdown_output}...")
    save_markdown_inventory(unique_messages, summary, markdown_output)

    print("\n✓ Complete inventory created successfully!")
    print(f"  JSON: {json_output}")
    print(f"  Markdown: {markdown_output}")

    # Print summary
    print("\n=== SUMMARY ===")
    print(f"Total unique error messages: {summary['total_messages']:,}")
    print(f"Total files with errors: {summary['total_files']}")
    print("\nBy category:")
    for category, count in sorted(summary['by_error_category'].items()):
        print(f"  {category}: {count:,}")


if __name__ == '__main__':
    main()
