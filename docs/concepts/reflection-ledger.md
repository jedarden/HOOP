# Reflection Ledger

> The **Reflection Ledger** is HOOP's learned-rules store. When you repeat an instruction across Stitches, the agent proposes a durable rule you can approve. Approved rules inject into every subsequent agent session.

## What the Ledger does

The Reflection Ledger captures patterns from your work and turns them into reusable rules:

| Stage | What happens |
|-------|--------------|
| **Detect** | After each closed operator Stitch, scan for repeated patterns |
| **Propose** | Agent suggests rules based on patterns (you see these in the UI) |
| **Approve** | You accept, reject, or modify each proposal |
| **Inject** | Approved rules become part of the agent's system prompt |

## Rule lifecycle

```
Repeated instruction → Detection → Proposal → Approval → Injection
                                                        ↓
                                              Every future session
```

1. **Detection** — Pattern detector finds repetition in closed Stitches
2. **Proposal** — Agent drafts a rule from the pattern
3. **Approval** — Operator reviews and approves/rejects
4. **Injection** — Approved rules load into every new agent session

## What makes a rule

| Field | Description |
|-------|-------------|
| `id` | Unique UUID |
| `name` | Short identifier |
| `pattern` | The trigger (what to look for) |
| `action` | What to do when pattern matches |
| `confidence` | How strong the pattern is (0-1) |
| `approved_at` | When operator approved |
| `source_stitches` | Which Stitches taught this rule |

## Example rules

### From code review patterns

```
Pattern: "always add tests for new functions"
Action: When creating a Stitch that adds functions, include a test-writing step

Confidence: 0.85
Source: 12 closed Stitches with this pattern
```

### From cost optimization

```
Pattern: "use Sonnet for drafts, Opus for review"
Action: Suggest model selection based on task type

Confidence: 0.92
Source: 23 closed Stitches showing this pattern
```

### From project conventions

```
Pattern: "kalshi-weather uses UTC for all timestamps"
Action: When working in kalshi-weather, remind to use UTC

Confidence: 0.78
Source: 8 closed Stitches in this project
```

## How proposals work

After you close an operator Stitch:

1. The **reflection detector** scans the Stitch for patterns
2. If a pattern matches 3+ times across your history, a proposal is created
3. You see a banner in the UI:
   ```
   💡 New rule proposal
      "Always add tests for new functions"

      Based on: 12 Stitches over 30 days
      Confidence: 85%

      [Approve] [Reject] [Modify]
   ```
4. Your choice is recorded in the ledger

## Viewing and managing rules

### Via the UI

Navigate to **Settings → Reflection Ledger** to see:

- All approved rules with confidence scores
- When each rule was learned
- Which Stitches taught it
- Options to disable or delete rules

### Via the CLI

```bash
# List all rules
hoop reflection list

# Show a specific rule
hoop reflection show <rule-id>

# Disable a rule (doesn't delete, just stops injection)
hoop reflection disable <rule-id>

# Delete a rule permanently
hoop reflection delete <rule-id>

# Export rules for backup
hoop reflection export > reflection-rules.json
```

## Rule injection

Approved rules are injected into every agent session:

```
[System prompt base...]

=== Reflection Rules ===
1. Always add tests for new functions (confidence: 0.85)
2. Use Sonnet for drafts, Opus for review (confidence: 0.92)
3. kalshi-weather uses UTC for all timestamps (confidence: 0.78)
=== End Reflection Rules ===

[User message...]
```

The agent sees these rules as part of its context and applies them automatically.

## Privacy considerations

- Rules are **per-operator** — each user has their own ledger
- Rules **never include secrets** — patterns are abstracted before storage
- Rules are **stored in fleet.db** — backed up with your other HOOP data
- **Nothing is learned silently** — every rule requires approval

## Disabling the Reflection Ledger

If you don't want learned rules:

1. **Via UI**: Settings → Reflection Ledger → Disable all proposals
2. **Via config**:
   ```yaml
   agent:
     reflection_ledger_enabled: false
   ```

Disabling doesn't delete existing rules — it just stops new proposals and injection.

## When proposals arrive

Proposals appear **after** a Stitch closes, not during work. This ensures:

- No interruption while you're focused
- Proposals are based on complete, closed work
- You can approve/reject at your convenience

You'll see a "What's New" banner on next login with pending proposals.

## Related concepts

- **Human-Interface Agent** — Receives injected rules
- **Stitches** — Source of pattern data
- **Morning Brief** — May mention new proposals
