# HOOP Debug Build Output

**Bead:** bf-u8z59  
**Date:** 2026-07-02  
**Build Command:** `cargo build`

## Summary

The debug build fails with **22 compilation errors** and **74 warnings**.

## Errors (22 total)

All errors are related to missing `ToSchema` trait implementations for OpenAPI documentation:

1. **`ScriptRunRequest`** (hoop-daemon/src/api_scripts.rs:162) - Missing `ToSchema` and `PartialSchema`
2. **`EnableTourRequest`** (hoop-daemon/src/api_tour_project.rs:34) - Missing `ToSchema` and `PartialSchema` (4 occurrences)
3. **`ListJobsQuery`** (hoop-daemon/src/api_transcription.rs:19) - Missing `ToSchema` and `PartialSchema`
4. **`CreateScreenCaptureRequest`** (hoop-daemon/src/api_screen_capture.rs:34) - Missing `ToSchema` and `PartialSchema`
5. **`StartStreamingUploadRequest`** (hoop-daemon/src/api_screen_capture.rs:352) - Missing `ToSchema` and `PartialSchema`
6. **`CompleteStreamingUploadRequest`** (hoop-daemon/src/api_screen_capture.rs:469) - Missing `ToSchema` and `PartialSchema`

These structs are used in OpenAPI path definitions but lack the required `#[derive(ToSchema)]` attribute.

## Warnings (74 total)

Warnings include:
- Unused imports (PathBuf, warn, State, Connection, params, Deserialize, get, Arc, etc.)
- Unused variables (start, remote_addr, required_role, elapsed_ms, config, etc.)
- Unused mut variables (conn, gemini_dirs, opencode_dirs, shared_files, etc.)

## Full Output

See `/tmp/hoop_debug_build.log` for complete compilation output.
