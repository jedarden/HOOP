# Verify ToSchema compilation — bf-c68i1

## Task
Verify `cargo check --features openapi` passes and all `ToSchema` derives in
`hoop-daemon/src/api_screen_capture.rs` compile successfully.

## Outcome
✅ **Verified — no changes required.** All `ToSchema` derives were already
present and compile cleanly. The OpenAPI schema generation is wired up correctly
for every screen-capture endpoint.

## Verification performed
1. `cargo check -p hoop-daemon --features openapi`
   → `Finished dev profile [unoptimized + debuginfo] target(s) in 46.22s`
2. `cargo check -p hoop-daemon --features openapi --bin generate_openapi`
   → `Finished dev profile [unoptimized + debuginfo] target(s) in 0.14s`

The second check builds the binary that actually instantiates every
`ToSchema`-derived type into the OpenAPI doc — its clean link is the strongest
proof the schema registration is correct.

Both runs emitted **zero errors** and zero warnings touching
`api_screen_capture.rs`. The only diagnostics were 14 pre-existing warnings
unrelated to screen capture (in `prompt_substitute.rs`, `capacity.rs`,
`sessions.rs`, `stitch_percentile_index.rs`, etc.).

## Acceptance criteria — all met
- ✅ `cargo check --features openapi` completes successfully.
- ✅ No `ToSchema`-related compilation errors in `api_screen_capture.rs`.
- ✅ All 3 request structs have `ToSchema` derives:
  - `CreateScreenCaptureRequest` — `api_screen_capture.rs:34-35`
  - `StartStreamingUploadRequest` — `api_screen_capture.rs:353-354`
  - `CompleteStreamingUploadRequest` — `api_screen_capture.rs:471-472`

## Supporting derives (also confirmed)
For the request schemas to compile, every type they reference must also derive
`ToSchema`. Confirmed present:
- `FrameSample` (referenced by both complete-style requests) — `screen_capture.rs:29-30`
- `RawBytes` (used as the `append_stream_chunk` request body) — `api_screen_capture.rs:28-29`
- `CreateScreenCaptureResponse` — `api_screen_capture.rs:43-44`

## Notes
- The `openapi` feature is on by default (`hoop-daemon/Cargo.toml`: `default = ["openapi"]`).
- Build environment: `nix-shell --run 'cargo ...'` (bare cargo fails on NixOS per AGENTS.md).
- 48G free on root disk at build time — above the 20G clear-target threshold.
