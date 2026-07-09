# bf-4g2dk: Verify ToSchema derive on EnableTourRequest

## Task
Add `#[derive(ToSchema)]` to `EnableTourRequest` struct in `hoop-daemon/src/api_tour_project.rs:34`

## Finding
The `ToSchema` derive was already present on `EnableTourRequest` at line 34 using conditional compilation:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EnableTourRequest {
    /// Optional custom path (defaults to ~/.hoop/tour/)
    pub path: Option<String>,
}
```

This is the correct pattern used throughout the HOOP codebase - the derive is only applied when the `openapi` feature is enabled.

## Verification
- ✅ `cargo check -p hoop-daemon` passed without errors
- ✅ No 'trait bound `EnableTourRequest: ToSchema is not satisfied' error
- ✅ All other structs in the same file (`TourProjectResponse`, `TourStitchInfo`) also use the same pattern

## Conclusion
No changes required - the derive was already properly configured.
