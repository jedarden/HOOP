# Error category taxonomy

The machine-readable source of truth for catalog classification is
[`error_category_taxonomy.json`](error_category_taxonomy.json). It defines a
closed set of semantic `error_type` values and a single-label decision
procedure for the normalized handoff at
[`normalized_error_catalog.json`](normalized_error_catalog.json).

## Input and preservation contract

The extraction `category` field is the extractor's pattern name (for example,
`expect with failure`); it is not a semantic error type. The preceding
normalization step reads `test_error_messages.json`, preserves every input
field, and writes the normalized handoff. Classification reads these fields:

| Handoff field | Source field | Treatment |
| --- | --- | --- |
| `source_file` | `file` | Preserve; use for source fallback and module derivation |
| `source_line` | `line` | Preserve as a positive one-based integer |
| `extraction_kind` | `category` | Preserve as extraction metadata |
| `source_text` | `line_content` | Preserve verbatim |
| `original_message` | derived | Use for message matching; may be `null` |
| `source_module` | derived | Normalize from `source_file` |
| `exception_flags` | derived | Preserve and extend for known exceptions |

Same-location records from overlapping extraction patterns are distinct input
records. They must not be deduplicated.

## Supported types and precedence

The only permitted output values are:

`security`, `authentication`, `authorization`, `timeout`, `performance`,
`concurrency`, `configuration`, `persistence`, `filesystem`, `parsing`,
`network`, `runtime`, `state`, `resource`, `validation`, and `other`.

The first matching message rule wins, in this exact order:

1. security
2. authentication
3. authorization
4. timeout
5. performance
6. concurrency
7. configuration
8. persistence
9. filesystem
10. parsing
11. network
12. runtime
13. state
14. resource
15. validation

Patterns are case-insensitive regular expressions applied to the extracted
message only. There is no confidence scoring, tie-breaking based on match
length, or classifier-specific category invention. Thus a message mentioning
both a WebSocket and a deadline is `timeout`, while a message saying that JSON
parsing failed is `parsing` rather than `network`.

The exact patterns, rule IDs, and category descriptions are in the JSON file;
the order above is normative and must not be inferred from JSON object order.

## Unknown and ambiguous messages

If no message rule matches, source-file fallback rules are considered. A source
file hint is usable only when it yields exactly one distinct category. Zero
matching hints or conflicting hints are ambiguous and resolve to `other`.
Rule order never resolves conflicting source hints.

`other` is therefore the explicit fallback for unknown, empty, generic, and
ambiguous messages, malformed source paths, and records whose meaning cannot be
determined from the handoff. A null `original_message` does not justify
dropping a record: retain it, keep or add `no_original_message` in
`exception_flags`, and use the source fallback or `other`.

## Source module normalization

For a validated repository-relative `file` value:

1. Convert `\\` separators to `/` and remove leading `./` segments.
2. Remove the final `.rs` suffix.
3. Replace `-` with `_` in each path segment.
4. Join non-empty segments with `::`.
5. Preserve case and do not collapse `lib.rs` or `mod.rs`; `file` remains the
   precise grouping identity.

Examples:

```text
hoop-daemon/tests/acceptance/s3_bead_creation_from_chat.rs
  -> hoop_daemon::tests::acceptance::s3_bead_creation_from_chat

./hoop-cli/tests/no-interactive.rs
  -> hoop_cli::tests::no_interactive
```

An invalid source path gets `source_module=unknown_module`, retains its
original `source_file`, and receives an exception flag. It must not be
reconstructed from an absolute path or a path containing `..` segments.

## Deterministic output

After classification, sort records by `source_module`, `error_type`, `file`,
numeric `line`, extraction `category`, and `line_content`, all ascending. The
original identity and all unrelated fields remain unchanged. This makes the
catalog reproducible while retaining every overlapping extraction record.

The checked-in categorized intermediate is
[`categorized_error_catalog.json`](categorized_error_catalog.json). The
machine-readable run record at
[`error_category_catalog_run.json`](error_category_catalog_run.json) records
the taxonomy, normalized input, output, applied precedence, per-type counts,
and the provenance of every fallback or ambiguous record. Run
`python3 bin/categorize_error_messages.py` from the repository root to
reproduce both files; the script defaults to the normalized input and these
documented output paths.

## Source-module grouped intermediate

The same run also writes
[`categorized_error_catalog_by_module.json`](categorized_error_catalog_by_module.json),
the source-module grouping consumed by the final export child. It nests the
complete categorized records as `source_module -> error_type -> records`
(never reduced summaries), so no consumer has to join the grouped view back
against the flat catalog. Modules are sorted lexicographically and error
types follow `allowed_error_types` order inside each module. The document
carries `record_count`, `module_count`, taxonomy-ordered
`counts_by_error_type`, sorted `counts_by_source_module`, and
`unresolved_module_mappings` — the explicit list of records whose `file`
could not be mapped (each also grouped under the `unknown_module` fallback
value from the taxonomy's `source_module_normalization` block). A count of
zero there means every record resolved to a concrete module.

The run record mirrors this for the flat catalog: `source_modules` reports
`module_count`, per-module counts, the `unknown_module` value, and the
unresolved mappings; `field_coverage` records how many records carry
`error_type` and `source_module`. Classification refuses to write any
artifact when a record is missing either field, and the final export child
must treat a nonzero `unresolved_module_mappings.count` as a handoff
exception to resolve or flag, never as data to drop.
