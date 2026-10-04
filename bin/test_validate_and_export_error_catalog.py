#!/usr/bin/env python3
"""Focused tests for the categorized catalog release validator."""

from __future__ import annotations

import csv
import json
import sys
import tempfile
import unittest
from pathlib import Path


sys.path.insert(0, str(Path(__file__).parent))
from validate_and_export_error_catalog import (  # noqa: E402
    UNKNOWN_MESSAGE,
    build_final_rows,
    load_csv,
    validate_intermediate,
    validate_outputs,
    write_csv,
)


class ReleaseCatalogTests(unittest.TestCase):
    def setUp(self) -> None:
        self.source = [
            {
                "file": "tests/example.rs",
                "line": 7,
                "category": "expect with failure",
                "line_content": 'anyhow::bail!(\\"Failed to parse JSON\\");',
            },
            {
                "file": "tests/example.rs",
                "line": 8,
                "category": "assert_eq detailed message",
                "line_content": 'assert_eq!(actual, expected);',
            },
        ]
        self.intermediate = [
            {
                **self.source[0],
                "error_type": "parsing",
                "source_module": "tests::example",
                "exception_flags": [],
            },
            {
                **self.source[1],
                "error_type": "other",
                "source_module": "tests::example",
                "exception_flags": ["no_original_message"],
            },
        ]
        self.allowed = {"parsing", "other"}
        self.ranks = {"parsing": 0, "other": 1}

    def test_identity_taxonomy_and_unknown_message_are_validated(self) -> None:
        report = validate_intermediate(self.source, self.intermediate, self.allowed)

        self.assertEqual(report["errors"], [])
        self.assertTrue(report["identity_parity"])
        self.assertEqual(report["message_records"], {"known": 1, "unknown": 1})

        final = build_final_rows(self.intermediate, self.ranks)
        self.assertEqual(final[1]["original_message"], UNKNOWN_MESSAGE)
        self.assertIn("no_original_message", final[1]["exception_flags"])

    def test_json_and_csv_exports_share_source_identity(self) -> None:
        final = build_final_rows(self.intermediate, self.ranks)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "catalog.csv"
            write_csv(final, path)
            csv_rows, header = load_csv(path)
            report = validate_outputs(
                self.source,
                final,
                csv_rows,
                header,
                self.allowed,
                self.ranks,
            )

        self.assertEqual(report["errors"], [])
        self.assertTrue(report["json_identity_parity"])
        self.assertTrue(report["csv_identity_parity"])
        self.assertTrue(report["json_csv_identity_parity"])
        self.assertTrue(report["module_file_grouping_deterministic"])

    def test_location_change_breaks_identity_parity(self) -> None:
        changed = [dict(row) for row in self.intermediate]
        changed[0]["line"] = 99
        report = validate_intermediate(self.source, changed, self.allowed)

        self.assertFalse(report["identity_parity"])
        self.assertTrue(any("identity" in error for error in report["errors"]))

    def test_preserved_handoff_message_is_used_before_reextracting_source(self) -> None:
        intermediate = [
            {
                **self.intermediate[0],
                "original_message": "Preserved handoff message",
            }
        ]
        self.assertEqual(
            build_final_rows(intermediate, self.ranks)[0]["original_message"],
            "Preserved handoff message",
        )


if __name__ == "__main__":
    unittest.main()
