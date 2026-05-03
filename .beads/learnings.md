# Workspace Learnings

This file is automatically managed by NEEDLE. Learnings from completed beads are captured here.

### 2026-04-30 | bead: hoop-ttb.11.4 | worker: claude-code-glm-4.7-oscar | type: other | reinforced: 13
- **Observation:** For future property testing tasks: (1) define clear invariants with proptest strategies, (2) include both property tests and unit tests for coverage, (3) document shrinking behavior and seed reproduction, (4) when using writeln! with JSON, always escape braces as `{{` and `}}`.
- **Confidence:** high
- **Source:** reusable-pattern from hoop-ttb.11.4

### 2026-04-30 | bead: hoop-ttb.11.4 | worker: claude-code-glm-4.7-oscar | type: other | reinforced: 0
- **Observation:** Initial compilation attempts were blocked by unrelated OpenAPI generation errors in openapi.rs. Additionally, inconsistent brace escaping in writeln! calls caused syntax errors (some lines had `{{}}` while others had `{}`).
- **Confidence:** low
- **Source:** what-didnt-work from hoop-ttb.11.4

### 2026-04-30 | bead: hoop-ttb.17.3 | worker: claude-code-glm-4.7-kilo | type: other | reinforced: 2
- **Observation:** N/A - implementation was already in place.
- **Confidence:** low
- **Source:** what-didnt-work from hoop-ttb.17.3

### 2026-04-30 | bead: hoop-ttb.5.8.1 | worker: claude-code-glm-4.7-foxtrot | type: other | reinforced: 0
- **Observation:** Initial approach tried to compute similarity against all historical Stitches, which would be too slow. Switched to pre-computed percentile buckets.
- **Confidence:** low
- **Source:** what-didnt-work from hoop-ttb.5.8.1

### 2026-04-30 | bead: hoop-ttb.9.1.1 | worker: claude-code-glm-4.7-golf | type: other | reinforced: 0
- **Observation:** For beads tied to plan sections that reference existing implementations, verify the code first before starting new work.
- **Confidence:** high
- **Source:** reusable-pattern from hoop-ttb.9.1.1

