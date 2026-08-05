# Argo UI Artifact Accessibility Verification (bf-26mz1)

## Investigation Date
2026-07-05

## Goal
Verify that test results and artifacts are properly viewable in the Argo UI, including both success and failure scenarios.

## Methodology
1. Attempted to submit test workflow runs
2. Checked workflow status and pod scheduling
3. Examined existing workflow runs
4. Investigated artifact repository configuration

## Findings

### 1. Cluster Resource Constraints
The iad-ci cluster has severe resource constraints that prevent workflows from scheduling:

**Evidence:**
- Submitted `hoop-test-manual-kjmzp` workflow - pods stuck in Pending state
- Pod status shows: `0/6 nodes are available: 3 Insufficient cpu, 3 Insufficient memory`
- The workflow requires significant resources (4-8 CPU cores, 8-16GB memory per step)
- Workflow `hoop-ci-test-manual-b2lqf` has been stuck for ~25 hours with pods still Pending

**Impact:** Workflows cannot run, so test results cannot be generated or viewed.

### 2. Artifact Repository Not Configured
The Argo Workflows installation lacks an artifact repository, which is required for artifact storage and retrieval.

**Evidence:**
- Test workflow `artifact-test-r28d6` failed with error: `artifact storage is not configured; see the docs for setup instructions: https://argo-workflows.readthedocs.io/en/latest/configure-artifact-repository/`
- No `workflow-controller-configmap` found in argo-workflows namespace
- No artifact repository configuration found in cluster ConfigMaps

**Impact:** Even if workflows run successfully, artifacts cannot be stored or accessed in the Argo UI.

### 3. Stale/Failed Workflows
Multiple workflows have been stuck in failed states for extended periods:

**Examples:**
- Multiple `sigil-ci` workflows stuck in Running state for 1-2 days
- `needle-ci-54jjl` stuck in Running for 32 hours
- `hoop-ci-test-manual-b2lqf` stuck for 25 hours with multiple failed steps:
  - e2e-tests: Failed
  - schema-drift: Failed
  - beads-removal-recovery: Failed
  - openapi-spec-check: Failed
  - coverage: Failed

**Impact:** The workflow queue appears to be clogged with stuck workflows, preventing new workflows from scheduling.

## What Should Be Available (Based on Workflow Templates)

The `hoop-test` workflow template should produce the following artifacts when working properly:

### Unit Test Artifacts
- Cargo test output (stdout/stderr logs)
- Compile-fail test results
- Clippy warnings
- Format check results

### Integration Test Artifacts
- Coverage report (`lcov.info`)
- Coverage percentage output
- Beads removal recovery test results
- OpenAPI spec check results

### E2E Test Artifacts
- Playwright test results (desktop smoke tests)
- Mobile responsiveness test results
- Visual regression test results
- Screenshots (if tests capture them)

## Acceptance Criteria Status

| Criteria | Status | Notes |
|----------|--------|-------|
| Test artifacts visible in Argo UI artifacts panel | ❌ FAIL | Artifact repository not configured |
| Test output readable from UI | ❌ FAIL | Workflows cannot run due to resource constraints |
| Failed run artifacts retained and accessible | ❌ FAIL | Cannot test - no successful/failed workflow completions |
| Successful run artifacts retained and accessible | ❌ FAIL | Cannot test - no workflows completing |

## Recommendations

### Immediate Actions Required

1. **Configure Artifact Repository**
   - Set up S3/MinIO/GCS artifact repository for Argo Workflows
   - Update workflow controller config with artifact repository settings
   - Test artifact storage with simple workflow

2. **Clear Stale Workflows**
   - Delete stuck workflows that have been pending for >24 hours
   - Implement workflow TTL limits to prevent future accumulation
   - Check workflow controller logs for root cause of stuck workflows

3. **Address Resource Constraints**
   - Scale up cluster node pool or add spot instances
   - Reduce workflow resource requests if possible
   - Implement workflow priority classes to manage resource allocation

### Testing Plan After Fixes

Once the above issues are resolved:

1. **Submit Simple Test Workflow**
   ```bash
   kubectl create -f - <<EOF
   apiVersion: argoproj.io/v1alpha1
   kind: Workflow
   metadata:
     generateName: artifact-verification-
     namespace: argo-workflows
   spec:
     entrypoint: artifact-example
     serviceAccountName: argo-workflow
     templates:
     - container:
         image: debian:bookworm
         command: [bash, -c]
         args: ["echo 'Test output' > /tmp/test.txt && echo 'Done'"]
       name: artifact-example
       outputs:
         artifacts:
         - name: test-artifact
           path: /tmp/test.txt
   EOF
   ```

2. **Access Argo UI** at https://argo-ci.ardenone.com
   - Verify workflow appears in list
   - Click through to workflow details
   - Check that artifacts panel shows `test-artifact`
   - Download and verify artifact content

3. **Test with HOOP Workflow**
   - Submit `hoop-test` workflow with commit parameter
   - Verify logs are accessible for each step
   - Verify artifacts (coverage reports, test results) appear
   - Test both success and failure scenarios

## Conclusion

The Argo UI artifact accessibility **cannot be verified** until two critical infrastructure issues are resolved:

1. **Artifact repository must be configured** - Without this, no artifacts can be stored or retrieved
2. **Cluster resource constraints must be addressed** - Without available resources, workflows cannot run to completion

These are infrastructure/environment issues that must be resolved before the acceptance criteria can be met. The workflow templates are properly configured to produce artifacts, but the cluster infrastructure prevents their execution and storage.

## Related Beads
- Depends on: bf-1kyrc (CI infrastructure setup)
