# GitHub Webhook Setup for HOOP

## Overview

This document describes the complete webhook setup for automatic CI/CD execution on push to the HOOP repository.

## Current Infrastructure Status

### 1. Argo Events Configuration (✅ DEPLOYED)

The GitHub webhook infrastructure for HOOP is already deployed in the iad-ci cluster:

**Event Source**: `jedarden/declarative-config/k8s/iad-ci/argo-events/github-eventsource.yml`
- Repository: `jedarden/HOOP`
- Events: `push`
- Endpoint: `/hoop`
- URL: `https://webhooks-ci.ardenone.com`
- Port: `12000`
- Status: Active and listening for webhook events

**Sensor**: `jedarden/declarative-config/k8s/iad-ci/argo-events/hoop-ci-sensor.yml`
- Triggers on: Push to `refs/heads/main`
- Submits: Workflow `hoop-ci` in `argo-workflows` namespace
- Filters: Only triggers on main branch pushes (not PRs or other branches)

### 2. Workflow Template (✅ DEPLOYED)

**hoop-ci** workflow template includes comprehensive CI/CD pipeline:
- Build and release (with GitHub release creation)
- Coverage tests (80% threshold)
- OpenAPI spec check
- Schema drift tests
- Beads removal recovery tests
- E2E tests (Playwright)
- Security audit (cargo audit, pnpm audit, trivy)
- Load test (conditional via `run_load_test` parameter)
- Docker push to ronaldraygun/hoop:latest
- Image validation and security scanning

### 3. Manual Trigger Capability (✅ AVAILABLE)

Workflows can be triggered manually for testing:

```bash
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig create -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  generateName: hoop-ci-manual-
  namespace: argo-workflows
spec:
  workflowTemplateRef:
    name: hoop-ci
  arguments:
    parameters:
      - name: run_load_test
        value: "true"  # or "false" to skip load test
EOF
```

## Setup Checklist

### Cluster Infrastructure (✅ COMPLETE)
- [x] EventSource configured for HOOP repository
- [x] Sensor configured with main branch filter
- [x] WorkflowTemplate deployed with full CI/CD pipeline
- [x] Manual trigger available and tested

### GitHub Repository Configuration (⚠️ PENDING)
- [ ] Webhook configured in GitHub repository settings
- [ ] Webhook URL: `https://webhooks-ci.ardenone.com/hook`
- [ ] Content type: `application/json`
- [ ] Secret: configured from `github-webhook-secret`
- [ ] Events: "Pushes" enabled
- [ ] Active: webhook enabled and receiving events

## GitHub Webhook Configuration Steps

### Step 1: Access GitHub Repository Settings

1. Navigate to: https://github.com/jedarden/HOOP/settings/hooks
2. Click "Add webhook" if no webhook exists, or edit existing webhook

### Step 2: Configure Webhook Settings

**Payload URL**: `https://webhooks-ci.ardenone.com/hook`

**Content type**: `application/json`

**Secret**: The webhook secret from the cluster secret `github-webhook-secret`
- Retrieve with: `kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig get secret github-webhook-secret -n argo-events -o jsonpath='{.data.webhook-secret}' | base64 -d`

**Which events would you like to trigger this webhook?**
- Select: "Just the push event"
- This triggers on all branch pushes (sensor filters for main only)

**Active**: ✅ Enable the webhook (checkbox should be checked)

### Step 3: Save and Verify

1. Click "Add webhook" or "Update webhook"
2. GitHub will send a test ping event
3. Scroll down to "Recent Deliveries" to verify the ping was successful (should show 200 OK)

### Step 4: Test Automatic Trigger

1. Push a commit to the main branch:
   ```bash
   cd /home/coding/HOOP
   git commit --allow-empty -m "Test webhook trigger"
   git push origin main
   ```

2. Verify workflow execution in Argo:
   ```bash
   kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig get workflows -n argo-workflows --sort-by=.metadata.creationTimestamp | grep hoop-ci | tail -5
   ```

3. Check workflow logs:
   ```bash
   # Get the latest workflow name
   WORKFLOW=$(kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig get workflows -n argo-workflows --sort-by=.metadata.creationTimestamp -o jsonpath='{.items[-1].metadata.name}' | grep hoop-ci)
   
   # Follow workflow progress
   kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig get workflow $WORKFLOW -n argo-workflows -w
   ```

## Verification Commands

### Check EventSource Status
```bash
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig get eventsource github-webhooks -n argo-events -o yaml
```

### Check Sensor Status
```bash
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig get sensor hoop-ci-sensor -n argo-events -o yaml
```

### Check Recent Workflow Runs
```bash
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig get workflows -n argo-workflows --sort-by=.metadata.creationTimestamp | grep hoop-ci | tail -10
```

### Check Webhook Secret (for GitHub configuration)
```bash
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig get secret github-webhook-secret -n argo-events -o jsonpath='{.data.webhook-secret}' | base64 -d
```

## Troubleshooting

### Webhook not triggering workflow

1. **Check EventSource logs**:
   ```bash
   kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig logs -n argo-events -l eventsource-name=github-webhooks --tail=50
   ```

2. **Check Sensor logs**:
   ```bash
   kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig logs -n argo-events -l sensor-name=hoop-ci-sensor --tail=50
   ```

3. **Check GitHub webhook deliveries**:
   - Go to: https://github.com/jedarden/HOOP/settings/hooks
   - Click on the webhook
   - Review "Recent Deliveries" for any failed deliveries

4. **Verify webhook secret matches**:
   - The secret in GitHub must match the cluster secret
   - Mismatched secrets cause webhook validation to fail

### Workflow triggers but fails

1. **Check workflow status**:
   ```bash
   kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig get workflow <workflow-name> -n argo-workflows -o yaml
   ```

2. **Check workflow logs**:
   ```bash
   kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig logs -n argo-workflows <pod-name> -c main
   ```

3. **Check Argo UI**: https://argo-ci.ardenone.com (VPN required)

### Sensor filters not working

The sensor filters for `refs/heads/main` only. Pushes to other branches will not trigger the workflow. To test webhook connectivity without triggering on main, temporarily modify the sensor filter.

## Load Test Configuration

The `hoop-ci` workflow includes conditional load test execution:

- **Parameter**: `run_load_test` (default: "true")
- **Scale**: 5 projects × 2 workers × 50 beads (CI-optimized)
- **Can be disabled**: Set `run_load_test="false"` for faster PR validation
- **Separate load test workflows**: Available for manual execution at different scales

## Security Considerations

- The webhook secret is stored as a Kubernetes Secret in the `argo-events` namespace
- The EventSource and Sensor use the secret to validate webhook signatures
- GitHub webhooks use HMAC-SHA1 signature validation
- Ensure the secret is not exposed in logs or documentation

## Related Documentation

- **Argo Events**: https://argoproj.github.io/argo-events/
- **Argo Workflows**: https://argoproj.github.io/argo-workflows/
- **GitHub Webhooks**: https://docs.github.com/en/developers/webhooks-and-events/webhooks/about-webhooks
