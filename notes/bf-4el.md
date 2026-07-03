# bf-4el: systemd installation - BLOCKED by compilation errors

## Status: BLOCKED

The systemd installation task cannot proceed because `hoop` does not currently compile.

## Blocker

Bug bead **bf-8gad6** - "Fix ToSchema trait errors - structs missing derive in api_screen_capture"

## Build Failure Details

Running:
```bash
nix-shell --run 'cargo build --release'
```

Results in multiple compilation errors of the form:
```
error[E0277]: the trait bound `CreateScreenCaptureRequest: ToSchema` is not satisfied
```

### Affected structs
- `CreateScreenCaptureRequest` (api_screen_capture.rs:34)
- `StartStreamingUploadRequest` (api_screen_capture.rs:352)
- `CompleteStreamingUploadRequest` (api_screen_capture.rs:469)

## Root cause

These structs are used in utoipa `request_body` parameters but lack the `#[derive(ToSchema)]` attribute.

## Next steps

Once bf-8gad6 is resolved and `cargo build --release` succeeds, the systemd installation can proceed with:
1. `hoop install-systemd`
2. Verify `~/.config/systemd/user/hoop.service`
3. `systemctl --user enable --now hoop`
4. Verify service status and health endpoint
5. Configure `~/.hoop/config.yml`
