# Test Failures Summary - `new` Test Suite

## Test Run: `cargo test -p hoop-cli --tests new`

**Date:** 2026-07-04  
**Status:** ✅ ALL TESTS PASSED

## Results

No failures detected. All 8 tests in the `new` module passed successfully:

- `new::tests::parse_missing_closing_fence_errors` - ✅ PASSED
- `new::tests::parse_missing_opening_fence_errors` - ✅ PASSED  
- `new::tests::parse_minimal_frontmatter` - ✅ PASSED
- `new::tests::template_contains_project` - ✅ PASSED
- `new::tests::parse_valid_frontmatter` - ✅ PASSED
- `restore::tests::test_newer_version_rejection_clear_diagnostic` - ✅ PASSED
- `restore::tests::test_newer_version_rejection_diagnostic` - ✅ PASSED
- `restore::tests::test_newer_version_rejection_happens_before_move_aside` - ✅ PASSED

## Full Test Suite

Additionally verified the full hoop-cli test suite (56 tests) - all passed with no failures.

## Conclusion

The `new` command test suite is functioning correctly. No error patterns or failures to categorize.
