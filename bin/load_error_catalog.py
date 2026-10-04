#!/usr/bin/env python3
"""Load the canonical error-message extraction catalog.

Child 1's handoff is ``test_error_messages.json``: a UTF-8 JSON array of
objects.  Every object must contain the lossless source fields ``file``,
``line``, ``category``, and ``line_content``.  The loader validates only that
input contract.  It does not classify, group, sort, deduplicate, or otherwise
interpret records.

The extraction catalog does not always carry a separate ``original_message``
field.  When it is absent, the loader adds the nullable handoff field by using
the source-catalog message extractor; the original ``line_content`` remains
unchanged.  All other input fields are copied unchanged, including metadata
added by an earlier stage.
"""

from __future__ import annotations

import copy
import json
import re
from pathlib import Path
from typing import Any


DEFAULT_CATALOG_PATH = Path("test_error_messages.json")
REQUIRED_FIELDS = ("file", "line", "category", "line_content")


class CatalogLoadError(ValueError):
    """A catalog cannot be loaded without violating its input contract."""


def _record_error(path: Path, index: int, message: str) -> CatalogLoadError:
    return CatalogLoadError(f"{path}: record {index + 1}: {message}")


def _extract_original_message(line_content: str) -> str | None:
    """Extract an expectation/assertion message without changing source text.

    This is deliberately limited to the extraction handoff.  Semantic error
    categorization remains the responsibility of the later classifier.
    """

    source = line_content.replace(r'\"', '"')
    literals = re.findall(r'"((?:\\.|[^"\\])*)"', source)
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
        literals = re.findall(r'"((?:\\.|[^"\\])*)"', ",".join(arguments[2:]))
        if not literals:
            return None

    # JSON-compatible escaping is what the extraction catalog retains.
    messages: list[str] = []
    for literal in literals[-1:]:
        try:
            messages.append(json.loads(f'"{literal}"'))
        except json.JSONDecodeError:
            messages.append(literal.replace(r'\"', '"'))
    return " ".join(messages) or None


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


def _validate_record(path: Path, index: int, value: Any) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise _record_error(path, index, "must be a JSON object")

    missing = [field for field in REQUIRED_FIELDS if field not in value]
    if missing:
        raise _record_error(path, index, f"missing required field(s): {', '.join(missing)}")

    file_value = value["file"]
    if not isinstance(file_value, str) or not file_value.strip():
        raise _record_error(path, index, "file must be a non-empty string")
    # Keep non-empty but semantically invalid paths in the representation. The
    # classifier deliberately turns those into ``unknown_module`` while
    # retaining the source location, so loading must not discard them.

    line_value = value["line"]
    if isinstance(line_value, bool) or not isinstance(line_value, int) or line_value < 1:
        raise _record_error(path, index, "line must be a positive one-based integer")

    for field in ("category", "line_content"):
        field_value = value[field]
        if not isinstance(field_value, str) or not field_value.strip():
            raise _record_error(path, index, f"{field} must be a non-empty string")

    if "original_message" in value and value["original_message"] is not None and not isinstance(
        value["original_message"], str
    ):
        raise _record_error(path, index, "original_message must be a string or null")

    if "exception_flags" in value and (
        not isinstance(value["exception_flags"], list)
        or not all(isinstance(flag, str) for flag in value["exception_flags"])
    ):
        raise _record_error(path, index, "exception_flags must be an array of strings")

    # Deep-copy the decoded object so callers can safely enrich an in-memory
    # record without changing the representation returned by a future load.
    record = copy.deepcopy(value)
    record.setdefault("original_message", _extract_original_message(record["line_content"]))
    return record


def load_catalog(path: Path = DEFAULT_CATALOG_PATH) -> list[dict[str, Any]]:
    """Load and validate ``path`` as the canonical JSON catalog.

    Malformed JSON, an invalid top-level value, or an invalid record raises
    :class:`CatalogLoadError` with a stable path/record/field description.
    No malformed record is skipped and no valid record is returned alongside
    an error.
    """

    path = Path(path)
    try:
        raw = path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        raise CatalogLoadError(f"{path}: unable to read catalog: {error}") from error

    try:
        value = json.loads(raw)
    except json.JSONDecodeError as error:
        raise CatalogLoadError(f"{path}: invalid JSON at line {error.lineno}, column {error.colno}") from error

    if not isinstance(value, list):
        raise CatalogLoadError(f"{path}: top level must be a JSON array")

    return [_validate_record(path, index, record) for index, record in enumerate(value)]
