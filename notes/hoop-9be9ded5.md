# Grouped-catalog handoff for the export child (hoop-9be9ded5)

- **Date:** 2026-10-12
- **Parent bead:** hoop-4d8451f4 ("Add source-module grouping to the categorized
  catalog"), last child of the four-way auto-split (commit `38c054d7`)
- **Deliverable commit:** `8dbfd82c416a907a7c04d0e002784cfcbd87a14a` — ancestor of
  the verified HEAD; this entry documents its artifacts so the export child needs
  no re-derivation.

## Output paths

| Artifact | Path | Role for the export child |
| --- | --- | --- |
| Grouped catalog (primary input) | `docs/categorized_error_catalog_by_module.json` | `source_module -> error_type -> records`, complete records, never summaries |
| Flat catalog | `docs/categorized_error_catalog.json` | Sorted 1:1 counterpart; bijection with the grouped view verified |
| Run record | `docs/error_category_catalog_run.json` | Counts, field coverage, unresolved mappings, provenance |
| Taxonomy + rules | `docs/error_category_taxonomy.md`, `docs/error_category_taxonomy.json` | Normative classification and module-derivation rules |

Reproduce with `python3 bin/categorize_error_messages.py` from the repository
root (one run writes all of the above).

## Mapping rule (no re-derivation needed)

`source_module` is derived per
[error_category_taxonomy.md § Source module normalization](../docs/error_category_taxonomy.md):
forward-slash the path, strip `./`, drop the trailing `.rs`, `-`→`_` per
segment, join with `::`, preserve case, never collapse `lib.rs`/`mod.rs`.
Invalid paths (absolute, or containing `..`) get
`source_module="unknown_module"`, keep their original `source_file`, and gain
an exception flag — they are never reconstructed.

## Counts (verified at committed HEAD)

- **3466** records total; `field_coverage`: 3466/3466 carry both `error_type`
  and `source_module`.
- **97** source modules (`module_count` in both the run record and the grouped
  artifact; per-module dict identical in both, summing to 3466).
- `counts_by_error_type` (taxonomy order, sums to 3466): authorization 21,
  concurrency 33, configuration 238, filesystem 698, network 537, parsing 379,
  persistence 248, performance 19, resource 242, runtime 460, security 2,
  state 314, timeout 29, validation 162, authentication 0, other 84.
- Grouped view internals re-verified: every nested `records` list agrees with
  its module `record_count` and per-module `counts_by_error_type`; modules
  sorted lexicographically; error types follow
  `allowed_error_types` order; multiset of
  (`source_file`, `source_line`, `line_content`, `category`) identities is
  identical between flat and grouped views.

## Unresolved module mappings: 0

`unresolved_module_mappings` is explicit and empty in **both** the run record
(`source_modules.unresolved_module_mappings`) and the grouped artifact:
`{"count": 0, "records": []}`. Every record resolved to a concrete module:
`unknown_module` does not appear among the 97 grouped module keys and no module
has empty `records` — the `unknown_module` name survives only as the declared
fallback value in both documents' schemas.

Standing rule for the export child: a **nonzero**
`unresolved_module_mappings.count` is a handoff exception to resolve or flag —
never data to drop.

## Verification performed on this bead

Clean `git archive HEAD` extraction (post-commit): `py_compile` exit 0;
categorizer suite `python3 -m pytest bin/test_categorize_error_messages.py`
exit 0 (22 tests); regeneration `python3 bin/categorize_error_messages.py`
exit 0 and byte-identical (sha256) to all three committed artifacts; the
acceptance assertions above re-run in the extraction, exit 0. No source file
was modified by this bead; the sole change is this handoff entry.
