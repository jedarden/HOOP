# Pattern Service Implementation (hoop-ttb.3.32)

## Summary

This bead completes the Pattern service implementation with CRUD operations and query-based auto-include functionality.

## What Was Already Implemented

### 1. Database Schema (fleet.rs)
- `patterns` table with status constraint (planned, active, blocked, done, abandoned)
- `pattern_members` table for many-to-many relationship between patterns and stitches
- `pattern_queries` table for saved queries
- Foreign key cascade deletion
- Cycle prevention trigger for parent_pattern

### 2. Pattern Query Evaluator (pattern_query_evaluator.rs)
- Query DSL parser supporting:
  - `title:regex` - Match title against regex
  - `label:name` - Match beads with label
  - `project:name` - Match project name exactly
  - `kind:name` - Match kind exactly
  - `AND`, `OR`, `NOT` operators
  - Parentheses for grouping
- Query evaluation against stitch context
- Idempotent member insertion
- Event emission for UI updates
- Slow query logging (100ms threshold)

### 3. Read API (api_patterns.rs)
- `GET /api/patterns` - List all patterns with aggregate stats
- `GET /api/patterns/:id` - Single pattern detail with members
- Parent chain resolution for nested patterns
- Progress calculation based on closed members
- Token aggregation across members

### 4. CLI Interface (hoop-cli/src/patterns.rs)
- `hoop pattern new <title>` - Create pattern
- `hoop pattern list` - List all patterns
- `hoop pattern show <id>` - Show pattern details
- `hoop pattern update <id>` - Update pattern
- `hoop pattern close <id>` - Close pattern (status → done)
- `hoop pattern delete <id>` - Delete pattern
- `hoop pattern add-member <id> <stitch_id>` - Add stitch member
- `hoop pattern remove-member <id> <stitch_id>` - Remove stitch member
- `hoop pattern add-query <id> <query>` - Add saved query
- `hoop pattern remove-query <id> <query>` - Remove saved query

### 5. Write API (api_pattern_mutations.rs)
- `POST /api/patterns` - Create pattern
- `PUT /api/patterns/:id` - Update pattern
- `DELETE /api/patterns/:id` - Delete pattern
- `POST /api/patterns/:id/members` - Add stitch member
- `DELETE /api/patterns/:id/members/:stitch_id` - Remove stitch member
- `POST /api/patterns/:id/queries` - Add saved query
- `DELETE /api/patterns/:id/queries/:query` - Remove saved query

### 6. Status Transition Validation
Valid transitions implemented:
- Done → Active (reopening)
- Planned → Active (starting work)
- Active → Blocked (blocked by dependency)
- Blocked → Active (unblocked)
- Active → Done (completion)
- Any → Abandoned (abandonment)
- Abandoned → Active (reactivating)

Invalid transitions rejected:
- Abandoned → Done (must go through Active first)
- Direct Planned → Done (must go through Active first)

### 7. Integration Points
Pattern query evaluation is called on stitch creation in:
- api_beads.rs (bead creation with stitch label)
- api_stitch_decompose.rs (stitch submission)
- api_dictated_notes.rs (dictated notes)
- screen_capture.rs (screen capture sessions)

## What Was Added

### 1. Module Registration (lib.rs)
- Added `pub mod api_pattern_mutations;` import
- Registered `.merge(api_pattern_mutations::router())` in main router

This was the missing piece - the mutation API existed but wasn't connected to the HTTP router.

## Acceptance Criteria Verification

### ✓ Create pattern via UI form or CLI
- CLI: `hoop pattern new <title>` implemented in patterns.rs
- API: `POST /api/patterns` implemented in api_pattern_mutations.rs

### ✓ Add/remove Stitch members
- Add: `POST /api/patterns/:id/members`
- Remove: `DELETE /api/patterns/:id/members/:stitch_id`
- CLI: `hoop pattern add-member` / `remove-member`

### ✓ Saved query evaluated on every new Stitch
- Implemented in `sync_and_emit_pattern_queries()`
- Called on stitch creation in multiple code paths
- Idempotent insertion prevents duplicates
- Event emission updates UI in real-time

### ✓ Status transitions valid
- Validation in `is_valid_transition()` function
- Clear error messages with valid transitions listed
- Database CHECK constraint for status values

## Testing

Integration tests exist in pattern_query_evaluator_integration.rs:
- Basic query evaluation
- Multiple pattern matches
- Complex expressions (AND, OR, NOT)
- Kind filtering
- Standalone word as label
- Idempotent member insertion

## Notes

The implementation was already complete - only the module registration in lib.rs was missing. All the core functionality existed:
- Database schema with migrations
- Query evaluation engine
- Read and write APIs
- CLI interface
- Status validation
- Integration with stitch creation

The only change needed was to expose the mutation API via the HTTP router.
