# Normalized source error catalog

This is the handoff from extraction to categorization.

## Input artifact and source schema

- Input: [`test_error_messages.json`](../test_error_messages.json)
- Format: UTF-8 JSON array; one object per extracted source match.
- Source identity: `(file, line, category, line_content)`.
- Required fields:
  - `file`: repository-relative Rust source path.
  - `line`: positive, one-based source line number.
  - `category`: extractor pattern label; it is not the semantic error taxonomy.
  - `line_content`: original extracted source line, including its string literal.
- Existing metadata in the input is copied unchanged. The current input also
  contains `error_type` and `source_module`; normalization does not recompute
  or overwrite either field.

The input has 3,466 records. Every record has all four required fields. The
existing `test_error_messages.csv` projection has the same 3,466 source
records, but the JSON array is authoritative.

## Working artifact

- Normalized catalog: [`docs/normalized_error_catalog.json`](normalized_error_catalog.json)
- Generator: [`bin/normalize_source_error_catalog.py`](../bin/normalize_source_error_catalog.py)
- Verification: `python3 bin/normalize_source_error_catalog.py --verify`

The normalized artifact is still a JSON array so the next categorization child
can consume it with the existing catalog loader. It preserves source order and
all source keys, and adds these deterministic fields:

| Field | Meaning |
| --- | --- |
| `source_record_index` | One-based position in the input array. |
| `source_record_id` | Stable position identifier, `r0000001`, etc. |
| `normalized_file` | Slash-normalized repository-relative path, or `null` when invalid. The original `file` is untouched. |
| `normalized_line` | Validated positive line number, or `null` when invalid. The original `line` is untouched. |
| `normalized_category` | Trimmed category label, or `null` when missing. The original `category` is untouched. |
| `original_message` | Extracted literal for expectation/assertion messages, or `null` when no safe literal exists. |
| `message_status` | `extracted`, `not_present`, `provided`, `provided_null`, or `ambiguous`. |
| `normalization_status` | `ready` for structurally valid source records, otherwise `invalid`. |
| `normalization_flags` | Sorted, de-duplicated exception flags. |

## Deterministic exception handling

- Missing or invalid required fields are retained in their original form,
  never dropped or repaired in place. The corresponding normalized field is
  `null`, and a `missing_*`, `missing_required_field:*`, or `invalid_*` flag is
  added; the record is marked `invalid`.
- A value-only `assert_eq!`/`assert_ne!` or a source line without a safe string
  literal gets `original_message: null`, `message_status: not_present`, and
  `no_original_message`. The normalizer never invents text from compared
  values.
- A malformed assertion/message expression gets
  `message_status: ambiguous` and `ambiguous_original_message`; it remains in
  the catalog for explicit downstream review.
- Existing `original_message` values, when present, take precedence and are
  preserved. A non-string non-null supplied value is not coerced; it is flagged
  as invalid and normalized to `null`.
- Multiple matches at one `(file,line)` are expected because extraction
  patterns overlap. Every row is retained and receives
  `source_location_collision`; rows are never deduplicated by location. The
  full four-field source identity remains the handoff key.
- Comment/doc examples remain records and receive `comment_or_doc_source`.

## Verification result for the current handoff

The source validator reports 3,466 input rows, 3,466 normalized rows, zero
missing required fields, zero malformed source records, and zero exact source
duplicates. It reports 1,036 extra rows in location-collision groups and 95
records without a literal message; both are intentional and represented by
flags in the normalized catalog.

The next child should use `docs/normalized_error_catalog.json` as its input and
must retain `source_record_index` plus the four-field source identity through
categorization.
