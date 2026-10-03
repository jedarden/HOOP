#!/usr/bin/env python3
"""Add semantic error types and source-module identifiers to the test catalog.

The extraction catalog's existing ``category`` field describes the Rust
assertion/extraction pattern (for example, ``expect with failure``).  This
script adds ``error_type`` for the operation that failed and ``source_module``
for grouping entries without losing the original source path and line.

The classifier is deliberately single-label: rules are evaluated in the order
below, so a message that mentions both a timeout and a network operation is
categorized as ``timeout``.  ``other`` is retained for messages whose
meaning cannot be inferred reliably from the catalog entry.
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


# Rules are mutually exclusive by ordered evaluation.  These patterns are
# matched against the message text extracted from ``line_content`` rather than
# the complete Rust expression.  That avoids classifying ``resp.json()`` as a
# network error when the message itself says that JSON parsing failed.
RULES = (
    (
        "security",
        re.compile(
            r"\bsecret\b|redact|traversal|privacy|\bsensitive\b|"
            r"access[_ -]?key|age[_ -]?key|credential leak",
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
            r"must be readable|created|workspace root|manifest_dir|\.jsonl\b",
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
            r"exit|execute|stdin|stdout|stderr|\bcommand\b|\brun\b",
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

REQUIRED_FIELDS = {"file", "line", "category", "line_content"}
DERIVED_FIELDS = ("error_type", "source_module")


def _split_macro_args(arguments: str) -> list[str]:
    """Split a Rust macro argument list without splitting nested expressions."""

    parts: list[str] = []
    start = 0
    depth = 0
    in_string = False
    escaped = False
    for index, character in enumerate(arguments):
        if in_string:
            if escaped:
                escaped = False
            elif character == "\\":
                escaped = True
            elif character == '"':
                in_string = False
            continue
        if character == '"':
            in_string = True
        elif character in "([{":
            depth += 1
        elif character in ")]}":
            depth -= 1
        elif character == "," and depth == 0:
            parts.append(arguments[start:index])
            start = index + 1
    parts.append(arguments[start:])
    return parts


def message_text(line_content: str) -> str:
    """Return the likely message text from a Child 1 source-line record.

    Child 1 stores the complete source line in ``line_content`` rather than a
    separate message field.  The extractor serializes quotes as ``\\"`` in the
    JSON value, so normalize those first and then inspect only quoted text.
    For ``expect`` calls the final literal is the error message; earlier
    literals are usually input values.  ``assert_eq!`` and ``assert_ne!`` only
    carry a message when they have a third macro argument.  Keeping the
    complete line as a fallback makes the classifier tolerant of future
    catalog rows that contain no quoted literal.
    """

    normalized = line_content.replace('\\"', '"')
    literals = re.findall(r'"((?:\\.|[^"\\])*)"', normalized)
    if not literals:
        return normalized
    if re.search(r"assert_(?:eq|ne)!\s*\(", normalized):
        opening = normalized.find("(")
        closing = normalized.rfind(")")
        if opening >= 0 and closing > opening:
            arguments = _split_macro_args(normalized[opening + 1 : closing])
            if len(arguments) < 3:
                return ""
            literals = re.findall(r'"((?:\\.|[^"\\])*)"', ",".join(arguments[2:]))
    elif ".expect" in normalized or "expect_err!" in normalized:
        literals = literals[-1:]
    return " ".join(literals)


def source_module(source_file: str) -> str:
    """Return a stable crate-like module path while preserving ``file``."""

    module = source_file.replace("\\", "/")
    if module.endswith(".rs"):
        module = module[:-3]
    return module.replace("-", "_").replace("/", "::")


def error_type(row: dict[str, object]) -> str:
    """Classify one catalog entry into exactly one semantic category."""

    text = message_text(str(row.get("line_content", "")))
    for category, pattern in RULES:
        if pattern.search(text):
            return category

    # Generic messages such as ``"failed"`` carry too little information by
    # themselves.  A few source-file names are stable domain hints and are only
    # consulted after message rules, so explicit message text always wins.
    source = str(row.get("file", ""))
    for category, pattern in (
        ("authentication", re.compile(r"(?:^|[/_])auth(?:entication)?(?:[/_.]|$)", re.I)),
        ("network", re.compile(r"(?:network|websocket|socket|http)", re.I)),
        ("filesystem", re.compile(r"(?:file[_-]?io|filesystem|path)", re.I)),
        ("persistence", re.compile(r"(?:database|migration|backup|restore|storage)", re.I)),
        ("concurrency", re.compile(r"(?:concurr|parallel|race|lock|shutdown)", re.I)),
        ("configuration", re.compile(r"(?:config|no[_-]?interactive|adapter)", re.I)),
    ):
        if pattern.search(source):
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
        if not REQUIRED_FIELDS <= row.keys():
            raise ValueError(f"catalog entry is missing required fields: {row!r}")
        updated = dict(row)
        updated["error_type"] = error_type(row)
        updated["source_module"] = source_module(str(row["file"]))
        categorized.append(updated)
    return sorted(categorized, key=sort_key)


def sort_key(row: dict[str, object]) -> tuple[str, str, str, int]:
    """Sort records for identical, review-friendly JSON and CSV output."""

    try:
        line = int(row["line"])
    except (TypeError, ValueError) as error:
        raise ValueError(f"catalog line must be an integer: {row!r}") from error
    return (
        str(row["source_module"]),
        str(row["error_type"]),
        str(row["file"]),
        line,
    )


def write_csv(rows: list[dict[str, object]], path: Path) -> None:
    fields = []
    for row in rows:
        for field in row:
            if field not in fields and field not in DERIVED_FIELDS:
                fields.append(field)
    fields.extend(DERIVED_FIELDS)
    from io import StringIO

    buffer = StringIO(newline="")
    writer = csv.DictWriter(buffer, fieldnames=fields, lineterminator="\n")
    writer.writeheader()
    writer.writerows({field: row.get(field, "") for field in fields} for row in rows)
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
