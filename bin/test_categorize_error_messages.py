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
    classification_report,
    error_type,
    group_by_source_module,
    load_taxonomy,
    module_grouping_document,
    source_module,
    verify_required_derived_fields,
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
            'anyhow::bail!(\\"Invalid token\\");': "authentication",
            'anyhow::bail!(\\"Invalid project config\\");': "configuration",
            'anyhow::bail!(\\"Forbidden workspace\\");': "authorization",
            'anyhow::bail!(\\"Failed to fetch response\\");': "network",
            'anyhow::bail!(\\"Failed to parse JSON\\");': "parsing",
            'anyhow::bail!(\\"Daemon failed to start\\");': "runtime",
            'anyhow::bail!(\\"Expected value to be equal\\");': "validation",
        }

        for line_content, expected in examples.items():
            with self.subTest(line_content=line_content):
                self.assertEqual(error_type(row(line_content)), expected)

    def test_precedence_makes_multi_signal_messages_single_label(self) -> None:
        # timeout precedes network in the approved taxonomy. The same message
        # must never produce both labels or depend on source-file hints.
        record = row(
            'anyhow::bail!(\\"HTTP request timed out while waiting for response\\");',
            file="src/network/client.rs",
        )
        self.assertEqual(error_type(record), "timeout")

        # Authentication precedes network when a credential failure is
        # reported by a remote request.
        self.assertEqual(
            error_type(row('anyhow::bail!(\\"Request rejected: invalid token\\");')),
            "authentication",
        )

    def test_generic_message_uses_stable_source_hint(self) -> None:
        self.assertEqual(
            error_type(row('assert_eq!(value, \\"failed\\");', file="src/network.rs")),
            "network",
        )
        self.assertEqual(error_type(row('anyhow::bail!(\\"failed\\");')), "other")

    def test_conflicting_source_hints_use_documented_fallback(self) -> None:
        self.assertEqual(
            error_type(row("assert_eq!(actual, expected);", file="src/network/path.rs")),
            "other",
        )

    def test_handoff_message_wins_over_unrelated_source_literals(self) -> None:
        record = row('anyhow::bail!(\\"Failed to fetch response\\");')
        record["original_message"] = "Failed to parse JSON"
        self.assertEqual(error_type(record), "parsing")

    def test_classification_preserves_loaded_message_and_location(self) -> None:
        original = row(
            'anyhow::bail!(\\"network source text\\");',
            file="hoop-daemon/src/auth.rs",
        )
        original["line"] = 41
        original["original_message"] = "Invalid token"

        with tempfile.TemporaryDirectory() as directory:
            input_path = Path(directory) / "input.json"
            input_path.write_text(json.dumps([original]), encoding="utf-8")
            categorized = classify_catalog(input_path)

        self.assertEqual(categorized[0]["error_type"], "authentication")
        for field in ("original_message", "file", "line", "category", "line_content"):
            self.assertEqual(categorized[0][field], original[field])

    def test_source_module_normalization_and_invalid_fallback(self) -> None:
        self.assertEqual(
            source_module(r"./hoop-cli/tests/no-interactive.rs"),
            "hoop_cli::tests::no_interactive",
        )
        self.assertEqual(source_module("/tmp/outside.rs"), UNKNOWN_MODULE)
        self.assertEqual(source_module("hoop-cli/tests/../outside.rs"), UNKNOWN_MODULE)

    def test_source_module_fallback_is_documented_and_used_for_grouping(self) -> None:
        taxonomy = load_taxonomy()
        self.assertEqual(
            UNKNOWN_MODULE,
            taxonomy["source_module_normalization"]["fallback_value"],
        )

        records = [
            row(
                'anyhow::bail!(\\"Failed to parse JSON\\");',
                file="hoop-daemon/src/catalog.rs",
            ),
            {
                **row(
                    'anyhow::bail!(\\"Failed to fetch response\\");',
                    file="hoop-daemon/src/catalog.rs",
                ),
                "line": 8,
                "message": "keep this metadata",
                "column": 4,
            },
            row(
                'anyhow::bail!(\\"Failed to write file\\");',
                file="/tmp/generated.rs",
            ),
            row(
                'anyhow::bail!(\\"Failed to fetch response\\");',
                file="hoop-cli/src/status.rs",
            ),
        ]
        with tempfile.TemporaryDirectory() as directory:
            input_path = Path(directory) / "input.json"
            input_path.write_text(json.dumps(records), encoding="utf-8")
            categorized = classify_catalog(input_path)

        grouped = group_by_source_module(categorized)
        self.assertEqual(
            list(grouped),
            [
                "hoop_cli::src::status",
                "hoop_daemon::src::catalog",
                UNKNOWN_MODULE,
            ],
        )
        self.assertEqual(
            list(grouped["hoop_daemon::src::catalog"]),
            ["network", "parsing"],
        )
        grouped_record = grouped["hoop_daemon::src::catalog"]["network"][0]
        self.assertEqual(grouped_record["file"], "hoop-daemon/src/catalog.rs")
        self.assertEqual(grouped_record["line"], 8)
        self.assertEqual(grouped_record["message"], "keep this metadata")
        self.assertEqual(grouped[UNKNOWN_MODULE]["filesystem"][0]["file"], "/tmp/generated.rs")
        self.assertIn("invalid_source_file", grouped[UNKNOWN_MODULE]["filesystem"][0]["exception_flags"])

    def test_grouping_and_flat_output_are_independent_of_input_order(self) -> None:
        records = [
            row('anyhow::bail!(\\"Failed to parse JSON\\");', file="hoop-daemon/src/a.rs"),
            row('anyhow::bail!(\\"Failed to fetch response\\");', file="hoop-daemon/src/a.rs"),
            row('anyhow::bail!(\\"Failed to fetch response\\");', file="hoop-cli/src/a.rs"),
        ]
        with tempfile.TemporaryDirectory() as directory:
            first_path = Path(directory) / "first.json"
            second_path = Path(directory) / "second.json"
            first_path.write_text(json.dumps(records), encoding="utf-8")
            second_path.write_text(json.dumps(list(reversed(records))), encoding="utf-8")
            first = classify_catalog(first_path)
            second = classify_catalog(second_path)

        self.assertEqual(first, second)
        self.assertEqual(
            list(group_by_source_module(first)),
            ["hoop_cli::src::a", "hoop_daemon::src::a"],
        )

    def categorized_fixture(self) -> list[dict[str, object]]:
        records = [
            {
                **row('anyhow::bail!(\\"Failed to parse JSON\\");', file="hoop-daemon/src/a.rs"),
                "source_record_id": "r0000001",
                "source_record_index": 1,
                "original_message": "Failed to parse JSON",
            },
            {
                **row('anyhow::bail!(\\"Failed to fetch response\\");', file="hoop-daemon/src/a.rs"),
                "source_record_id": "r0000002",
                "source_record_index": 2,
                "original_message": "Failed to fetch response",
            },
            {
                **row('anyhow::bail!(\\"Failed to fetch response\\");', file="hoop-cli/src/a.rs"),
                "source_record_id": "r0000003",
                "source_record_index": 3,
                "original_message": "Failed to fetch response",
            },
            {
                **row('anyhow::bail!(\\"Failed to write file\\");', file="/tmp/generated.rs"),
                "source_record_id": "r0000004",
                "source_record_index": 4,
                "original_message": "Failed to write file",
            },
        ]
        with tempfile.TemporaryDirectory() as directory:
            input_path = Path(directory) / "input.json"
            input_path.write_text(json.dumps(records), encoding="utf-8")
            return classify_catalog(input_path)

    def test_module_grouping_document_groups_records_with_counts(self) -> None:
        categorized = self.categorized_fixture()
        document = module_grouping_document(
            categorized, source_path=Path("docs/categorized_error_catalog.json")
        )

        self.assertEqual(document["record_count"], len(categorized))
        self.assertEqual(document["module_count"], 3)
        self.assertEqual(
            list(document["modules"]),
            sorted(document["modules"]),
        )
        self.assertEqual(
            document["counts_by_source_module"],
            {
                "hoop_cli::src::a": 1,
                "hoop_daemon::src::a": 2,
                UNKNOWN_MODULE: 1,
            },
        )
        self.assertEqual(
            document["modules"]["hoop_daemon::src::a"]["counts_by_error_type"],
            {"network": 1, "parsing": 1},
        )
        self.assertEqual(
            document["modules"]["hoop_daemon::src::a"]["record_count"], 2
        )

        nested = [
            record
            for module in document["modules"].values()
            for records in module["records"].values()  # type: ignore[union-attr]
            for record in records
        ]
        self.assertEqual(
            sorted(
                json.dumps(record, sort_keys=True) for record in nested
            ),
            sorted(json.dumps(record, sort_keys=True) for record in categorized),
        )
        top_level = document["counts_by_error_type"]
        self.assertEqual(sum(top_level.values()), len(categorized))  # type: ignore[union-attr]
        self.assertEqual(
            list(top_level),
            [
                "authentication",
                "authorization",
                "concurrency",
                "configuration",
                "filesystem",
                "network",
                "parsing",
                "persistence",
                "performance",
                "resource",
                "runtime",
                "security",
                "state",
                "timeout",
                "validation",
                "other",
            ],
        )

    def test_module_grouping_document_reports_unresolved_mappings(self) -> None:
        categorized = self.categorized_fixture()
        document = module_grouping_document(categorized, source_path=Path("flat.json"))

        unresolved = document["unresolved_module_mappings"]
        self.assertEqual(unresolved["count"], 1)  # type: ignore[union-attr]
        reference = unresolved["records"][0]  # type: ignore[union-attr]
        self.assertEqual(reference["file"], "/tmp/generated.rs")
        self.assertEqual(reference["source_record_id"], "r0000004")
        self.assertEqual(reference["reason"], "invalid_source_file")

        unknown_group = document["modules"][UNKNOWN_MODULE]
        self.assertEqual(unknown_group["record_count"], 1)
        record = unknown_group["records"]["filesystem"][0]
        self.assertEqual(record["file"], "/tmp/generated.rs")
        self.assertIn("invalid_source_file", record["exception_flags"])

    def test_module_grouping_document_is_independent_of_input_order(self) -> None:
        records = [
            row('anyhow::bail!(\\"Failed to parse JSON\\");', file="hoop-daemon/src/a.rs"),
            row('anyhow::bail!(\\"Failed to fetch response\\");', file="hoop-daemon/src/a.rs"),
            row('anyhow::bail!(\\"Failed to fetch response\\");', file="hoop-cli/src/a.rs"),
        ]
        with tempfile.TemporaryDirectory() as directory:
            first_path = Path(directory) / "first.json"
            second_path = Path(directory) / "second.json"
            first_path.write_text(json.dumps(records), encoding="utf-8")
            second_path.write_text(json.dumps(list(reversed(records))), encoding="utf-8")
            first = module_grouping_document(
                classify_catalog(first_path), source_path=Path("flat.json")
            )
            second = module_grouping_document(
                classify_catalog(second_path), source_path=Path("flat.json")
            )

        self.assertEqual(first, second)

    def test_report_includes_module_summary_and_field_coverage(self) -> None:
        categorized = self.categorized_fixture()
        report = classification_report(
            categorized,
            input_path=Path("normalized.json"),
            taxonomy_path=Path("taxonomy.json"),
            output_path=Path("categorized.json"),
            command="classifier",
            grouped_output=Path("by_module.json"),
        )

        self.assertEqual(report["grouped_output"], "by_module.json")
        coverage = report["field_coverage"]
        self.assertEqual(coverage["record_count"], len(categorized))
        self.assertEqual(coverage["records_with_error_type"], len(categorized))
        self.assertEqual(coverage["records_with_source_module"], len(categorized))
        summary = report["source_modules"]
        self.assertEqual(summary["module_count"], 3)
        self.assertEqual(summary["unknown_module"], UNKNOWN_MODULE)
        self.assertEqual(summary["unresolved_module_mappings"]["count"], 1)
        self.assertEqual(
            summary["counts_by_source_module"],
            {
                "hoop_cli::src::a": 1,
                "hoop_daemon::src::a": 2,
                UNKNOWN_MODULE: 1,
            },
        )

    def test_missing_derived_fields_are_rejected_before_output(self) -> None:
        categorized = self.categorized_fixture()
        incomplete = [dict(categorized[0])]
        del incomplete[0]["source_module"]

        with self.assertRaises(ValueError):
            verify_required_derived_fields(incomplete)
        with self.assertRaises(ValueError):
            module_grouping_document(incomplete, source_path=Path("flat.json"))
        with self.assertRaises(ValueError):
            classification_report(
                incomplete,
                input_path=Path("normalized.json"),
                taxonomy_path=Path("taxonomy.json"),
                output_path=Path("categorized.json"),
                command="classifier",
            )

        empty_type = [dict(categorized[0])]
        empty_type[0]["error_type"] = "  "
        with self.assertRaises(ValueError):
            verify_required_derived_fields(empty_type)

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

    def test_normalized_handoff_fields_are_preserved_and_reported(self) -> None:
        records = [
            {
                **row('anyhow::bail!(\\"Invalid token\\");', file="hoop-cli/src/auth.rs"),
                "source_record_id": "r0000042",
                "source_record_index": 42,
                "original_message": "Invalid token",
                "message_status": "extracted",
                "normalization_flags": [],
                "unrelated_metadata": {"owner": "test"},
            },
            {
                **row("assert_eq!(actual, expected);", file="src/network/path.rs"),
                "source_record_id": "r0000043",
                "source_record_index": 43,
                "original_message": None,
                "message_status": "not_present",
                "normalization_flags": ["no_original_message"],
            },
        ]
        with tempfile.TemporaryDirectory() as directory:
            input_path = Path(directory) / "normalized.json"
            input_path.write_text(json.dumps(records), encoding="utf-8")
            categorized = classify_catalog(input_path)
            report = classification_report(
                categorized,
                input_path=input_path,
                taxonomy_path=Path("taxonomy.json"),
                output_path=Path("categorized.json"),
                command="classifier",
            )

        by_id = {record["source_record_id"]: record for record in categorized}
        self.assertEqual(by_id["r0000042"]["error_type"], "authentication")
        self.assertEqual(by_id["r0000042"]["unrelated_metadata"], {"owner": "test"})
        self.assertEqual(by_id["r0000043"]["error_type"], "other")
        self.assertEqual(by_id["r0000043"]["original_message"], None)
        for field, value in records[0].items():
            self.assertEqual(by_id["r0000042"][field], value)
        self.assertEqual(report["record_count"], 2)
        self.assertEqual(
            report["message_rule_precedence"],
            [
                "security",
                "authentication",
                "authorization",
                "timeout",
                "performance",
                "concurrency",
                "configuration",
                "persistence",
                "filesystem",
                "parsing",
                "network",
                "runtime",
                "state",
                "resource",
                "validation",
            ],
        )
        self.assertEqual(report["fallback_error_type"], "other")
        self.assertEqual(report["counts_by_error_type"]["authentication"], 1)
        self.assertEqual(report["fallback"]["count"], 1)
        self.assertEqual(report["fallback"]["records"][0]["source_record_id"], "r0000043")
        self.assertIsNone(report["fallback"]["records"][0]["original_message"])


if __name__ == "__main__":
    unittest.main()
