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
from categorize_error_messages import classify_catalog, error_type, write_csv  # noqa: E402


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
