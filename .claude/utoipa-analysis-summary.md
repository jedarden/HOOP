# Unused utoipa::ToSchema Imports Analysis

## Summary
Found **25 unused `utoipa::ToSchema` imports** across the HOOP workspace (not 23 as initially expected).

## Distribution by Crate

| Crate | Count | Percentage |
|-------|-------|------------|
| hoop-daemon | 25 | 100% |
| hoop-cli | 0 | 0% |
| hoop-mcp | 0 | 0% |
| hoop-schema | 0 | 0% |

## File Breakdown

### API Modules (23 files)
All API files with unused ToSchema imports:

1. `api_agent.rs:248`
2. `api_bead_files.rs:19`
3. `api_beads.rs:28`
4. `api_blame.rs:23`
5. `api_config.rs:13`
6. `api_conversations.rs:17`
7. `api_diff.rs:10`
8. `api_morning_brief.rs:19`
9. `api_pattern_mutations.rs:22`
10. `api_patterns.rs:20`
11. `api_presence.rs:24`
12. `api_propagation.rs:17`
13. `api_reflection_ledger.rs:21`
14. `api_scripts.rs:29`
15. `api_screen_capture.rs:28`
16. `api_skills.rs:61`
17. `api_stitch_traversal.rs:19`
18. `api_timeline.rs:20`
19. `api_transcription.rs:18`
20. `api_tour_project.rs:23`
21. `api_unassigned.rs:37`
22. `api_uploads.rs:14`

### Core Modules (2 files)
1. `adb_dictate.rs:29`
2. `cross_project_propagation.rs:22`

## Next Steps
These unused imports should be removed to clean up the codebase. The manifest file `.claude/utoipa-unused-imports.txt` contains the complete list for reference during removal.
