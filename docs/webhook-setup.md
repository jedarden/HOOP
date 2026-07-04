# GitHub Webhook Setup for HOOP

## Current Infrastructure Status

### 1. Argo Events Configuration (DEPLOYED)

The GitHub webhook infrastructure for HOOP is already deployed in the iad-ci cluster:

**Event Source**: `/home/coding/declarative-config/k8s/iad-ci/argo-events/github-eventsource.yml`
- Repository: `jedarden/HOOP`
- Events: `push`
- Endpoint: `/hoop`
- URL: `https://webhooks-ci.ardenone.com`
- Port: `12000`

**Sensor**: `/home/coding/declarative-config/k8s/iad-ci/argo-events/hoop-ci-sensor.yml`
- Triggers on: Push to `refs/heads/main`
- Submits: Workflow `hoop-ci` in `argo-workflows` namespace

### 2. Workflow Template (DEPLOYED)

**hoop-ci** workflow template includes:
- Build and release
- Coverage tests (80% threshold)
- OpenAPI spec check
- Schema drift tests
- Beads removal recovery tests
- E2E tests (Playwright)
- Security audit (cargo audit, pnpm audit, trivy)
- Load test (conditional via `run_load_test` parameter)
- Docker push
- Image validation and security scanning

### 3. Manual Trigger Capability

Workflows can be triggered manually via:

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

## Verification Status

✅ **Webhook configuration in Argo Events**: Deployed and active
✅ **Sensor configuration**: Deployed and active  
✅ **Workflow template**: Comprehensive CI/CD pipeline deployed
✅ **Manual trigger**: Available and tested (recent run: hoop-ci-test-manual-b2lqf)
⚠️ **GitHub repository webhook**: Needs verification in GitHub UI

## Next Steps for Full Automation

1. Verify webhook is configured in GitHub repository settings:
   - Navigate to: https://github.com/jedarden/HOOP/settings/hooks
   - Look for webhook pointing to: `https://webhooks-ci.ardenone.com/hook`
   - Verify "Push" events are enabled

2. If webhook is missing in GitHub, add it with:
   - URL: `https://webhooks-ci.ardenone.com/hook`
   - Content type: `application/json`
   - Secret: (use the webhook secret from github-webhook-secret)
   - Events: Pushes

3. Test automatic trigger by pushing to main branch and verifying workflow execution

## Load Test Specific Configuration

The `hoop-ci` workflow includes conditional load test execution:

- **Parameter**: `run_load_test` (default: "true")
- **Scale**: 5 projects × 2 workers × 50 beads (CI-optimized)
- **Can be disabled**: Set `run_load_test="false"` for faster PR validation
- **Separate load test workflows**: Available in `.argo/workflowtemplates/hoop-load-test.yaml`
