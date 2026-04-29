# Security Notes

This document tracks accepted security findings and baseline vulnerabilities for HOOP.

## Baseline (2026-04-29)

Initial security scan captured as baseline. New vulnerabilities detected in future scans should be evaluated against this baseline.

### Rust Dependencies (cargo audit)

**No vulnerabilities found.**

**Unmaintained packages (informational):**
| ID | Package | Severity | Reason | Action |
|----|---------|----------|--------|--------|
| RUSTSEC-2025-0141 | bincode 1.3.3 | Unmaintained | Used by syntect (syntax highlighting) | Monitor; alternatives: wincode, postcard, bitcode |
| RUSTSEC-2024-0436 | paste 1.0.15 | Unmaintained | Transitive via tokenizers→fastembed | Monitor; not in production path |

### npm Dependencies (pnpm audit)

**2 moderate vulnerabilities (dev dependencies only):**
| Advisory | Package | Severity | Path | Recommendation |
|----------|---------|----------|------|----------------|
| GHSA-67mh-4w82-2f99 | esbuild <=0.24.2 | Moderate | vitest→vite→esbuild | Upgrade to >=0.25.0 when available |
| GHSA-4w7w-66w2-5vf9 | vite <=6.4.1 | Moderate | vitest→vite | Upgrade to >=6.4.2 when available |

**Note:** These affect only the development build process (vitest). Production builds use esbuild bundled with vite, which runs in CI only. No runtime exposure.

### Image Vulnerabilities (trivy image)

Baseline pending - requires image to be built and pushed.

### Filesystem Scan (trivy fs)

Baseline pending - requires trivy installation or CI run.

## Acceptance Criteria

New vulnerabilities are evaluated based on:
1. **Severity:** HIGH/CRITICAL requires immediate action; MODERATE evaluated case-by-case
2. **Exposure:** Dev-only vs production runtime
3. **Exploitability:** Public exploit known vs theoretical
4. **Remediation cost:** Simple update vs major refactoring

## Remediation Log

| Date | Action | Result |
|------|--------|--------|
| 2026-04-29 | Initial baseline captured | 2 unmaintained warnings, 2 moderate dev vulns |
