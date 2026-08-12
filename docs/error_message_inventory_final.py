#!/usr/bin/env python3
"""
HOOP Error Message Inventory - Compilation Script
Combines error extraction results from multiple sources into a unified inventory.
"""

import json
from pathlib import Path
from typing import List, Dict, Set
from datetime import datetime
from collections import defaultdict

def load_error_extraction() -> List[Dict]:
    """Load error-extraction data (bf-5z50g)."""
    path = Path('/home/coding/HOOP/docs/error-extraction/error-messages-with-lines.json')
    if not path.exists():
        print(f"Warning: {path} not found")
        return []

    with open(path, 'r') as f:
        data = json.load(f)

    # Normalize schema: file, line, pattern_type, message, full_context
    normalized = []
    for entry in data:
        normalized.append({
            'file': entry.get('file', ''),
            'line': entry.get('line', 0),
            'pattern_type': entry.get('pattern_type', ''),
            'message': entry.get('message', ''),
            'full_context': entry.get('full_context', ''),
            'source': 'error-extraction'
        })
    return normalized

def load_test_analysis() -> List[Dict]:
    """Load test-analysis data (bf-3ysoc)."""
    path = Path('/home/coding/HOOP/docs/test-analysis/error_messages_catalog.json')
    if not path.exists():
        print(f"Warning: {path} not found")
        return []

    with open(path, 'r') as f:
        data = json.load(f)

    # Normalize schema: text -> message, add test_function
    normalized = []
    for entry in data:
        normalized.append({
            'file': entry.get('file', ''),
            'line': entry.get('line', 0),
            'pattern_type': entry.get('pattern_type', ''),
            'message': entry.get('text', ''),
            'test_function': entry.get('test_function', ''),
            'full_context': entry.get('full_context', ''),
            'source': 'test-analysis'
        })
    return normalized

def create_dedup_key(entry: Dict) -> str:
    """Create a deduplication key for an error message entry."""
    return f"{entry['file']}:{entry['line']}:{entry['pattern_type']}:{entry['message'][:100]}"

def combine_and_deduplicate(error_data: List[Dict], test_data: List[Dict]) -> List[Dict]:
    """Combine both sources and remove duplicates."""
    combined = []
    seen_keys: Set[str] = set()

    # Track duplicates
    duplicate_count = 0
    source_stats = {'error-extraction': 0, 'test-analysis': 0, 'both': 0}

    # Process error-extraction data first
    for entry in error_data:
        key = create_dedup_key(entry)
        if key not in seen_keys:
            seen_keys.add(key)
            combined.append(entry)
            source_stats['error-extraction'] += 1

    # Process test-analysis data, tracking duplicates
    for entry in test_data:
        key = create_dedup_key(entry)
        if key in seen_keys:
            duplicate_count += 1
            source_stats['both'] += 1
            # Mark as duplicate from both sources
            existing = next(e for e in combined if create_dedup_key(e) == key)
            existing['source'] = 'both'
            existing['duplicate'] = True
        else:
            seen_keys.add(key)
            combined.append(entry)
            source_stats['test-analysis'] += 1

    print(f"Combined {len(combined)} unique error messages")
    print(f"Removed {duplicate_count} duplicates")
    print(f"Source stats: {source_stats}")

    return combined

def generate_statistics(data: List[Dict]) -> Dict:
    """Generate statistics for the inventory."""
    stats = {
        'total_messages': len(data),
        'by_pattern_type': defaultdict(int),
        'by_file': defaultdict(int),
        'by_source': defaultdict(int)
    }

    for entry in data:
        stats['by_pattern_type'][entry['pattern_type']] += 1
        stats['by_file'][entry['file']] += 1
        stats['by_source'][entry['source']] += 1

    return stats

def save_json_inventory(data: List[Dict], stats: Dict, output_path: Path):
    """Save the inventory as JSON."""
    inventory = {
        'metadata': {
            'generated': datetime.utcnow().isoformat() + 'Z',
            'total_messages': len(data),
            'sources': ['error-extraction (bf-5z50g)', 'test-analysis (bf-3ysoc)'],
            'bead': 'bf-5hyh6'
        },
        'statistics': {
            'by_pattern_type': dict(stats['by_pattern_type']),
            'top_files': sorted(stats['by_file'].items(), key=lambda x: x[1], reverse=True)[:20],
            'by_source': dict(stats['by_source'])
        },
        'messages': data
    }

    with open(output_path, 'w') as f:
        json.dump(inventory, f, indent=2)

    print(f"JSON inventory saved to {output_path}")

def save_markdown_inventory(data: List[Dict], stats: Dict, output_path: Path):
    """Save the inventory as Markdown."""
    lines = []

    # Header
    lines.append("# HOOP Error Message Inventory")
    lines.append("")
    lines.append(f"**Generated:** {datetime.utcnow().isoformat() + 'Z'}")
    lines.append(f"**Bead:** bf-5hyh6")
    lines.append(f"**Total Messages:** {len(data)}")
    lines.append("")

    # Sources
    lines.append("## Sources")
    lines.append("")
    lines.append("- error-extraction (bf-5z50g): Error/anyhow patterns from 1527 messages")
    lines.append("- test-analysis (bf-3ysoc): Comprehensive test assertion patterns from 3230 messages")
    lines.append("")

    # Statistics
    lines.append("## Statistics")
    lines.append("")
    lines.append("### By Pattern Type")
    lines.append("")
    lines.append("| Pattern Type | Count | Percentage |")
    lines.append("|--------------|-------|------------|")
    total = len(data)
    for pattern, count in sorted(stats['by_pattern_type'].items(), key=lambda x: x[1], reverse=True):
        pct = (count / total) * 100 if total > 0 else 0
        lines.append(f"| {pattern} | {count} | {pct:.1f}% |")
    lines.append("")

    lines.append("### By Source")
    lines.append("")
    lines.append("| Source | Count |")
    lines.append("|--------|-------|")
    for source, count in sorted(stats['by_source'].items()):
        lines.append(f"| {source} | {count} |")
    lines.append("")

    lines.append("### Top Files by Error Count")
    lines.append("")
    lines.append("| File | Count |")
    lines.append("|------|-------|")
    for file, count in sorted(stats['by_file'].items(), key=lambda x: x[1], reverse=True)[:20]:
        lines.append(f"| {file} | {count} |")
    lines.append("")

    # Detailed catalog grouped by file
    lines.append("## Detailed Catalog")
    lines.append("")

    # Group by file
    by_file = defaultdict(list)
    for entry in data:
        by_file[entry['file']].append(entry)

    for file_path in sorted(by_file.keys()):
        entries = sorted(by_file[file_path], key=lambda x: x['line'])
        lines.append(f"### {file_path}")
        lines.append("")
        lines.append(f"**Total messages in file:** {len(entries)}")
        lines.append("")
        lines.append("| Line | Pattern Type | Message |")
        lines.append("|------|--------------|---------|")

        for entry in entries:
            # Truncate long messages for table
            message = entry['message'][:80] + '...' if len(entry['message']) > 80 else entry['message']
            # Escape pipe characters
            message = message.replace('|', '\\|')
            lines.append(f"| {entry['line']} | {entry['pattern_type']} | {message} |")

        lines.append("")

    with open(output_path, 'w') as f:
        f.write('\n'.join(lines))

    print(f"Markdown inventory saved to {output_path}")

def main():
    print("=" * 60)
    print("HOOP Error Message Inventory Compilation")
    print("Bead: bf-5hyh6")
    print("=" * 60)
    print()

    # Load both sources
    print("Loading error-extraction data...")
    error_data = load_error_extraction()
    print(f"  Loaded {len(error_data)} entries")

    print("Loading test-analysis data...")
    test_data = load_test_analysis()
    print(f"  Loaded {len(test_data)} entries")
    print()

    # Combine and deduplicate
    print("Combining and deduplicating...")
    combined = combine_and_deduplicate(error_data, test_data)
    print()

    # Generate statistics
    print("Generating statistics...")
    stats = generate_statistics(combined)
    print(f"  Pattern types: {len(stats['by_pattern_type'])}")
    print(f"  Files: {len(stats['by_file'])}")
    print()

    # Save outputs
    output_dir = Path('/home/coding/HOOP/docs')
    json_path = output_dir / 'hoop_error_message_inventory_final.json'
    md_path = output_dir / 'hoop_error_message_inventory_final.md'

    print("Saving JSON inventory...")
    save_json_inventory(combined, stats, json_path)

    print("Saving Markdown inventory...")
    save_markdown_inventory(combined, stats, md_path)

    print()
    print("=" * 60)
    print("Compilation complete!")
    print(f"Total unique error messages: {len(combined)}")
    print(f"JSON: {json_path}")
    print(f"Markdown: {md_path}")
    print("=" * 60)

if __name__ == '__main__':
    main()
