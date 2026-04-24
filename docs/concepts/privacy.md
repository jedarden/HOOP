# Privacy & Redaction

> Plan reference: §18

HOOP handles voice transcripts, screen-capture recordings, file contents, bead
drafts, morning briefs, and agent conversations. All of these surfaces can
inadvertently contain secrets — API keys, tokens, passwords, credentials.

This document enumerates every ingestion surface, its scanner hook, and the
operator controls available to act on findings.

---

## 1. Guiding principle: flag, don't block

Detected secrets are **flagged, not silently deleted**. The operator sees a
warning banner listing what was found and can choose:

- **Redact-in-place** — the secret in the stored text/transcript is replaced
  with `[REDACTED]`; audio is muted at the word's timestamps (for voice notes).
- **Redact-and-delete** — the entire attachment or record is discarded.
- **Proceed anyway** — the operator acknowledges the finding and stores as-is.

Nothing happens silently. Every finding, and every operator action taken on a
finding, is recorded in the audit log.

---

## 2. Scanner patterns

All surfaces use the same pattern set, implemented in
`hoop-daemon/src/redaction.rs`. Named patterns:

| Pattern name | What it matches | Example |
|---|---|---|
| `anthropic_api_key` | `sk-ant-...` Anthropic API keys | `sk-ant-api03-...` |
| `generic_sk_key` | `sk-...` generic secret keys (OpenAI, etc.) | `sk-ABCDEFGH...` |
| `aws_access_key` | AWS Access Key IDs | `AKIAIOSFODNN7EXAMPLE` |
| `github_token_ghp` | Classic GitHub PATs | `ghp_16C7e42F...` |
| `github_token_ghs` | GitHub server tokens | `ghs_...` |
| `github_token_ghu` | GitHub user tokens | `ghu_...` |
| `github_pat` | Fine-grained GitHub PATs | `github_pat_...` |
| `slack_bot_token` | Slack bot tokens | `xoxb-...` |
| `slack_user_token` | Slack user tokens | `xoxp-...` |
| `jwt` | JSON Web Tokens | `eyJhbGci...` |
| `bearer_token` | Bearer tokens in HTTP headers | `Authorization: Bearer ...` |
| `env_var_secret` | Environment variable assignments | `API_KEY=<value>` |
| `json_secret_field` | JSON object secret fields | `{"password": "..."}` |

The scanner has two modes:

- **Detection** (`scan_text_for_secrets`): returns `Vec<SecretFinding>` with
  pattern name and byte position. Text is unchanged. Used at ingestion
  boundaries.
- **Redaction** (`redact_text`, `redact_json_value`): replaces matches with
  `[REDACTED]`. Used on the read side before returning projections to the UI or
  the human-interface agent.

---

## 3. Ingestion surfaces and scanner hooks

Every new data path that enters HOOP has a corresponding scanner call. The
table below is the authoritative coverage report; it mirrors the
`coverage_report_all_surfaces_accounted_for` test in
`hoop-daemon/tests/privacy_surface_audit.rs`.

### Phase 0 — CLI session JSONL (read-side)

| Surface | Scanner function | Call site |
|---|---|---|
| CLI session JSONL projections (Claude, Codex, OpenCode, Gemini, Aider) | `redact_text` / `redact_json_value` | `hoop-daemon/src/redaction.rs` — called by the session tailer before any projection is emitted to the UI or the human-interface agent. Raw files on disk are never modified. |

### Phase 3 — Screen-capture frames + voice transcripts

| Surface | Scanner function | Call site |
|---|---|---|
| Voice transcripts (Whisper output) | `scan_voice_transcript` | `hoop-daemon/src/api_dictated_notes.rs` — `create_note` (when a pre-computed transcript is provided) and `update_note` (when a transcript is set post-transcription). |
| Screen-capture frame text (OCR / narration) | `scan_screen_capture_text` | Called when the screen-capture endpoint processes frame samples or the attached narration transcript. Hook is in `hoop-daemon/src/redaction.rs`; wired into the endpoint when the screen-capture feature is built (Phase 3). |

### Phase 4 — Bulk-draft bodies + imported markdown lists

| Surface | Scanner function | Call site |
|---|---|---|
| Draft title + description (single creation) | `scan_draft_body` | `hoop-daemon/src/api_draft_queue.rs` — `create_draft`, executed before the draft is inserted into the queue. |
| Bulk-imported markdown lists | `scan_draft_body` (one call per item) | Same handler — each item in a bulk import is scanned individually before the corresponding draft row is written. |

### Phase 5 — Morning-brief outputs + cross-project propagation drafts

| Surface | Scanner function | Call site |
|---|---|---|
| Morning brief markdown content | `scan_morning_brief` | `hoop-daemon/src/fleet.rs` — `insert_morning_brief`, executed before the row is written to `fleet.db`. Prevents secrets from leaking laterally into Stitches when the brief is forwarded. |
| Cross-project propagation drafts | `scan_propagation_draft` | Propagation drafts are synthesised by the human-interface agent and submitted through `create_draft`, which applies `scan_draft_body`. The named `scan_propagation_draft` wrapper is available for call sites that need to distinguish propagation drafts from operator drafts at the call-site level. |

---

## 4. Read-side redaction (§18.3)

When HOOP reads CLI session JSONL files to build projections, it applies
`redact_text` / `redact_json_value` to every string value before emitting the
result. **The raw files on disk are never modified** — only HOOP's own
projections, the UI transcripts, and data forwarded to the human-interface
agent are redacted.

This means a `claude` session that contained an API key in scrollback will not
leak through HOOP's lens, even though the raw file retains the original content
(which belongs to the CLI, not to HOOP).

---

## 5. Per-phase integration tests

Tests live in `hoop-daemon/tests/privacy_surface_audit.rs`. Each phase has
named tests of the form `phase{N}_{surface}_{outcome}`:

```
# Phase 3
phase3_screen_capture_frame_flags_anthropic_key
phase3_screen_capture_frame_flags_github_token
phase3_screen_capture_frame_clean_text_no_findings
phase3_voice_transcript_flags_anthropic_key
phase3_voice_transcript_flags_jwt
phase3_voice_transcript_flags_env_var_style_secret
phase3_voice_transcript_clean_no_findings
phase3_finding_position_metadata_accurate

# Phase 4
phase4_draft_title_with_secret_flagged
phase4_draft_body_with_secret_flagged
phase4_bulk_import_markdown_list_flags_secret_item
phase4_draft_body_json_style_secret_field_flagged
phase4_clean_draft_no_findings
phase4_draft_body_bearer_token_flagged

# Phase 5
phase5_morning_brief_flags_api_key
phase5_morning_brief_flags_github_token
phase5_morning_brief_clean_no_findings
phase5_propagation_draft_title_secret_flagged
phase5_propagation_draft_body_secret_flagged
phase5_propagation_draft_clean_no_findings
phase5_propagation_draft_jwt_lateral_leak_blocked

# Coverage report
coverage_report_all_surfaces_accounted_for
```

Run all of them with:

```bash
cargo test -p hoop-daemon --test privacy_surface_audit
```

---

## 6. Operator controls (planned, §18.5)

The following controls are planned for Phase 4–5 delivery and will be
configured in `~/.hoop/config.yml`:

```yaml
privacy:
  # Action taken when secrets are detected:
  #   warn    — flag and log; operator sees a banner (default)
  #   redact  — auto-redact-in-place before storage
  #   reject  — refuse to store/process the content
  action_on_detection: warn

  # Per-project overrides (e.g. stricter for customer-data projects)
  project_overrides:
    customer-data:
      action_on_detection: reject

  # Entropy threshold for high-entropy string detection (0.0 = disabled)
  entropy_threshold: 0.0

  # Custom PII patterns (email, phone, etc.)
  custom_patterns: []
```

Every detection event and operator action is recorded in the `fleet.db` actions
table with the surface name, pattern name, and actor.

---

## 7. What HOOP cannot prevent

The CLI agents (Claude Code, Codex, etc.) write their own session JSONL files to
disk. If an agent session contained a secret, it remains in that CLI's native
cache — HOOP cannot clean up files it does not own. HOOP's redaction applies
only to HOOP's own surfaces: its UI projections, its transcripts, its
attachments, and content forwarded to the human-interface agent.

For disk-level cleanup of CLI session files, use the CLI's own session
management commands or a separate housekeeping script.
