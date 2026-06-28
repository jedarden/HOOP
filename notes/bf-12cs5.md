# Bead bf-12cs5: Categorize unused utoipa imports by crate

## Summary

Categorized all 23 parsed utoipa issues (actual count: 15 imports found) by crate.

## Results

All 15 imports belong to a single crate:

### hoop-daemon (15 imports)
- **Unused import:** 1
  - `api_beads.rs:28` — unused `utoipa::path`

- **Missing ToSchema imports:** 14
  - `api_draft_queue.rs` — lines 34, 72, 79, 89, 99, 106, 115, 123, 131, 138, 145, 161, 167, 175

### Other crates
- hoop-cli: 0 imports
- hoop-mcp: 0 imports  
- hoop-schema: 0 imports

## Files Created

- `.claude/utoipa-by-crate.json` — Categorized data with crate breakdown
