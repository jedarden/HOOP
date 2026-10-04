#!/usr/bin/env python3
"""Tests for the lossless source-catalog normalizer."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from normalize_source_error_catalog import normalize_records, verify_lossless  # noqa: E402


class NormalizeSourceCatalogTests(unittest.TestCase):
    def test_preserves_source_fields_metadata_and_collision_rows(self) -> None:
        source = [
            {
                "file": "tests/example.rs",
                "line": 7,
                "category": "expect with failure",
                "line_content": r'.expect(\"Could not read config\");',
                "error_type": "configuration",
                "source_module": "tests::example",
                "metadata": {"owner": "extractor"},
            },
            {
                "file": "tests/example.rs",
                "line": 7,
                "category": "expect call with detailed message",
                "line_content": r'.expect(\"Could not read config\");',
                "error_type": "configuration",
                "source_module": "tests::example",
                "metadata": {"owner": "extractor"},
            },
        ]

        normalized = normalize_records(source)
        report = verify_lossless(source, normalized)

        self.assertEqual(report["source_records"], 2)
        self.assertTrue(report["row_count_preserved"])
        self.assertEqual(normalized[0]["original_message"], "Could not read config")
        self.assertIn("source_location_collision", normalized[0]["normalization_flags"])
        self.assertIn("source_location_collision", normalized[1]["normalization_flags"])
        self.assertEqual(normalized[0]["metadata"], source[0]["metadata"])
        self.assertEqual(normalized[0]["file"], source[0]["file"])
        self.assertEqual(normalized[0]["line"], source[0]["line"])

    def test_value_only_assertion_is_retained_without_guessing(self) -> None:
        source = [
            {
                "file": "tests/assertions.rs",
                "line": 12,
                "category": "assert_eq detailed message",
                "line_content": r'assert_eq!(actual, expected);',
            }
        ]

        normalized = normalize_records(source)

        self.assertIsNone(normalized[0]["original_message"])
        self.assertEqual(normalized[0]["message_status"], "not_present")
        self.assertIn("no_original_message", normalized[0]["normalization_flags"])

    def test_missing_and_ambiguous_fields_are_flagged_and_source_is_retained(self) -> None:
        source = [
            {
                "file": "../outside.rs",
                "line": "unknown",
                "category": "",
                "line_content": 'assert_eq!(left, right, "unterminated',
                "error_type": "other",
            },
            {"file": "tests/missing.rs", "line": 1},
        ]

        normalized = normalize_records(source)

        self.assertEqual(normalized[0]["file"], source[0]["file"])
        self.assertEqual(normalized[0]["line"], source[0]["line"])
        self.assertIn("invalid_file", normalized[0]["normalization_flags"])
        self.assertIn("invalid_line", normalized[0]["normalization_flags"])
        self.assertIn("missing_category", normalized[0]["normalization_flags"])
        self.assertIn("ambiguous_original_message", normalized[0]["normalization_flags"])
        self.assertIn("missing_required_field:category", normalized[1]["normalization_flags"])
        self.assertEqual(normalized[1]["normalization_status"], "invalid")


if __name__ == "__main__":
    unittest.main()
