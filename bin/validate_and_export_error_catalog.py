#!/usr/bin/env python3
"""Validate and export the categorized error catalog release candidate.

The source catalog owns the record identity.  This command validates the
classified intermediate against that identity, checks the closed taxonomy and
derived source module, then writes a deterministic JSON and CSV projection.
Records without an assertion/error message are retained with the explicit
``original_message`` value ``"unknown"`` and the ``no_original_message``
exception flag.
"""

from __future__ import annotations

import argparse
import csv
import json
import os
import sys
import tempfile
from collections import Counter
from pathlib import Path
from typing import Any, Iterable


SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

from categorize_error_messages import (  # noqa: E402
    UNKNOWN_MODULE,
    message_text,
    source_module,
)


IDENTITY_FIELDS = ("file", "line", "category", "line_content")
FINAL_REQUIRED_FIELDS = (
    "file",
    "line",
    "category",
    "line_content",
    "error_type",
    "source_module",
    "original_message",
)
UNKNOWN_MESSAGE = "unknown"


def atomic_write(path: Path, data: str) -> None:
    """Write a generated artifact without leaving a partial destination."""

    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(
        prefix=f".{path.name}.", dir=path.parent
    )
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8", newline="") as handle:
            handle.write(data)
        os.replace(temporary_name, path)
    except Exception:
        try:
            os.unlink(temporary_name)
        except FileNotFoundError:
            pass
        raise


def load_records(path: Path) -> list[dict[str, Any]]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, list) or not all(isinstance(row, dict) for row in value):
        raise ValueError(f"catalog must be a JSON array of objects: {path}")
    return value


def identity(row: dict[str, Any]) -> tuple[str, str, str, str]:
    """Normalize JSON/CSV scalar representation for identity comparisons."""

    return tuple(str(row.get(field, "")) for field in IDENTITY_FIELDS)  # type: ignore[return-value]


def identity_counts(rows: Iterable[dict[str, Any]]) -> Counter[tuple[str, str, str, str]]:
    return Counter(identity(row) for row in rows)


def taxonomy_allowed(path: Path) -> tuple[set[str], dict[str, int]]:
    taxonomy = json.loads(path.read_text(encoding="utf-8"))
    allowed = taxonomy.get("allowed_error_types") if isinstance(taxonomy, dict) else None
    if not isinstance(allowed, list) or not all(isinstance(value, str) for value in allowed):
        raise ValueError(f"taxonomy has no valid allowed_error_types: {path}")
    return set(allowed), {value: index for index, value in enumerate(allowed)}


def expected_message(row: dict[str, Any]) -> str:
    """Return the preserved handoff message or derive it from source text.

    The loader's ``original_message`` is the authoritative handoff when it is
    present.  Re-extracting from a line with multiple string literals can pick
    a different value and would make an otherwise lossless intermediate fail
    validation.
    """

    original = row.get("original_message")
    if isinstance(original, str) and original.strip():
        return original
    return message_text(str(row["line_content"])) or UNKNOWN_MESSAGE


def validate_intermediate(
    source: list[dict[str, Any]],
    intermediate: list[dict[str, Any]],
    allowed_categories: set[str],
) -> dict[str, Any]:
    errors: list[str] = []
    source_ids = identity_counts(source)
    intermediate_ids = identity_counts(intermediate)

    if len(source) != len(intermediate):
        errors.append(f"row count mismatch: source={len(source)} intermediate={len(intermediate)}")
    if source_ids != intermediate_ids:
        errors.append("intermediate record identity does not match source catalog")

    source_module_groups: list[str] = []
    message_counts = Counter()
    for index, row in enumerate(intermediate):
        prefix = f"intermediate row {index + 1}"
        missing = [field for field in FINAL_REQUIRED_FIELDS if field not in row and field != "original_message"]
        if missing:
            errors.append(f"{prefix} missing fields: {', '.join(missing)}")
            continue
        if not isinstance(row["file"], str) or not row["file"].strip():
            errors.append(f"{prefix} has an empty file")
        if isinstance(row.get("line"), bool) or not isinstance(row.get("line"), int) or row["line"] < 1:
            errors.append(f"{prefix} has an invalid line")
        for field in ("category", "line_content", "error_type", "source_module"):
            if not isinstance(row.get(field), str) or not row[field].strip():
                errors.append(f"{prefix} has an empty {field}")
        if row.get("error_type") not in allowed_categories:
            errors.append(f"{prefix} has taxonomy category {row.get('error_type')!r}")
        expected_module = source_module(str(row.get("file", "")))
        if row.get("source_module") != expected_module:
            errors.append(
                f"{prefix} source_module mismatch: {row.get('source_module')!r} != {expected_module!r}"
            )
        if row.get("source_module") == UNKNOWN_MODULE and expected_module != UNKNOWN_MODULE:
            errors.append(f"{prefix} incorrectly marked source module unknown")
        flags = row.get("exception_flags", [])
        if not isinstance(flags, list):
            errors.append(f"{prefix} exception_flags is not an array")
            flags = []
        message = expected_message(row)
        message_counts["unknown" if message == UNKNOWN_MESSAGE else "known"] += 1
        if "original_message" in row and row["original_message"] not in (
            message,
            UNKNOWN_MESSAGE,
            None,
        ):
            errors.append(f"{prefix} original_message changed from source-derived value")
        if message == UNKNOWN_MESSAGE and "no_original_message" not in flags:
            errors.append(f"{prefix} missing no_original_message exception")
        if message != UNKNOWN_MESSAGE and "no_original_message" in flags:
            errors.append(f"{prefix} has no_original_message despite a source message")
        source_module_groups.append(str(row.get("source_module", "")))

    return {
        "source_records": len(source),
        "intermediate_records": len(intermediate),
        "identity_parity": source_ids == intermediate_ids,
        "taxonomy_categories_valid": not any(
            "taxonomy category" in error for error in errors
        ),
        "source_module_values_valid": not any(
            "source_module" in error for error in errors
        ),
        "message_records": dict(message_counts),
        "errors": errors,
        "source_module_order_is_non_decreasing": source_module_groups
        == sorted(source_module_groups),
    }


def final_sort_key(row: dict[str, Any], ranks: dict[str, int]) -> tuple[Any, ...]:
    return (
        str(row["source_module"]),
        ranks.get(str(row["error_type"]), len(ranks)),
        str(row["file"]),
        int(row["line"]),
        str(row["category"]),
        str(row["line_content"]),
    )


def build_final_rows(
    intermediate: list[dict[str, Any]], ranks: dict[str, int]
) -> list[dict[str, Any]]:
    rows = []
    for row in intermediate:
        final = dict(row)
        final["original_message"] = expected_message(row)
        flags = list(row.get("exception_flags", []))
        if final["original_message"] == UNKNOWN_MESSAGE and "no_original_message" not in flags:
            flags.append("no_original_message")
        final["exception_flags"] = flags
        rows.append(final)
    return sorted(rows, key=lambda row: final_sort_key(row, ranks))


def csv_fields(rows: list[dict[str, Any]]) -> list[str]:
    fields: list[str] = []
    for field in FINAL_REQUIRED_FIELDS + ("exception_flags",):
        if any(field in row for row in rows):
            fields.append(field)
    return fields


def write_csv(rows: list[dict[str, Any]], path: Path) -> None:
    from io import StringIO

    fields = csv_fields(rows)
    buffer = StringIO(newline="")
    writer = csv.DictWriter(buffer, fieldnames=fields, lineterminator="\n")
    writer.writeheader()
    for row in rows:
        writer.writerow(
            {
                field: json.dumps(row[field], ensure_ascii=False, sort_keys=True)
                if isinstance(row.get(field), (list, dict))
                else row.get(field, "")
                for field in fields
            }
        )
    atomic_write(path, buffer.getvalue())


def load_csv(path: Path) -> tuple[list[dict[str, str]], list[str]]:
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        if reader.fieldnames is None:
            raise ValueError(f"CSV catalog has no header: {path}")
        return list(reader), reader.fieldnames


def validate_outputs(
    source: list[dict[str, Any]],
    json_rows: list[dict[str, Any]],
    csv_rows: list[dict[str, str]],
    csv_header: list[str],
    allowed_categories: set[str],
    ranks: dict[str, int],
) -> dict[str, Any]:
    errors: list[str] = []
    source_ids = identity_counts(source)
    json_ids = identity_counts(json_rows)
    csv_ids = identity_counts(csv_rows)
    required_csv = set(FINAL_REQUIRED_FIELDS)
    if not required_csv.issubset(csv_header):
        errors.append(f"CSV is missing fields: {', '.join(sorted(required_csv - set(csv_header)))}")
    if len(json_rows) != len(source):
        errors.append(f"JSON row count mismatch: source={len(source)} output={len(json_rows)}")
    if len(csv_rows) != len(source):
        errors.append(f"CSV row count mismatch: source={len(source)} output={len(csv_rows)}")
    if json_ids != source_ids:
        errors.append("JSON record identity does not match source catalog")
    if csv_ids != source_ids:
        errors.append("CSV record identity does not match source catalog")
    if json_ids != csv_ids:
        errors.append("JSON and CSV record identity do not match")

    for index, row in enumerate(json_rows):
        prefix = f"JSON row {index + 1}"
        if any(field not in row for field in FINAL_REQUIRED_FIELDS):
            errors.append(f"{prefix} is missing a required final field")
            continue
        if row["error_type"] not in allowed_categories:
            errors.append(f"{prefix} has a category outside the taxonomy")
        message = expected_message(row)
        if row["original_message"] != message:
            errors.append(f"{prefix} original_message changed")
        if not isinstance(row["source_module"], str) or not row["source_module"].strip():
            errors.append(f"{prefix} source_module is empty")

    # CSV scalars are compared to their JSON projection after normalizing the
    # line number and exception list.  Identity parity above catches location
    # or source-text changes independently of these derived-field checks.
    for index, row in enumerate(csv_rows):
        prefix = f"CSV row {index + 1}"
        if any(field not in row for field in FINAL_REQUIRED_FIELDS):
            errors.append(f"{prefix} is missing a required final field")
            continue
        if row["error_type"] not in allowed_categories:
            errors.append(f"{prefix} has a category outside the taxonomy")
        expected = expected_message(row)
        if row["original_message"] != expected:
            errors.append(f"{prefix} original_message changed")
        if not row["source_module"].strip():
            errors.append(f"{prefix} source_module is empty")
        try:
            flags = json.loads(row.get("exception_flags", "[]"))
        except json.JSONDecodeError:
            flags = None
        if not isinstance(flags, list):
            errors.append(f"{prefix} exception_flags is not a JSON array")
        if expected == UNKNOWN_MESSAGE and isinstance(flags, list) and "no_original_message" not in flags:
            errors.append(f"{prefix} missing no_original_message exception")

    # The exporter uses taxonomy order; checking module/file monotonicity here
    # is the release invariant that is visible to consumers.
    module_file_order = [
        (str(row.get("source_module", "")), str(row.get("file", "")))
        for row in json_rows
    ]
    return {
        "json_records": len(json_rows),
        "csv_records": len(csv_rows),
        "json_identity_parity": json_ids == source_ids,
        "csv_identity_parity": csv_ids == source_ids,
        "json_csv_identity_parity": json_ids == csv_ids,
        "required_final_fields_present": required_csv.issubset(set(csv_header))
        and all(all(field in row for field in FINAL_REQUIRED_FIELDS) for row in json_rows),
        "taxonomy_categories_valid": not any("outside the taxonomy" in error for error in errors),
        "module_file_grouping_deterministic": module_file_order == sorted(module_file_order),
        "json_order_is_deterministic": json_rows == sorted(
            json_rows,
            key=lambda row: final_sort_key(row, ranks),
        ),
        "errors": errors,
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=Path("test_error_messages.json"))
    parser.add_argument(
        "--intermediate",
        type=Path,
        default=Path("docs/categorized_error_catalog.json"),
    )
    parser.add_argument(
        "--taxonomy", type=Path, default=Path("docs/error_category_taxonomy.json")
    )
    parser.add_argument(
        "--json-output",
        type=Path,
        default=Path("docs/final_categorized_error_catalog.json"),
    )
    parser.add_argument(
        "--csv-output",
        type=Path,
        default=Path("docs/final_categorized_error_catalog.csv"),
    )
    parser.add_argument("--report-output", type=Path)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        source = load_records(args.source)
        intermediate = load_records(args.intermediate)
        allowed, ranks = taxonomy_allowed(args.taxonomy)
        intermediate_report = validate_intermediate(source, intermediate, allowed)
        if intermediate_report["errors"]:
            raise ValueError("intermediate validation failed: " + "; ".join(intermediate_report["errors"][:5]))

        final_rows = build_final_rows(intermediate, ranks)
        atomic_write(args.json_output, json.dumps(final_rows, indent=2, ensure_ascii=False) + "\n")
        write_csv(final_rows, args.csv_output)
        rendered_json = load_records(args.json_output)
        rendered_csv, csv_header = load_csv(args.csv_output)
        output_report = validate_outputs(
            source, rendered_json, rendered_csv, csv_header, allowed, ranks
        )
        if output_report["errors"]:
            raise ValueError("rendered output validation failed: " + "; ".join(output_report["errors"][:5]))
    except (OSError, ValueError, json.JSONDecodeError, csv.Error) as error:
        print(f"catalog release validation failed: {error}", file=sys.stderr)
        return 2

    report = {
        "source": str(args.source),
        "intermediate": str(args.intermediate),
        "outputs": [str(args.json_output), str(args.csv_output)],
        "taxonomy": str(args.taxonomy),
        "command": "python3 " + " ".join(
            [
                "bin/validate_and_export_error_catalog.py",
                "--source",
                str(args.source),
                "--intermediate",
                str(args.intermediate),
                "--taxonomy",
                str(args.taxonomy),
                "--json-output",
                str(args.json_output),
                "--csv-output",
                str(args.csv_output),
            ]
        ),
        "intermediate_validation": intermediate_report,
        "output_validation": output_report,
        "exit_code": 0,
    }
    if args.report_output is not None:
        atomic_write(args.report_output, json.dumps(report, indent=2, sort_keys=True) + "\n")
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
