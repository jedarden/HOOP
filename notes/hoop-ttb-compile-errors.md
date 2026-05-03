# HOOP Compilation Errors Summary

Date: 2026-05-03

## Current State

The HOOP codebase has **131 compilation errors** that must be fixed before the project can proceed.

## Major Error Categories

### 1. Missing ToSchema Implementations (~80 errors)
OpenAPI documentation trait not implemented for:
- `SecretPattern` (config_resolver.rs)
- `OnboardingPrompt` (api_onboarding.rs)
- `ProposalsResponse`, `ReflectionsResponse`, `ApproveProposalRequest`, `RejectProposalRequest` (api_reflection_ledger.rs)
- `StitchLinkInfo`, `ClosureNodeInfo` (api_stitch_traversal.rs)
- `EnableTourRequest`, `TourProjectResponse` (api_tour_project.rs)
- `HighlightResult` (api_files.rs)
- `Bytes` (api_screen_capture.rs)

### 2. WsEvent Missing Fields (16 errors)
`WsEvent` struct initializers missing:
- `cost_anomaly_alert`
- `saturation_alert`

### 3. Type Mismatches (~20 errors)
- `api_ui_state.rs`: `while let Some(row)` should be `while let Ok(Some(row))`
- `api_tour_project.rs`: unwrap_or type issues, FromSql trait issues
- `lib.rs`: Several type mismatches (audit_retention_days, shutdown_coordinator, etc.)

### 4. Missing Fields (5 errors)
- `DaemonState` missing `reflection_tx`
- `HoopConfig` missing `embedding` field

### 5. Other Issues (~10 errors)
- `UnassignedTracker` missing Debug trait
- `ActionKind::SaturationAlert` not covered in match
- Various API handler signature issues

## Next Steps

These errors need to be fixed in order of dependency:
1. Add missing struct fields first
2. Fix ToSchema implementations
3. Fix type mismatches
4. Add missing trait implementations

