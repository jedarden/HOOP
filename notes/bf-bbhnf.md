# Blocking Errors with Locations

**Date:** 2026-07-02  
**Bead:** bf-bbhnf  
**Total Blocking Errors:** 14 (all E0277 trait bound violations)  
**Source:** bf-89p2v_errors.md

---

## Blocking Errors List

### Error Group 1: ListJobsQuery (2 errors)

**File:** `hoop-daemon/src/openapi.rs:500`  
**Referenced struct location:** `hoop-daemon/src/api_transcription.rs:19`  
**Error type:** E0277 (trait bound not satisfied)

**Error message:**
```
error[E0277]: the trait bound `ListJobsQuery: ToSchema` is not satisfied
   --> hoop-daemon/src/openapi.rs:500:13
    |
500 |             crate::api_transcription::ListJobsQuery,
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
```

**Root cause:** `ListJobsQuery` struct does not implement `ToSchema` trait required by utoipa OpenAPI generation.

---

### Error Group 2: CreateScreenCaptureRequest (2 errors)

**File:** `hoop-daemon/src/api_screen_capture.rs:84`  
**Referenced struct location:** `hoop-daemon/src/api_screen_capture.rs:34`  
**Error type:** E0277 (trait bound not satisfied)

**Error message:**
```
error[E0277]: the trait bound `CreateScreenCaptureRequest: ToSchema` is not satisfied
  --> hoop-daemon/src/api_screen_capture.rs:84:20
   |
84 |     request_body = CreateScreenCaptureRequest,
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
```

**Root cause:** `CreateScreenCaptureRequest` struct does not implement `ToSchema` trait.

---

### Error Group 3: StartStreamingUploadRequest (2 errors)

**File:** `hoop-daemon/src/api_screen_capture.rs:366`  
**Referenced struct location:** `hoop-daemon/src/api_screen_capture.rs:352`  
**Error type:** E0277 (trait bound not satisfied)

**Error message:**
```
error[E0277]: the trait bound `StartStreamingUploadRequest: ToSchema` is not satisfied
  --> hoop-daemon/src/api_screen_capture.rs:366:20
   |
366 |     request_body = StartStreamingUploadRequest,
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
```

**Root cause:** `StartStreamingUploadRequest` struct does not implement `ToSchema` trait.

---

### Error Group 4: CompleteStreamingUploadRequest (2 errors)

**File:** `hoop-daemon/src/api_screen_capture.rs:484`  
**Referenced struct location:** `hoop-daemon/src/api_screen_capture.rs:469`  
**Error type:** E0277 (trait bound not satisfied)

**Error message:**
```
error[E0277]: the trait bound `CompleteStreamingUploadRequest: ToSchema` is not satisfied
  --> hoop-daemon/src/api_screen_capture.rs:484:20
   |
484 |     request_body = CompleteStreamingUploadRequest,
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
```

**Root cause:** `CompleteStreamingUploadRequest` struct does not implement `ToSchema` trait.

---

## Summary Table

| Error Group | File Location | Line | Struct | Error Count |
|-------------|---------------|------|--------|-------------|
| 1 | `hoop-daemon/src/openapi.rs` | 500 | `ListJobsQuery` | 2 |
| 2 | `hoop-daemon/src/api_screen_capture.rs` | 84 | `CreateScreenCaptureRequest` | 2 |
| 3 | `hoop-daemon/src/api_screen_capture.rs` | 366 | `StartStreamingUploadRequest` | 2 |
| 4 | `hoop-daemon/src/api_screen_capture.rs` | 484 | `CompleteStreamingUploadRequest` | 2 |

**Note:** Each struct generates 2 errors (one for `ToSchema`, one for `PartialSchema`), totaling 14 errors.

---

## Required Fixes

Add `#[derive(ToSchema)]` to the following structs:

1. `hoop-daemon/src/api_transcription.rs:19` - `ListJobsQuery`
2. `hoop-daemon/src/api_screen_capture.rs:34` - `CreateScreenCaptureRequest`
3. `hoop-daemon/src/api_screen_capture.rs:352` - `StartStreamingUploadRequest`
4. `hoop-daemon/src/api_screen_capture.rs:469` - `CompleteStreamingUploadRequest`

---

## Non-Blocking Issues (Excluded)

The following were identified but excluded from this blocking error list:
- 74 warnings (unused imports, unused variables, unnecessary mut keywords)
- These do not prevent compilation and can be addressed separately
