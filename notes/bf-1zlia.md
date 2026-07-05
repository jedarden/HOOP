# Test Workflow Submission to Argo - bf-1zlia

## Summary
Successfully submitted a test workflow to Argo Workflows in the `iad-ci` cluster.

## Workflow Details
- **Workflow Name:** `hoop-test-manual-6gqxb`
- **Workflow UID (Run ID):** `31035a9c-1a93-480c-993c-af26c1c8c33b`
- **Template:** `hoop-test`
- **Namespace:** `argo-workflows`
- **Started At:** 2026-07-05T01:57:21Z
- **Final Phase:** Running (stuck pending due to resource constraints)

## Parameters
- **repo:** jedarden/HOOP
- **branch:** main
- **commit:** "" (empty = use branch HEAD)

## Submission Process
1. Listed available workflow templates using `kubectl get workflowtemplates`
2. Found the `hoop-test` template designed for HOOP comprehensive testing
3. Submitted workflow via `kubectl create -f -` with proper YAML specification
4. Initial attempts failed due to missing `commit` parameter
5. Final successful submission included all three required parameters

## Workflow Components
The `hoop-test` template runs three test suites:
1. **Unit Tests** - cargo test, clippy, fmt check (1 hour timeout, 8 CPU / 16Gi RAM)
2. **Integration Tests** - coverage, beads removal recovery, OpenAPI spec (40 min timeout, 4 CPU / 8Gi RAM)
3. **E2E Tests** - Playwright smoke/responsiveness/visual regression (1 hour timeout, 4 CPU / 8Gi RAM)

## Current Status
The workflow was submitted successfully but is stuck in the pending phase due to cluster resource constraints:
- Pod: `hoop-test-manual-6gqxb-unit-tests-1123717064`
- Status: Pending
- Scheduling Error: "0/6 nodes are available: 3 Insufficient cpu, 3 Insufficient memory, 3 node(s) didn't match Pod's node affinity/selector"

## Acceptance Criteria
- ✅ Workflow submitted successfully
- ⚠️ Workflow stuck pending due to cluster resource constraints (not a submission failure)
- ✅ Workflow name and ID captured: `hoop-test-manual-6gqxb` / `31035a9c-1a93-480c-993c-af26c1c8c33b`

## Notes
The `hoop-test` workflow requires significant resources (up to 8 CPU / 16Gi RAM for unit tests). The `iad-ci` cluster currently does not have sufficient available capacity to run this workflow. For future test runs, consider:
1. Using less resource-intensive templates (e.g., `rust-verify` for simple validation)
2. Scheduling during off-peak hours when cluster capacity is available
3. Checking cluster resource availability before submitting: `kubectl top nodes`
