#!/usr/bin/env python3
"""Focused tests for the canonical error-catalog input contract."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path


sys.path.insert(0, str(Path(__file__).parent))
from load_error_catalog import CatalogLoadError, load_catalog  # noqa: E402


class ErrorCatalogLoaderTests(unittest.TestCase):
    def test_valid_records_preserve_message_location_and_metadata(self) -> None:
        records = [
            {
                "file": "hoop-daemon/tests/example.rs",
                "line": 17,
                "category": "expect with failure",
                "line_content": r'.expect(\"Original source message\");',
                "error_type": "runtime",
                "source_module": "hoop_daemon::tests::example",
                "exception_flags": ["existing-flag"],
                "metadata": {"owner": "extraction", "sequence": 4},
            },
            {
                "file": "tests/assertions.rs",
                "line": 23,
                "category": "assert_eq detailed message",
                "line_content": r'assert_eq!(actual, expected, \"Explicit message\");',
                "original_message": "Explicit message",
                "custom_field": {"keep": True},
            },
        ]

        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "catalog.json"
            path.write_text(json.dumps(records), encoding="utf-8")
            loaded = load_catalog(path)

        self.assertEqual(len(loaded), 2)
        self.assertEqual(loaded[0]["file"], records[0]["file"])
        self.assertEqual(loaded[0]["line"], records[0]["line"])
        self.assertEqual(loaded[0]["line_content"], records[0]["line_content"])
        self.assertEqual(loaded[0]["original_message"], "Original source message")
        self.assertEqual(loaded[0]["metadata"], records[0]["metadata"])
        self.assertEqual(loaded[0]["exception_flags"], records[0]["exception_flags"])
        self.assertEqual(loaded[1]["original_message"], records[1]["original_message"])
        self.assertEqual(loaded[1]["custom_field"], records[1]["custom_field"])

    def test_message_less_record_is_retained_with_nullable_message(self) -> None:
        record = {
            "file": "tests/assertions.rs",
            "line": 24,
            "category": "assert_eq detailed message",
            "line_content": "assert_eq!(actual, expected);",
        }

        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "catalog.json"
            path.write_text(json.dumps([record]), encoding="utf-8")
            loaded = load_catalog(path)

        self.assertEqual(loaded[0]["original_message"], None)
        self.assertEqual(loaded[0]["line_content"], record["line_content"])

    def test_invalid_record_fails_with_stable_location(self) -> None:
        invalid = {
            "file": "tests/example.rs",
            "line": "not-a-line",
            "category": "expect with failure",
            "line_content": r'.expect(\"message\");',
        }

        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "catalog.json"
            path.write_text(json.dumps([invalid]), encoding="utf-8")
            with self.assertRaisesRegex(
                CatalogLoadError,
                rf"{path}: record 1: line must be a positive one-based integer",
            ):
                load_catalog(path)


if __name__ == "__main__":
    unittest.main()

