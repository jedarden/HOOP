#!/usr/bin/env python3
"""Focused tests for the error-catalog categorizer."""

from __future__ import annotations

import csv
import json
import sys
import tempfile
import unittest
from pathlib import Path


sys.path.insert(0, str(Path(__file__).parent))
from categorize_error_messages import (  # noqa: E402
    UNKNOWN_MODULE,
    classify_catalog,
    error_type,
    source_module,
    write_csv,
)


def row(line_content: str, *, file: str = "tests/example.rs") -> dict[str, object]:
    return {
        "file": file,
        "line": 7,
        "category": "expect with failure",
        "line_content": line_content,
    }


class CategorizerTests(unittest.TestCase):
    def test_semantic_categories_are_single_label(self) -> None:
        examples = {
            'anyhow::bail!(\\"Invalid project config\\");': "configuration",
            'anyhow::bail!(\\"Forbidden workspace\\");': "authorization",
            'anyhow::bail!(\\"Failed to fetch response\\");': "network",
            'anyhow::bail!(\\"Failed to parse JSON\\");': "parsing",
            'anyhow::bail!(\\"Daemon failed to start\\");': "runtime",
            'anyhow::bail!(\\"Expected one event\\");': "state",
        }

        for line_content, expected in examples.items():
            with self.subTest(line_content=line_content):
                self.assertEqual(error_type(row(line_content)), expected)

    def test_generic_message_uses_stable_source_hint(self) -> None:
        self.assertEqual(
            error_type(row('assert_eq!(value, \\"failed\\");', file="src/network.rs")),
            "network",
        )
        self.assertEqual(error_type(row('anyhow::bail!(\\"failed\\");')), "other")

    def test_handoff_message_wins_over_unrelated_source_literals(self) -> None:
        record = row('anyhow::bail!(\\"Failed to fetch response\\");')
        record["original_message"] = "Failed to parse JSON"
        self.assertEqual(error_type(record), "parsing")

    def test_source_module_normalization_and_invalid_fallback(self) -> None:
        self.assertEqual(
            source_module(r"./hoop-cli/tests/no-interactive.rs"),
            "hoop_cli::tests::no_interactive",
        )
        self.assertEqual(source_module("/tmp/outside.rs"), UNKNOWN_MODULE)
        self.assertEqual(source_module("hoop-cli/tests/../outside.rs"), UNKNOWN_MODULE)

    def test_message_less_and_invalid_records_get_explicit_flags(self) -> None:
        records = [
            row("assert_eq!(actual, expected);"),
            row('anyhow::bail!(\\"failed\\");', file="/tmp/outside.rs"),
        ]
        with tempfile.TemporaryDirectory() as directory:
            input_path = Path(directory) / "input.json"
            input_path.write_text(json.dumps(records), encoding="utf-8")
            categorized = classify_catalog(input_path)

        by_file = {record["file"]: record for record in categorized}
        self.assertEqual(by_file["tests/example.rs"]["error_type"], "other")
        self.assertIn("no_original_message", by_file["tests/example.rs"]["exception_flags"])
        self.assertEqual(by_file["/tmp/outside.rs"]["source_module"], UNKNOWN_MODULE)
        self.assertIn("invalid_source_file", by_file["/tmp/outside.rs"]["exception_flags"])

    def test_sort_keeps_taxonomy_groups_then_source_order(self) -> None:
        records = [
            row('anyhow::bail!(\\"Failed to fetch response\\");', file="z.rs"),
            row('anyhow::bail!(\\"Failed to parse JSON\\");', file="a.rs"),
            row('anyhow::bail!(\\"Failed to fetch response\\");', file="a.rs"),
        ]
        with tempfile.TemporaryDirectory() as directory:
            input_path = Path(directory) / "input.json"
            input_path.write_text(json.dumps(records), encoding="utf-8")
            categorized = classify_catalog(input_path)

        self.assertEqual(
            [(record["error_type"], record["file"]) for record in categorized],
            [("network", "a.rs"), ("parsing", "a.rs"), ("network", "z.rs")],
        )

    def test_input_fields_and_extra_fields_are_preserved(self) -> None:
        records = [
            row('anyhow::bail!(\\"Failed to parse JSON\\");'),
            {
                **row('anyhow::bail!(\\"Failed to write file\\");', file="src/io.rs"),
                "message": "Failed to write file",
                "column": 4,
            },
        ]

        with tempfile.TemporaryDirectory() as directory:
            input_path = Path(directory) / "input.json"
            csv_path = Path(directory) / "output.csv"
            input_path.write_text(json.dumps(records), encoding="utf-8")

            categorized = classify_catalog(input_path)
            write_csv(categorized, csv_path)

            self.assertEqual(len(categorized), len(records))
            originals_by_location = {(record["file"], record["line"]): record for record in records}
            for updated in categorized:
                original = originals_by_location[(updated["file"], updated["line"])]
                self.assertEqual(
                    {key: updated[key] for key in original},
                    original,
                )
            with csv_path.open(newline="", encoding="utf-8") as handle:
                csv_rows = list(csv.DictReader(handle))
            self.assertIn("message", csv_rows[0])
            self.assertIn("column", csv_rows[0])
            self.assertEqual(len(csv_rows), len(records))


if __name__ == "__main__":
    unittest.main()
