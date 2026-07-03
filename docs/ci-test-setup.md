# HOOP CI Test Infrastructure Setup

## Overview

This document describes the HOOP CI test infrastructure configured in Argo Workflows on the `iad-ci` cluster.

## Workflow Template

The `hoop-test` workflow template is defined in:
```
jedarden/declarative-config → k8s/iad-ci/argo-workflows/hoop-test-workflowtemplate.yml
```

### Workflow Structure

The test workflow runs three test stages in parallel:

1. **Unit Tests** (`unit-tests`)
   - Runs `cargo test --workspace`
   - Executes formatting checks (`cargo fmt --check`)
   - Executes clippy lints (`cargo clippy`)
   - Runs compile-fail tests for create-only invariants

2. **Integration Tests** (`integration-tests`)
   - Runs coverage tests with `cargo llvm-cov`
   - Executes beads removal recovery test
   - Runs OpenAPI spec validation

3. **E2E Tests** (`e2e-tests`)
   - Starts hoop-daemon in background
   - Runs Playwright smoke tests
   - Runs mobile responsiveness tests
   - Runs visual regression tests

### Resource Limits

The workflow uses the following resource allocations:

- **Unit Tests**: 4-8 CPU cores, 8-16GB memory
- **Integration Tests**: 2-4 CPU cores, 4-8GB memory
- **E2E Tests**: 2-4 CPU cores, 4-8GB memory

These limits ensure the full test suite can run without resource constraints.

## Manual Workflow Submission

To manually trigger the test workflow:

```bash
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig create -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  generateName: hoop-test-
  namespace: argo-workflows
spec:
  workflowTemplateRef:
    name: hoop-test
  arguments:
    parameters:
      - name: repo
        value: "jedarden/HOOP"
      - name: branch
        value: "main"
EOF
```

### Parameters

- `repo`: GitHub repository in "owner/repo" format (default: "jedarden/HOOP")
- `branch`: Git branch to test (default: "main")
- `commit`: Optional specific commit SHA to test

## GitHub Webhook Setup

To trigger the workflow automatically on push to the HOOP repository:

### 1. Configure GitHub Webhook

Navigate to: `https://github.com/jedarden/HOOP/settings/hooks`

Create a new webhook with:
- **Payload URL**: `https://<argo-workflows-url>/api/v1/events/`
- **Content type**: `application/json`
- **Secret**: Use the value from `github-webhook-secret` in `iad-ci` cluster
- **Events**: Select "Push" events

### 2. Obtain Argo Workflows URL

The Argo Workflows server is accessible via Tailscale. Check the service:

```bash
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig get svc -n argo-workflows
```

### 3. Verify Webhook Secret

Get the webhook secret:

```bash
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig \
  get secret github-webhook-secret -n argo-workflows \
  -o jsonpath='{.data.token}' | base64 -d
```

### 4. Test Webhook

After configuration, test the webhook by:
1. Pushing a commit to the HOOP repository
2. Checking workflow submissions:
```bash
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig \
  get workflows -n argo-workflows --sort-by=.metadata.creationTimestamp | grep hoop-test
```

## Monitoring Test Results

### Argo UI

Access the Argo Workflows UI at `https://argo-ci.ardenone.com` (VPN required).

### CLI Monitoring

```bash
# List recent test runs
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig \
  get workflows -n argo-workflows --sort-by=.metadata.creationTimestamp | grep hoop-test

# Get workflow status
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig \
  get workflow <workflow-name> -n argo-workflows -o jsonpath='{.status.phase}'

# Get workflow logs
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig \
  logs -n argo-workflows <pod-name> -c main
```

## Test Result Storage

Test results are captured in:
- **Argo Workflow logs**: Available in the Argo UI and via kubectl logs
- **Coverage reports**: Generated as `lcov.info` in the integration-tests step
- **Playwright reports**: Generated in the e2e-tests step

### Artifact Storage (Future)

To enable persistent artifact storage:
1. Configure an S3-compatible storage backend
2. Update workflow template to archive test results
3. Configure artifact retention policies

## Dependencies

- **Cluster**: `iad-ci` (Rackspace Spot, us-east-iad-1)
- **Namespace**: `argo-workflows`
- **Service Account**: `argo-workflow`
- **Secrets**: `github-webhook-secret`

## Troubleshooting

### Workflow Not Triggering

1. Verify GitHub webhook is configured correctly
2. Check webhook delivery logs in GitHub repository settings
3. Ensure Argo Workflows server is accessible from GitHub

### Tests Failing

1. Check workflow logs in Argo UI
2. Verify resource limits are adequate (increase if OOMKilled)
3. Check for leaked test processes: `ps aux | grep 'HOOP/target'`

### Resource Exhaustion

If the cluster runs out of resources:
1. Check available resources:
```bash
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig top nodes
```
2. Clean up completed workflows:
```bash
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig \
  delete workflows -n argo-workflows --field-selector=status.phase=Succeeded
```

## Related Documentation

- [HOOP CLAUDE.md](/home/coding/HOOP/CLAUDE.md) - Project-specific instructions
- [AGENTS.md](/home/coding/HOOP/AGENTS.md) - Repository agent guide
- [declarative-config](https://github.com/jedarden/declarative-config) - Kubernetes configuration

## Version History

- 2026-07-03: Initial setup of HOOP test workflow infrastructure
