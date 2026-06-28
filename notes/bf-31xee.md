---
name: bf-31xee
description: Remove unused utoipa imports from api_reflection_ledger, api_scripts, api_screen_capture
metadata:
  type: task
---

## Task: Remove unused utoipa imports from 3 API modules

### Verification Result

The target files do NOT contain unused `use utoipa::ToSchema;` imports.

**Files examined:**
1. `hoop-daemon/src/api_reflection_ledger.rs`
2. `hoop-daemon/src/api_scripts.rs`
3. `hoop-daemon/src/api_screen_capture.rs`

**Current state:** All three files correctly use the full path in derive attributes:
```rust
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
```

This pattern does NOT require a separate `use utoipa::ToSchema;` import statement.

### Acceptance Criteria Status

- [x] All 3 unused imports removed — **N/A** (no such imports existed)
- [x] Files use correct pattern — **PASS** (inline full-path usage)
- [x] Only utoipa::ToSchema affected — **PASS** (no ToResponse usage affected)

### Conclusion

No code changes required. The files already follow the correct pattern for conditional OpenAPI schema derivation.
