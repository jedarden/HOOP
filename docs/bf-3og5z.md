# Bead bf-3og5z: Verify ToSchema derive on CompleteStreamingUploadRequest

## Task
Add ToSchema derive to CompleteStreamingUploadRequest

## Finding
The ToSchema derive was already present on `CompleteStreamingUploadRequest` in `hoop-daemon/src/api_screen_capture.rs` at line 471:

```rust
/// Request body for completing a streaming upload
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct CompleteStreamingUploadRequest {
    duration_secs: f64,
    frame_samples: Vec<screen_capture::FrameSample>,
}
```

## Verification
- ✅ ToSchema derive present via `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]`
- ✅ Compilation verified: `cargo check --package hoop-daemon` passes with no errors
- ✅ No related compilation errors found

## Conclusion
Task already complete - no changes required.
