# bf-mce0l: testrepo fixture file count verification

## Investigation

Plan §14.1 specifies: "A realistic file tree (Rust crate + docs + config, ~500 files)"

### Current testrepo state (as of 2026-05-31)

| Metric | Count |
|--------|-------|
| Total git-tracked files | **565** |
| .rs files | 222 |
| .md files | 121 |
| Files in src/ | 99 |
| Files in docs/ | 115 |
| Files in benches/ | 21 |
| Files in fixtures/ | 26 |
| Files in proto/ | 20 |
| Files in cli-sessions/ | 10 |

### Conclusion

**The testrepo fixture already meets plan §14.1 specification.** The current 565 files is within the expected "~500 files" range (±10% is reasonable for "~" notation).

The bead description stating "Currently testrepo/src/ has 13 files" was based on outdated or incorrect information. The actual count is 99 files in src/ and 565 total.

## Populate script behavior

`scripts/populate-testrepo.sh` generates synthetic load test data in memory (for performance budget verification tests) and does NOT create the fixture file tree. The file tree already exists in the repository and is version-controlled.

## Recommendation

No changes to the fixture are needed. The plan §14.1 specification is satisfied by the current testrepo structure.

If future expansion is needed, the populate-testrepo.sh script should remain focused on synthetic data generation only. Fixture file tree changes should be made directly to the testrepo/ directory as normal code changes.
