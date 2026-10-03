#!/usr/bin/env python3
"""Add semantic error types and source-module identifiers to the test catalog.

The extraction catalog's existing ``category`` field describes the Rust
assertion/extraction pattern (for example, ``expect with failure``).  This
script adds ``error_type`` for the operation that failed and ``source_module``
for grouping entries without losing the original source path and line.

The classifier is deliberately single-label: rules are evaluated in the order
below, so a message that mentions both a timeout and a network operation is
categorized as ``timeout``.  ``other`` is retained for messages whose
meaning cannot be inferred reliably from the source line.
"""

from __future__ import annotations

import argparse
import csv
import json
import os
import re
import tempfile
from pathlib import Path


CATEGORY_DESCRIPTIONS = {
    "authentication": "Identity or credential verification",
    "authorization": "Permission, confirmation, or access checks",
    "concurrency": "Tasks, locks, races, shutdown, or reconnect coordination",
    "configuration": "CLI flags, adapters, settings, and configuration files",
    "filesystem": "Files, directories, paths, and local filesystem I/O",
    "network": "HTTP, WebSocket, socket, port, or endpoint communication",
    "parsing": "Parsing, serialization, encoding, or structured output",
    "persistence": "SQLite, audit, backup, restore, or durable storage",
    "performance": "Load, latency, throughput, or performance budgets",
    "resource": "Creation, lookup, or lifecycle of a domain resource",
    "runtime": "Daemon, process, subprocess, or service lifecycle",
    "security": "Secrets, redaction, traversal, or privacy handling",
    "state": "Events, beads, Stitches, workers, sessions, or projections",
    "timeout": "Timeouts, readiness waits, and deadline failures",
    "validation": "Expected values, invariants, required fields, or mismatches",
    "other": "Insufficient context for a more specific classification",
}


# Rules are mutually exclusive by ordered evaluation.  Keep these patterns
# narrow: common words such as "response" and "created" occur in many domains.
RULES = (
    (
        "security",
        re.compile(
            r"\bsecret\b|redact|traversal|privacy|\bsensitive\b|"
            r"access[_ -]?key|age[_ -]?key",
            re.I,
        ),
    ),
    (
        "authentication",
        re.compile(
            r"authenticat|\bauth\b|credential|\btoken\b|api[_ -]?key|"
            r"password|\blogin\b|oauth",
            re.I,
        ),
    ),
    (
        "authorization",
        re.compile(
            r"forbidden|unauthori[sz]|permission denied|access denied|"
            r"not allowed|must confirm|confirm requirement",
            re.I,
        ),
    ),
    (
        "timeout",
        re.compile(
            r"timeout|timed out|within \d+ seconds|within timeout|"
            r"did not become ready|failed to become ready|waiting for|wait for",
            re.I,
        ),
    ),
    (
        "performance",
        re.compile(
            r"performance|load test|load data|budget|latency|percentile|"
            r"throughput|slow",
            re.I,
        ),
    ),
    (
        "concurrency",
        re.compile(
            r"concurr|parallel|race|lock|mutex|deadlock|shutdown|cancel|"
            r"\bepoch\b|\btask\b|reconnect",
            re.I,
        ),
    ),
    (
        "configuration",
        re.compile(
            r"config(?:uration)?|projects\.ya?ml|no[_ -]?interactive|\bflag\b|"
            r"adapter|model|setting|wizard|confirm",
            re.I,
        ),
    ),
    (
        "persistence",
        re.compile(
            r"sqlite|database|fleet\.db|\baudit\b|hash|migration|transaction|"
            r"\bquery\b|\brow\b|backup|restore|wal|init fleet db|rebuild|"
            r"\bcount\b",
            re.I,
        ),
    ),
    (
        "filesystem",
        re.compile(
            r"\bfile\b|\bdirectory\b|\bdir\b|\bpath\b|\.beads|\.hoop|"
            r"\bread\b|\bwrite\b|\bremove\b|mkdir|temp|should exist|"
            r"must be readable|created|workspace root|manifest_dir",
            re.I,
        ),
    ),
    (
        "parsing",
        re.compile(
            r"\bparse\b|parsing|deserial|serializ|utf-?8|malformed|"
            r"valid json|json output|valid regex",
            re.I,
        ),
    ),
    (
        "network",
        re.compile(
            r"bind|random port|local address|\bport\b|http|https|request|"
            r"response|fetch|connect|receive|send|socket|websocket|readyz|"
            r"health|endpoint|client|server|stream ended",
            re.I,
        ),
    ),
    (
        "runtime",
        re.compile(
            r"daemon|spawn|start|stop|crash|panic|process|service|running|"
            r"exit|execute",
            re.I,
        ),
    ),
    (
        "state",
        re.compile(
            r"event|state|bead|stitch|project|worker|session|claim|dispatch|"
            r"heartbeat|snapshot|status|timeline|message|subscription|fixture|"
            r"actor|payload|capacity|replay|array|object|title|name|timestamp|"
            r"default",
            re.I,
        ),
    ),
    (
        "resource",
        re.compile(
            r"costaggregator|metadata|draft|proposal|entry|table|index|schema|"
            r"pattern|skill|presence|notification|action|rule|script|approved|"
            r"populate|\b(create|insert|list|get|approve|reject|edit|update)\b",
            re.I,
        ),
    ),
    (
        "validation",
        re.compile(
            r"invalid|expected|should|must|mismatch|equal|success|failure|"
            r"found|match|contain|preserv|remain|output",
            re.I,
        ),
    ),
)


def source_module(source_file: str) -> str:
    """Return a stable crate-like module path while preserving ``file``."""

    module = source_file.replace("\\", "/")
    if module.endswith(".rs"):
        module = module[:-3]
    return module.replace("-", "_").replace("/", "::")


def error_type(row: dict[str, object]) -> str:
    """Classify one source line into exactly one semantic category."""

    source_line = str(row.get("line_content", "")).replace('\\"', '"')
    for category, pattern in RULES:
        if pattern.search(source_line):
            return category
    return "other"


def atomic_write(path: Path, data: str) -> None:
    """Write generated output atomically so an interrupted run cannot truncate it."""

    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    try:
        with os.fdopen(fd, "w", encoding="utf-8", newline="") as handle:
            handle.write(data)
        os.replace(temporary_name, path)
    except Exception:
        try:
            os.unlink(temporary_name)
        except FileNotFoundError:
            pass
        raise


def classify_catalog(input_path: Path) -> list[dict[str, object]]:
    rows = json.loads(input_path.read_text(encoding="utf-8"))
    if not isinstance(rows, list):
        raise ValueError(f"expected a JSON array in {input_path}")

    categorized = []
    for row in rows:
        if not isinstance(row, dict):
            raise ValueError("every catalog entry must be a JSON object")
        if not {"file", "line", "category", "line_content"} <= row.keys():
            raise ValueError(f"catalog entry is missing required fields: {row!r}")
        updated = dict(row)
        updated["error_type"] = error_type(row)
        updated["source_module"] = source_module(str(row["file"]))
        categorized.append(updated)
    return categorized


def write_csv(rows: list[dict[str, object]], path: Path) -> None:
    fields = ["file", "line", "category", "error_type", "source_module", "line_content"]
    output = []
    from io import StringIO

    buffer = StringIO(newline="")
    writer = csv.DictWriter(buffer, fieldnames=fields, lineterminator="\n")
    writer.writeheader()
    grouped_rows = sorted(
        rows,
        key=lambda row: (
            str(row["source_module"]),
            str(row["error_type"]),
            int(row["line"]),
        ),
    )
    writer.writerows({field: row.get(field, "") for field in fields} for row in grouped_rows)
    atomic_write(path, buffer.getvalue())


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, default=Path("test_error_messages.json"))
    parser.add_argument("--json-output", type=Path)
    parser.add_argument("--csv-output", type=Path)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    json_output = args.json_output or args.input
    csv_output = args.csv_output or args.input.with_suffix(".csv")
    rows = classify_catalog(args.input)
    atomic_write(json_output, json.dumps(rows, indent=2, ensure_ascii=False) + "\n")
    write_csv(rows, csv_output)

    print(f"categorized {len(rows)} entries")
    for category in CATEGORY_DESCRIPTIONS:
        count = sum(row["error_type"] == category for row in rows)
        print(f"{category}: {count}")


if __name__ == "__main__":
    main()
