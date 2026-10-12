#!/usr/bin/env python3
"""Group the preclassified error catalog by stable source module.

The extraction catalog's existing ``category`` field describes the Rust
assertion/extraction pattern (for example, ``expect with failure``).  This
script consumes the upstream ``error_type`` classification and derives
``source_module`` for grouping entries without losing the original source path
and line.  A missing ``error_type`` is classified for compatibility with the
standalone fixture tests, but a supplied value is never rewritten.

Outputs: the flat, deterministically sorted catalog (``--json-output``), the
same records nested ``source_module -> error_type`` with per-module and
per-type counts (``--grouped-output``), and the audit report with counts,
field coverage, and unresolved module mappings (``--report-output``).

The classifier is deliberately single-label: rules are evaluated in the
documented taxonomy order, so a message that mentions both a timeout and a
network operation is categorized as ``timeout``.  ``other`` is the explicit
fallback for messages whose meaning cannot be inferred reliably from the
catalog entry.
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
from collections import Counter
from copy import deepcopy
from pathlib import Path

from load_error_catalog import load_catalog


DERIVED_FIELDS = ("error_type", "source_module", "exception_flags")
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
    module_normalization = taxonomy.get("source_module_normalization")
    if not isinstance(allowed, list) or not allowed or not all(
        isinstance(value, str) and value for value in allowed
    ):
        raise ValueError(f"taxonomy has no valid allowed_error_types: {path}")
    if len(set(allowed)) != len(allowed):
        raise ValueError(f"taxonomy has no valid allowed_error_types: {path}")
    if (
        not isinstance(precedence, list)
        or not all(isinstance(value, str) and value for value in precedence)
    ):
        raise ValueError(f"taxonomy has an invalid message_rule_precedence: {path}")
    if len(set(precedence)) != len(precedence) or set(precedence) != set(allowed) - {"other"}:
        raise ValueError(f"taxonomy has an invalid message_rule_precedence: {path}")
    categories = taxonomy.get("categories")
    if not isinstance(categories, dict) or set(categories) != set(allowed):
        raise ValueError(f"taxonomy categories do not match allowed_error_types: {path}")
    rule_categories = [rule.get("category") for rule in rules] if isinstance(rules, list) else []
    if (
        not isinstance(rules, list)
        or not all(
            isinstance(rule, dict)
            and isinstance(rule.get("pattern"), str)
            and isinstance(rule.get("category"), str)
            for rule in rules
        )
    ):
        raise ValueError(f"taxonomy is missing classification rules: {path}")
    if len(rule_categories) != len(set(rule_categories)) or set(rule_categories) != set(precedence):
        raise ValueError(f"taxonomy is missing classification rules: {path}")
    if not isinstance(fallback, dict) or fallback.get("category", "other") != "other":
        raise ValueError(f"taxonomy is missing the explicit other fallback: {path}")
    if not isinstance(module_normalization, dict):
        raise ValueError(f"taxonomy is missing source-module normalization: {path}")
    module_fallback = module_normalization.get("fallback_value")
    if not isinstance(module_fallback, str) or not module_fallback.strip():
        raise ValueError(f"taxonomy has no valid source-module fallback value: {path}")
    return taxonomy


TAXONOMY = load_taxonomy()
UNKNOWN_MODULE = str(TAXONOMY["source_module_normalization"]["fallback_value"])
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
FALLBACK_ERROR_TYPE = "other"
REPOSITORY_ROOT = Path(__file__).resolve().parents[1]


def classification_details(
    message: str, source_file: object
) -> tuple[str, str, tuple[str, ...]]:
    """Return the label, decision path, and any competing source hints.

    The decision path is reporting metadata only.  ``RULES`` remains the
    normative ordered decision list, and the first matching message rule is
    still the sole source of the returned label.
    """

    for category, pattern in RULES:
        if pattern.search(message):
            return category, f"message:{category}", ()

    source = _normalized_source_path(source_file)
    if source is None:
        return FALLBACK_ERROR_TYPE, "fallback:invalid_source_file", ()

    matching_categories = tuple(
        sorted({category for category, pattern in SOURCE_FALLBACK_RULES if pattern.search(source)})
    )
    if len(matching_categories) == 1:
        return matching_categories[0], "source_file:unique_hint", matching_categories
    if len(matching_categories) > 1:
        return FALLBACK_ERROR_TYPE, "fallback:ambiguous_source_hints", matching_categories
    return FALLBACK_ERROR_TYPE, "fallback:no_matching_rule", ()


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


def source_module(source_file: object) -> str:
    """Return a stable crate-like module path while preserving ``file``."""

    normalized = _normalized_source_path(source_file)
    if normalized is None:
        return UNKNOWN_MODULE
    module = normalized[:-3]
    segments = [segment.replace("-", "_") for segment in module.split("/") if segment]
    return "::".join(segments) or UNKNOWN_MODULE


def classify_message(message: str, source_file: object) -> str:
    """Classify message context into one approved, deterministic category.

    Message rules are intentionally evaluated as an ordered decision list. A
    message can match several patterns, but returning on the first match makes
    the result mutually exclusive and keeps precedence independent of the
    source-file hint. Source fallback is considered only when no message rule
    matches; conflicting source hints resolve to the documented fallback.
    """

    return classification_details(message, source_file)[0]


def error_type(row: dict[str, object]) -> str:
    """Classify one loaded catalog entry into exactly one semantic category."""

    return classify_message(message_for_row(row), row.get("file"))


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


def display_path(path: Path) -> str:
    """Use repository-relative paths in committed run metadata when possible."""

    resolved = Path(path).resolve()
    try:
        return str(resolved.relative_to(REPOSITORY_ROOT))
    except ValueError:
        return str(path)


def classify_catalog(input_path: Path) -> list[dict[str, object]]:
    rows = load_catalog(input_path)

    categorized = []
    for row in rows:
        updated = dict(row)
        message = message_for_row(row)
        if "error_type" not in row:
            # Keep the helper useful for small standalone fixtures, while the
            # real handoff preserves the previous child's classification.
            updated["error_type"] = error_type(row)
        updated["source_module"] = source_module(row.get("file"))
        updated["exception_flags"] = _exception_flags(
            row, message=message, module=str(updated["source_module"])
        )
        categorized.append(updated)
    return sorted(categorized, key=sort_key)


def _record_reference(row: dict[str, object], *, reason: str, hints: tuple[str, ...]) -> dict[str, object]:
    """Identify a report entry with enough provenance to inspect it."""

    reference: dict[str, object] = {
        field: deepcopy(row[field])
        for field in (
            "source_record_index",
            "source_record_id",
            "file",
            "line",
            "category",
            "line_content",
            "original_message",
            "message_status",
            "normalization_flags",
            "exception_flags",
            "error_type",
        )
        if field in row
    }
    reference["reason"] = reason
    if hints:
        reference["competing_source_hints"] = list(hints)
    return reference


def verify_required_derived_fields(rows: list[dict[str, object]]) -> None:
    """Reject the handoff when any record lacks a derived grouping field.

    Downstream grouping and export both key on ``error_type`` and
    ``source_module``, so a record missing either would silently split or
    drop entries.  The check runs before any artifact is written.
    """

    missing: list[str] = []
    for index, row in enumerate(rows):
        for field in ("error_type", "source_module"):
            value = row.get(field)
            if not isinstance(value, str) or not value.strip():
                missing.append(f"record {index + 1} is missing {field}")
    if missing:
        raise ValueError(
            "categorized catalog has records without required derived fields: "
            + "; ".join(missing[:5])
        )


def classification_report(
    rows: list[dict[str, object]],
    *,
    input_path: Path,
    taxonomy_path: Path,
    output_path: Path,
    command: str,
    grouped_output: Path | None = None,
) -> dict[str, object]:
    """Build a deterministic audit report for the categorized handoff."""

    verify_required_derived_fields(rows)
    ordered_rows = sorted(rows, key=sort_key)
    counts = Counter(str(row["error_type"]) for row in ordered_rows)
    unexpected = sorted(set(counts) - set(ALLOWED_ERROR_TYPES))
    if unexpected:
        raise ValueError(f"catalog contains error_type outside taxonomy: {unexpected}")
    fallback_records: list[dict[str, object]] = []
    ambiguous_records: list[dict[str, object]] = []
    for row in ordered_rows:
        message = message_for_row(row)
        classified_category, classified_reason, classified_hints = classification_details(
            message, row.get("file")
        )
        recorded_category = str(row["error_type"])
        if recorded_category == classified_category:
            reason = classified_reason
            hints = classified_hints
        else:
            # The previous catalog child owns semantic classification.  Keep
            # this report diagnostic-only so grouping cannot silently rewrite
            # an upstream label when taxonomy rules evolve.
            reason = f"preclassified:{recorded_category}"
            hints = ()

        flags = set()
        for field in ("normalization_flags", "exception_flags"):
            value = row.get(field)
            if isinstance(value, list):
                flags.update(str(flag) for flag in value)
        message_ambiguous = row.get("message_status") == "ambiguous" or any(
            "ambiguous" in flag for flag in flags
        )
        source_ambiguous = classified_reason == "fallback:ambiguous_source_hints"
        if message_ambiguous or source_ambiguous:
            ambiguous_records.append(
                _record_reference(row, reason=reason, hints=hints)
            )
        if recorded_category == FALLBACK_ERROR_TYPE:
            fallback_records.append(_record_reference(row, reason=reason, hints=hints))

    ordered_counts = {category: counts.get(category, 0) for category in ALLOWED_ERROR_TYPES}
    module_counts = Counter(str(row["source_module"]) for row in ordered_rows)
    unresolved = [
        _record_reference(row, reason="invalid_source_file", hints=())
        for row in ordered_rows
        if str(row["source_module"]) == UNKNOWN_MODULE
    ]
    return {
        "taxonomy": display_path(taxonomy_path),
        "input": display_path(input_path),
        "output": display_path(output_path),
        "classification_command": command,
        "output_format": "JSON array",
        "message_rule_precedence": [category for category, _ in RULES],
        "fallback_error_type": FALLBACK_ERROR_TYPE,
        "error_type_source": "preserved_from_input",
        "record_count": len(ordered_rows),
        "counts_by_error_type": ordered_counts,
        "field_coverage": {
            "records_with_error_type": sum(
                1 for row in ordered_rows if str(row.get("error_type", "")).strip()
            ),
            "records_with_source_module": sum(
                1 for row in ordered_rows if str(row.get("source_module", "")).strip()
            ),
            "record_count": len(ordered_rows),
        },
        "source_modules": {
            "unknown_module": UNKNOWN_MODULE,
            "module_count": len(module_counts),
            "counts_by_source_module": {
                module: module_counts[module] for module in sorted(module_counts)
            },
            "unresolved_module_mappings": {
                "count": len(unresolved),
                "records": unresolved,
            },
        },
        "grouped_output": display_path(grouped_output) if grouped_output is not None else None,
        "fallback": {
            "error_type": FALLBACK_ERROR_TYPE,
            "count": len(fallback_records),
            "records": fallback_records,
        },
        "ambiguous": {
            "count": len(ambiguous_records),
            "records": ambiguous_records,
        },
    }


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


def group_by_source_module(
    rows: list[dict[str, object]],
) -> dict[str, dict[str, list[dict[str, object]]]]:
    """Return a deterministic ``source_module -> error_type -> records`` view.

    The records are the categorized dictionaries themselves, not reduced
    summaries.  This keeps source identity, the extracted message, and any
    unrelated metadata available to every grouping consumer.  Sorting before
    inserting into the nested dictionaries makes both module and error-type
    iteration order stable for the same input.
    """

    grouped: dict[str, dict[str, list[dict[str, object]]]] = {}
    for row in sorted(rows, key=sort_key):
        module = str(row["source_module"])
        category = str(row["error_type"])
        grouped.setdefault(module, {}).setdefault(category, []).append(row)
    return grouped


def module_grouping_document(
    rows: list[dict[str, object]],
    *,
    source_path: Path,
) -> dict[str, object]:
    """Build the grouped intermediate consumed by the final export child.

    The document nests the complete categorized records under
    ``source_module`` and then ``error_type``, so a consumer never has to join
    the grouped view back against the flat catalog.  Modules are sorted
    lexicographically and error types follow taxonomy rank order, which makes
    the serialized document byte-stable for the same input.  Per-module and
    per-type counts accompany the records, and records whose path could not be
    mapped are listed explicitly under ``unresolved_module_mappings`` instead
    of being silently grouped away.
    """

    verify_required_derived_fields(rows)
    grouped = group_by_source_module(rows)
    modules: dict[str, dict[str, object]] = {}
    for module in sorted(grouped):
        by_type = grouped[module]
        ordered_types = sorted(by_type, key=lambda category: (
            ERROR_TYPE_RANK.get(category, len(ERROR_TYPE_RANK)),
            category,
        ))
        modules[module] = {
            "record_count": sum(len(by_type[category]) for category in ordered_types),
            "counts_by_error_type": {
                category: len(by_type[category]) for category in ordered_types
            },
            "records": {category: by_type[category] for category in ordered_types},
        }
    type_counts = Counter(str(row["error_type"]) for row in rows)
    unresolved = [
        _record_reference(row, reason="invalid_source_file", hints=())
        for row in sorted(rows, key=sort_key)
        if str(row["source_module"]) == UNKNOWN_MODULE
    ]
    return {
        "source": display_path(source_path),
        "unknown_module": UNKNOWN_MODULE,
        "record_count": len(rows),
        "module_count": len(modules),
        "counts_by_error_type": {
            category: type_counts.get(category, 0) for category in ALLOWED_ERROR_TYPES
        },
        "counts_by_source_module": {
            module: sum(len(records) for records in grouped[module].values())
            for module in sorted(grouped)
        },
        "unresolved_module_mappings": {
            "count": len(unresolved),
            "records": unresolved,
        },
        "modules": modules,
    }


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
    parser.add_argument(
        "--input", type=Path, default=Path("docs/normalized_error_catalog.json")
    )
    parser.add_argument("--taxonomy", type=Path, default=TAXONOMY_PATH)
    parser.add_argument(
        "--json-output", type=Path, default=Path("docs/categorized_error_catalog.json")
    )
    parser.add_argument(
        "--grouped-output",
        type=Path,
        default=Path("docs/categorized_error_catalog_by_module.json"),
    )
    parser.add_argument("--csv-output", type=Path)
    parser.add_argument(
        "--report-output", type=Path, default=Path("docs/error_category_catalog_run.json")
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    global ALLOWED_ERROR_TYPES, ERROR_TYPE_RANK, CATEGORY_DESCRIPTIONS, RULES, SOURCE_FALLBACK_RULES, UNKNOWN_MODULE
    taxonomy = load_taxonomy(args.taxonomy)
    UNKNOWN_MODULE = str(taxonomy["source_module_normalization"]["fallback_value"])
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
    verify_required_derived_fields(rows)

    command = " ".join(
        shlex.quote(argument) for argument in [sys.executable, *sys.argv]
    )
    grouped = module_grouping_document(rows, source_path=args.json_output)
    report = classification_report(
        rows,
        input_path=args.input,
        taxonomy_path=args.taxonomy,
        output_path=args.json_output,
        command=command,
        grouped_output=args.grouped_output,
    )

    atomic_write(args.json_output, json.dumps(rows, indent=2, ensure_ascii=False) + "\n")
    if args.csv_output is not None:
        write_csv(rows, args.csv_output)
    atomic_write(
        args.grouped_output, json.dumps(grouped, indent=2, ensure_ascii=False) + "\n"
    )
    atomic_write(args.report_output, json.dumps(report, indent=2, ensure_ascii=False) + "\n")

    print(f"categorized {len(rows)} entries")
    print(f"input: {args.input}")
    print(f"taxonomy: {args.taxonomy}")
    print(f"json_output: {args.json_output}")
    print(f"grouped_output: {args.grouped_output}")
    if args.csv_output is not None:
        print(f"csv_output: {args.csv_output}")
    print(f"report_output: {args.report_output}")
    print(f"classification_command: {command}")
    for category in CATEGORY_DESCRIPTIONS:
        count = sum(row["error_type"] == category for row in rows)
        print(f"{category}: {count}")


if __name__ == "__main__":
    main()
