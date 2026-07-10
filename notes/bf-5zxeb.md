# Bead bf-5zxeb: Verify ToSchema derive on CreateScreenCaptureRequest

## Task
Add ToSchema derive to CreateScreenCaptureRequest

## Finding
The ToSchema derive was already present on the struct.

## Location
File: `/home/coding/HOOP/hoop-daemon/src/api_screen_capture.rs`
Lines: 32-40

## Current State
```rust
/// Request body for creating a screen capture
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct CreateScreenCaptureRequest {
    video_data: String,
    video_content_type: String,
    duration_secs: f64,
    frame_samples: Vec<FrameSample>,
}
```

The `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` attribute is present on line 34.

## Related
This verification is part of a pattern of similar beads (bf-2rd49, bf-56xil) that were verifying ToSchema derives on request/response structs.
