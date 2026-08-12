# HOOP Test Error Messages Summary

**Generated:** 2026-08-12T13:53:14.235548Z
**Total findings:** 2462
**Files processed:** 220

---

## Unique Error Messages by Pattern Type

### panic (51 unique messages)

- `Event type mismatch at index {}`
- `Expected Claim, got {other:?}`
- `Expected Complete, got {other:?}`
- `Expected Crash, got {other:?}`
- `Expected Dispatch, got {other:?}`
- `Expected Executing state, got {other:?}`
- `Expected Fail, got {other:?}`
- `Expected Knot state, got {other:?}`
- `Expected Projects command`
- `Expected Projects command at Level 1`
- `Expected Projects subcommand`
- `Expected Projects::Scan command`
- `Expected Quiet with 999 days`
- `Expected Release, got {other:?}`
- `Expected Remove command`
- `Expected Remove command at Level 2`
- `Expected Scan command`
- `Expected Timeout, got {other:?}`
- `Expected Variant0 project`
- `Expected text message (iteration {})`
- `Expected text message for init, got {:?}`
- `Expected text message, got {:?}`
- `Failed to connect WS client {}: {}`
- `Failed to parse event line: {e}\n  Line: {line}`
- `Failed to parse fixture {} as JSON: {}`
- `Failed to parse normalized JSON for {}: {}`
- `Failed to read fixture {}: {}`
- `Failed to read scenario directory {scenario_path:?}: {e}`
- `Failed to read {:?}: {}`
- `Failed to run hoop with args: {:?}`
- ... and 21 more

### assert (394 unique messages)

- `AND query should match`
- `ANSI strip too slow: {:?}`
- `API key should have high entropy: {}`
- `Adapter build should succeed`
- `All Init command no_interactive tests verified`
- `All Remove command no_interactive tests verified`
- `All Scan command no_interactive tests verified`
- `All complex multi-command tests should pass`
- `Audit log should contain DraftApproved entry`
- `Audit log should contain DraftCreated entry`
- `Audit should have DraftApproved`
- `Audit should have DraftCreated`
- `Backup should fail when encryption enabled but age key missing`
- `Bead creation should succeed`
- `Beads must be an array`
- `Beads response should be an array`
- `Beads response should not be empty`
- `Binary files should not be scanned`
- `CLI must parse flag as true`
- `Capacity endpoint should return 200`
- `Capacity should be object or array`
- `Child must receive no_interactive flag`
- `Clean attachment should have no findings`
- `Code blocks should be preserved`
- `Config should have encryption disabled`
- `Config should have encryption enabled`
- `Config status endpoint should return 200`
- `Config status must include 'valid' field`
- `Configured subprocess should succeed`
- `Confirm flag must be true`
- ... and 364 more

### assert_eq (250 unique messages)

- `API should still be accessible`
- `Active adapter should be zai`
- `Active model should be glm-5`
- `Adapter should be zai`
- `Adapter switch should succeed`
- `Agent should be active`
- `Agent should be active after switch`
- `Agent should still be active`
- `Agent spawn should succeed`
- `All 4 messages should be stored`
- `All 4 test cases should succeed`
- `All WebSocket connections should receive init`
- `All approved rules should be preserved`
- `All concurrent requests should succeed`
- `All history messages should be stored`
- `All test cases should succeed`
- `Audit query should return 200`
- `Bead B workspace should match`
- `Bead C workspace should match`
- `Bead list endpoint should return 200`
- `Beads endpoint should return 200`
- `Both Reflection Ledger entries should be preserved`
- `Both flags should produce true`
- `Both positions must yield the same value`
- `Both runtimes should still exist`
- `Both should produce true`
- `Codex should parse 4 good lines`
- `Codex should quarantine 1 bad line`
- `Command should be 'init'`
- `Command should be 'remove'`
- ... and 220 more

### assert_ne (1 unique messages)

- `content hash must change on valid edit`

---

## Pattern Types Without Explicit Messages

- **unwrap** (1476 occurrences): These are implicit assertions with no custom error message
- **unwrap_err** (25 occurrences): These expect an error but have no custom message

---

## Usage Patterns

### Most Common Message Categories

#### Expectation violations (493 unique messages)

- `AND query should match`
- `API key should have high entropy: {}`
- `API should still be accessible`
- `Active adapter should be zai`
- `Active model should be glm-5`
- `Adapter build should succeed`
- `Adapter should be zai`
- `Adapter switch should succeed`
- `Agent should be active`
- `Agent should be active after switch`
- ... and 483 more

#### Precondition violations (99 unique messages)

- `Beads must be an array`
- `Both positions must yield the same value`
- `CLI must parse flag as true`
- `Child must receive no_interactive flag`
- `Config status must include 'valid' field`
- `Confirm flag must be true`
- `Each bead must have a status`
- `Each bead must have a title`
- `Each bead must have an id`
- `Environment variable must be '1'`
- ... and 89 more

#### Operation failures (18 unique messages)

- `Failed to connect WS client {}: {}`
- `Failed to parse args: {:?}`
- `Failed to parse command without flag`
- `Failed to parse event line: {e}\n  Line: {line}`
- `Failed to parse fixture {} as JSON: {}`
- `Failed to parse normalized JSON for {}: {}`
- `Failed to parse with -y flag`
- `Failed to parse with flag after command`
- `Failed to parse with flag before command`
- `Failed to parse without flag`
- ... and 8 more

#### Invalid state (1 unique messages)

- `invalid JSON in fixture {}: {}`

