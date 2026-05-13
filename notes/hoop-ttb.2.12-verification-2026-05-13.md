# hoop-ttb.2.12: Tag-join resolver verification

**Date:** 2026-05-13
**Status:** ✅ COMPLETE - Implementation verified

## Summary

The tag-join resolver implementation is **complete and production-ready**. All acceptance criteria have been met and verified.

## Implementation Location

- **Primary module:** `hoop-daemon/src/tag_join.rs` (544 lines)
- **Integration:** `hoop-daemon/src/sessions.rs`
- **Event emission:** `SessionEvent::TagJoinBound` variant

## Acceptance Criteria Status

### ✅ 1. Regex validated and tested against fixtures (all four adapters)

**Well-formed tag regex:**
```rust
r"^\[needle:([^:]+):([^:]+):([^:\]]*)\]"
```

**Test coverage:**
- `test_worker_tag_full` - Complete tag with all components
- `test_worker_tag_empty_strand` - Tag with empty strand field
- `test_worker_tag_no_strand_value` - Tag with no strand value
- `test_worker_tag_from_first_user_content` - Tag extraction from first user message
- `test_worker_tag_prefers_first_user_content` - Priority ordering validation

**Adapter-specific tests:**
- `test_adapter_claude_code` - Claude Code adapter
- `test_adapter_codex` - Codex (OpenAI) adapter
- `test_adapter_opencode` - OpenCode adapter
- `test_adapter_gemini` - Gemini adapter
- `test_fixture_aider_session_001` - Aider adapter

**Fixture validation:**
All testrepo fixtures verified with correct tag format:
- Claude: `[needle:alpha:bd-abc123:pluck]`
- Codex: `[needle:bravo:bd-def456:mend]`
- OpenCode: `[needle:charlie:bd-ghi789:weave]`
- Gemini: `[needle:delta:bd-jkl012:unravel]`
- Aider: `[needle:echo:bd-mno345:knot]`

### ✅ 2. Missing tag → session classified as AdHoc, not auto-joined

**Tests:**
- `test_missing_tag_plain_title` - No tag in title
- `test_missing_tag_empty_strings` - Empty content
- `test_missing_tag_with_first_user_content` - No tag in either source
- `test_tag_not_at_start` - Tag not at beginning ignored

**Implementation:**
```rust
pub fn resolve(title: &str, first_user_content: Option<&str>) -> TagJoinResult {
    // ... tag extraction attempts ...

    // Default: ad-hoc (no prefix, no binding)
    TagJoinResult {
        kind: ParsedSessionKind::Variant2(ParsedSessionKindVariant2::AdHoc),
        binding: None,
    }
}
```

### ✅ 3. Malformed tag logged at warn, treated as missing

**Malformed tag detection regex:**
```rust
r"^\[needle:[^\]]*\]"
```

**Tests:**
- `test_malformed_tag_too_few_parts` - Only 1-2 parts
- `test_malformed_tag_two_parts` - Missing strand component
- `test_malformed_tag_too_many_parts` - 4+ parts
- `test_malformed_tag_empty_brackets` - Completely empty tag
- `test_malformed_in_first_user_content` - Malformed in first message

**Implementation:**
```rust
for content in sources.iter().copied().flatten() {
    if malformed_needle_re().is_match(content) {
        warn!(
            "Malformed needle tag in session, treating as missing: {}",
            content.chars().take(80).collect::<String>()
        );
        break;
    }
}
```

### ✅ 4. Binding emitted as `session_bound` event (dual-identity invariant §B1)

**Event definition:**
```rust
TagJoinBound {
    session_id: String,           // HOOP internal stable session ID
    provider_session_id: String,   // Provider-native session ID
    bead_id: String,
    worker: String,
    strand: Option<String>,
    ts: DateTime<Utc>,
}
```

**Idempotent emission function:**
```rust
fn maybe_emit_session_bound(
    event_tx: &broadcast::Sender<SessionEvent>,
    session_id: &str,
    provider_session_id: &str,
    kind: &ParsedSessionKind,
    state: &mut SessionTailerState,
) {
    if let ParsedSessionKind::Variant0 { worker, bead, strand } = kind {
        let key = (bead.clone(), provider_session_id.to_string());
        if state.session_bound_seen.insert(key) {
            // First meeting — emit event and update indexes
            state.bead_to_provider_session.insert(bead.clone(), provider_session_id.to_string());
            state.provider_session_to_bead.insert(provider_session_id.to_string(), bead.clone());
            let _ = event_tx.send(SessionEvent::TagJoinBound { ... });
        }
    }
}
```

**Tests (in sessions.rs):**
- `session_bound_emitted_on_first_join` - First join emits event
- `session_bound_replay_fixture_twice_emits_exactly_once` - Idempotent emission
- `session_bound_different_pairs_each_emit_once` - Different pairs emit independently
- `session_bound_indexes_both_ids` - Forward and reverse indexes populated
- `session_bound_ad_hoc_kind_emits_nothing` - AdHoc sessions don't emit

## Data Structures

### TagJoinResult
```rust
pub struct TagJoinResult {
    pub kind: ParsedSessionKind,
    pub binding: Option<TagBinding>,
}
```

### TagBinding
```rust
pub struct TagBinding {
    pub worker: String,
    pub bead: String,
    pub strand: Option<String>,
}
```

## Integration Points

1. **Session discovery** (`sessions.rs`):
   - Parses first user message from session files
   - Calls `tag_join::resolve(title, first_user_content)`
   - Emits `TagJoinBound` event for worker sessions

2. **Event flow**:
   ```
   Session tailer discovers session
   → Parses first user message
   → Calls tag_join::resolve()
   → If Worker kind: calls maybe_emit_session_bound()
   → TagJoinBound event emitted (idempotent)
   → Event broadcast to WebSocket subscribers
   ```

## Plan Reference

- §5.1 Data flows (tag-join arrow)
- §3 principle 4 (dual-identity invariance)
- §B1 (session_bound idempotency)
- notes/interop-with-needle.md Hook 1
- notes/reference-feature-inventory.md §1

## Conclusion

All acceptance criteria met. The tag-join resolver is production-ready and fully integrated into the session tailer. No additional work required.
