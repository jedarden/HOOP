# Compilation Errors Blocking Unit Tests

**Bead ID:** bf-aokf2  
**Date:** 2026-07-03  
**Total Compilation Errors:** 82

## Summary

The HOOP workspace unit tests cannot run due to **82 compilation errors** in `hoop-daemon`. These are not runtime test failures but compilation errors that prevent the test binaries from being built.

## Error Types

| Error Code | Description | Count |
|------------|-------------|-------|
| E0061 | Function argument count mismatch | 8 |
| E0063 | Missing struct fields | 61 |
| E0308 | Type mismatch | 7 |
| E0369 | Binary operation on invalid type | 1 |
| E0599 | Missing associated function/constant | 3 |
| E0277 | Unpinned async block | 2 |

## Detailed Errors by File

### 1. `api_beads.rs` (1 error)
- **Line 1097:** `resolve_actor()` takes 2 arguments but 1 supplied

### 2. `api_preview.rs` (1 error)
- **Line 621:** Missing field `attachments_count` in `PreviewRequest` initializer

### 3. `api_stitch_decompose.rs` (10+ errors)

#### Constructor argument mismatches:
- **Line 1214:** `ProjectSupervisor::new()` takes 9 arguments but 0 supplied
- **Line 1220:** `CostAggregator::new()` takes 1 argument but 0 supplied
- **Line 1222:** `UploadRegistry::new()` takes 1 argument but 0 supplied  
- **Line 1232:** `WorkerAckMonitor::new()` takes 1 argument but 0 supplied

#### Missing associated functions:
- **Line 1230:** `ResolvedConfig::default()` not found
- **Line 1237:** `RedactionPolicyState::default()` not found

#### Missing struct fields:
- **Line 1203:** `DaemonState` missing fields `br_semaphore` and `br_semaphore_target_permits`

#### Type mismatches:
- **Line 1205:** Type mismatch at `started_at` (Instant vs expected type)
- **Line 1220:** Type mismatch at `cost_aggregator`
- **Line 1222:** Type mismatch at `upload_registry`

### 4. `capacity.rs` (60+ errors)

All in `CapacityMeterConfig` initializers across multiple test functions:

**Missing fields variations:**
- `accounts_file, gcp_quota_config, gemini_dirs, opencode_dirs` (4 fields)
- `accounts_file, gcp_quota_config, opencode_dirs` (3 fields)
- `accounts_file, opencode_dirs` (2 fields)

**Locations:**
- Lines: 2457, 2503, 2573, 2774, 2851, 2913, 3058, 3111, and ~50 more test cases

### 5. `syntax_highlight_stream.rs` (2 errors)
- **Line 163:** Async block cannot be unpinned
- **Line 174:** Async block cannot be unpinned

### 6. `risk_patterns.rs` (1 error)
- Type error: Binary operation `==` cannot be applied to `RiskSeverity`

### 7. Other affected files (remaining errors):
- Missing fields in various struct initializers
- Missing `default()` implementations
- Function signature changes

## Root Cause Analysis

These errors appear to be from:

1. **API changes without test updates** - Function signatures changed but test code not updated
2. **New struct fields added** - Structs gained new required fields but test initializers not updated
3. **Removed default implementations** - Some types lost `Default` implementations

## Passing Tests

Before the compilation failures, these test suites passed:
- **hoop (lib):** 32 tests passed
- **hoop-cli:** 56 tests passed

## Next Steps

The compilation errors must be fixed before unit tests can run. This involves:
1. Updating function calls to match new signatures
2. Adding missing struct fields to all initializers
3. Implementing missing `Default` traits or providing explicit values
4. Fixing type mismatches

## Checklist

- [x] Identify all compilation errors
- [x] Categorize by error type
- [x] Document affected files
- [ ] Fix compilation errors (requires separate bead)
- [ ] Re-run tests to find runtime failures
