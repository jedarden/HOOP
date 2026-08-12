# HOOP Error Message Validation Report

**Generated:** "unknown"
**Standards Source:** bf-4vtp7 (error_message_standards.md, error_message_informational_actionability_standards.md)

## Summary

- **Total messages validated:** 5,904
- **Total violations found:** 4,826
- **Compliance rate:** 18.3%

## Violations by Category

| Category | Count | Percentage |
|----------|-------|------------|
| missing_info | 2,595 | 53.8% |
| capitalization | 1,513 | 31.4% |
| wording | 700 | 14.5% |
| formatting | 10 | 0.2% |
| punctuation | 8 | 0.2% |

## Detailed Violations by Category

### WORDING

Wording convention violations (patterns, terminology)

#### Subcategory: other (123 violations)

**1.** `clap_test_utils.rs:24` (assert_eq)

- **Message:** `scan`
- **Issue:** Message 'scan' is too minimal or cryptic

**2.** `clap_test_utils.rs:28` (assert_eq)

- **Message:** `/tmp`
- **Issue:** Message '/tmp' is too minimal or cryptic

**3.** `clap_test_utils.rs:485` (assert_eq)

- **Message:** `/tmp`
- **Issue:** Message '/tmp' is too minimal or cryptic

**4.** `clap_test_utils.rs:582` (assert_eq)

- **Message:** `scan`
- **Issue:** Message 'scan' is too minimal or cryptic

**5.** `clap_test_utils.rs:587` (assert_eq)

- **Message:** `scan`
- **Issue:** Message 'scan' is too minimal or cryptic

**6.** `clap_test_utils.rs:592` (assert_eq)

- **Message:** `--no-interactive`
- **Issue:** Message '--no-interactive' is too minimal or cryptic

**7.** `clap_test_utils.rs:921` (assert_eq)

- **Message:** `scan`
- **Issue:** Message 'scan' is too minimal or cryptic

**8.** `clap_test_utils.rs:927` (assert_eq)

- **Message:** `-y`
- **Issue:** Message '-y' is too minimal or cryptic

**9.** `clap_test_utils.rs:933` (assert_eq)

- **Message:** `scan`
- **Issue:** Message 'scan' is too minimal or cryptic

**10.** `clap_test_utils.rs:939` (assert_eq)

- **Message:** `/tmp`
- **Issue:** Message '/tmp' is too minimal or cryptic

*... and 113 more violations in this subcategory*

#### Subcategory: Message with 'should' might be missing 'when' clause (158 violations)

**1.** `clap_test_utils.rs:682` (assert_eq)

- **Message:** `no_interactive should be true with flag before command`
- **Issue:** Message with 'should' might be missing 'when' clause: no_interactive should be true with flag before command

**2.** `clap_test_utils.rs:704` (assert_eq)

- **Message:** `no_interactive should be true with flag after command`
- **Issue:** Message with 'should' might be missing 'when' clause: no_interactive should be true with flag after command

**3.** `clap_test_utils.rs:726` (assert_eq)

- **Message:** `no_interactive should be true with -y flag`
- **Issue:** Message with 'should' might be missing 'when' clause: no_interactive should be true with -y flag

**4.** `cli_test_helpers.rs:287` (assert_eq)

- **Message:** `Flag should be true`
- **Issue:** Message with 'should' might be missing 'when' clause: Flag should be true

**5.** `cli_test_helpers.rs:345` (assert_eq)

- **Message:** `Flag should be true at any position`
- **Issue:** Message with 'should' might be missing 'when' clause: Flag should be true at any position

**6.** `cli_test_helpers.rs:2003` (assert_eq)

- **Message:** `Flag position should not affect value for {}`
- **Issue:** Message with 'should' might be missing 'when' clause: Flag position should not affect value for {}

**7.** `cli_test_helpers.rs:2058` (assert_eq)

- **Message:** `Primary subcommand should be {}`
- **Issue:** Message with 'should' might be missing 'when' clause: Primary subcommand should be {}

**8.** `cli_test_helpers.rs:2064` (assert_eq)

- **Message:** `Nested subcommand should be {}`
- **Issue:** Message with 'should' might be missing 'when' clause: Nested subcommand should be {}

**9.** `cli_test_helpers.rs:2867` (assert_eq)

- **Message:** `Flag should be consistent at both positions`
- **Issue:** Message with 'should' might be missing 'when' clause: Flag should be consistent at both positions

**10.** `cli_test_helpers.rs:339` (assert)

- **Message:** `Parsing should succeed even with flag at end`
- **Issue:** Message with 'should' might be missing 'when' clause: Parsing should succeed even with flag at end

*... and 148 more violations in this subcategory*

#### Subcategory: Message uses 'must' but might be better with 'should' (419 violations)

**1.** `cli_test_helpers.rs:2211` (assert_eq)

- **Message:** `Flag position must not affect value for {}`
- **Issue:** Message uses 'must' but might be better with 'should': Flag position must not affect value for {}

**2.** `cli_test_utils.rs:597` (assert_eq)

- **Message:** `no_interactive value must be consistent regardless of flag position`
- **Issue:** Message uses 'must' but might be better with 'should': no_interactive value must be consistent regardless of flag position

**3.** `cli_test_utils.rs:698` (assert_eq)

- **Message:** `no_interactive value must be consistent regardless of flag position`
- **Issue:** Message uses 'must' but might be better with 'should': no_interactive value must be consistent regardless of flag position

**4.** `cli_test_utils.rs:769` (assert_eq)

- **Message:** `Flag value must be consistent regardless of position`
- **Issue:** Message uses 'must' but might be better with 'should': Flag value must be consistent regardless of position

**5.** `cli_test_utils.rs:939` (assert_eq)

- **Message:** `Flag must be consistent regardless of position`
- **Issue:** Message uses 'must' but might be better with 'should': Flag must be consistent regardless of position

**6.** `init_no_interactive_flag.rs:162` (assert)

- **Message:** `run_init_wizard must accept no_interactive parameter`
- **Issue:** Message uses 'must' but might be better with 'should': run_init_wizard must accept no_interactive parameter

**7.** `init_no_interactive_flag.rs:168` (assert)

- **Message:** `run_init_wizard must check no_interactive flag`
- **Issue:** Message uses 'must' but might be better with 'should': run_init_wizard must check no_interactive flag

**8.** `init_no_interactive_flag.rs:185` (assert)

- **Message:** `Init must check no_interactive flag early in the handler`
- **Issue:** Message uses 'must' but might be better with 'should': Init must check no_interactive flag early in the handler

**9.** `init_no_interactive_flag.rs:191` (assert)

- **Message:** `Init must exit with code 2 when no_interactive is true`
- **Issue:** Message uses 'must' but might be better with 'should': Init must exit with code 2 when no_interactive is true

**10.** `init_no_interactive_flag.rs:197` (assert)

- **Message:** `Init must explain why it cannot run non-interactively`
- **Issue:** Message uses 'must' but might be better with 'should': Init must explain why it cannot run non-interactively

*... and 409 more violations in this subcategory*

---

### FORMATTING

Formatting violations (placeholders, structure)

#### Subcategory: Placeholders should be at end of message (10 violations)

**1.** `golden_transcripts_regression.rs:280` (assert)

- **Message:** `Simple turn scenario {:?} for adapter '{}' must contain at least one text event`
- **Issue:** Placeholders should be at end of message: Simple turn scenario {:?} for adapter '{}' must contain at least one text event

**2.** `golden_transcripts_regression.rs:344` (assert)

- **Message:** `Tool-heavy scenario {:?} for adapter '{}' must contain at least one tool event`
- **Issue:** Placeholders should be at end of message: Tool-heavy scenario {:?} for adapter '{}' must contain at least one tool event

**3.** `golden_transcripts_regression.rs:391` (assert)

- **Message:** `Failure scenario {:?} for adapter '{}' must contain at least one error indication`
- **Issue:** Placeholders should be at end of message: Failure scenario {:?} for adapter '{}' must contain at least one error indication

**4.** `golden_transcripts_regression.rs:561` (assert)

- **Message:** `Simple turn scenario {:?} for adapter '{}' must parse to at least one TextDelta event`
- **Issue:** Placeholders should be at end of message: Simple turn scenario {:?} for adapter '{}' must parse to at least one TextDelta event

**5.** `golden_transcripts_regression.rs:613` (assert)

- **Message:** `Tool-heavy scenario {:?} for adapter '{}' must parse to at least one ToolUse event`
- **Issue:** Placeholders should be at end of message: Tool-heavy scenario {:?} for adapter '{}' must parse to at least one ToolUse event

**6.** `golden_transcripts_regression.rs:619` (assert)

- **Message:** `Tool-heavy scenario {:?} for adapter '{}' must parse to at least one ToolResult event`
- **Issue:** Placeholders should be at end of message: Tool-heavy scenario {:?} for adapter '{}' must parse to at least one ToolResult event

**7.** `golden_transcripts_regression.rs:665` (assert)

- **Message:** `Failure scenario {:?} for adapter '{}' must parse to at least one Error event`
- **Issue:** Placeholders should be at end of message: Failure scenario {:?} for adapter '{}' must parse to at least one Error event

**8.** `needle_events_roundtrip.rs:515` (assert)

- **Message:** `heartbeat line {} failed to parse: {line}`
- **Issue:** Placeholders should be at end of message: heartbeat line {} failed to parse: {line}

**9.** `secrets_scanner_parity.rs:244` (assert_eq)

- **Message:** `Fixture '{}' references pattern_id '{}' which doesn't exist in default patterns`
- **Issue:** Placeholders should be at end of message: Fixture '{}' references pattern_id '{}' which doesn't exist in default patterns

**10.** `secrets_scanner_parity.rs:256` (assert)

- **Message:** `Fixture '{}' references pattern_id '{}' which doesn't exist in default patterns`
- **Issue:** Placeholders should be at end of message: Fixture '{}' references pattern_id '{}' which doesn't exist in default patterns

---

### PUNCTUATION

Punctuation violations (periods, quotes)

#### Subcategory: Unnecessary quotes around simple value (7 violations)

**1.** `scan_no_interactive_flag.rs:29` (assert_eq)

- **Message:** `Command should be 'scan'`
- **Issue:** Unnecessary quotes around simple value: Command should be 'scan'

**2.** `scan_no_interactive_flag.rs:49` (assert_eq)

- **Message:** `Command should be 'scan'`
- **Issue:** Unnecessary quotes around simple value: Command should be 'scan'

**3.** `scan_no_interactive_flag.rs:69` (assert_eq)

- **Message:** `Command should be 'scan'`
- **Issue:** Unnecessary quotes around simple value: Command should be 'scan'

**4.** `scan_no_interactive_flag.rs:81` (assert_eq)

- **Message:** `Command should be 'scan'`
- **Issue:** Unnecessary quotes around simple value: Command should be 'scan'

**5.** `scan_no_interactive_flag.rs:93` (assert_eq)

- **Message:** `Command should be 'scan'`
- **Issue:** Unnecessary quotes around simple value: Command should be 'scan'

**6.** `scan_no_interactive_flag.rs:106` (assert_eq)

- **Message:** `Command should be 'scan'`
- **Issue:** Unnecessary quotes around simple value: Command should be 'scan'

**7.** `scan_no_interactive_flag.rs:123` (assert_eq)

- **Message:** `Command should be 'scan'`
- **Issue:** Unnecessary quotes around simple value: Command should be 'scan'

#### Subcategory: Message has trailing period (1 violations)

**1.** `phase2_exit_gate.rs:415` (assert)

- **Message:** `Phase 2 exit gate FAILED: {} of 13 core deliverables lack passing tests. \
        Marquee features (14-17) cannot merge until all core deliverables are verified.`
- **Issue:** Message has trailing period: Phase 2 exit gate FAILED: {} of 13 core deliverables lack passing tests. \
        Marquee features (14-17) cannot merge until all core deliverables are verified.

---

### CAPITALIZATION

Capitalization violations (case conventions)

#### Subcategory: First word should be capitalized (1492 violations)

**1.** `clap_test_utils.rs:24` (assert_eq)

- **Message:** `scan`
- **Issue:** First word should be capitalized: scan

**2.** `clap_test_utils.rs:582` (assert_eq)

- **Message:** `scan`
- **Issue:** First word should be capitalized: scan

**3.** `clap_test_utils.rs:587` (assert_eq)

- **Message:** `scan`
- **Issue:** First word should be capitalized: scan

**4.** `clap_test_utils.rs:682` (assert_eq)

- **Message:** `no_interactive should be true with flag before command`
- **Issue:** First word should be capitalized: no_interactive should be true with flag before command

**5.** `clap_test_utils.rs:704` (assert_eq)

- **Message:** `no_interactive should be true with flag after command`
- **Issue:** First word should be capitalized: no_interactive should be true with flag after command

**6.** `clap_test_utils.rs:726` (assert_eq)

- **Message:** `no_interactive should be true with -y flag`
- **Issue:** First word should be capitalized: no_interactive should be true with -y flag

**7.** `clap_test_utils.rs:772` (assert_eq)

- **Message:** `no_interactive should default to false`
- **Issue:** First word should be capitalized: no_interactive should default to false

**8.** `clap_test_utils.rs:921` (assert_eq)

- **Message:** `scan`
- **Issue:** First word should be capitalized: scan

**9.** `clap_test_utils.rs:933` (assert_eq)

- **Message:** `scan`
- **Issue:** First word should be capitalized: scan

**10.** `clap_test_utils.rs:1026` (assert_eq)

- **Message:** `scan`
- **Issue:** First word should be capitalized: scan

*... and 1482 more violations in this subcategory*

#### Subcategory: Acronym 'cli' should be 'CLI' (21 violations)

**1.** `adapter_failover_test.rs:157` (expect)

- **Message:** `Failed to create client`
- **Issue:** Acronym 'cli' should be 'CLI': Failed to create client

**2.** `adapter_failover_test.rs:189` (expect)

- **Message:** `Failed to create client`
- **Issue:** Acronym 'cli' should be 'CLI': Failed to create client

**3.** `adapter_failover_test.rs:267` (expect)

- **Message:** `Failed to create client`
- **Issue:** Acronym 'cli' should be 'CLI': Failed to create client

**4.** `adapter_failover_test.rs:346` (expect)

- **Message:** `Failed to create client`
- **Issue:** Acronym 'cli' should be 'CLI': Failed to create client

**5.** `adapter_failover_test.rs:404` (expect)

- **Message:** `Failed to create client`
- **Issue:** Acronym 'cli' should be 'CLI': Failed to create client

**6.** `adapter_failover_test.rs:482` (expect)

- **Message:** `Failed to create client`
- **Issue:** Acronym 'cli' should be 'CLI': Failed to create client

**7.** `adapter_failover_test.rs:541` (expect)

- **Message:** `Failed to create client`
- **Issue:** Acronym 'cli' should be 'CLI': Failed to create client

**8.** `adapter_failover_test.rs:599` (expect)

- **Message:** `Failed to create client`
- **Issue:** Acronym 'cli' should be 'CLI': Failed to create client

**9.** `adapter_failover_test.rs:814` (expect)

- **Message:** `Failed to create client`
- **Issue:** Acronym 'cli' should be 'CLI': Failed to create client

**10.** `adapter_failover_test.rs:919` (expect)

- **Message:** `Failed to create client`
- **Issue:** Acronym 'cli' should be 'CLI': Failed to create client

*... and 11 more violations in this subcategory*

---

### MISSING_INFO

Missing informational content (what, target, expected)

#### Subcategory: Message doesn't indicate what failed (2497 violations)

**1.** `clap_test_utils.rs:24` (assert_eq)

- **Message:** `scan`
- **Issue:** Message doesn't indicate what failed: scan

**2.** `clap_test_utils.rs:28` (assert_eq)

- **Message:** `/tmp`
- **Issue:** Message doesn't indicate what failed: /tmp

**3.** `clap_test_utils.rs:485` (assert_eq)

- **Message:** `/tmp`
- **Issue:** Message doesn't indicate what failed: /tmp

**4.** `clap_test_utils.rs:582` (assert_eq)

- **Message:** `scan`
- **Issue:** Message doesn't indicate what failed: scan

**5.** `clap_test_utils.rs:587` (assert_eq)

- **Message:** `scan`
- **Issue:** Message doesn't indicate what failed: scan

**6.** `clap_test_utils.rs:592` (assert_eq)

- **Message:** `--no-interactive`
- **Issue:** Message doesn't indicate what failed: --no-interactive

**7.** `clap_test_utils.rs:921` (assert_eq)

- **Message:** `scan`
- **Issue:** Message doesn't indicate what failed: scan

**8.** `clap_test_utils.rs:927` (assert_eq)

- **Message:** `-y`
- **Issue:** Message doesn't indicate what failed: -y

**9.** `clap_test_utils.rs:933` (assert_eq)

- **Message:** `scan`
- **Issue:** Message doesn't indicate what failed: scan

**10.** `clap_test_utils.rs:939` (assert_eq)

- **Message:** `/tmp`
- **Issue:** Message doesn't indicate what failed: /tmp

*... and 2487 more violations in this subcategory*

#### Subcategory: Comparison missing 'expected' value (98 violations)

**1.** `scan_no_interactive_flag.rs:715` (assert)

- **Message:** `Scan should have local --yes flag defined with arg attribute`
- **Issue:** Comparison missing 'expected' value: Scan should have local --yes flag defined with arg attribute

**2.** `s1_morning_review.rs:59` (assert)

- **Message:** `total_workers must be numeric, got: {}`
- **Issue:** Comparison missing 'expected' value: total_workers must be numeric, got: {}

**3.** `s1_morning_review.rs:73` (assert)

- **Message:** `total_spend_usd must be non-negative, got: {}`
- **Issue:** Comparison missing 'expected' value: total_spend_usd must be non-negative, got: {}

**4.** `s2_transcript_archaeology.rs:65` (assert)

- **Message:** `Bead events endpoint should return 200 or 404, got: {}`
- **Issue:** Comparison missing 'expected' value: Bead events endpoint should return 200 or 404, got: {}

**5.** `s2_transcript_archaeology.rs:155` (assert)

- **Message:** `Stitch read endpoint should return 200 or 404, got: {}`
- **Issue:** Comparison missing 'expected' value: Stitch read endpoint should return 200 or 404, got: {}

**6.** `s2_transcript_archaeology.rs:193` (assert)

- **Message:** `Endpoint {} should return 200 or 404, got: {}`
- **Issue:** Comparison missing 'expected' value: Endpoint {} should return 200 or 404, got: {}

**7.** `s6_machine_mode.rs:114` (assert_eq)

- **Message:** `hoop status --json should exit with code 0, got: {:?}`
- **Issue:** Comparison missing 'expected' value: hoop status --json should exit with code 0, got: {:?}

**8.** `s6_machine_mode.rs:266` (assert)

- **Message:** `stdout should not contain interactive prompts, got: {}`
- **Issue:** Comparison missing 'expected' value: stdout should not contain interactive prompts, got: {}

**9.** `s6_machine_mode.rs:284` (assert)

- **Message:** `Output should be concise without prompts, got {} lines`
- **Issue:** Comparison missing 'expected' value: Output should be concise without prompts, got {} lines

**10.** `s6_machine_mode.rs:412` (assert)

- **Message:** `stdout should not contain prompt '{}' for args {:?}, got: {}`
- **Issue:** Comparison missing 'expected' value: stdout should not contain prompt '{}' for args {:?}, got: {}

*... and 88 more violations in this subcategory*

---

## Recommendations

### Priority 1: High-Impact Fixes

1. **Cryptic/Minimal Messages** - Add descriptive context to messages that are too brief
2. **Missing Expected Values** - Always include "expected X, got Y" for comparisons
3. **Trailing Periods** - Remove periods from error messages (standard violation)

### Priority 2: Consistency Improvements

1. **Missing "when" Clauses** - Add conditional context to "should" statements
2. **Capitalization** - Standardize acronyms (CLI not cli) and first-word capitalization
3. **Unnecessary Quotes** - Remove quotes around simple values (true, false, scan, etc.)

### Priority 3: Enhanced Quality

1. **Placeholder Placement** - Move format placeholders to end of messages
2. **MUST vs SHOULD** - Reserve "must" for invariants, use "should" for preferences

---

**Validation Tool:** bin/validate_error_messages.py
**Standards Documents:** error_message_standards.md, error_message_informational_actionability_standards.md
