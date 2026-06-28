# Bead bf-20z0s: Unused utoipa imports already removed

## Finding

The unused `utoipa::ToSchema` imports specified in this bead were already removed in prior commits before this bead was created.

## Timeline

- **2026-06-28 00:58:16 -0400**: Commit `16086f9` - "refactor: remove unused utoipa::ToSchema from request-only structs"
  - Removed `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` from `SkillRunRequest` in `api_skills.rs`
  - Also removed from other request-only structs across multiple files
  
- **2026-06-28 01:15:17 -0400**: Commit `0e24592` - "refactor: remove unused utoipa::ToSchema imports from core modules"
  - Removed explicit `use utoipa::ToSchema;` import statements from other modules

- **2026-06-28 05:37:14Z**: Bead `bf-20z0s` created

## Current State

All three target files are now clean:

### `api_skills.rs`
- No explicit `use utoipa::ToSchema;` import
- Only response type has ToSchema: `SkillRunResponse` (correct)
- Request type `SkillRunRequest` does NOT have ToSchema (correct)

### `api_stitch_traversal.rs`
- No explicit `use utoipa::ToSchema;` import
- Only response types have ToSchema:
  - `ParentsResponse`
  - `ChildrenResponse`
  - `ReferencedByResponse`
  - `ClosureResponse`
  - `StitchLinkInfo`
  - `ClosureNodeInfo`

### `api_timeline.rs`
- No explicit `use utoipa::ToSchema;` import
- Only response types have ToSchema:
  - `TimelineSegment`
  - `WorkerTimeline`
  - `TimelineResponse`

## Acceptance Criteria

✓ All 3 unused imports removed (already done in prior commits)
✓ Each file compiles (no syntax errors)
✓ Only utoipa::ToSchema removed, no utoipa::ToResponse touched (no ToResponse in these files)

## Conclusion

No additional work is required. The bead's acceptance criteria are already met by commits `16086f9` and `0e24592`.
