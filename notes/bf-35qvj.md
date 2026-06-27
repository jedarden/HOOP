# Bead bf-35qvj: Unused validation functions in config_resolver.rs

## Finding

The bead description requested fixes for three unused validation functions:
- `yaml_validate_str` (line 985)
- `yaml_validate_u64_range` (line 1022)
- `yaml_validate_f64_range` (line 1059)

## Investigation

These functions do not exist in `hoop-daemon/src/config_resolver.rs`. Running `cargo clippy` shows no `unused_function` warnings for `config_resolver.rs`.

The actual line numbers in the current file contain:
- Line 985: `fn yaml_type_name(v: &serde_yaml::Value) -> &str {` (helper function, in use)
- Line 1022: Comment `// Server` (in resolve function)
- Line 1059: `None,` (in a resolve_opt call)

## Conclusion

The task appears to be already complete. Either:
1. These functions were never added, or
2. They were already removed/fixed in a prior commit

No action required - clippy confirms no unused function warnings in config_resolver.rs.
