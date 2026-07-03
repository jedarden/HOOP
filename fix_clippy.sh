#!/usr/bin/env bash
# Workflow script to fix all clippy errors in the HOOP codebase

set -e

echo "Running clippy to get all errors..."
nix-shell -p pkg-config openssl --run 'cargo clippy --workspace -- -D warnings 2>&1' | grep "^error:" | wc -l

echo "Total clippy errors found. Proceeding with fixes..."

# The clippy errors fall into these categories:
# 1. needless_borrow - remove unnecessary &
# 2. manual_clamp - use .clamp() instead of .min().max()
# 3. double_ended_iterator_last - use .next_back() instead of .last()
# 4. redundant_closure - use function directly instead of closure
# 5. unnecessary_sort_by - use .sort_by_key() instead of .sort_by()
# 6. unnecessary_lazy_evaluations - use .unwrap_or() instead of .unwrap_or_else()
# 7. manual_flatten - use .flatten() instead of if let Ok()
# 8. useless_conversion - remove unnecessary PathBuf::from()
# 9. option_if_let_else - use map() instead of if let
# 10. cast_ref_to_ptr - use .as_ptr() instead of &*expr
# 11. needless_borrows_for_generic_args - remove & before generic args
# 12. useless_format - remove unnecessary format! macro
# 13. disallowed_methods - std::fs::write, std::fs::File::create (test code)
# 14. many_more - various specific clippy lints

echo "Clippy fixes needed. Will proceed file by file."
echo "See /tmp/clippy_output.txt for full details."

cat /tmp/clippy_output.txt | grep -E "^error:" | wc -l
