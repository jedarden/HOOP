# HOOP Clippy Warnings Analysis

## Summary

- **Total Warnings**: 99
- **Total Errors**: 8 (compilation failures - these must be fixed first)
- **Files with warnings**: 30+

## Critical Errors (Compilation Blockers)

These errors prevent compilation and must be fixed before addressing warnings:

### E0425: Function not found (2 errors)
- `hoop-daemon/src/lib.rs:1980` - `update_per_project_patterns` not found in `secrets_scanner`
- `hoop-daemon/src/lib.rs:2024` - `update_per_project_patterns` not found in `secrets_scanner`

### E0609: Field not found (2 errors)
- `hoop-daemon/src/lib.rs:1979` - `redaction` field not found on `ResolvedConfig`
- `hoop-daemon/src/lib.rs:2023` - `redaction` field not found on `ResolvedConfig`

### E0061/E0277: Function signature mismatch (4 errors)
- `hoop-daemon/src/lib.rs:3101` - `update_all_orphan_metrics` called with 2 args, takes 1
- `hoop-daemon/src/lib.rs:3109` - `update_all_orphan_metrics` called with 2 args, takes 1
- Both calls also incorrectly `.await` a non-async function

---

## Warnings by Category

### 1. Unused Imports (66 warnings)

The most common category. Most are `utoipa::ToSchema` imports that can be removed.

#### utoipa::ToSchema (25 occurrences)
Files affected:
- api_agent.rs, api_bead_files.rs, api_config.rs, api_morning_brief.rs
- api_pattern_mutations.rs, api_patterns.rs, api_propagation.rs
- api_timeline.rs, api_transcription.rs, api_uploads.rs
- api_presence.rs, api_reflection_ledger.rs, api_stitch_traversal.rs
- adb_dictate.rs, api_beads.rs, api_conversations.rs
- api_scripts.rs, api_unassigned.rs, api_skills.rs
- api_tour_project.rs, api_blame.rs, api_diff.rs
- api_screen_capture.rs, dictated_notes.rs, cross_project_propagation.rs

#### Other unused imports (41 occurrences)
- PathBuf (2), warn (2), State, Connection, params, Deserialize (2)
- get (2), Arc, ReplayOptions, ParsedSessionKind, RecommendedWatcher
- Duration, OpenCodeLimits, chrono::Utc, HashMap (2)
- serde::Serialize (2), anyhow, anyhow::Result, bail, json
- SubstitutionContext, SimilarStitch, DateTime, delete (2), put (2)
- self, Path, TcpStream, log_rotation, AgentConfigChanged, utoipa::path

### 2. Unused Variables (22 warnings)

Variables that are declared but never used:

- start (timing) - 4 occurrences
- remote_addr, required_role (2), elapsed_ms, timed_out (2)
- config, event_type, initial_hash, cfg, link_kind, schedule
- overlap_policy, workspace, transition_secs, created_by, conn (2)
- sim, source_labels, create_req, attachments_dir, dashboard, abs_path

### 3. Unused Mut (7 warnings)

Variables declared as `mut` but never mutated:

- conn (4 occurrences), gemini_dirs, opencode_dirs
- shared_files, shared_labels

### 4. Unused Assignments (3 warnings)

Values assigned but never read:

- timed_out - hoop-mcp/src/skills.rs:321
- timed_out - hoop-daemon/src/api_scripts.rs:371
- timed_out - hoop-daemon/src/api_skills.rs:354

### 5. Dead Code (4 warnings)

Public functions never called:

- project_workspace - hoop-mcp/src/notes.rs:23
- redact_json_string - hoop-mcp/src/redaction.rs:26
- skills_to_mcp_tools - hoop-mcp/src/skills.rs:406
- find_skill_by_tool_name - hoop-mcp/src/skills.rs:421

### 6. Iterator Safety (2 warnings)

Potential infinite loops with `flatten()` on `io::Lines`:

- hoop-mcp/src/skills.rs:300 - Suggests `map_while(Result::ok)`
- hoop-mcp/src/skills.rs:308 - Suggests `map_while(Result::ok)`

### 7. Unknown Lints (3 warnings)

Command-line warnings (incorrect flags):
- clippy::dead_code - Use `-W dead_code` instead
- clippy::unused_* - Invalid wildcard syntax
- clippy::warnings - Use `-W warnings` instead

---

## Files Ranked by Warning Count

### Most Warnings (5+)

| File | Warnings | Categories |
|------|----------|------------|
| cross_project_propagation.rs | 6 | unused_import (3), unused_variables (3), unused_mut (2) |
| lib.rs | 3 | unused_import (1), unused_variable (1), plus errors (4) |
| capacity.rs | 4 | unused_import (2), unused_variables (2), unused_mut (2) |
| auth.rs | 3 | unused_variables (3) |
| api_scripts.rs | 4 | unused_import (1), unused_variables (2), unused_assignment (1) |
| api_skills.rs | 4 | unused_import (2), unused_variables (2), unused_assignment (1) |
| config_watcher.rs | 2 | unused_variables (2) |
| fix_patterns.rs | 2 | unused_mut (2) |

### hoop-mcp warnings (10 total)

- skills.rs - 6 warnings
- notes.rs - 1 warning
- redaction.rs - 1 warning

---

## Recommended Fix Priority

### Priority 1: Fix Compilation Errors (8)
Without fixing these, the code doesn't compile:
1. Implement or remove `update_per_project_patterns` calls
2. Fix `redaction` field access on `ResolvedConfig`
3. Fix `update_all_orphan_metrics` calls (remove semaphore arg, remove .await)

### Priority 2: Safety Fixes (2)
Infinite loop potential in hoop-mcp/src/skills.rs:
- Replace `flatten()` with `map_while(Result::ok)` on lines 300 and 308

### Priority 3: High-Volume Cleanup (66)
Remove unused imports - these are quick wins:
- Bulk remove `utoipa::ToSchema` imports (25 occurrences)
- Remove other unused imports (41 occurrences)

### Priority 4: Code Cleanliness (33)
Fix unused variables/assignments:
- Prefix with underscore (22 variables)
- Remove `mut` keyword (7 variables)
- Fix or remove unused assignments (3)

### Priority 5: Dead Code Review (4)
Determine if these functions should be:
- Kept for future use (add `#[allow(dead_code)]`)
- Removed entirely
- Actually used somewhere we missed

---

## Corrected Clippy Command

The command used incorrect lint names. The correct version:

```bash
cargo clippy --workspace -- -W warnings -D warnings
```

Or to focus on specific categories:
```bash
cargo clippy --workspace -- -W unused_imports -W unused_variables -W dead_code
```
