# Final categorized error catalog

The published, validated release of the categorized error message catalog,
ready for downstream consumers. It is generated from the previous child's
source-module-enriched handoff and is byte-reproducible.

## Pipeline

| Stage | Path | Role |
| --- | --- | --- |
| Source extraction | `test_error_messages.json` | Owns record identity (`file`, `line`, `category`, `line_content`) |
| Normalized handoff | `docs/normalized_error_catalog.json` | Preserves every source field, adds `original_message` |
| Classified intermediate | `docs/categorized_error_catalog.json` | Adds `error_type` (closed taxonomy) and `source_module` |
| Grouped handoff | `docs/categorized_error_catalog_by_module.json` | Same records nested `source_module -> error_type -> records`; the export's primary input |
| Taxonomy | `docs/error_category_taxonomy.json` | Closed `allowed_error_types` set, normative order |
| **Final export** | `docs/final_categorized_error_catalog.json` | Published JSON catalog |
| **Final export** | `docs/final_categorized_error_catalog.csv` | Published CSV catalog, cell-equivalent to the JSON |
| **Run record** | `docs/final_error_category_catalog_run.json` | Provenance, validation results, category/module counts |

## Regenerate and validate

```bash
python3 bin/validate_and_export_error_catalog.py --report-output docs/final_error_category_catalog_run.json
pytest bin/test_validate_and_export_error_catalog.py
```

Run both from the repository root. The exporter exits 0 only when every
validation below passes; on any failure it writes nothing.

## Record schema

Required fields (present on every record, in both formats):

| Field | Type | Meaning |
| --- | --- | --- |
| `file` | string | Repository-relative source path (identity) |
| `line` | integer ≥ 1 | One-based source line (identity) |
| `category` | string | Extraction pattern name (identity; not a semantic type) |
| `line_content` | string | Verbatim source text (identity) |
| `error_type` | string | Closed taxonomy value; first matching rule wins |
| `source_module` | string | Rust-style module path derived from `file` |
| `original_message` | string | Extracted message, or the explicit `unknown` |
| `exception_flags` | array | e.g. `no_original_message` when no message could be extracted |

Provenance fields beyond the contract (`source_record_index`,
`source_record_id`, `normalized_*`, `message_status`, `normalization_status`,
`normalization_flags`) are retained losslessly and appended to the CSV header
in sorted order.

## Guarantees enforced by the exporter

1. **Retention** — every source row appears exactly once; record-identity
   multisets of source, JSON, and CSV are equal, and every CSV cell equals the
   JSON projection of the same record.
2. **Grouped-handoff parity** — the export retains every record of
   `categorized_error_catalog_by_module.json` exactly once with agreeing
   `error_type`/`source_module`, and its category/module counts match the
   grouped counts. A nonzero `unresolved_module_mappings.count` is a handoff
   exception: the export refuses to publish rather than drop data.
3. **Schema and taxonomy** — required fields are present and non-empty;
   `error_type` is always in `allowed_error_types`; `source_module` matches
   the derivation rule in [`error_category_taxonomy.md`](error_category_taxonomy.md).
4. **Deterministic ordering** — records sort by `source_module`, `error_type`
   (taxonomy order), `file`, numeric `line`, `category`, `line_content`,
   ascending; reruns are byte-identical.
5. **Counts** — the run record carries `final_counts` with `record_count`,
   `module_count`, zero-filled taxonomy-ordered `counts_by_error_type`, and
   sorted `counts_by_source_module`; both count projections sum to
   `record_count`.

## Current release (2026-10-12, HEAD `4c524592`)

3466 records across 97 source modules. Counts by error type:
authentication 0, authorization 21, concurrency 33, configuration 238,
filesystem 698, network 537, parsing 379, persistence 252, performance 19,
resource 242, runtime 460, security 2, state 314, timeout 29, validation 162,
other 80. Unresolved module mappings: 0.
