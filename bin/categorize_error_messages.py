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
import shlex
import sys
import tempfile
from pathlib import Path


REQUIRED_FIELDS = {"file", "line", "category", "line_content"}
DERIVED_FIELDS = ("error_type", "source_module", "exception_flags")
UNKNOWN_MODULE = "unknown_module"
TAXONOMY_PATH = Path(__file__).resolve().parents[1] / "docs" / "error_category_taxonomy.json"


def load_taxonomy(path: Path = TAXONOMY_PATH) -> dict[str, object]:
    """Load and minimally validate the approved machine-readable taxonomy."""

    taxonomy = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(taxonomy, dict):
        raise ValueError(f"taxonomy must be a JSON object: {path}")
    allowed = taxonomy.get("allowed_error_types")
    precedence = taxonomy.get("message_rule_precedence")
    rules = taxonomy.get("message_rules")
    fallback = taxonomy.get("source_file_fallback")
    if not isinstance(allowed, list) or not all(isinstance(value, str) for value in allowed):
        raise ValueError(f"taxonomy has no valid allowed_error_types: {path}")
    if not isinstance(precedence, list) or precedence != [value for value in precedence if value in allowed]:
        raise ValueError(f"taxonomy has an invalid message_rule_precedence: {path}")
    if not isinstance(rules, list) or not isinstance(fallback, dict):
        raise ValueError(f"taxonomy is missing classification rules: {path}")
    return taxonomy


TAXONOMY = load_taxonomy()
ALLOWED_ERROR_TYPES = tuple(TAXONOMY["allowed_error_types"])
ERROR_TYPE_RANK = {category: index for index, category in enumerate(ALLOWED_ERROR_TYPES)}
CATEGORY_DESCRIPTIONS = {
    category: str(details["description"])
    for category, details in TAXONOMY["categories"].items()
}
_rules_by_category = {
    rule["category"]: re.compile(rule["pattern"], re.I)
    for rule in TAXONOMY["message_rules"]
}
RULES = tuple(
    (category, _rules_by_category[category])
    for category in TAXONOMY["message_rule_precedence"]
)
SOURCE_FALLBACK_RULES = tuple(
    (rule["category"], re.compile(rule["pattern"], re.I))
    for rule in TAXONOMY["source_file_fallback"]["rules"]
)


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
    carry a message when they have a third macro argument.  A line without a
    message literal has an empty message by contract; source
    fallback rules, rather than unrelated Rust syntax, handle that case.
    """

    normalized = line_content.replace('\\"', '"')
    literals = re.findall(r'"((?:\\.|[^"\\])*)"', normalized)
    if not literals:
        return ""
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
    return " ".join(_unescape_literal(literal) for literal in literals)


def _unescape_literal(value: str) -> str:
    """Decode the JSON-compatible escapes retained by the source extractor."""

    try:
        return json.loads(f'"{value}"')
    except json.JSONDecodeError:
        return value.replace('\\"', '"')


def message_for_row(row: dict[str, object]) -> str:
    """Apply the taxonomy's handoff-message precedence to one record."""

    original = row.get("original_message")
    if isinstance(original, str) and original.strip():
        return original
    return message_text(str(row.get("line_content", "")))


def _normalized_source_path(source_file: object) -> str | None:
    """Return the normalized source path, or ``None`` for malformed input."""

    if not isinstance(source_file, str) or not source_file.strip():
        return None
    normalized = source_file.replace("\\", "/")
    while normalized.startswith("./"):
        normalized = normalized[2:]
    if (
        normalized.startswith("/")
        or re.match(r"^[A-Za-z]:/", normalized)
        or ".." in normalized.split("/")
        or not normalized.endswith(".rs")
    ):
        return None
    return normalized


def source_module(source_file: str) -> str:
    """Return a stable crate-like module path while preserving ``file``."""

    normalized = _normalized_source_path(source_file)
    if normalized is None:
        return UNKNOWN_MODULE
    module = normalized[:-3]
    segments = [segment.replace("-", "_") for segment in module.split("/") if segment]
    return "::".join(segments) or UNKNOWN_MODULE


def error_type(row: dict[str, object]) -> str:
    """Classify one catalog entry into exactly one semantic category."""

    text = message_for_row(row)
    for category, pattern in RULES:
        if pattern.search(text):
            return category

    # Generic messages such as ``"failed"`` carry too little information by
    # themselves.  A few source-file names are stable domain hints and are only
    # consulted after message rules, so explicit message text always wins.
    source = _normalized_source_path(row.get("file"))
    if source is None:
        return "other"
    matching_categories = {
        category for category, pattern in SOURCE_FALLBACK_RULES if pattern.search(source)
    }
    if len(matching_categories) == 1:
        return matching_categories.pop()
    return "other"


def _exception_flags(row: dict[str, object], *, message: str, module: str) -> list[object]:
    """Retain existing flags and append only deterministic handoff exceptions."""

    existing = row.get("exception_flags", [])
    flags = list(existing) if isinstance(existing, list) else ["invalid_exception_flags"]
    if not message.strip() and "no_original_message" not in flags:
        flags.append("no_original_message")
    if module == UNKNOWN_MODULE and "invalid_source_file" not in flags:
        flags.append("invalid_source_file")
    return flags


def _canonical_extra_fields(row: dict[str, object]) -> str:
    """Make ties deterministic without changing the taxonomy's visible order."""

    extras = {key: value for key, value in row.items() if key not in DERIVED_FIELDS}
    return json.dumps(extras, ensure_ascii=False, sort_keys=True, default=str)


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
        message = message_for_row(row)
        updated["error_type"] = error_type(row)
        updated["source_module"] = source_module(row["file"])
        updated["exception_flags"] = _exception_flags(
            row, message=message, module=str(updated["source_module"])
        )
        categorized.append(updated)
    return sorted(categorized, key=sort_key)


def sort_key(row: dict[str, object]) -> tuple[str, int, str, int, str, str, str]:
    """Sort records for identical, review-friendly JSON and CSV output."""

    try:
        line = int(row["line"])
    except (TypeError, ValueError) as error:
        raise ValueError(f"catalog line must be an integer: {row!r}") from error
    return (
        str(row["source_module"]),
        ERROR_TYPE_RANK.get(str(row["error_type"]), len(ERROR_TYPE_RANK)),
        str(row["file"]),
        line,
        str(row["category"]),
        str(row["line_content"]),
        _canonical_extra_fields(row),
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
    writer.writerows(
        {
            field: json.dumps(row[field], ensure_ascii=False, sort_keys=True)
            if isinstance(row.get(field), (list, dict))
            else row.get(field, "")
            for field in fields
        }
        for row in rows
    )
    atomic_write(path, buffer.getvalue())


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, default=Path("test_error_messages.json"))
    parser.add_argument("--taxonomy", type=Path, default=TAXONOMY_PATH)
    parser.add_argument("--json-output", type=Path, default=Path("categorized_error_catalog.json"))
    parser.add_argument("--csv-output", type=Path)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    global ALLOWED_ERROR_TYPES, ERROR_TYPE_RANK, CATEGORY_DESCRIPTIONS, RULES, SOURCE_FALLBACK_RULES
    taxonomy = load_taxonomy(args.taxonomy)
    ALLOWED_ERROR_TYPES = tuple(taxonomy["allowed_error_types"])
    ERROR_TYPE_RANK = {category: index for index, category in enumerate(ALLOWED_ERROR_TYPES)}
    CATEGORY_DESCRIPTIONS = {
        category: str(details["description"])
        for category, details in taxonomy["categories"].items()
    }
    rules_by_category = {
        rule["category"]: re.compile(rule["pattern"], re.I)
        for rule in taxonomy["message_rules"]
    }
    RULES = tuple(
        (category, rules_by_category[category]) for category in taxonomy["message_rule_precedence"]
    )
    SOURCE_FALLBACK_RULES = tuple(
        (rule["category"], re.compile(rule["pattern"], re.I))
        for rule in taxonomy["source_file_fallback"]["rules"]
    )
    rows = classify_catalog(args.input)
    atomic_write(args.json_output, json.dumps(rows, indent=2, ensure_ascii=False) + "\n")
    if args.csv_output is not None:
        write_csv(rows, args.csv_output)

    print(f"categorized {len(rows)} entries")
    print(f"input: {args.input}")
    print(f"taxonomy: {args.taxonomy}")
    print(f"json_output: {args.json_output}")
    if args.csv_output is not None:
        print(f"csv_output: {args.csv_output}")
    print(
        "classification_command: "
        + " ".join(shlex.quote(argument) for argument in [sys.executable, *sys.argv])
    )
    for category in CATEGORY_DESCRIPTIONS:
        count = sum(row["error_type"] == category for row in rows)
        print(f"{category}: {count}")


if __name__ == "__main__":
    main()
