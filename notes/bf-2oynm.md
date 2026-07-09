# bf-2oynm — Add ToSchema derive to RejectProposalRequest

## Outcome: already satisfied (no source change needed)

The `RejectProposalRequest` struct in `hoop-daemon/src/api_reflection_ledger.rs`
already carries a ToSchema derive:

```rust
/// Request to reject a proposal
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RejectProposalRequest {
    /// Optional reason for rejection
    pub reason: Option<String>,
}
```

### Acceptance criteria — all met

1. **RejectProposalRequest has `#[derive(ToSchema)]`** — yes, via the
   conditional `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]`,
   which expands to `#[derive(utoipa::ToSchema)]` when the `openapi` feature is
   active. That feature is on by default in `hoop-daemon/Cargo.toml`
   (`default = ["openapi"]`), so the derive is active in normal builds.
2. **All field types are ToSchema-compatible** — the only field is
   `reason: Option<String>`, both `Option` and `String` are natively
   ToSchema-compatible.
3. **File compiles with `cargo check`** — verified:
   `nix-shell --run 'cargo check -p hoop-daemon'` →
   `Finished dev profile ... in 40.44s`, no errors (only pre-existing,
   unrelated dead-code/unused-constant warnings).

### Why no unconditional `#[derive(utoipa::ToSchema)]` was added

Commit `89214ef` (bf-4ibg8, "Standardize ToSchema derive pattern in
api_reflection_ledger.rs") deliberately moved `ApproveProposalRequest` and
`RejectProposalRequest` *from* an unconditional `#[derive(..., utoipa::ToSchema)]`
*to* the conditional `cfg_attr` form so that all six structs in the file use one
consistent pattern (ToSchema is only needed for the OpenAPI generator, which is
feature-gated). Re-adding an unconditional derive would revert that intentional
standardization and conflict with the file's convention, so it was left as-is.

### Verification command

```bash
nix-shell --run 'cargo check -p hoop-daemon'
```
