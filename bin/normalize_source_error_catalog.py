#!/usr/bin/env python3
"""Create the lossless working catalog for error-message categorization.

The extraction handoff is a JSON array.  This normalizer keeps every source
field and every existing metadata field unchanged, then appends deterministic
fields used by later categorization steps.  It deliberately preserves source
order and source-location collisions; ``(file, line)`` is not a record key.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import sys
from collections import Counter
from pathlib import Path
from typing import Any


REQUIRED_FIELDS = ("file", "line", "category", "line_content")
NORMALIZED_FIELDS = (
    "source_record_index",
    "source_record_id",
    "normalized_file",
    "normalized_line",
    "normalized_category",
    "original_message",
    "message_status",
    "normalization_status",
    "normalization_flags",
)


class CatalogNormalizationError(ValueError):
    """The source or normalized catalog violates its lossless contract."""


def _source_text(value: str) -> str:
    return value.replace(r'\"', '"')


def _split_macro_args(arguments: str) -> list[str]:
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


def _extract_message(line_content: str) -> tuple[str | None, str]:
    """Return ``(message, status)`` without changing the source line.

    ``not_present`` means the source is a valid value-only assertion or has no
    string literal.  ``ambiguous`` is reserved for a malformed assertion
    shape where guessing a message would be unsafe.
    """

    source = _source_text(line_content)
    literals = re.findall(r'"((?:\\.|[^"\\])*)"', source)
    is_assertion = re.search(r"assert_(?:eq|ne)!\s*\(", source)
    if is_assertion:
        opening = source.find("(")
        closing = source.rfind(")")
        if opening < 0 or closing <= opening:
            return None, "ambiguous"
        arguments = _split_macro_args(source[opening + 1 : closing])
        if len(arguments) < 3:
            return None, "not_present"
        literals = re.findall(r'"((?:\\.|[^"\\])*)"', ",".join(arguments[2:]))

    if not literals:
        return None, "not_present"

    literal = literals[-1]
    try:
        message = json.loads(f'"{literal}"')
    except json.JSONDecodeError:
        message = literal.replace(r'\"', '"')
    if not isinstance(message, str) or not message.strip():
        return None, "ambiguous"
    return message, "extracted"


def _normalize_file(value: Any) -> tuple[str | None, str | None]:
    if not isinstance(value, str) or not value.strip():
        return None, "missing_file"
    normalized = value.replace("\\", "/")
    while normalized.startswith("./"):
        normalized = normalized[2:]
    if (
        normalized.startswith("/")
        or re.match(r"^[A-Za-z]:/", normalized)
        or ".." in normalized.split("/")
        or not normalized.endswith(".rs")
    ):
        return None, "invalid_file"
    return normalized, None


def _normalize_line(value: Any) -> tuple[int | None, str | None]:
    if isinstance(value, bool) or not isinstance(value, int) or value < 1:
        return None, "invalid_line"
    return value, None


def _is_comment_or_doc(line_content: Any) -> bool:
    return isinstance(line_content, str) and _source_text(line_content).lstrip().startswith(("//", "/*", "*"))


def _load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise CatalogNormalizationError(f"{path}: unable to read JSON catalog: {error}") from error


def normalize_records(source: Any) -> list[dict[str, Any]]:
    """Normalize records while retaining malformed records for inspection."""

    if not isinstance(source, list):
        raise CatalogNormalizationError("source catalog top level must be a JSON array")

    locations: Counter[tuple[str, int]] = Counter()
    for record in source:
        if not isinstance(record, dict):
            continue
        line = record.get("line")
        if isinstance(record.get("file"), str) and isinstance(line, int) and not isinstance(line, bool):
            locations[(record["file"], line)] += 1

    normalized: list[dict[str, Any]] = []
    for index, source_record in enumerate(source, start=1):
        if not isinstance(source_record, dict):
            normalized.append(
                {
                    "source_record_index": index,
                    "source_record_id": f"r{index:07d}",
                    "normalization_status": "invalid",
                    "normalization_flags": ["record_not_object"],
                    "source_record": copy.deepcopy(source_record),
                }
            )
            continue

        record = copy.deepcopy(source_record)
        flags: list[str] = []
        for field in REQUIRED_FIELDS:
            if field not in record:
                flags.append(f"missing_required_field:{field}")

        normalized_file, file_flag = _normalize_file(record.get("file"))
        if file_flag is not None:
            flags.append(file_flag)
        normalized_line, line_flag = _normalize_line(record.get("line"))
        if line_flag is not None:
            flags.append(line_flag)

        category = record.get("category")
        normalized_category = category.strip() if isinstance(category, str) and category.strip() else None
        if normalized_category is None:
            flags.append("missing_category")

        source_line = record.get("line_content")
        if not isinstance(source_line, str) or not source_line.strip():
            flags.append("missing_line_content")

        if "original_message" in record:
            supplied_message = record["original_message"]
            if supplied_message is None:
                message, message_status = None, "provided_null"
            elif isinstance(supplied_message, str):
                message, message_status = supplied_message, "provided"
            else:
                message, message_status = None, "ambiguous"
                flags.append("invalid_original_message")
        elif isinstance(source_line, str) and source_line.strip():
            message, message_status = _extract_message(source_line)
        else:
            message, message_status = None, "not_present"

        if message_status in {"not_present", "provided_null"}:
            flags.append("no_original_message")
        elif message_status == "ambiguous":
            flags.append("ambiguous_original_message")
        if _is_comment_or_doc(source_line):
            flags.append("comment_or_doc_source")
        if normalized_file is not None and normalized_line is not None:
            if locations[(record.get("file"), record.get("line"))] > 1:
                flags.append("source_location_collision")

        for field, value in (
            ("source_record_index", index),
            ("source_record_id", f"r{index:07d}"),
            ("normalized_file", normalized_file),
            ("normalized_line", normalized_line),
            ("normalized_category", normalized_category),
            ("original_message", message),
            ("message_status", message_status),
            ("normalization_status", "invalid" if any(flag.startswith(("missing_required_field:", "missing_", "invalid_")) for flag in flags) else "ready"),
            ("normalization_flags", sorted(set(flags))),
        ):
            record[field] = value
        normalized.append(record)
    return normalized


def _source_projection(record: dict[str, Any]) -> dict[str, Any]:
    return {field: copy.deepcopy(record[field]) for field in record if field not in NORMALIZED_FIELDS}


def verify_lossless(source: Any, normalized: Any) -> dict[str, Any]:
    if not isinstance(source, list) or not isinstance(normalized, list):
        raise CatalogNormalizationError("source and normalized catalogs must both be JSON arrays")
    errors: list[str] = []
    if len(source) != len(normalized):
        errors.append(f"row count changed: source={len(source)} normalized={len(normalized)}")
    for index, source_record in enumerate(source):
        if index >= len(normalized):
            break
        output_record = normalized[index]
        if not isinstance(source_record, dict) or not isinstance(output_record, dict):
            if source_record != output_record.get("source_record"):
                errors.append(f"record {index + 1}: non-object source record changed")
            continue
        for field, value in source_record.items():
            if field not in output_record or output_record[field] != value:
                errors.append(f"record {index + 1}: source field changed: {field}")
        if output_record.get("source_record_index") != index + 1:
            errors.append(f"record {index + 1}: source_record_index is not stable")
        if output_record.get("source_record_id") != f"r{index + 1:07d}":
            errors.append(f"record {index + 1}: source_record_id is not stable")
    if errors:
        raise CatalogNormalizationError("; ".join(errors[:10]))
    required_presence = {
        field: {
            "source": sum(isinstance(record, dict) and field in record for record in source),
            "normalized": sum(isinstance(record, dict) and field in record for record in normalized),
        }
        for field in REQUIRED_FIELDS
    }
    return {
        "source_records": len(source),
        "normalized_records": len(normalized),
        "row_count_preserved": len(source) == len(normalized),
        "source_fields_preserved": True,
        "required_field_presence": required_presence,
        "sha256_normalized": hashlib.sha256(
            json.dumps(normalized, ensure_ascii=False, indent=2).encode("utf-8")
        ).hexdigest(),
    }


def _write(path: Path, records: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(records, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, default=Path("test_error_messages.json"))
    parser.add_argument("--output", type=Path, default=Path("docs/normalized_error_catalog.json"))
    parser.add_argument("--verify", action="store_true", help="verify an existing output instead of writing it")
    args = parser.parse_args()

    try:
        source = _load_json(args.input)
        if args.verify:
            normalized = _load_json(args.output)
        else:
            normalized = normalize_records(source)
            _write(args.output, normalized)
        report = verify_lossless(source, normalized)
    except CatalogNormalizationError as error:
        print(f"normalization failed: {error}", file=sys.stderr)
        return 2
    print(json.dumps({"input": str(args.input), "output": str(args.output), **report}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
