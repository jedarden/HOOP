# bf-2rd49: Verify ToSchema derive on StartStreamingUploadRequest

## Task
Add `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` to `StartStreamingUploadRequest` struct in `hoop-daemon/src/api_screen_capture.rs`.

## Finding
The `ToSchema` derive was already present on `StartStreamingUploadRequest` at line 353 using conditional compilation:

```rust
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct StartStreamingUploadRequest {
    video_content_type: String,
}
```

This is the correct pattern used throughout the HOOP codebase - the derive is only applied when the `openapi` feature is enabled. The struct is referenced by the `start_streaming_upload` utoipa endpoint (line 368 `request_body`) and handler (line 379).

## Verification
- ✅ `cargo check -p hoop-daemon --features hoop-daemon/openapi` passed (`Finished dev profile`) with only pre-existing warnings unrelated to this struct
- ✅ No "trait bound `StartStreamingUploadRequest: ToSchema` is not satisfied" error
- ✅ Sibling request structs in the same file (e.g. `CreateScreenCaptureRequest`) use the same pattern

## Conclusion
No changes required - the derive was already properly configured.
