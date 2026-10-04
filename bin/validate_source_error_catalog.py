#!/usr/bin/env python3
"""Validate the extraction catalog without assigning taxonomy labels.

The extraction step writes ``test_error_messages.json`` as a JSON array.  Its
source schema is intentionally small and lossless:

``file``
    Repository-relative Rust source path.
``line``
    One-based source line number.
``category``
    The extractor's pattern name (not a semantic error taxonomy).
``line_content``
    The source line, including the original Rust string literal.

The validator never rewrites the catalog and never interprets an existing
semantic ``error_type`` value.  It reports records that need an explicit
preservation rule for the taxonomy stage: value-only assertions without an
error message, overlapping matches at one source location, and pre-existing
derived fields such as ``source_module``.
"""

from __future__ import annotations

import argparse
import csv
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any, Iterable


REQUIRED_FIELDS = ("file", "line", "category", "line_content")
EXPECTED_ROOTS = ("hoop-daemon/tests/", "hoop-cli/tests/", "tests/")
DERIVED_FIELDS = ("error_type", "source_module")
HANDOFF_SCHEMA = {
    "record_identity": "(file, line, category, line_content)",
    "source_file": "file: non-empty repository-relative .rs path",
    "source_line": "line: positive one-based integer",
    "extraction_kind": "category: extractor pattern name; do not treat as taxonomy",
    "source_text": "line_content: original extracted source line",
    "original_message": "string when a message argument exists, otherwise null",
    "source_module": "derived crate-like path from file; preserve a supplied matching value",
    "exception_flags": "array; retain records instead of dropping or deduplicating them",
}


def _json_key(record: dict[str, Any]) -> tuple[str, str, str, str]:
    """Return an identity using only fields owned by extraction."""

    return tuple(str(record.get(field, "")) for field in REQUIRED_FIELDS)


def _source_text(value: str) -> str:
    """Undo the extractor's JSON-safe quote escaping for source inspection."""

    return value.replace(r'\"', '"')


def _string_literals(value: str) -> list[str]:
    """Return quoted Rust literals from one source line.

    This is deliberately only a validation heuristic.  It does not parse or
    normalize the message; the source line remains the authoritative payload.
    """

    return re.findall(r'"((?:\\.|[^"\\])*)"', _source_text(value))


def _split_macro_args(value: str) -> list[str]:
    """Split a macro argument list while respecting strings and nesting."""

    parts: list[str] = []
    start = 0
    depth = 0
    in_string = False
    escaped = False
    for index, character in enumerate(value):
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
            parts.append(value[start:index])
            start = index + 1
    parts.append(value[start:])
    return parts


def original_message(line_content: str) -> str | None:
    """Extract a message literal when the source expression has one.

    ``assert_eq!`` and ``assert_ne!`` require a third macro argument for an
    assertion message.  For expectation/error macros the final literal is the
    message.  Returning ``None`` is intentional for value-only assertions;
    those records remain in the handoff with an exception marker.
    """

    source = _source_text(line_content)
    literals = _string_literals(source)
    if not literals:
        return None

    if re.search(r"assert_(?:eq|ne)!\s*\(", source):
        opening = source.find("(")
        closing = source.rfind(")")
        if opening < 0 or closing <= opening:
            return None
        arguments = _split_macro_args(source[opening + 1 : closing])
        if len(arguments) < 3:
            return None
        message_literals = _string_literals(",".join(arguments[2:]))
        return message_literals[-1] if message_literals else None

    return literals[-1]


def _is_nonempty_string(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def _validate_records(records: Any) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    if not isinstance(records, list):
        raise ValueError("catalog top level must be a JSON array")

    report: dict[str, Any] = {
        "records": len(records),
        "handoff_schema": HANDOFF_SCHEMA,
        "schema": {
            "required_fields": list(REQUIRED_FIELDS),
            "extra_fields": [],
            "field_presence": {},
        },
        "empty": {
            "records_with_empty_required_field": 0,
            "by_field": Counter(),
            "empty_original_message": 0,
        },
        "malformed": {
            "records": 0,
            "by_reason": Counter(),
        },
        "duplicates": {
            "exact_source_records": 0,
            "exact_source_record_groups": 0,
            "source_location_collision_records": 0,
            "source_location_collision_groups": 0,
        },
        "exceptions": {
            "records_without_original_message": 0,
            "comment_or_doc_records": 0,
            "missing_source_module": 0,
            "source_module_mismatches": 0,
            "preexisting_error_type_fields": 0,
        },
        "message_check": {
            "records_with_original_message": 0,
            "records_without_original_message": 0,
        },
    }

    objects: list[dict[str, Any]] = []
    all_fields: set[str] = set()
    for record in records:
        reasons: list[str] = []
        if not isinstance(record, dict):
            report["malformed"]["records"] += 1
            report["malformed"]["by_reason"]["record_not_object"] += 1
            continue

        all_fields.update(record)
        for field in REQUIRED_FIELDS:
            if field not in record:
                report["schema"]["field_presence"].setdefault(field, {"present": 0, "missing": 0})["missing"] += 1
                reasons.append(f"missing_{field}")
            else:
                report["schema"]["field_presence"].setdefault(field, {"present": 0, "missing": 0})["present"] += 1

        if "file" in record:
            if not _is_nonempty_string(record["file"]):
                reasons.append("file_not_nonempty_string")
            elif record["file"].startswith("/") or ".." in Path(record["file"]).parts:
                reasons.append("file_not_repository_relative")
            elif not record["file"].endswith(".rs"):
                reasons.append("file_not_rust_source")
            elif not record["file"].startswith(EXPECTED_ROOTS):
                reasons.append("file_outside_extraction_roots")

        if "line" in record:
            line = record["line"]
            if isinstance(line, bool) or not isinstance(line, int) or line < 1:
                reasons.append("line_not_positive_integer")

        for field in ("category", "line_content"):
            if field in record and not _is_nonempty_string(record[field]):
                reasons.append(f"{field}_not_nonempty_string")

        if "source_module" in record:
            if not _is_nonempty_string(record["source_module"]):
                reasons.append("source_module_not_nonempty_string")
            elif _is_nonempty_string(record.get("file")):
                expected = record["file"].replace("\\", "/")
                if expected.endswith(".rs"):
                    expected = expected[:-3]
                expected = expected.replace("-", "_").replace("/", "::")
                if record["source_module"] != expected:
                    report["exceptions"]["source_module_mismatches"] += 1

        if "error_type" in record:
            report["exceptions"]["preexisting_error_type_fields"] += 1

        if reasons:
            report["malformed"]["records"] += 1
            for reason in reasons:
                report["malformed"]["by_reason"][reason] += 1
        else:
            objects.append(record)

    report["schema"]["extra_fields"] = sorted(all_fields - set(REQUIRED_FIELDS))
    report["schema"]["field_presence"] = {
        field: dict(values)
        for field, values in sorted(report["schema"]["field_presence"].items())
    }

    for field in REQUIRED_FIELDS:
        empty_count = sum(
            isinstance(record, dict)
            and field in record
            and (record[field] is None or (isinstance(record[field], str) and not record[field].strip()))
            for record in records
        )
        report["empty"]["by_field"][field] = empty_count
    report["empty"]["records_with_empty_required_field"] = sum(
        1 for record in records
        if isinstance(record, dict)
        and any(
            field in record
            and (record[field] is None or (isinstance(record[field], str) and not record[field].strip()))
            for field in REQUIRED_FIELDS
        )
    )

    exact_counts = Counter(_json_key(record) for record in objects)
    exact_groups = [count for count in exact_counts.values() if count > 1]
    report["duplicates"]["exact_source_record_groups"] = len(exact_groups)
    report["duplicates"]["exact_source_records"] = sum(count - 1 for count in exact_groups)

    by_location: defaultdict[tuple[str, str], int] = defaultdict(int)
    for record in objects:
        by_location[(str(record["file"]), str(record["line"]))] += 1
    location_groups = [count for count in by_location.values() if count > 1]
    report["duplicates"]["source_location_collision_groups"] = len(location_groups)
    report["duplicates"]["source_location_collision_records"] = sum(count - 1 for count in location_groups)

    for record in objects:
        message = original_message(record["line_content"])
        if message is None:
            report["exceptions"]["records_without_original_message"] += 1
            report["message_check"]["records_without_original_message"] += 1
        else:
            report["message_check"]["records_with_original_message"] += 1
            if not message.strip():
                report["empty"]["empty_original_message"] += 1
        if _source_text(record["line_content"]).lstrip().startswith(("//", "/*", "*")):
            report["exceptions"]["comment_or_doc_records"] += 1
        if "source_module" not in record:
            report["exceptions"]["missing_source_module"] += 1

    return report, objects


def _load_csv(path: Path) -> tuple[list[dict[str, str]], dict[str, Any]]:
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        if reader.fieldnames is None:
            raise ValueError(f"CSV catalog has no header: {path}")
        rows = list(reader)
        return rows, {"fields": reader.fieldnames, "records": len(rows)}


def _compare_csv(report: dict[str, Any], objects: Iterable[dict[str, Any]], path: Path) -> None:
    rows, metadata = _load_csv(path)
    expected = Counter(_json_key(record) for record in objects)
    actual = Counter(tuple(row.get(field, "") for field in REQUIRED_FIELDS) for row in rows)
    report["csv"] = {
        **metadata,
        "required_fields_present": all(field in metadata["fields"] for field in REQUIRED_FIELDS),
        "same_source_records": expected == actual,
        "missing_source_records": sum((expected - actual).values()),
        "extra_source_records": sum((actual - expected).values()),
    }


def _print_text(report: dict[str, Any], input_path: Path) -> None:
    schema = report["schema"]
    print(f"source_catalog: {input_path}")
    print("authoritative_format: JSON array of source records")
    print(f"records: {report['records']}")
    print(f"required_fields: {', '.join(schema['required_fields'])}")
    print("handoff_schema: source_file=file, source_line=line, extraction_kind=category, source_text=line_content, original_message=string|null, source_module=derived, exception_flags=array")
    print(f"extra_fields_ignored_for_source_validation: {', '.join(schema['extra_fields']) or '(none)'}")
    print(f"empty_required_field_records: {report['empty']['records_with_empty_required_field']}")
    print(f"malformed_records: {report['malformed']['records']}")
    print(f"exact_duplicate_records: {report['duplicates']['exact_source_records']}")
    print(f"source_location_collision_groups: {report['duplicates']['source_location_collision_groups']}")
    print(f"source_location_collision_extra_records: {report['duplicates']['source_location_collision_records']}")
    print(f"records_with_original_message: {report['message_check']['records_with_original_message']}")
    print(f"records_without_original_message: {report['message_check']['records_without_original_message']}")
    print(f"empty_original_message_records: {report['empty']['empty_original_message']}")
    print(f"comment_or_doc_records: {report['exceptions']['comment_or_doc_records']}")
    print(f"missing_source_module_records: {report['exceptions']['missing_source_module']}")
    print(f"source_module_mismatches: {report['exceptions']['source_module_mismatches']}")
    print(f"preexisting_error_type_fields: {report['exceptions']['preexisting_error_type_fields']}")
    if "csv" in report:
        csv_report = report["csv"]
        print(f"csv_projection_records: {csv_report['records']}")
        print(f"csv_projection_matches_json: {csv_report['same_source_records']}")
    print("handoff_identity: (file, line, category, line_content)")
    print("handoff_exception_policy: preserve every record; use null original_message plus exception flags when needed")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, default=Path("test_error_messages.json"))
    parser.add_argument("--csv", type=Path, help="Optional CSV projection to compare with the JSON source")
    parser.add_argument("--json", action="store_true", dest="as_json", help="Emit the report as JSON")
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Return non-zero for any malformed, empty, duplicate, message-less, or mismatched record",
    )
    args = parser.parse_args()

    try:
        records = json.loads(args.input.read_text(encoding="utf-8"))
        report, objects = _validate_records(records)
        csv_path = args.csv
        if csv_path is None:
            sibling = args.input.with_suffix(".csv")
            csv_path = sibling if sibling.exists() else None
        if csv_path is not None:
            _compare_csv(report, objects, csv_path)
    except (OSError, json.JSONDecodeError, ValueError, csv.Error) as error:
        print(f"catalog validation failed: {error}", file=sys.stderr)
        return 2

    if args.as_json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        _print_text(report, args.input)

    if not args.strict:
        return 0

    exceptions = report["exceptions"]
    has_problem = any(
        (
            report["empty"]["records_with_empty_required_field"],
            report["empty"]["empty_original_message"],
            report["malformed"]["records"],
            report["duplicates"]["exact_source_records"],
            report["duplicates"]["source_location_collision_records"],
            report["message_check"]["records_without_original_message"],
            exceptions["missing_source_module"],
            exceptions["source_module_mismatches"],
            report.get("csv", {}).get("same_source_records", True) is False,
        )
    )
    return 1 if has_problem else 0


if __name__ == "__main__":
    raise SystemExit(main())
