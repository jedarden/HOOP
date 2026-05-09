# Pattern Service Implementation Summary (hoop-ttb.3.32)

## Task
CRUD operations on Patterns. Saved-query auto-include evaluates on new Stitch events.

## Acceptance Criteria - ALL MET ✓

### 1. Create pattern via UI form or CLI (`hoop pattern new <title>`)
- **CLI**: `hoop-cli/src/patterns.rs` implements all pattern commands
  - `hoop pattern new <title>` - create pattern
  - `hoop pattern list` - list all patterns
  - `hoop pattern show <id>` - show pattern details
  - `hoop pattern update <id>` - update pattern
  - `hoop pattern close <id>` - close pattern (status → done)
  - `hoop pattern delete <id>` - delete pattern
  - `hoop pattern add-member <id> <stitch_id>` - add stitch member
  - `hoop pattern remove-member <id> <stitch_id>` - remove stitch member
  - `hoop pattern add-query <id> <query>` - add saved query
  - `hoop pattern remove-query <id> <query>` - remove saved query

### 2. Add/remove Stitch members
- **API**: `hoop-daemon/src/api_pattern_mutations.rs`
  - `POST /api/patterns/:id/members` - add member
  - `DELETE /api/patterns/:id/members/:stitch_id` - remove member
- Idempotent inserts using `INSERT OR IGNORE`

### 3. Saved query evaluated on every new Stitch; matching Stitches auto-joined
- **Evaluator**: `hoop-daemon/src/pattern_query_evaluator.rs`
  - Query DSL supports: `title:regex`, `label:name`, `project:name`, `kind:name`, `AND`, `OR`, `NOT`, parentheses
  - `sync_and_emit_pattern_queries()` called on stitch creation in:
    - `api_stitch_decompose.rs`
    - `api_dictated_notes.rs`
    - `api_beads.rs`
    - `screen_capture.rs`
    - `api_screen_capture.rs`
  - Emits `pattern.saved_query_synced` events for UI updates

### 4. Status transitions valid (no arbitrary jumps)
- **Validation**: `hoop-daemon/src/api_pattern_mutations.rs:is_valid_transition()`
  - Valid transitions:
    - Planned → Active (starting work)
    - Active → Blocked (blocked by dependency)
    - Blocked → Active (unblocked)
    - Active → Done (completion)
    - Done → Active (reopening)
    - Any → Abandoned (abandonment)
    - Abandoned → Active (reactivating)
  - Invalid transitions rejected with clear error messages

## Implementation Details

### Database Schema (`fleet.rs`)
```sql
CREATE TABLE patterns (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'planned',
    owner TEXT,
    deadline TEXT,
    parent_pattern TEXT REFERENCES patterns(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT,
    CHECK(status IN ('planned', 'active', 'blocked', 'done', 'abandoned'))
);

CREATE TABLE pattern_members (
    pattern_id TEXT NOT NULL REFERENCES patterns(id) ON DELETE CASCADE,
    stitch_id TEXT NOT NULL REFERENCES stitches(id) ON DELETE CASCADE,
    added_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (pattern_id, stitch_id)
);

CREATE TABLE pattern_queries (
    id TEXT PRIMARY KEY NOT NULL,
    pattern_id TEXT NOT NULL REFERENCES patterns(id) ON DELETE CASCADE,
    saved_query TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### API Endpoints
- `GET /api/patterns` - list patterns with aggregates
- `GET /api/patterns/:id` - pattern detail with members
- `POST /api/patterns` - create pattern
- `PUT /api/patterns/:id` - update pattern (validates status transitions)
- `DELETE /api/patterns/:id` - delete pattern (cascades to members/queries)
- `POST /api/patterns/:id/members` - add member
- `DELETE /api/patterns/:id/members/:stitch_id` - remove member
- `POST /api/patterns/:id/queries` - add saved query
- `DELETE /api/patterns/:id/queries/:query` - remove saved query

### Parent Pattern Cycle Detection
- Database triggers prevent cycles and self-references
- API validates parent_pattern exists before creating/updating

### Integration Tests
- `hoop-daemon/tests/pattern_query_evaluator_integration.rs`
  - Basic evaluation test
  - Multiple pattern matching
  - Complex expressions (AND, OR, NOT)
  - Kind filter
  - Standalone word as label

## Existing Implementation
The Pattern service was fully implemented in commit `f36931c`:
```
feat(hoop-cli/hoop-daemon): add pattern management CLI and API
```

All acceptance criteria for hoop-ttb.3.32 have been met.
