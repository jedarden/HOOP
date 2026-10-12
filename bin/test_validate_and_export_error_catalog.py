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
    final_counts,
    load_csv,
    load_grouped,
    validate_grouped,
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
        self.assertTrue(report["json_csv_values_equivalent"])
        self.assertTrue(report["csv_preserves_json_fields"])
        self.assertTrue(report["module_file_grouping_deterministic"])

    def test_json_and_csv_must_match_every_emitted_value_and_metadata(self) -> None:
        intermediate = [
            {
                **self.intermediate[0],
                "metadata": {"owner": "catalog", "ordinal": 1},
            },
            self.intermediate[1],
        ]
        final = build_final_rows(intermediate, self.ranks)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "catalog.csv"
            write_csv(final, path)
            csv_rows, header = load_csv(path)
            self.assertIn("metadata", header)

            csv_rows[0]["error_type"] = "runtime"
            report = validate_outputs(
                self.source,
                final,
                csv_rows,
                header,
                self.allowed,
                self.ranks,
            )

        self.assertFalse(report["json_csv_values_equivalent"])
        self.assertTrue(
            any("field 'error_type'" in error for error in report["errors"])
        )

    def test_exports_are_byte_reproducible_on_rerun(self) -> None:
        final = build_final_rows(self.intermediate, self.ranks)
        rendered_json = json.dumps(final, indent=2, ensure_ascii=False) + "\n"
        with tempfile.TemporaryDirectory() as directory:
            directory_path = Path(directory)
            json_one = directory_path / "one.json"
            json_two = directory_path / "two.json"
            csv_one = directory_path / "one.csv"
            csv_two = directory_path / "two.csv"
            json_one.write_text(rendered_json, encoding="utf-8")
            json_two.write_text(rendered_json, encoding="utf-8")
            write_csv(final, csv_one)
            write_csv(final, csv_two)

            self.assertEqual(json_one.read_bytes(), json_two.read_bytes())
            self.assertEqual(csv_one.read_bytes(), csv_two.read_bytes())

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

    def _grouped_doc(self, rows: list[dict]) -> dict:
        """Nest rows into the source-module -> error_type -> records shape."""

        modules: dict[str, dict] = {}
        for row in rows:
            entry = modules.setdefault(
                str(row["source_module"]),
                {"record_count": 0, "counts_by_error_type": {}, "records": {}},
            )
            entry["records"].setdefault(str(row["error_type"]), []).append(row)
            entry["record_count"] += 1
            type_counts = entry["counts_by_error_type"]
            type_counts[str(row["error_type"])] = type_counts.get(str(row["error_type"]), 0) + 1
        doc = {
            "record_count": len(rows),
            "module_count": len(modules),
            "counts_by_error_type": {
                name: sum(
                    len(module_records["records"].get(name, []))
                    for module_records in modules.values()
                )
                for name in ("parsing", "other")
            },
            "counts_by_source_module": {
                name: entry["record_count"] for name, entry in modules.items()
            },
            "unresolved_module_mappings": {"count": 0, "records": []},
            "modules": modules,
        }
        return doc

    def _write_grouped(self, doc: dict) -> Path:
        """Persist a grouped document and load it back through the real loader."""

        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "grouped.json"
            path.write_text(json.dumps(doc), encoding="utf-8")
            return load_grouped(path)

    def test_grouped_handoff_parity_passes_on_matching_records(self) -> None:
        final = build_final_rows(self.intermediate, self.ranks)
        grouped_records, doc = self._write_grouped(self._grouped_doc(final))
        report = validate_grouped(final, grouped_records, doc)

        self.assertEqual(report["errors"], [])
        self.assertTrue(report["identity_parity"])
        self.assertTrue(report["derived_fields_agree"])
        self.assertTrue(report["counts_match"])
        self.assertEqual(report["unresolved_count"], 0)

    def test_grouped_handoff_catches_missing_record(self) -> None:
        final = build_final_rows(self.intermediate, self.ranks)
        grouped_records, doc = self._write_grouped(self._grouped_doc(final[:-1]))
        report = validate_grouped(final, grouped_records, doc)

        self.assertFalse(report["identity_parity"])
        self.assertFalse(report["counts_match"])
        self.assertTrue(report["errors"])

    def test_grouped_handoff_catches_reclassified_record(self) -> None:
        final = build_final_rows(self.intermediate, self.ranks)
        relabeled = [dict(final[0])]
        relabeled[0]["error_type"] = "runtime"
        grouped_records, doc = self._write_grouped(self._grouped_doc(final))
        report = validate_grouped(relabeled + final[1:], grouped_records, doc)

        self.assertFalse(report["derived_fields_agree"])
        self.assertFalse(report["counts_match"])

    def test_grouped_handoff_flags_unresolved_mappings(self) -> None:
        final = build_final_rows(self.intermediate, self.ranks)
        _, doc = self._write_grouped(self._grouped_doc(final))
        doc["unresolved_module_mappings"] = {"count": 1, "records": [{"file": "x"}]}
        grouped_records, _ = self._write_grouped(doc)
        report = validate_grouped(final, grouped_records, doc)

        self.assertEqual(report["unresolved_count"], 1)
        self.assertTrue(
            any("handoff exception" in error for error in report["errors"])
        )

    def test_final_counts_sum_to_record_count(self) -> None:
        final = build_final_rows(self.intermediate, self.ranks)
        counts = final_counts(final, ["parsing", "other"])

        self.assertEqual(counts["record_count"], 2)
        self.assertEqual(counts["module_count"], 1)
        self.assertEqual(counts["counts_by_error_type"], {"parsing": 1, "other": 1})
        self.assertEqual(counts["counts_by_source_module"], {"tests::example": 2})
        self.assertTrue(counts["type_counts_sum_to_record_count"])
        self.assertTrue(counts["module_counts_sum_to_record_count"])


if __name__ == "__main__":
    unittest.main()
