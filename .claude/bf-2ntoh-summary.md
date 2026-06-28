# Bead bf-2ntoh: Catalog Unused utoipa::ToSchema Imports

## Summary
Successfully cataloged all unused `utoipa::ToSchema` imports across the HOOP workspace.

## Findings
- **Total unused imports: 25**
- **Affected crates:** 1 (hoop-daemon only)
- **Clean crates:** hoop-cli (0), hoop-mcp (0), hoop-schema (0)

## Distribution
All 25 unused imports are in `hoop-daemon`:
- 23 API modules (api_*.rs files)
- 2 core modules (adb_dictate.rs, cross_project_propagation.rs)

## Output
Manifest saved to `.claude/utoipa-unused-imports.txt` with complete file paths and line numbers for each unused import.

## Generated
2025-06-27 via cargo clippy analysis
