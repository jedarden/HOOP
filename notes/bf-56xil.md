# bf-56xil: Verify ToSchema derive on CompleteStreamingUploadRequest

## Task
Add `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` to `CompleteStreamingUploadRequest` struct in `hoop-daemon/src/api_screen_capture.rs`.

## Finding
The `ToSchema` derive was already present on `CompleteStreamingUploadRequest` at line 471 using conditional compilation:

```rust
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct CompleteStreamingUploadRequest {
    duration_secs: f64,
    frame_samples: Vec<screen_capture::FrameSample>,
}
```

This is the correct pattern used throughout the HOOP codebase — the derive is only applied when the `openapi` feature is enabled. The struct is referenced by the `complete_streaming_upload` utoipa endpoint (line 487 `request_body`) and handler (line 499).

Its field type `screen_capture::FrameSample` also already carries the `ToSchema` derive (`hoop-daemon/src/screen_capture.rs:29`), so the full schema chain compiles.

## Verification
- ✅ `cargo check -p hoop-daemon --features hoop-daemon/openapi` passed (`Finished dev profile`) with only pre-existing warnings unrelated to this struct
- ✅ No "trait bound `CompleteStreamingUploadRequest: ToSchema` is not satisfied" error
- ✅ Sibling request structs in the same file (e.g. `StartStreamingUploadRequest`) use the same pattern

## Conclusion
No source changes required — the derive was already properly configured.
