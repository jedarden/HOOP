# Secrets Scanner Unification (§18)

## Overview

The client-side and backend secrets scanners are unified to use a single pattern set sourced from `config.yml`. This ensures parity between client pre-upload warnings and backend authoritative scanning/redaction.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          SINGLE SOURCE OF TRUTH                              │
│                                                                              │
│  config.yml (secrets_patterns) → Fallback: default_secret_patterns()        │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        BACKEND (hoop-daemon)                                │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │ config_resolver::resolve()                                           │  │
│  │   - Reads config.yml                                                  │  │
│  │   - Falls back to default_secret_patterns()                           │  │
│  │   - Returns ResolvedConfig with secrets_patterns                      │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                    │                                        │
│                                    ▼                                        │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │ lib.rs (daemon startup)                                               │  │
│  │   - Initializes patterns: update_patterns_with_names()               │  │
│  │   - Updates on config change                                          │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                    │                                        │
│                    ┌───────────────┴───────────────┐                       │
│                    ▼                               ▼                       │
│  ┌─────────────────────────────┐   ┌───────────────────────────────────┐  │
│  │ redaction::scan_text_*()    │   │ api_config::get_secrets_patterns()│  │
│  │   - Authoritative scanning  │   │   - Serves patterns to client      │  │
│  │   - Redaction with [REDACTED]│   │   - GET /api/config/secrets-patterns│  │
│  └─────────────────────────────┘   └───────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        CLIENT (hoop-ui)                                     │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │ secretsScanner.ts                                                    │  │
│  │   - prefetchSecretPatterns(): Fetches from /api/config/secrets-patterns│  │
│  │   - scanForSecretsSync(): Synchronous scanning for React             │  │
│  │   - scanForSecrets(): Async scanning with pattern fetch              │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                    │                                        │
│                                    ▼                                        │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │ StitchesTab.tsx (pre-upload warning)                                 │  │
│  │   - Displays warning banner when secrets detected                    │  │
│  │   - Shows severity, type, and truncated value                        │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Pattern Definitions

### Default Patterns (`config_resolver.rs::default_secret_patterns()`)

| ID | Name | Severity | Patterns |
|----|------|----------|----------|
| `anthropic_api_key` | Anthropic API Key | high | `sk-ant-[a-zA-Z0-9_-]{20,}` |
| `generic_sk_key` | Generic API Key | high | `\bsk-[a-zA-Z0-9]{20,}\b` |
| `aws_access_key` | AWS Access Key | high | `\bAKIA[A-Z0-9]{16}\b` |
| `github_token` | GitHub Token | high | `ghp_`, `ghs_`, `ghu_`, `github_pat_` patterns |
| `slack_token` | Slack Token | high | `xoxb-`, `xoxp-` patterns |
| `jwt` | JWT | high | `\bey[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b` |
| `bearer_token` | Bearer Token | high | `(?i)bearer\s+[A-Za-z0-9._\-+/]{20,}` |
| `env_var_secret` | Environment Variable Secret | high | Complex pattern for env assignments |
| `json_secret_field` | JSON Secret Field | high | Pattern for JSON secret fields |

### Custom Patterns via config.yml

```yaml
secrets_patterns:
  - id: "custom_api_key"
    name: "Custom API Key"
    severity: "high"
    patterns:
      - "custom-key-[a-zA-Z0-9]{20,}"
```

## Parity Tests

### Client Tests (`hoop-ui/web/src/secretsScanner.test.ts`)

Tests client-side detection using mocked backend patterns:

- `detects Anthropic API key`
- `detects generic API key`
- `detects AWS access key`
- `detects GitHub token`
- `detects Slack token`
- `detects JWT`
- `detects environment variable secret`
- `detects JSON secret field`
- `detects all secrets in multi-secret text`
- `removes overlapping matches keeping the most specific`
- `classifies severity correctly`
- `truncates secrets for display`

### Backend Tests (`hoop-daemon/src/redaction.rs`)

Parity tests using the same fixtures as client tests:

- `test_parity_anthropic_key`
- `test_parity_generic_sk_key`
- `test_parity_aws_access_key`
- `test_parity_github_token`
- `test_parity_slack_token`
- `test_parity_jwt`
- `test_parity_env_var_secret`
- `test_parity_json_secret`
- `test_parity_multiple_secrets`
- `test_parity_clean_text`
- `test_parity_match_positions`
- `test_parity_redaction_matches_scanning`

## Running Tests

### Client Tests
```bash
cd hoop-ui/web
npm test -- secretsScanner.test.ts
```

### Backend Tests
```bash
cargo test -p hoop-daemon redaction::tests::test_parity
```

## Verification Checklist

- [x] Client fetches patterns from `/api/config/secrets-patterns`
- [x] Backend serves patterns from `config.yml` (or defaults)
- [x] Client uses patterns for pre-upload warning (`StitchesTab.tsx`)
- [x] Backend uses patterns for authoritative scanning (`redaction.rs`)
- [x] Backend parity tests use same fixtures as client tests
- [x] Patterns are validated at compile time
- [x] Custom patterns can be added via `config.yml`

## Acceptance Criteria

- ✅ Config.yml exposes pattern set; backend + client read same file or shared schema
- ✅ Client warns pre-upload; backend blocks/redacts authoritatively
- ✅ Tests confirm parity on fixture secrets

## References

- Client scanner: `hoop-ui/web/src/components/secretsScanner.ts`
- Client tests: `hoop-ui/web/src/secretsScanner.test.ts`
- Backend scanner: `hoop-daemon/src/redaction.rs`
- Backend tests: `hoop-daemon/src/redaction.rs` (test_parity_*)
- Config resolver: `hoop-daemon/src/config_resolver.rs`
- API endpoint: `hoop-daemon/src/api_config.rs::get_secrets_patterns()`
- Plan reference: §18
