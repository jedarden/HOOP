#!/usr/bin/env python3
"""Validate and export the categorized error catalog release candidate.

The source catalog owns the record identity.  This command validates the
classified intermediate against that identity, cross-checks the previous
child's source-module grouped handoff, checks the closed taxonomy and derived
source module, then writes a deterministic JSON and CSV projection.  Records
without an assertion/error message are retained with the explicit
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
from collections import Counter, defaultdict
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


def taxonomy_allowed(path: Path) -> tuple[set[str], list[str], dict[str, int]]:
    taxonomy = json.loads(path.read_text(encoding="utf-8"))
    allowed = taxonomy.get("allowed_error_types") if isinstance(taxonomy, dict) else None
    if not isinstance(allowed, list) or not all(isinstance(value, str) for value in allowed):
        raise ValueError(f"taxonomy has no valid allowed_error_types: {path}")
    ranks = {value: index for index, value in enumerate(allowed)}
    return set(allowed), allowed, ranks


def load_grouped(path: Path) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    """Flatten the previous child's grouped handoff back into records.

    The grouped document nests complete records as
    ``source_module -> error_type -> records`` and never reduced summaries, so
    the export can validate itself against it without a join.
    """

    doc = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(doc, dict):
        raise ValueError(f"grouped catalog must be a JSON object: {path}")
    modules = doc.get("modules")
    if not isinstance(modules, dict) or not modules:
        raise ValueError(f"grouped catalog has no module entries: {path}")
    records: list[dict[str, Any]] = []
    for module_name, entry in modules.items():
        if not isinstance(entry, dict):
            raise ValueError(f"grouped module {module_name!r} is not an object: {path}")
        nested = entry.get("records")
        if not isinstance(nested, dict) or not nested:
            raise ValueError(
                f"grouped module {module_name!r} has no error-type records: {path}"
            )
        for error_type, rows in nested.items():
            if not isinstance(rows, list) or not all(isinstance(row, dict) for row in rows):
                raise ValueError(
                    f"grouped module {module_name!r} type {error_type!r} is not a record array: {path}"
                )
            records.extend(rows)
    return records, doc


def validate_grouped(
    final_rows: list[dict[str, Any]],
    grouped_records: list[dict[str, Any]],
    grouped_doc: dict[str, Any],
) -> dict[str, Any]:
    """Cross-check the final rows against the source-module grouped handoff.

    Retains the handoff's standing rule: a nonzero
    ``unresolved_module_mappings.count`` is a handoff exception to resolve or
    flag, never data to drop, so the export refuses to publish while it is set.
    """

    errors: list[str] = []
    grouped_ids = identity_counts(grouped_records)
    final_ids = identity_counts(final_rows)
    if grouped_ids != final_ids:
        errors.append(
            "export does not retain every grouped record exactly once: "
            f"grouped={len(grouped_records)} export={len(final_rows)}"
        )

    grouped_fields: dict[tuple[str, str, str, str], list[tuple[str, str]]] = defaultdict(list)
    for row in grouped_records:
        key = identity(row)
        grouped_fields[key].append(
            (str(row.get("error_type", "")), str(row.get("source_module", "")))
        )
    for row in final_rows:
        key = identity(row)
        derived = (str(row.get("error_type", "")), str(row.get("source_module", "")))
        candidates = grouped_fields.get(key, [])
        if derived in candidates:
            candidates.remove(derived)
        else:
            errors.append(
                f"export record {key} disagrees with the grouped handoff on "
                f"error_type/source_module: {derived!r}"
            )

    type_counts = Counter(str(row.get("error_type", "")) for row in final_rows)
    module_counts = Counter(str(row.get("source_module", "")) for row in final_rows)
    counts_match = True
    for name, actual in (
        ("counts_by_error_type", type_counts),
        ("counts_by_source_module", module_counts),
    ):
        # The grouped handoff zero-fills taxonomy types that have no records;
        # compare semantic content, so explicit zeros on either side are equal.
        expected = grouped_doc.get(name)
        if not isinstance(expected, dict) or {
            name_: count for name_, count in actual.items() if count
        } != {name_: count for name_, count in expected.items() if count}:
            errors.append(f"{name} does not match the grouped handoff")
            counts_match = False
    if grouped_doc.get("record_count") != len(final_rows):
        errors.append(
            f"record_count mismatch: grouped={grouped_doc.get('record_count')} export={len(final_rows)}"
        )
    modules = grouped_doc.get("modules")
    grouped_module_count = len(modules) if isinstance(modules, dict) else None
    if grouped_doc.get("module_count") != len(module_counts) or grouped_module_count != len(
        module_counts
    ):
        errors.append(
            f"module_count mismatch: grouped={grouped_doc.get('module_count')} export={len(module_counts)}"
        )

    unresolved = grouped_doc.get("unresolved_module_mappings")
    unresolved_count: Any = None
    if not isinstance(unresolved, dict):
        errors.append("grouped handoff does not report unresolved_module_mappings")
        unresolved = None
    else:
        unresolved_count = unresolved.get("count")
        if unresolved_count != 0:
            errors.append(
                "grouped handoff has unresolved source-module mappings "
                f"(count={unresolved_count!r}) - a handoff exception to resolve or "
                "flag, never data to drop"
            )

    return {
        "grouped_records": len(grouped_records),
        "grouped_module_count": grouped_module_count,
        "identity_parity": grouped_ids == final_ids,
        "derived_fields_agree": not any(
            "disagrees with the grouped handoff" in error for error in errors
        ),
        "counts_match": counts_match,
        "unresolved_module_mappings": unresolved,
        "unresolved_count": unresolved_count,
        "errors": errors,
    }


def final_counts(rows: list[dict[str, Any]], taxonomy_order: list[str]) -> dict[str, Any]:
    """Summarize and validate the category/module counts of the final rows."""

    type_counts = Counter(str(row.get("error_type", "")) for row in rows)
    module_counts = Counter(str(row.get("source_module", "")) for row in rows)
    return {
        "record_count": len(rows),
        "module_count": len(module_counts),
        "counts_by_error_type": {name: type_counts.get(name, 0) for name in taxonomy_order},
        "counts_by_source_module": dict(sorted(module_counts.items())),
        "type_counts_sum_to_record_count": sum(type_counts.values()) == len(rows),
        "module_counts_sum_to_record_count": sum(module_counts.values()) == len(rows),
    }


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
    """Return a stable header that includes every final-record field.

    The required fields have a documented consumer-facing order.  Any
    extraction metadata beyond that contract is appended lexicographically so
    CSV remains lossless and reruns do not depend on dictionary insertion
    order.
    """

    preferred = FINAL_REQUIRED_FIELDS + ("exception_flags",)
    fields = [field for field in preferred if any(field in row for row in rows)]
    extras = sorted({field for row in rows for field in row if field not in fields})
    return fields + extras


def csv_cell(row: dict[str, Any], field: str) -> str:
    """Render one JSON value exactly as :func:`write_csv` renders it."""

    value = row.get(field, "")
    if isinstance(value, (list, dict)):
        return json.dumps(value, ensure_ascii=False, sort_keys=True)
    return "" if value is None else str(value)


def write_csv(rows: list[dict[str, Any]], path: Path) -> None:
    from io import StringIO

    fields = csv_fields(rows)
    buffer = StringIO(newline="")
    writer = csv.DictWriter(buffer, fieldnames=fields, lineterminator="\n")
    writer.writeheader()
    for row in rows:
        writer.writerow({field: csv_cell(row, field) for field in fields})
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

    expected_header = csv_fields(json_rows)
    if csv_header != expected_header:
        errors.append(
            "CSV header does not preserve the JSON fields in deterministic order: "
            f"expected {expected_header!r}, got {csv_header!r}"
        )

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

    # Identity parity alone would allow a CSV row to retain the same location
    # while silently changing its category, source module, message, or
    # metadata.  Compare every emitted field against the exact CSV rendering
    # of the JSON row, preserving structured values through their canonical
    # JSON encoding.
    if len(json_rows) == len(csv_rows):
        for index, (json_row, csv_row) in enumerate(zip(json_rows, csv_rows), start=1):
            for field in expected_header:
                expected_cell = csv_cell(json_row, field)
                actual_cell = csv_row.get(field, "")
                if actual_cell != expected_cell:
                    errors.append(
                        f"JSON/CSV value mismatch at row {index}, field {field!r}: "
                        f"expected {expected_cell!r}, got {actual_cell!r}"
                    )

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
        "json_csv_values_equivalent": not any(
            "JSON/CSV value mismatch" in error for error in errors
        ),
        "csv_preserves_json_fields": csv_header == expected_header,
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
        "--grouped",
        type=Path,
        default=Path("docs/categorized_error_catalog_by_module.json"),
        help="source-module grouped handoff to cross-check the export against",
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
        allowed, taxonomy_order, ranks = taxonomy_allowed(args.taxonomy)
        intermediate_report = validate_intermediate(source, intermediate, allowed)
        if intermediate_report["errors"]:
            raise ValueError("intermediate validation failed: " + "; ".join(intermediate_report["errors"][:5]))

        final_rows = build_final_rows(intermediate, ranks)

        grouped_report = None
        if args.grouped is not None:
            grouped_records, grouped_doc = load_grouped(args.grouped)
            grouped_report = validate_grouped(final_rows, grouped_records, grouped_doc)
            if grouped_report["errors"]:
                raise ValueError(
                    "grouped handoff validation failed: "
                    + "; ".join(grouped_report["errors"][:5])
                )

        counts = final_counts(final_rows, taxonomy_order)
        if not (
            counts["type_counts_sum_to_record_count"]
            and counts["module_counts_sum_to_record_count"]
        ):
            raise ValueError("final category or module counts do not sum to the record count")

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

    command = [
        "bin/validate_and_export_error_catalog.py",
        "--source",
        str(args.source),
        "--intermediate",
        str(args.intermediate),
    ]
    if args.grouped is not None:
        command.extend(["--grouped", str(args.grouped)])
    command.extend(
        [
            "--taxonomy",
            str(args.taxonomy),
            "--json-output",
            str(args.json_output),
            "--csv-output",
            str(args.csv_output),
        ]
    )
    report = {
        "source": str(args.source),
        "intermediate": str(args.intermediate),
        "grouped": str(args.grouped) if args.grouped is not None else None,
        "outputs": [str(args.json_output), str(args.csv_output)],
        "taxonomy": str(args.taxonomy),
        "command": "python3 " + " ".join(command),
        "intermediate_validation": intermediate_report,
        "grouped_validation": grouped_report,
        "final_counts": counts,
        "output_validation": output_report,
        "exit_code": 0,
    }
    if args.report_output is not None:
        atomic_write(args.report_output, json.dumps(report, indent=2, sort_keys=True) + "\n")
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
