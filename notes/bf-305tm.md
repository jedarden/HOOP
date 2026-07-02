# Blocking Issues Documentation (bf-305tm)

**Generated:** 2026-07-02  
**Purpose:** Document all blocking/critical compilation errors with file locations and full error messages

---

## Executive Summary

**Total Blocking Errors:** 8 compilation errors  
**All errors are:** Trait bound violations (missing `ToSchema` and `PartialSchema` implementations)  
**Blocking Phase 1 CI Gate:** YES - prevents `cargo build` and `cargo test`

---

## Detailed Error List

### Error Group 1: ListJobsQuery (2 errors)

**File:** `hoop-daemon/src/api_transcription.rs:19`  
**Referenced in:** `hoop-daemon/src/openapi.rs:500`

#### Error 1
```text
error[E0277]: the trait bound `ListJobsQuery: ToSchema` is not satisfied
```

**Location:** `hoop-daemon/src/api_transcription.rs:19`  
**Severity:** BLOCKING  
**Traits Missing:** `ToSchema`  
**Context:** Struct is used in OpenAPI schema definition but lacks required derive macro

#### Error 2
```text
error[E0277]: the trait bound `ListJobsQuery: PartialSchema` is not satisfied
```

**Location:** `hoop-daemon/src/api_transcription.rs:19`  
**Severity:** BLOCKING  
**Traits Missing:** `PartialSchema`  
**Context:** Struct is used in OpenAPI schema definition but lacks required derive macro

---

### Error Group 2: CreateScreenCaptureRequest (2 errors)

**File:** `hoop-daemon/src/api_screen_capture.rs:34`  
**Referenced in:** `hoop-daemon/src/api_screen_capture.rs:84`

#### Error 3
```text
error[E0277]: the trait bound `CreateScreenCaptureRequest: ToSchema` is not satisfied
```

**Location:** `hoop-daemon/src/api_screen_capture.rs:34`  
**Severity:** BLOCKING  
**Traits Missing:** `ToSchema`  
**Context:** Struct is used as `request_body` in `#[utoipa::path]` macro but lacks required derive

#### Error 4
```text
error[E0277]: the trait bound `CreateScreenCaptureRequest: PartialSchema` is not satisfied
```

**Location:** `hoop-daemon/src/api_screen_capture.rs:34`  
**Severity:** BLOCKING  
**Traits Missing:** `PartialSchema`  
**Context:** Struct is used as `request_body` in `#[utoipa::path]` macro but lacks required derive

---

### Error Group 3: StartStreamingUploadRequest (2 errors)

**File:** `hoop-daemon/src/api_screen_capture.rs:352`  
**Referenced in:** `hoop-daemon/src/api_screen_capture.rs:366`

#### Error 5
```text
error[E0277]: the trait bound `StartStreamingUploadRequest: ToSchema` is not satisfied
```

**Location:** `hoop-daemon/src/api_screen_capture.rs:352`  
**Severity:** BLOCKING  
**Traits Missing:** `ToSchema`  
**Context:** Struct is used as `request_body` in `#[utoipa::path]` macro but lacks required derive

#### Error 6
```text
error[E0277]: the trait bound `StartStreamingUploadRequest: PartialSchema` is not satisfied
```

**Location:** `hoop-daemon/src/api_screen_capture.rs:352`  
**Severity:** BLOCKING  
**Traits Missing:** `PartialSchema`  
**Context:** Struct is used as `request_body` in `#[utoipa::path]` macro but lacks required derive

---

### Error Group 4: CompleteStreamingUploadRequest (2 errors)

**File:** `hoop-daemon/src/api_screen_capture.rs:469`  
**Referenced in:** `hoop-daemon/src/api_screen_capture.rs:484`

#### Error 7
```text
error[E0277]: the trait bound `CompleteStreamingUploadRequest: ToSchema` is not satisfied
```

**Location:** `hoop-daemon/src/api_screen_capture.rs:469`  
**Severity:** BLOCKING  
**Traits Missing:** `ToSchema`  
**Context:** Struct is used as `request_body` in `#[utoipa::path]` macro but lacks required derive

#### Error 8
```text
error[E0277]: the trait bound `CompleteStreamingUploadRequest: PartialSchema` is not satisfied
```

**Location:** `hoop-daemon/src/api_screen_capture.rs:469`  
**Severity:** BLOCKING  
**Traits Missing:** `PartialSchema`  
**Context:** Struct is used as `request_body` in `#[utoipa::path]` macro but lacks required derive

---

## Structured JSON Format

```json
{
  "blocking_errors": [
    {
      "id": 1,
      "struct": "ListJobsQuery",
      "file": "hoop-daemon/src/api_transcription.rs",
      "line": 19,
      "referenced_in": "hoop-daemon/src/openapi.rs:500",
      "traits_missing": ["ToSchema"],
      "error_code": "E0277",
      "error_message": "the trait bound `ListJobsQuery: ToSchema` is not satisfied",
      "severity": "BLOCKING",
      "category": "trait_bound_violation"
    },
    {
      "id": 2,
      "struct": "ListJobsQuery",
      "file": "hoop-daemon/src/api_transcription.rs",
      "line": 19,
      "referenced_in": "hoop-daemon/src/openapi.rs:500",
      "traits_missing": ["PartialSchema"],
      "error_code": "E0277",
      "error_message": "the trait bound `ListJobsQuery: PartialSchema` is not satisfied",
      "severity": "BLOCKING",
      "category": "trait_bound_violation"
    },
    {
      "id": 3,
      "struct": "CreateScreenCaptureRequest",
      "file": "hoop-daemon/src/api_screen_capture.rs",
      "line": 34,
      "referenced_in": "hoop-daemon/src/api_screen_capture.rs:84",
      "traits_missing": ["ToSchema"],
      "error_code": "E0277",
      "error_message": "the trait bound `CreateScreenCaptureRequest: ToSchema` is not satisfied",
      "severity": "BLOCKING",
      "category": "trait_bound_violation"
    },
    {
      "id": 4,
      "struct": "CreateScreenCaptureRequest",
      "file": "hoop-daemon/src/api_screen_capture.rs",
      "line": 34,
      "referenced_in": "hoop-daemon/src/api_screen_capture.rs:84",
      "traits_missing": ["PartialSchema"],
      "error_code": "E0277",
      "error_message": "the trait bound `CreateScreenCaptureRequest: PartialSchema` is not satisfied",
      "severity": "BLOCKING",
      "category": "trait_bound_violation"
    },
    {
      "id": 5,
      "struct": "StartStreamingUploadRequest",
      "file": "hoop-daemon/src/api_screen_capture.rs",
      "line": 352,
      "referenced_in": "hoop-daemon/src/api_screen_capture.rs:366",
      "traits_missing": ["ToSchema"],
      "error_code": "E0277",
      "error_message": "the trait bound `StartStreamingUploadRequest: ToSchema` is not satisfied",
      "severity": "BLOCKING",
      "category": "trait_bound_violation"
    },
    {
      "id": 6,
      "struct": "StartStreamingUploadRequest",
      "file": "hoop-daemon/src/api_screen_capture.rs",
      "line": 352,
      "referenced_in": "hoop-daemon/src/api_screen_capture.rs:366",
      "traits_missing": ["PartialSchema"],
      "error_code": "E0277",
      "error_message": "the trait bound `StartStreamingUploadRequest: PartialSchema` is not satisfied",
      "severity": "BLOCKING",
      "category": "trait_bound_violation"
    },
    {
      "id": 7,
      "struct": "CompleteStreamingUploadRequest",
      "file": "hoop-daemon/src/api_screen_capture.rs",
      "line": 469,
      "referenced_in": "hoop-daemon/src/api_screen_capture.rs:484",
      "traits_missing": ["ToSchema"],
      "error_code": "E0277",
      "error_message": "the trait bound `CompleteStreamingUploadRequest: ToSchema` is not satisfied",
      "severity": "BLOCKING",
      "category": "trait_bound_violation"
    },
    {
      "id": 8,
      "struct": "CompleteStreamingUploadRequest",
      "file": "hoop-daemon/src/api_screen_capture.rs",
      "line": 469,
      "referenced_in": "hoop-daemon/src/api_screen_capture.rs:484",
      "traits_missing": ["PartialSchema"],
      "error_code": "E0277",
      "error_message": "the trait bound `CompleteStreamingUploadRequest: PartialSchema` is not satisfied",
      "severity": "BLOCKING",
      "category": "trait_bound_violation"
    }
  ],
  "summary": {
    "total_errors": 8,
    "unique_structs_affected": 4,
    "files_affected": 2,
    "error_type": "trait_bound_violation",
    "blocking_phase_1_ci_gate": true,
    "fix_type": "add_derive_macro"
  }
}
```

---

## Fix Pattern

All blocking errors follow the same fix pattern. Add the `#[derive(ToSchema)]` attribute to each struct:

```rust
// hoop-daemon/src/api_transcription.rs:19
#[derive(ToSchema)]
pub struct ListJobsQuery {
    // ... existing fields ...
}

// hoop-daemon/src/api_screen_capture.rs:34
#[derive(ToSchema)]
struct CreateScreenCaptureRequest {
    // ... existing fields ...
}

// hoop-daemon/src/api_screen_capture.rs:352
#[derive(ToSchema)]
struct StartStreamingUploadRequest {
    // ... existing fields ...
}

// hoop-daemon/src/api_screen_capture.rs:469
#[derive(ToSchema)]
struct CompleteStreamingUploadRequest {
    // ... existing fields ...
}
```

---

## Impact Analysis

**Before Fix:**
- `cargo build`: FAILS (8 errors)
- `cargo test`: CANNOT RUN (build fails)
- `cargo clippy -- -D warnings`: NOT REACHABLE

**After Fix:**
- `cargo build`: Should succeed
- `cargo test`: Should run
- `cargo clippy -- -D warnings`: Should run (may have warnings to address separately)

---

## Related Documentation

- See `notes/bf-jmb87.md` for original error analysis
- See `notes/bf-19zug.md` for error categorization
- Supports Phase 1 CI gate (bead `bf-5mpcl`)

---

## Acceptance Criteria Met

✅ Filtered for blocking/critical errors only  
✅ Extracted file path and line number for each  
✅ Documented the full error message  
✅ Created a structured list (both markdown and JSON formats)

**Bead:** bf-305tm  
**Status:** COMPLETE
