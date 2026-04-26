#!/usr/bin/env python3
"""Regenerate CLI session fixtures for testrepo adapters."""

import json
import os
import sys
from datetime import datetime, timedelta

TESTREPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# Session templates per adapter
ADAPTER_SESSIONS = {
    'claude': [
        ('2026-04-21T18:42:10Z', 'br list', '[needle:alpha:bd-abc123:pluck] tr-open-001|Fix memory leak|open|bug'),
        ('2026-04-21T18:42:15Z', 'br show tr-open-001', '[needle:alpha:bd-abc123:pluck] ID: tr-open-001\nTitle: Fix memory leak in parser\nStatus: open\nType: bug\nPriority: 1'),
        ('2026-04-21T18:47:33Z', 'br claim tr-open-001', '[needle:alpha:bd-abc123:pluck] Claimed tr-open-001'),
        ('2026-04-21T18:52:01Z', 'br close tr-open-001 --body "Fixed leak in parser tokenization"', '[needle:alpha:bd-abc123:pluck] Closed tr-open-001'),
        ('2026-04-21T18:52:05Z', 'br list --status ready', '[needle:alpha:bd-abc123:pluck] tr-open-002|Add streaming support|open|feature\ntr-open-003|Update documentation|open|task'),
    ],
    'codex': [
        ('2026-04-21T18:55:00Z', 'br list', '[needle:bravo:bd-def456:mend] tr-open-002|Add streaming support|open|feature'),
        ('2026-04-21T18:55:30Z', 'br claim tr-open-002', '[needle:bravo:bd-def456:mend] Claimed tr-open-002'),
        ('2026-04-21T18:56:45Z', 'br update tr-open-002 --status failed', '[needle:bravo:bd-def456:mend] Updated tr-open-002 to failed'),
        ('2026-04-21T18:57:00Z', 'br release tr-open-002', '[needle:bravo:bd-def456:mend] Released tr-open-002'),
    ],
    'gemini': [
        ('2026-04-21T19:15:00Z', 'br list', '[needle:delta:bd-jkl012:weave] tr-closed-001|Initial scaffold|closed|task\ntr-closed-002|Add test suite|closed|task'),
        ('2026-04-21T19:15:30Z', 'br show tr-closed-001 --json', '[needle:delta:bd-jkl012:weave] {"id":"tr-closed-001","title":"Initial scaffold","status":"closed","closed_at":"2026-04-21T18:52:01Z"}'),
        ('2026-04-21T19:17:00Z', 'br crash', '[needle:delta:bd-jkl012:weave] Process crashed with exit code 139'),
    ],
    'opencode': [
        ('2026-04-21T19:05:00Z', 'br list --status ready', '[needle:charlie:bd-ghi789:explore] tr-open-003|Update documentation|open|task'),
        ('2026-04-21T19:05:30Z', 'br show tr-open-003', '[needle:charlie:bd-ghi789:explore] ID: tr-open-003\nTitle: Update documentation\nStatus: open\nType: task\nPriority: 2'),
        ('2026-04-21T19:10:00Z', 'br timeout', '[needle:charlie:bd-ghi789:explore] Command timed out'),
    ],
    'aider': [
        ('2026-04-21T19:20:00Z', 'br list --status in_progress', '[needle:alpha:bd-mno345:pluck] tr-claimed-001|Implement retry logic|in_progress|feature\ntr-claimed-002|Refactor database layer|in_progress|refactor'),
        ('2026-04-21T19:20:30Z', 'br update tr-claimed-001 --add-label "wip"', '[needle:alpha:bd-mno345:pluck] Added label \'wip\' to tr-claimed-001'),
        ('2026-04-21T19:25:01Z', 'br close tr-claimed-001', '[needle:alpha:bd-mno345:pluck] Closed tr-claimed-001'),
    ],
}

def generate_session(adapter: str) -> None:
    """Generate CLI session JSONL for an adapter."""
    if adapter not in ADAPTER_SESSIONS:
        print(f"Unknown adapter: {adapter}", file=sys.stderr)
        sys.exit(1)
    
    session_dir = os.path.join(TESTREPO_ROOT, 'cli-sessions', adapter)
    os.makedirs(session_dir, exist_ok=True)
    
    session_file = os.path.join(session_dir, 'session.jsonl')
    
    with open(session_file, 'w') as f:
        for ts, cmd, output in ADAPTER_SESSIONS[adapter]:
            entry = {
                'ts': ts,
                'cmd': cmd,
                'output': output
            }
            f.write(json.dumps(entry) + '\n')
    
    print(f"Generated {adapter} session with {len(ADAPTER_SESSIONS[adapter])} entries")

def main():
    if len(sys.argv) != 2:
        print("Usage: regenerate-cli-sessions.py <adapter>", file=sys.stderr)
        print(f"Available adapters: {', '.join(ADAPTER_SESSIONS.keys())}", file=sys.stderr)
        sys.exit(1)
    
    adapter = sys.argv[1]
    generate_session(adapter)

if __name__ == '__main__':
    main()
