# bf-64pqx: Verify hoop-mcp dependencies

## Summary

All dependencies declared in `hoop-mcp/Cargo.toml` have been verified as available.

## Dependencies by Category

### Workspace Dependencies
- `tokio` - defined in `[workspace.dependencies]` with `features = ["full"]`
- `tracing` - defined in `[workspace.dependencies]`
- `tracing-subscriber` - defined in `[workspace.dependencies]` with `features = ["env-filter"]`
- `serde_json` - defined in `[workspace.dependencies]`

### Local Workspace Dependency
- `hoop-schema = { path = "../hoop-schema" }` - verified: `hoop-schema/Cargo.toml` exists and workspace includes `"hoop-schema"` in `members`

### External Crates (crates.io verified)
All external crates are published on crates.io:

**Main dependencies:**
- anyhow = "1.0" ✓
- serde_yaml = "0.9" ✓
- chrono = "0.4" (with serde feature) ✓ (also in workspace)
- uuid = "1" (with v4 feature) ✓
- dirs = "5.0" ✓
- rusqlite = "0.30" (with bundled feature) ✓
- sha2 = "0.10" ✓
- hex = "0.4" ✓
- regex = "1" ✓
- ignore = "0.4" ✓
- clap = "4.5" (with derive feature) ✓
- reqwest = "0.12" (with json, blocking, rustls-tls features) ✓
- jsonschema = "0.26" ✓
- fnv = "1.0" ✓
- serde = "1.0" (with derive feature) ✓ (also in workspace)

**Dev dependencies:**
- tempfile = "3" ✓
- trybuild = "1.0" ✓

## Verification Method
- Local dependencies: Verified by checking file paths and workspace member list
- External crates: Verified using `curl -s -A "hoop-mcp-verification/1.0" "https://crates.io/api/v1/crates/<crate-name>"`

## Acceptance Status
✓ All dependencies listed in `hoop-mcp/Cargo.toml` are accounted for (either local workspace members or published crates on crates.io)
