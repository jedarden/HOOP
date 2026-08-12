# HOOP Error Messages Complete Inventory

**Generated:** 2026-08-12
**Total Error Messages:** 5,694
**Total Files:** 140

---

## Summary Statistics

### By Error Category
- **Error**: 2,012 instances
- **anyhow**: 43 instances
- **assertion**: 3,639 instances

### By Pattern Type
- **assert!**: 2,032 instances
- **.expect()**: 1,987 instances
- **assert_eq!**: 1,501 instances
- **panic!**: 93 instances
- **anyhow::bail!()**: 41 instances
- **.unwrap_err()**: 25 instances
- **assert_ne!**: 13 instances
- **.context()**: 2 instances

### Top 20 Files by Error Message Density
| File | Count |
|------|-------|
| `hoop-daemon/tests/integration_harness.rs` | 191 |
| `hoop-cli/tests/cli_test_helpers.rs` | 184 |
| `hoop-cli/tests/scan_no_interactive_flag.rs` | 183 |
| `hoop-daemon/tests/draft_queue_invariants.rs` | 164 |
| `hoop-cli/tests/no_interactive_flag_behavior.rs` | 163 |
| `hoop-cli/tests/remove_no_interactive_flag.rs` | 159 |
| `hoop-daemon/tests/adapter_failover_test.rs` | 148 |
| `hoop-daemon/tests_phase5/adapter_failover_test.rs` | 148 |
| `tests/cli_test_helpers.rs` | 135 |
| `hoop-cli/tests/restore_no_interactive_flag.rs` | 127 |
| `hoop-daemon/tests/config_field_validation.rs` | 119 |
| `hoop-cli/tests/cli_test_utils.rs` | 112 |
| `hoop-daemon/tests/acceptance/s3_bead_creation_from_chat.rs` | 98 |
| `hoop-daemon/tests/s3_bead_creation_from_chat.rs` | 98 |
| `hoop-daemon/tests/testrepo_integration.rs` | 98 |
| `hoop-daemon/tests/multi_operator_concurrency.rs` | 92 |
| `hoop-cli/tests/clap_test_utils.rs` | 89 |
| `hoop-daemon/tests/state_projections.rs` | 89 |
| `hoop-daemon/tests/testrepo_harness_integration.rs` | 88 |
| `hoop-daemon/tests/needle_events_roundtrip.rs` | 85 |

---

## Detailed Messages by Pattern Type

### .context() (2 instances)

#### hoop-daemon/src/integration_test_client.rs

- **Line 314**: Failed to send WebSocket message
- **Line 384**: Failed to send close message

### .expect() (1,987 instances)

#### hoop-cli/tests/clap_test_utils.rs

- **Line 681**: Should parse with flag before command
- **Line 703**: Should parse with flag after command
- **Line 725**: Should parse with -y flag
- **Line 771**: Should parse without flag
- **Line 803**: Should parse with flag before command
- **Line 811**: Should parse with flag after command
- **Line 819**: Should parse with -y flag
- **Line 840**: Should parse without flag

#### hoop-cli/tests/cli_test_helpers.rs

- **Line 229**: Failed to read main.rs
- **Line 753**: Failed to read mycommand.rs
- **Line 770**: Failed to read main.rs
- **Line 804**: Failed to read projects.rs
- **Line 835**: Failed to read init.rs
- **Line 876**: Failed to read main.rs
- **Line 891**: Failed to read projects.rs
- **Line 2174**: Flag before subcommand assertion failed
- **Line 2195**: Flag after subcommand assertion failed
- **Line 2237**: Default flag assertion failed
- **Line 2830**: Should parse flag before subcommand
- **Line 2838**: Should parse flag after subcommand
- **Line 2846**: Should parse short flag
- **Line 2855**: Should parse nested command
- **Line 2866**: Should parse nested command with flag
- **Line 2882**: Should parse command with multiple flags
- **Line 2890**: Should parse command without flag
- **Line 2911**: Should parse successfully
- **Line 2923**: Should parse flag-only args

#### hoop-cli/tests/cli_test_utils.rs

- **Line 407**: Failed to create .beads/ directory
- **Line 414**: Failed to create .hoop/ directory
- **Line 423**: Failed to write projects.yaml
- **Line 586**: Failed to parse with flag before command
- **Line 595**: Failed to parse with flag after command
- **Line 757**: Failed to parse with flag before subcommand
- **Line 764**: Failed to parse with flag after subcommand
- **Line 782**: Failed to parse with -y flag
- **Line 788**: Failed to parse without flag
- **Line 800**: Failed to parse with flag before subcommand
- **Line 805**: Failed to parse with flag after subcommand
- **Line 890**: Failed to create temp dir
- **Line 917**: Failed to create temp dir
- **Line 926**: Failed to parse remove with flag before
- **Line 933**: Failed to parse remove with flag after
- **Line 1139**: Failed to create temp dir
- **Line 1148**: Failed to create temp dir

#### hoop-cli/tests/cli_test_utils_examples.rs

- **Line 246**: Failed to create temp dir
- **Line 259**: Failed to create temp dir
- **Line 270**: Failed to read registry file
- **Line 279**: Failed to create temp dir
- **Line 313**: Should parse remove command successfully
- **Line 405**: Should parse successfully
- **Line 433**: Failed to create temp dir
- **Line 440**: Parse with flag before should succeed
- **Line 446**: Parse with flag after should succeed

#### hoop-cli/tests/init_no_interactive_flag.rs

- **Line 92**: Parse should succeed
- **Line 105**: Parse should succeed
- **Line 118**: Parse should succeed
- **Line 134**: Failed to read main.rs
- **Line 159**: Failed to read init.rs
- **Line 182**: Failed to read init.rs
- **Line 219**: Failed to read init.rs
- **Line 316**: Failed to read init.rs
- **Line 353**: Failed to read init.rs
- **Line 358**: Should find no_interactive check
- **Line 364**: Should find exit(2) in no_interactive section
- **Line 417**: Failed to read init.rs

#### hoop-cli/tests/no_interactive_flag_behavior.rs

- **Line 24**: Failed to create .beads/
- **Line 31**: Failed to create .hoop/
- **Line 33**: Failed to write registry
- **Line 42**: Failed to create temp dir
- **Line 60**: Failed to create temp dir
- **Line 94**: Failed to read projects.rs
- **Line 146**: Failed to read projects.rs
- **Line 167**: Failed to read main.rs
- **Line 181**: Failed to read projects.rs
- **Line 212**: Failed to read projects.rs
- **Line 271**: Failed to read projects.rs
- **Line 392**: Failed to read main.rs
- **Line 413**: Failed to read projects.rs
- **Line 432**: Failed to read projects.rs
- **Line 458**: Failed to read restore.rs
- **Line 476**: Failed to read restore.rs
- **Line 494**: Failed to read restore.rs
- **Line 549**: Failed to read restore.rs
- **Line 570**: Failed to read main.rs
- **Line 584**: Failed to read restore.rs
- **Line 615**: Failed to read restore.rs
- **Line 682**: Failed to read restore.rs
- **Line 820**: Failed to read main.rs
- **Line 840**: Failed to read restore.rs
- **Line 866**: Failed to read init.rs
- **Line 889**: Failed to read init.rs
- **Line 913**: Failed to read main.rs
- **Line 950**: Failed to read main.rs
- **Line 963**: Failed to read projects.rs
- **Line 982**: Failed to read projects.rs
- **Line 1001**: Failed to read restore.rs
- **Line 1020**: Failed to read init.rs
- **Line 1043**: Failed to read projects.rs
- **Line 1046**: Should find scan_projects function
- **Line 1084**: Failed to read projects.rs
- **Line 1086**: Failed to read restore.rs
- **Line 1107**: Failed to read init.rs

#### hoop-cli/tests/remove_no_interactive_flag.rs

- **Line 102**: Parse should succeed
- **Line 116**: Parse should succeed
- **Line 130**: Parse should succeed
- **Line 146**: Failed to read main.rs
- **Line 171**: Failed to read projects.rs
- **Line 195**: Failed to read projects.rs
- **Line 199**: Should find remove_project function
- **Line 203**: Should find confirm requirement check
- **Line 227**: Failed to read projects.rs
- **Line 231**: Should find remove_project function
- **Line 235**: Should find confirm requirement check
- **Line 242**: Should have prompt check after confirm requirement
- **Line 270**: Failed to read projects.rs
- **Line 274**: Should find remove_project function
- **Line 278**: Should find prompt check
- **Line 324**: Failed to read projects.rs
- **Line 328**: Should find remove_project function
- **Line 332**: Should find prompt check
- **Line 359**: Failed to read projects.rs
- **Line 363**: Should find remove_project function
- **Line 367**: Should find confirm requirement check
- **Line 371**: Should find end of confirm requirement block
- **Line 385**: Should find prompt check after confirm requirement
- **Line 607**: Should parse global --no-interactive flag
- **Line 622**: Should parse remove command without flags
- **Line 640**: Parse with global flag
- **Line 646**: Parse without flags
- **Line 655**: Should parse short -y flag
- **Line 678**: Parse flag before subcommand
- **Line 683**: Parse flag after subcommand
- **Line 729**: Failed to read projects.rs
- **Line 733**: Should find remove_project function
- **Line 737**: Should find confirm requirement check
- **Line 770**: Failed to read projects.rs
- **Line 774**: Should find remove_project function
- **Line 778**: Should find prompt check for interactive mode
- **Line 813**: Failed to read projects.rs
- **Line 817**: Should find remove_project function
- **Line 821**: Should find prompt check
- **Line 851**: Failed to read projects.rs
- **Line 855**: Should find remove_project function
- **Line 859**: Should find confirm requirement check
- **Line 863**: Should find prompt check after confirm requirement
- **Line 890**: Failed to read projects.rs
- **Line 894**: Should find remove_project function
- **Line 911**: Should find confirm requirement check
- **Line 915**: Should find prompt check after confirm requirement
- **Line 949**: Failed to read projects.rs
- **Line 953**: Should find remove_project function
- **Line 957**: Should find confirm requirement check
- **Line 961**: Should find end of confirm requirement block
- **Line 965**: Should find prompt check after confirm requirement
- **Line 970**: Should find removal call after checks
- **Line 988**: Failed to read main.rs
- **Line 992**: Should find Remove command handler in main.rs
- **Line 1010**: Failed to read main.rs
- **Line 1012**: Failed to read projects.rs

#### hoop-cli/tests/restore_no_interactive_flag.rs

- **Line 174**: Parse should succeed
- **Line 192**: Parse should succeed
- **Line 215**: Parse should succeed
- **Line 230**: Failed to read main.rs
- **Line 255**: Failed to read restore.rs
- **Line 281**: Failed to read restore.rs
- **Line 286**: Should find run_restore function
- **Line 291**: Should find confirm requirement check
- **Line 322**: Failed to read restore.rs
- **Line 327**: Should find run_restore function
- **Line 332**: Should find confirm requirement check
- **Line 340**: Should have prompt check after confirm requirement
- **Line 373**: Failed to read restore.rs
- **Line 378**: Should find run_restore function
- **Line 383**: Should find prompt check
- **Line 440**: Failed to read restore.rs
- **Line 445**: Should find run_restore function
- **Line 450**: Should find prompt check
- **Line 478**: Failed to read restore.rs
- **Line 483**: Should find run_restore function
- **Line 488**: Should find confirm requirement check
- **Line 493**: Should find end of confirm requirement block
- **Line 510**: Should find prompt check after confirm requirement
- **Line 531**: Failed to read restore.rs
- **Line 536**: run_restore must have dry_run mode
- **Line 565**: Failed to read restore.rs
- **Line 570**: restore.rs must define run_restore()
- **Line 575**: run_restore must call manifest.validate(current)
- **Line 578**: run_restore must call move_aside_for_rollback()
- **Line 593**: Failed to read restore.rs
- **Line 598**: restore.rs must define run_restore()
- **Line 603**: run_restore must check no_interactive && !confirm
- **Line 606**: run_restore must check !no_interactive for prompting
- **Line 619**: Failed to read restore.rs
- **Line 624**: Must have --confirm requirement check
- **Line 663**: Should parse flag before command
- **Line 668**: Should parse flag after command
- **Line 694**: Should parse -y flag
- **Line 710**: Failed to read main.rs
- **Line 712**: Failed to read restore.rs
- **Line 779**: run_restore function must exist
- **Line 788**: manifest.validate() must be called in function body
- **Line 791**: move_aside_for_rollback() must be called in function body

#### hoop-cli/tests/scan_no_interactive_flag.rs

- **Line 135**: Parse should succeed
- **Line 148**: Parse should succeed
- **Line 161**: Parse should succeed
- **Line 177**: Failed to read main.rs
- **Line 202**: Failed to read projects.rs
- **Line 222**: Failed to read main.rs
- **Line 232**: Should find Scan command handler
- **Line 236**: Should find scan_projects call with || logic
- **Line 252**: Failed to read projects.rs
- **Line 256**: Should find scan_projects function
- **Line 260**: Should find no_interactive check in scan_projects
- **Line 286**: Failed to read projects.rs
- **Line 290**: Should find scan_projects function
- **Line 322**: Failed to read projects.rs
- **Line 326**: Should find scan_projects function
- **Line 354**: Failed to read projects.rs
- **Line 358**: Should find scan_projects function
- **Line 362**: Should find no_interactive check in scan_projects
- **Line 687**: Failed to read main.rs
- **Line 700**: Failed to read main.rs
- **Line 704**: Should find Scan command documentation
- **Line 726**: Failed to read main.rs
- **Line 730**: Should find Scan command handler
- **Line 733**: Should find scan_projects call
- **Line 752**: Failed to read main.rs
- **Line 754**: Failed to read projects.rs
- **Line 827**: Should parse global --no-interactive flag
- **Line 842**: Should parse local --yes flag
- **Line 857**: Should parse both global --no-interactive and local --yes flags
- **Line 872**: Should parse scan command without flags
- **Line 913**: Parse with global flag
- **Line 919**: Parse with local flag
- **Line 925**: Parse with both flags
- **Line 931**: Parse without flags
- **Line 940**: Should parse short -y flag
- **Line 961**: Should parse with global flag only
- **Line 975**: Should parse with local flag only
- **Line 992**: Parse flag before subcommand
- **Line 997**: Parse flag after subcommand
- **Line 1048**: Failed to read projects.rs
- **Line 1052**: Should find scan_projects function
- **Line 1056**: Should find no_interactive check
- **Line 1098**: Failed to read projects.rs
- **Line 1102**: Should find scan_projects function
- **Line 1106**: Should find else branch with interactive prompts
- **Line 1153**: Failed to read projects.rs
- **Line 1157**: Should find scan_projects function
- **Line 1193**: Failed to read projects.rs
- **Line 1197**: Should find scan_projects function
- **Line 1201**: Should find no_interactive check
- **Line 1205**: Should find else branch for interactive mode
- **Line 1232**: Failed to read projects.rs
- **Line 1236**: Should find scan_projects function
- **Line 1248**: Should find no_interactive check
- **Line 1253**: Should find else branch after no_interactive check
- **Line 1286**: Failed to read projects.rs
- **Line 1290**: Should find scan_projects function
- **Line 1294**: Should find no_interactive check

#### hoop-daemon/examples/populate-testrepo.rs

- **Line 37**: workspace root is parent of hoop-daemon/

#### hoop-daemon/tests/acceptance/s1_morning_review.rs

- **Line 29**: Failed to spawn daemon
- **Line 38**: Failed to fetch dashboard
- **Line 49**: Failed to parse dashboard response
- **Line 58**: total_workers must be a number
- **Line 72**: total_spend_usd must be a number
- **Line 86**: longest_running must be an array
- **Line 94**: Failed to fetch worker timeline
- **Line 102**: Failed to parse timeline
- **Line 118**: Failed to spawn daemon
- **Line 128**: Failed to fetch dashboard
- **Line 153**: Failed to spawn daemon
- **Line 162**: Failed to fetch dashboard
- **Line 170**: Failed to parse response
- **Line 194**: Failed to spawn daemon
- **Line 203**: Failed to fetch dashboard
- **Line 205**: Failed to parse response
- **Line 215**: Failed to fetch dashboard
- **Line 217**: Failed to parse response
- **Line 238**: Failed to spawn daemon
- **Line 246**: Failed to fetch dashboard
- **Line 248**: Failed to parse response
- **Line 253**: total_spend_usd must be present
- **Line 263**: spend_by_project must be an array
- **Line 291**: Failed to spawn daemon
- **Line 299**: Failed to fetch dashboard
- **Line 301**: Failed to parse response
- **Line 305**: total_workers must be present
- **Line 309**: workers_by_project must be an array

#### hoop-daemon/tests/acceptance/s2_transcript_archaeology.rs

- **Line 31**: Failed to spawn daemon
- **Line 40**: Failed to fetch beads
- **Line 48**: Failed to parse beads
- **Line 55**: Bead should have an id
- **Line 62**: Failed to fetch bead events
- **Line 72**: Failed to parse events
- **Line 92**: Failed to spawn daemon
- **Line 101**: Failed to fetch beads
- **Line 103**: Failed to parse beads
- **Line 109**: Bead should have an id
- **Line 118**: Failed to fetch bead events
- **Line 143**: Failed to spawn daemon
- **Line 152**: Failed to connect to stitch endpoint
- **Line 173**: Failed to spawn daemon
- **Line 190**: Failed to connect to endpoint
- **Line 212**: Failed to spawn daemon
- **Line 221**: Failed to fetch conversations
- **Line 229**: Failed to parse conversations
- **Line 250**: Failed to spawn daemon
- **Line 259**: Failed to fetch beads
- **Line 261**: Failed to parse beads
- **Line 287**: Failed to spawn daemon
- **Line 296**: Failed to fetch cost trends
- **Line 304**: Failed to parse cost data
- **Line 326**: Failed to spawn daemon
- **Line 335**: Failed to fetch beads
- **Line 337**: Failed to parse beads
- **Line 343**: Bead should have an id
- **Line 350**: Failed to fetch bead events
- **Line 353**: Failed to parse events

#### hoop-daemon/tests/acceptance/s3_bead_creation_from_chat.rs

- **Line 41**: create temp dir
- **Line 56**: create br script
- **Line 57**: write br script
- **Line 62**: chmod br script
- **Line 107**: Failed to spawn daemon
- **Line 133**: Failed to create draft
- **Line 145**: Failed to parse draft response
- **Line 149**: draft_id should be present
- **Line 162**: Failed to list drafts
- **Line 169**: Failed to parse list response
- **Line 173**: drafts should be an array
- **Line 186**: Failed to get draft
- **Line 193**: Failed to parse draft
- **Line 217**: Failed to spawn daemon
- **Line 235**: Failed to create draft
- **Line 240**: Failed to parse draft response
- **Line 244**: draft_id should be present
- **Line 254**: Failed to approve draft
- **Line 266**: Failed to parse approve response
- **Line 270**: stitch_id should be present
- **Line 292**: Failed to get draft
- **Line 297**: Failed to parse draft
- **Line 318**: Failed to spawn daemon
- **Line 336**: Failed to create draft
- **Line 341**: Failed to parse draft response
- **Line 345**: draft_id should be present
- **Line 353**: Failed to approve draft
- **Line 358**: Failed to parse approve response
- **Line 362**: stitch_id should be present
- **Line 369**: Failed to query audit log
- **Line 376**: Failed to parse audit response
- **Line 380**: audit_rows should be an array
- **Line 392**: args should be an object
- **Line 404**: args should be an object
- **Line 412**: actor should be present
- **Line 434**: Failed to spawn daemon
- **Line 459**: Failed to create draft
- **Line 464**: Failed to parse response
- **Line 465**: draft_id present
- **Line 472**: Failed to list drafts
- **Line 474**: Failed to parse list
- **Line 475**: drafts array
- **Line 488**: Failed to approve draft
- **Line 493**: Failed to parse approve
- **Line 494**: stitch_id present
- **Line 509**: Failed to query audit
- **Line 511**: Failed to parse audit
- **Line 512**: audit_rows array
- **Line 526**: args object
- **Line 530**: args object
- **Line 534**: actor present
- **Line 556**: Failed to spawn daemon
- **Line 577**: Failed to create draft
- **Line 581**: Failed to parse
- **Line 582**: draft_id present
- **Line 589**: Failed to get draft
- **Line 593**: Failed to parse draft

#### hoop-daemon/tests/acceptance/s4_daemon_restart.rs

- **Line 32**: workspace root is parent of hoop-daemon/
- **Line 106**: create temp dir for test HOOP home
- **Line 108**: create .hoop dir
- **Line 123**: write projects.yaml
- **Line 132**: write config.yml
- **Line 134**: create data dir
- **Line 157**: init fleet.db
- **Line 161**: write claim
- **Line 162**: write complete
- **Line 163**: write claim
- **Line 172**: Failed to spawn first daemon
- **Line 197**: Failed to fetch beads from first daemon
- **Line 205**: Failed to parse beads
- **Line 214**: write complete
- **Line 215**: write claim
- **Line 228**: Failed to spawn second daemon
- **Line 251**: Failed to fetch beads from second daemon
- **Line 259**: Failed to parse beads
- **Line 294**: init fleet.db
- **Line 300**: write claim
- **Line 302**: write complete
- **Line 309**: Failed to spawn first daemon
- **Line 336**: Failed to spawn second daemon
- **Line 368**: Failed to fetch beads
- **Line 393**: init fleet.db
- **Line 398**: Failed to spawn first daemon
- **Line 424**: write claim
- **Line 425**: write complete
- **Line 426**: write claim
- **Line 438**: Failed to spawn second daemon
- **Line 457**: write complete
- **Line 458**: write claim
- **Line 472**: Failed to fetch beads
- **Line 496**: init fleet.db
- **Line 503**: write claim
- **Line 504**: write complete
- **Line 510**: Failed to spawn daemon
- **Line 533**: Failed to fetch beads
- **Line 537**: Failed to parse beads
- **Line 560**: write claim

#### hoop-daemon/tests/acceptance/s5_workspace_deleted.rs

- **Line 29**: Failed to create .beads dir
- **Line 31**: Failed to create issues.jsonl
- **Line 39**: Failed to create temp dir
- **Line 41**: Failed to create .hoop dir
- **Line 70**: Failed to write projects.yaml
- **Line 78**: Failed to write config.yml
- **Line 79**: Failed to create data dir
- **Line 121**: Failed to bind to random port
- **Line 122**: Failed to get local address
- **Line 167**: Failed to get readyz status
- **Line 174**: Failed to remove .beads from project A
- **Line 225**: Failed to bind to random port
- **Line 226**: Failed to get local address
- **Line 268**: Failed to remove .beads from project A
- **Line 278**: Failed to fetch projects
- **Line 282**: Failed to parse projects
- **Line 295**: Failed to check health
- **Line 328**: Failed to bind to random port
- **Line 329**: Failed to get local address
- **Line 372**: Failed to get readyz status
- **Line 377**: Failed to remove .beads from project A
- **Line 384**: Failed to get readyz status after deletion
- **Line 435**: Failed to bind to random port
- **Line 436**: Failed to get local address
- **Line 478**: Failed to remove .beads
- **Line 487**: Failed to check health

#### hoop-daemon/tests/acceptance/s6_machine_mode.rs

- **Line 32**: Failed to create temp dir
- **Line 34**: Failed to create .hoop dir
- **Line 42**: Failed to write config.yml
- **Line 47**: Failed to write projects.yaml
- **Line 61**: Failed to create project dir
- **Line 64**: Failed to create .beads dir
- **Line 68**: Failed to create issues.jsonl
- **Line 102**: Failed to write projects.yaml
- **Line 111**: Failed to run hoop status --json
- **Line 120**: Invalid UTF-8 in stdout
- **Line 124**: hoop status --json should produce valid JSON
- **Line 133**: projects should be an array
- **Line 176**: Failed to write projects.yaml
- **Line 192**: Failed to run hoop status --json
- **Line 206**: Failed to spawn jq
- **Line 209**: Failed to open jq stdin
- **Line 212**: Failed to write to jq stdin
- **Line 217**: Failed to read jq output
- **Line 236**: Failed to create root dir
- **Line 241**: Failed to move project
- **Line 260**: Failed to run hoop projects scan --yes
- **Line 262**: Invalid UTF-8 in stdout
- **Line 263**: Invalid UTF-8 in stderr
- **Line 313**: Failed to write projects.yaml
- **Line 322**: Failed to run hoop status
- **Line 353**: Failed to run hoop status
- **Line 361**: Invalid UTF-8 in stdout
- **Line 365**: Error output should still be valid JSON
- **Line 384**: Failed to create root dir
- **Line 387**: Failed to move project
- **Line 406**: Invalid UTF-8 in stdout
- **Line 446**: Failed to run hoop restore
- **Line 451**: Invalid UTF-8 in stderr
- **Line 486**: Failed to write projects.yaml
- **Line 495**: Failed to run hoop status --json
- **Line 497**: Invalid UTF-8 in stdout
- **Line 501**: Output should be valid JSON
- **Line 540**: Failed to run hoop status
- **Line 542**: Invalid UTF-8 in stdout
- **Line 543**: Invalid UTF-8 in stderr
- **Line 554**: Error output should be valid JSON
- **Line 584**: Failed to write projects.yaml
- **Line 594**: Failed to run hoop status without TTY
- **Line 602**: Invalid UTF-8 in stdout
- **Line 606**: Machine mode should produce valid JSON
- **Line 636**: Failed to write projects.yaml
- **Line 655**: Thread panicked

#### hoop-daemon/tests/adapter_failover.rs

- **Line 26**: create temp dir
- **Line 28**: create .hoop dir
- **Line 34**: init fleet.db
- **Line 73**: write config.yml
- **Line 156**: insert session
- **Line 174**: archive session as stitch
- **Line 177**: open db
- **Line 185**: query stitch
- **Line 201**: count messages
- **Line 212**: query linked stitch
- **Line 249**: insert session
- **Line 253**: archive session
- **Line 256**: open db
- **Line 264**: query archived session
- **Line 311**: insert session
- **Line 323**: open db
- **Line 331**: count active
- **Line 341**: get active adapter
- **Line 392**: insert entry 1
- **Line 393**: insert entry 2
- **Line 415**: insert session
- **Line 417**: archive session
- **Line 439**: insert new session
- **Line 443**: list approved entries
- **Line 484**: insert old session
- **Line 488**: archive old session
- **Line 510**: insert new session
- **Line 514**: list sessions
- **Line 530**: list sessions
- **Line 579**: insert session
- **Line 600**: archive as stitch
- **Line 603**: open db
- **Line 611**: query stitch metadata
- **Line 626**: prepare query
- **Line 628**: query messages
- **Line 676**: insert session
- **Line 694**: archive as stitch
- **Line 697**: open db
- **Line 701**: prepare query
- **Line 703**: query messages
- **Line 765**: insert entry 1
- **Line 766**: insert entry 2
- **Line 786**: insert rejected
- **Line 790**: list approved

#### hoop-daemon/tests/adapter_failover_integration.rs

- **Line 27**: create temp dir
- **Line 29**: create .hoop dir
- **Line 35**: init fleet.db
- **Line 140**: load active session
- **Line 141**: should have active session
- **Line 161**: archive session as stitch
- **Line 165**: archive agent session
- **Line 356**: archive session
- **Line 540**: list approved entries
- **Line 607**: load active session should succeed
- **Line 608**: should have an active session
- **Line 668**: list approved entries
- **Line 706**: load active session
- **Line 707**: should have active session
- **Line 711**: archive as stitch

#### hoop-daemon/tests/adapter_failover_test.rs

- **Line 155**: Failed to spawn daemon
- **Line 157**: Failed to create client
- **Line 160**: Health check failed
- **Line 164**: Failed to spawn agent
- **Line 172**: Failed to get agent status
- **Line 176**: Health check failed
- **Line 187**: Failed to spawn daemon
- **Line 189**: Failed to create client
- **Line 192**: Failed to spawn agent
- **Line 197**: Should have session_db_id
- **Line 203**: Failed to get agent status
- **Line 215**: Failed to switch adapter
- **Line 220**: Should have new session_db_id
- **Line 232**: Failed to list sessions
- **Line 252**: Failed to get agent status
- **Line 265**: Failed to spawn daemon
- **Line 267**: Failed to create client
- **Line 270**: Failed to spawn agent
- **Line 275**: Should have session_db_id
- **Line 281**: Failed to switch adapter
- **Line 287**: Failed to list sessions
- **Line 293**: Should find archived session
- **Line 311**: Failed to query stitch from fleet.db
- **Line 344**: Failed to spawn daemon
- **Line 346**: Failed to create client
- **Line 367**: Failed to insert reflection entry
- **Line 370**: Failed to spawn agent
- **Line 374**: Failed to switch adapter
- **Line 378**: Failed to list reflection entries
- **Line 389**: Entry should exist
- **Line 402**: Failed to spawn daemon
- **Line 404**: Failed to create client
- **Line 407**: Failed to spawn agent
- **Line 411**: Should have session_db_id
- **Line 417**: Failed to switch adapter
- **Line 423**: Failed to switch adapter back
- **Line 427**: Should have second session_db_id
- **Line 433**: Failed to list sessions
- **Line 450**: Should find first archived session
- **Line 454**: Should find second archived session
- **Line 480**: Failed to spawn daemon
- **Line 482**: Failed to create client
- **Line 485**: Failed to spawn agent
- **Line 506**: Failed to insert reflection entry
- **Line 512**: Failed to switch adapter
- **Line 518**: Failed to get agent status
- **Line 524**: Failed to list reflection entries
- **Line 539**: Failed to spawn daemon
- **Line 541**: Failed to create client
- **Line 544**: Failed to spawn agent
- **Line 567**: Switch 1 should complete
- **Line 570**: Switch 2 should complete
- **Line 579**: Health check failed
- **Line 597**: Failed to spawn daemon
- **Line 599**: Failed to create client
- **Line 602**: Failed to spawn agent
- **Line 607**: Should have session_db_id
- **Line 613**: Failed to get agent status
- **Line 639**: Failed to write updated config.yml
- **Line 650**: Failed to get agent status after config reload
- **Line 663**: Failed to list sessions
- **Line 683**: Should find original archived session
- **Line 700**: Failed to query stitch from fleet.db
- **Line 722**: Health check failed
- **Line 805**: Failed to start mock Anthropic server
- **Line 812**: Failed to spawn daemon
- **Line 814**: Failed to create client
- **Line 817**: Health check failed
- **Line 837**: Failed to write config with mock server URL
- **Line 853**: Health check failed
- **Line 865**: Ready endpoint request failed
- **Line 882**: Health check failed
- **Line 892**: Health check failed
- **Line 910**: Failed to start mock Anthropic server
- **Line 917**: Failed to spawn daemon
- **Line 919**: Failed to create client
- **Line 922**: Health check failed
- **Line 937**: Failed to write config
- **Line 943**: Health check failed
- **Line 950**: Adapter switch should succeed
- **Line 958**: Failed to get agent status
- **Line 963**: Health check failed

#### hoop-daemon/tests/agent_turn_audit_trail.rs

- **Line 25**: create temp dir
- **Line 27**: create .hoop dir
- **Line 33**: init fleet.db
- **Line 83**: insert draft
- **Line 87**: get draft
- **Line 88**: draft exists
- **Line 114**: create stitch with audit
- **Line 118**: open fleet.db
- **Line 136**: query stitch
- **Line 152**: count system messages
- **Line 162**: get system message content
- **Line 208**: write audit row
- **Line 212**: query audit rows
- **Line 217**: should find audit row for our stitch
- **Line 227**: args_json should be valid JSON
- **Line 292**: create stitch for reconstruction
- **Line 296**: open fleet.db
- **Line 311**: query stitch for reconstruction

#### hoop-daemon/tests/backup_config_deserialization.rs

- **Line 46**: YAML should parse
- **Line 49**: YAML→JSON conversion should succeed
- **Line 52**: BackupFileConfig should deserialize
- **Line 72**: YAML should parse
- **Line 75**: YAML→JSON conversion should succeed
- **Line 78**: BackupFileConfig should deserialize
- **Line 97**: Should deserialize from JSON directly

#### hoop-daemon/tests/backup_restore_cycle.rs

- **Line 638**: age-keygen should be installed for this test
- **Line 651**: age-keygen output should contain public key

#### hoop-daemon/tests/bead_created_by_hoop_broadcast.rs

- **Line 70**: Fleet notification should be received within 200ms
- **Line 71**: Fleet notification channel should not be closed
- **Line 110**: Should serialize
- **Line 111**: Should deserialize

#### hoop-daemon/tests/bead_real_line_deserialization.rs

- **Line 40**: Real br line must deserialize successfully
- **Line 62**: Minimal bead line (without created_by/dependencies) must deserialize
- **Line 194**: Bead line with extra unknown keys must deserialize
- **Line 216**: Bead line with null description must deserialize

#### hoop-daemon/tests/beads_deletion_http.rs

- **Line 111**: Failed to write projects.yaml
- **Line 114**: Failed to spawn daemon
- **Line 184**: project-a should be in degraded list
- **Line 313**: Failed to write projects.yaml
- **Line 316**: Failed to spawn daemon
- **Line 410**: Failed to spawn daemon

#### hoop-daemon/tests/beads_removal_recovery.rs

- **Line 26**: Failed to create temp dir
- **Line 31**: Failed to create .beads dir
- **Line 35**: Failed to create issues.jsonl
- **Line 39**: Failed to create events.jsonl
- **Line 47**: Failed to remove .beads dir
- **Line 53**: Failed to recreate .beads dir
- **Line 56**: Failed to recreate issues.jsonl
- **Line 59**: Failed to recreate events.jsonl
- **Line 104**: Failed to write projects.yaml
- **Line 107**: Failed to spawn test daemon
- **Line 118**: Failed to GET /api/projects
- **Line 124**: Failed to parse projects response
- **Line 147**: Failed to GET /readyz
- **Line 167**: Failed to GET /api/projects
- **Line 173**: Failed to parse projects response
- **Line 202**: Failed to GET /api/projects
- **Line 207**: Failed to parse projects response
- **Line 238**: Failed to GET /readyz
- **Line 249**: Failed to parse readiness response
- **Line 280**: Failed to POST /api/config/reload
- **Line 296**: Failed to GET /readyz
- **Line 315**: Failed to GET /readyz
- **Line 364**: Failed to write projects.yaml
- **Line 367**: Failed to spawn test daemon
- **Line 374**: Failed to GET /readyz
- **Line 394**: Failed to GET /readyz
- **Line 400**: Failed to parse readiness response
- **Line 420**: Failed to GET /api/projects
- **Line 425**: Failed to parse projects response

#### hoop-daemon/tests/config_reload_audit.rs

- **Line 48**: tempdir
- **Line 51**: init fleet db
- **Line 66**: tempdir for projects
- **Line 115**: write audit row
- **Line 124**: query
- **Line 133**: delta_keys should be array
- **Line 142**: hash chain should be valid
- **Line 154**: tempdir for projects
- **Line 189**: write audit row
- **Line 203**: query
- **Line 216**: hash chain should be valid
- **Line 224**: tempdir
- **Line 277**: tempdir for projects
- **Line 318**: write audit row
- **Line 322**: query
- **Line 351**: hash chain should be valid after round-trip

#### hoop-daemon/tests/config_reload_cycle.rs

- **Line 68**: tempdir
- **Line 71**: init fleet db
- **Line 90**: tempdir for projects
- **Line 102**: v1 should parse successfully
- **Line 135**: v2 should parse successfully
- **Line 167**: v3 should parse successfully
- **Line 199**: write rejected audit row
- **Line 221**: write success audit row
- **Line 230**: query rejected
- **Line 239**: query success
- **Line 247**: hash chain intact after full cycle
- **Line 336**: tempdir
- **Line 364**: YAML should parse fine
- **Line 411**: tempdir
- **Line 420**: valid config should load
- **Line 430**: YAML should still parse
- **Line 449**: fixed config should load
- **Line 490**: write rejected audit
- **Line 498**: query
- **Line 509**: hash chain intact

#### hoop-daemon/tests/create_only_stub.rs

- **Line 25**: create temp dir
- **Line 40**: create br script
- **Line 41**: write br script
- **Line 46**: chmod br script
- **Line 105**: run fake br
- **Line 307**: run fake br
- **Line 370**: run fake br

#### hoop-daemon/tests/create_stitch_no_auto_submit.rs

- **Line 143**: create temp dir for test project
- **Line 145**: create project dir
- **Line 148**: create .beads dir
- **Line 152**: create beads.db
- **Line 192**: create temp HOOP home
- **Line 194**: create .hoop dir
- **Line 207**: write projects.yaml
- **Line 217**: write config.yml
- **Line 227**: init fleet.db
- **Line 332**: create temp HOOP home
- **Line 334**: create .hoop dir
- **Line 337**: init fleet.db
- **Line 371**: insert draft
- **Line 375**: get draft
- **Line 376**: draft exists
- **Line 393**: update draft status
- **Line 397**: get approved draft
- **Line 398**: approved draft exists
- **Line 419**: create temp HOOP home
- **Line 421**: create .hoop dir
- **Line 424**: init fleet.db
- **Line 458**: insert first draft
- **Line 492**: insert second draft with force_create bypass
- **Line 496**: get first draft
- **Line 497**: first draft exists
- **Line 500**: get second draft
- **Line 501**: second draft exists
- **Line 525**: create temp HOOP home
- **Line 527**: create .hoop dir
- **Line 530**: init fleet.db
- **Line 564**: insert draft
- **Line 568**: get draft
- **Line 569**: draft exists
- **Line 599**: create temp HOOP home
- **Line 601**: create .hoop dir
- **Line 604**: init fleet.db

#### hoop-daemon/tests/cross_workspace_blockers.rs

- **Line 26**: Failed to create temp dir
- **Line 30**: Failed to open fleet.db
- **Line 42**: Failed to insert parent stitch
- **Line 50**: Failed to insert parent bead
- **Line 59**: Failed to insert child stitch B
- **Line 67**: Failed to insert child bead B
- **Line 76**: Failed to insert child stitch C
- **Line 84**: Failed to insert child bead C
- **Line 91**: Failed to insert link to child B
- **Line 97**: Failed to insert link to child C
- **Line 105**: Failed to prepare stitch_links query
- **Line 114**: Failed to query child stitches
- **Line 124**: Should find child stitch B
- **Line 130**: Should find child stitch C
- **Line 138**: Failed to prepare stitch_beads query
- **Line 142**: Failed to query child beads
- **Line 157**: Should find child bead B
- **Line 163**: Should find child bead C
- **Line 174**: Failed to create temp dir
- **Line 178**: Failed to open fleet.db
- **Line 189**: Failed to query workspace_from column
- **Line 200**: Failed to query workspace_to column
- **Line 209**: Failed to insert stitch link with workspaces
- **Line 217**: Failed to query workspace columns
- **Line 243**: Failed to create stitches table
- **Line 256**: Failed to create stitch_beads table
- **Line 271**: Failed to create stitch_links table
- **Line 277**: Failed to create idx_stitch_links_from
- **Line 282**: Failed to create idx_stitch_links_to
- **Line 287**: Failed to create idx_stitch_beads_project

#### hoop-daemon/tests/draft_queue_invariants.rs

- **Line 27**: create temp dir
- **Line 29**: create .hoop dir
- **Line 35**: init fleet.db
- **Line 82**: insert draft
- **Line 85**: get draft
- **Line 86**: draft exists
- **Line 129**: insert draft
- **Line 132**: get draft
- **Line 133**: draft exists
- **Line 206**: insert draft1
- **Line 207**: insert draft2
- **Line 212**: get draft1
- **Line 213**: draft1 exists
- **Line 218**: get draft2
- **Line 219**: draft2 exists
- **Line 266**: insert draft
- **Line 270**: list pending
- **Line 275**: list rejected
- **Line 281**: list pending
- **Line 282**: list edited
- **Line 326**: insert draft
- **Line 386**: insert draft
- **Line 399**: update draft status
- **Line 419**: write audit row
- **Line 429**: get draft
- **Line 430**: draft exists
- **Line 474**: insert draft
- **Line 488**: reject draft
- **Line 491**: get draft
- **Line 492**: draft exists
- **Line 529**: insert draft
- **Line 542**: reject draft
- **Line 545**: get draft
- **Line 546**: draft exists
- **Line 581**: write audit row
- **Line 631**: insert draft
- **Line 641**: edit draft
- **Line 644**: get draft
- **Line 645**: draft exists
- **Line 692**: insert draft
- **Line 706**: approve and submit draft
- **Line 709**: get draft
- **Line 710**: draft exists
- **Line 745**: write audit row
- **Line 756**: hash chain must be valid after draft actions
- **Line 774**: open_draft should succeed
- **Line 777**: get draft should succeed
- **Line 778**: draft should exist
- **Line 799**: first open should succeed
- **Line 802**: get draft should succeed
- **Line 803**: draft should exist
- **Line 808**: abandon should succeed
- **Line 811**: get draft should succeed
- **Line 812**: draft should exist
- **Line 818**: second open should succeed
- **Line 821**: get draft should succeed
- **Line 822**: draft should exist
- **Line 840**: open should succeed
- **Line 851**: autosave should succeed
- **Line 854**: get draft should succeed
- **Line 855**: draft should exist
- **Line 875**: second autosave should succeed
- **Line 878**: get draft should succeed
- **Line 879**: draft should exist
- **Line 895**: open should succeed
- **Line 898**: get draft should succeed
- **Line 899**: draft should exist
- **Line 906**: abandon should succeed
- **Line 909**: get draft should succeed
- **Line 910**: draft should exist
- **Line 953**: insert draft
- **Line 1002**: insert old draft
- **Line 1035**: insert recent draft
- **Line 1039**: cleanup should succeed
- **Line 1045**: get draft should succeed
- **Line 1051**: get draft should succeed
- **Line 1052**: recent abandoned draft should still exist
- **Line 1069**: open should succeed
- **Line 1072**: get draft should succeed
- **Line 1073**: draft should exist
- **Line 1087**: autosave should succeed
- **Line 1090**: get draft should succeed
- **Line 1091**: draft should exist
- **Line 1105**: second autosave should succeed
- **Line 1109**: abandon should succeed
- **Line 1112**: get draft should succeed
- **Line 1113**: draft should exist
- **Line 1120**: get draft should succeed
- **Line 1121**: abandoned draft should still exist

#### hoop-daemon/tests/epoch_sync_invariant.rs

- **Line 26**: Failed to spawn test daemon
- **Line 33**: Failed to connect to WebSocket
- **Line 40**: Timeout waiting for init message
- **Line 41**: WebSocket stream ended
- **Line 43**: Failed to receive init message
- **Line 47**: Failed to parse init event as JSON
- **Line 77**: Failed to spawn test daemon
- **Line 84**: Failed to connect to WebSocket
- **Line 105**: Failed to parse first message
- **Line 147**: Failed to spawn test daemon
- **Line 155**: Failed to connect to WebSocket
- **Line 188**: Failed to reconnect to WebSocket
- **Line 244**: Failed to spawn test daemon
- **Line 253**: Failed to connect to WebSocket (iteration {})
- **Line 263**: WebSocket stream ended
- **Line 272**: Failed to parse message as JSON
- **Line 292**: Failed to spawn test daemon
- **Line 305**: Failed to connect
- **Line 313**: Stream ended
- **Line 319**: Failed to parse
- **Line 332**: Task failed

#### hoop-daemon/tests/filesystem_failure_isolation.rs

- **Line 28**: Failed to create .beads dir
- **Line 30**: Failed to create issues.jsonl
- **Line 39**: Failed to create temp dir
- **Line 41**: Failed to create .hoop dir
- **Line 71**: Failed to write projects.yaml
- **Line 80**: Failed to write config.yml
- **Line 83**: Failed to create data dir
- **Line 125**: Failed to bind to random port
- **Line 126**: Failed to get local address
- **Line 173**: Failed to get readyz status
- **Line 184**: Failed to remove .beads from project A
- **Line 188**: Failed to get projects.yaml metadata
- **Line 189**: Failed to get modified time
- **Line 220**: project-a should be in degraded list
- **Line 275**: Failed to bind to random port
- **Line 276**: Failed to get local address
- **Line 323**: Failed to get readyz status
- **Line 334**: Failed to remove .beads from project A
- **Line 371**: Failed to read projects.yaml
- **Line 372**: Failed to write projects.yaml
- **Line 430**: Failed to bind to random port
- **Line 431**: Failed to get local address
- **Line 479**: Failed to get readyz status
- **Line 488**: Failed to connect to WebSocket
- **Line 493**: Failed to remove .beads from project A

#### hoop-daemon/tests/fix_patterns_integration.rs

- **Line 61**: pattern should exist
- **Line 85**: pattern should exist after update
- **Line 95**: pattern should exist

#### hoop-daemon/tests/fleet_notifications_integration.rs

- **Line 59**: Should serialize to JSON
- **Line 71**: Should deserialize from JSON

#### hoop-daemon/tests/golden_transcripts_regression.rs

- **Line 39**: workspace root is parent of hoop-daemon/

#### hoop-daemon/tests/hoop_dies_nothing_notices.rs

- **Line 30**: workspace root is parent of hoop-daemon/
- **Line 43**: create temp dir for test HOOP home
- **Line 45**: create .hoop dir
- **Line 61**: write projects.yaml
- **Line 71**: write config.yml
- **Line 74**: create data dir
- **Line 189**: testrepo should exist
- **Line 197**: init fleet.db
- **Line 205**: write claim event
- **Line 208**: write dispatch event
- **Line 219**: read events.jsonl
- **Line 243**: write complete event during HOOP absence
- **Line 246**: write claim event during HOOP absence
- **Line 263**: read events.jsonl after restart
- **Line 291**: testrepo should exist
- **Line 299**: init fleet.db
- **Line 310**: write claim before HOOP
- **Line 313**: write dispatch before HOOP
- **Line 332**: write claim during HOOP absence
- **Line 335**: write complete during HOOP absence
- **Line 362**: read events.jsonl after restart
- **Line 393**: testrepo should exist
- **Line 400**: init fleet.db
- **Line 412**: write claim event
- **Line 417**: write dispatch event
- **Line 424**: write complete event
- **Line 435**: read events.jsonl for rebuild
- **Line 475**: testrepo should exist
- **Line 482**: init fleet.db
- **Line 489**: write claim
- **Line 490**: write dispatch
- **Line 491**: write complete
- **Line 499**: write claim
- **Line 500**: write dispatch
- **Line 512**: read events after third run
- **Line 531**: testrepo should exist
- **Line 539**: init fleet.db
- **Line 573**: insert draft before restart
- **Line 577**: get draft before restart
- **Line 578**: draft should exist before restart
- **Line 596**: re-init fleet.db after restart
- **Line 600**: get draft after restart
- **Line 601**: draft should exist after restart
- **Line 622**: testrepo should exist
- **Line 629**: init fleet.db
- **Line 635**: write valid claim
- **Line 636**: write valid dispatch
- **Line 644**: open events.jsonl for corruption
- **Line 646**: write corrupted line
- **Line 650**: write valid claim after corruption
- **Line 651**: write valid complete after corruption
- **Line 655**: read events with corruption
- **Line 683**: testrepo should exist
- **Line 690**: init fleet.db
- **Line 696**: empty events.jsonl
- **Line 698**: create empty events.jsonl
- **Line 703**: read empty events.jsonl

#### hoop-daemon/tests/integration_harness.rs

- **Line 33**: workspace root is parent of hoop-daemon/
- **Line 61**: Failed to create temp dir for test HOOP home
- **Line 63**: Failed to create .hoop dir
- **Line 79**: Failed to write projects.yaml
- **Line 88**: Failed to write config.yml
- **Line 91**: Failed to create data dir
- **Line 125**: Failed to read events.jsonl
- **Line 141**: Failed to read heartbeats.jsonl
- **Line 385**: testrepo fixtures should be valid
- **Line 391**: events should parse correctly
- **Line 397**: heartbeats should parse correctly
- **Line 403**: bead event data should extract
- **Line 409**: bead projections should be correct
- **Line 415**: HOOP home setup should work
- **Line 421**: Failed to parse events
- **Line 456**: Failed to parse heartbeats
- **Line 483**: Failed to parse events
- **Line 558**: Failed to parse events
- **Line 710**: Failed to spawn test daemon
- **Line 719**: Failed to connect to healthz
- **Line 723**: Failed to parse healthz response
- **Line 732**: Failed to connect to readyz
- **Line 741**: Failed to spawn test daemon
- **Line 750**: Failed to GET /api/beads
- **Line 754**: Failed to parse beads response
- **Line 764**: Failed to GET /api/projects
- **Line 771**: Failed to parse projects response
- **Line 784**: Failed to spawn test daemon
- **Line 793**: Failed to connect to WebSocket
- **Line 800**: Timeout waiting for init message
- **Line 801**: WebSocket stream ended
- **Line 803**: Failed to receive init message
- **Line 807**: Failed to parse init event as JSON
- **Line 821**: Timeout waiting for workers_snapshot message
- **Line 822**: WebSocket stream ended
- **Line 824**: Failed to receive workers_snapshot
- **Line 828**: Failed to parse workers_snapshot event as JSON
- **Line 839**: Timeout waiting for beads_snapshot message
- **Line 840**: WebSocket stream ended
- **Line 842**: Failed to receive beads_snapshot
- **Line 846**: Failed to parse beads_snapshot event as JSON
- **Line 865**: Failed to send subscribe message
- **Line 871**: Failed to send close frame
- **Line 883**: Failed to spawn test daemon
- **Line 892**: Failed to connect to healthz
- **Line 901**: Failed to GET /api/beads
- **Line 910**: Failed to GET /api/projects
- **Line 933**: Failed to spawn test daemon
- **Line 942**: Failed to GET /api/projects
- **Line 949**: Failed to parse projects response
- **Line 969**: Failed to spawn test daemon
- **Line 978**: Failed to GET /api/beads
- **Line 982**: Failed to parse beads response
- **Line 996**: Failed to spawn test daemon
- **Line 1005**: Failed to GET /api/metrics
- **Line 1009**: Failed to read metrics response
- **Line 1026**: Failed to spawn test daemon
- **Line 1033**: Failed to connect to WebSocket
- **Line 1081**: Failed to spawn test daemon
- **Line 1088**: Failed to connect to WebSocket
- **Line 1095**: Timeout waiting for init
- **Line 1108**: Failed to send subscribe message
- **Line 1151**: Failed to spawn test daemon
- **Line 1160**: Failed to connect to healthz
- **Line 1183**: Failed to spawn test daemon
- **Line 1190**: Failed to connect to WebSocket
- **Line 1197**: Timeout waiting for init
- **Line 1205**: Failed to send malformed message
- **Line 1217**: Failed to send unknown event type
- **Line 1225**: Failed to send empty message
- **Line 1232**: Health check failed
- **Line 1248**: Failed to spawn test daemon
- **Line 1275**: Task failed
- **Line 1291**: Failed to spawn first daemon
- **Line 1305**: Failed to create bead
- **Line 1309**: Failed to parse bead
- **Line 1310**: Bead should have an ID
- **Line 1319**: Failed to spawn second daemon
- **Line 1326**: Failed to fetch beads
- **Line 1336**: Failed to spawn test daemon
- **Line 1376**: Task failed
- **Line 1390**: Failed to spawn test daemon
- **Line 1399**: Request failed
- **Line 1408**: Request failed
- **Line 1419**: Request failed
- **Line 1429**: Failed to spawn test daemon
- **Line 1437**: Failed to fetch metrics
- **Line 1441**: Failed to read metrics
- **Line 1464**: Failed to spawn test daemon
- **Line 1474**: Failed to list files
- **Line 1478**: Failed to parse files
- **Line 1489**: Failed to spawn test daemon
- **Line 1504**: Failed to create bead
- **Line 1508**: Failed to parse bead
- **Line 1509**: Bead should have an ID
- **Line 1516**: Failed to get bead
- **Line 1520**: Failed to parse fetched bead
- **Line 1529**: Failed to list beads
- **Line 1533**: Failed to parse beads list
- **Line 1543**: Failed to spawn test daemon
- **Line 1551**: Failed to fetch capacity
- **Line 1555**: Failed to parse capacity
- **Line 1566**: Failed to spawn test daemon
- **Line 1574**: Failed to fetch config status
- **Line 1578**: Failed to parse config status
- **Line 1592**: Failed to spawn test daemon
- **Line 1601**: Failed to GET /api/beads
- **Line 1609**: Failed to GET /api/projects

#### hoop-daemon/tests/load_test.rs

- **Line 209**: Failed to spawn test daemon
- **Line 214**: Load test should complete
- **Line 262**: Failed to spawn test daemon
- **Line 271**: Load test timed out after 10 minutes
- **Line 272**: Load test should complete
- **Line 282**: Performance budgets must be satisfied
- **Line 329**: Failed to spawn test daemon
- **Line 337**: Medium-scale load test timed out
- **Line 338**: Load test should complete
- **Line 345**: Medium-scale load test should pass performance budgets

#### hoop-daemon/tests/load_test_integration.rs

- **Line 72**: Failed to spawn daemon with load test data
- **Line 81**: Health check request failed
- **Line 100**: Failed to spawn daemon
- **Line 167**: Failed to spawn daemon
- **Line 219**: Failed to spawn daemon
- **Line 286**: Failed to spawn daemon
- **Line 345**: Failed to spawn daemon
- **Line 350**: Load test failed
- **Line 355**: Performance budget violations detected
- **Line 380**: Failed to populate testrepo with load test data
- **Line 396**: Failed to create project directory
- **Line 410**: Failed to serialize projects.yaml
- **Line 412**: Failed to write projects.yaml
- **Line 467**: Failed to spawn daemon with load test data
- **Line 472**: Failed to write daemon URL to file
- **Line 480**: Load test failed
- **Line 490**: Performance budget violations detected - blocking merge per hoop-ttb.7.11
- **Line 528**: Failed to spawn daemon
- **Line 557**: Failed to spawn daemon

#### hoop-daemon/tests/multi_operator_concurrency.rs

- **Line 26**: create temp dir
- **Line 28**: create .hoop dir
- **Line 34**: init fleet.db
- **Line 114**: insert draft_a
- **Line 115**: insert draft_b
- **Line 119**: get draft_a
- **Line 120**: draft_a exists
- **Line 124**: get draft_b
- **Line 125**: draft_b exists
- **Line 166**: insert draft
- **Line 176**: autosave draft
- **Line 179**: get draft
- **Line 180**: draft exists
- **Line 226**: insert draft
- **Line 229**: abandon draft
- **Line 232**: get draft
- **Line 233**: draft exists
- **Line 276**: insert existing draft
- **Line 284**: detect similar drafts
- **Line 307**: propose from operator A
- **Line 315**: propose from operator B
- **Line 322**: list proposals
- **Line 329**: parse source_stitches
- **Line 364**: insert proposal
- **Line 370**: approve proposal
- **Line 376**: get proposal
- **Line 377**: proposal exists
- **Line 383**: list approved entries
- **Line 416**: insert proposal
- **Line 420**: reject proposal
- **Line 426**: get proposal
- **Line 451**: update presence
- **Line 457**: query presence
- **Line 477**: update presence hidden
- **Line 483**: query presence
- **Line 498**: _HOOP_FLEET_DB_PATH not set
- **Line 500**: open db
- **Line 512**: insert stale presence
- **Line 518**: query presence
- **Line 535**: update presence
- **Line 541**: query presence
- **Line 549**: remove presence
- **Line 555**: query presence
- **Line 591**: insert session A
- **Line 613**: insert session B
- **Line 617**: list agent sessions
- **Line 665**: insert draft
- **Line 668**: get draft
- **Line 669**: draft exists
- **Line 702**: create stitch A
- **Line 713**: create stitch B
- **Line 717**: load stitch A
- **Line 718**: stitch A exists
- **Line 721**: load stitch B
- **Line 722**: stitch B exists

#### hoop-daemon/tests/needle_events_roundtrip.rs

- **Line 25**: workspace root is parent of hoop-daemon/
- **Line 64**: testrepo/.beads/events.jsonl must be readable
- **Line 85**: testrepo/.beads/heartbeats.jsonl must be readable
- **Line 106**: fixture must have a claim event
- **Line 132**: fixture must have a dispatch event
- **Line 163**: fixture must have a complete event
- **Line 202**: fixture must have a fail event
- **Line 230**: fixture must have a release event
- **Line 250**: fixture must have a timeout event
- **Line 270**: fixture must have a crash event
- **Line 451**: fixture must have an executing heartbeat
- **Line 454**: executing heartbeat must parse successfully
- **Line 476**: fixture must have an idle heartbeat
- **Line 479**: idle heartbeat must parse successfully
- **Line 491**: fixture must have a knot heartbeat
- **Line 494**: knot heartbeat must parse successfully

#### hoop-daemon/tests/path_traversal_hardening.rs

- **Line 147**: allowlist construction must succeed

#### hoop-daemon/tests/performance_budget.rs

- **Line 64**: Failed to populate testrepo with load test data
- **Line 81**: Failed to create project directory
- **Line 98**: Failed to serialize projects.yaml
- **Line 100**: Failed to write projects.yaml
- **Line 111**: Failed to spawn daemon
- **Line 125**: healthz request failed
- **Line 141**: readyz request failed
- **Line 157**: projects request failed
- **Line 181**: metrics request failed
- **Line 248**: Failed to populate testrepo
- **Line 261**: Failed to create project directory
- **Line 279**: Failed to spawn daemon
- **Line 288**: readyz request failed

#### hoop-daemon/tests/phase2_exit_gate.rs

- **Line 438**: Report must serialize to JSON

#### hoop-daemon/tests/projection_file_audit.rs

- **Line 195**: CARGO_MANIFEST_DIR not set
- **Line 198**: workspace root is the parent of hoop-daemon/
- **Line 216**: valid regex
- **Line 436**: valid regex

#### hoop-daemon/tests/protocol_contract.rs

- **Line 24**: workspace root
- **Line 47**: CreateDraftRequest must deserialize from fixture (daemon side)

#### hoop-daemon/tests/pure_functions.rs

- **Line 238**: sanitize should not fail
- **Line 247**: sanitize should not fail
- **Line 260**: sanitize should not fail
- **Line 269**: sanitize should not fail

#### hoop-daemon/tests/s1_morning_review.rs

- **Line 29**: Failed to spawn daemon
- **Line 38**: Failed to fetch dashboard
- **Line 49**: Failed to parse dashboard response
- **Line 58**: total_workers must be a number
- **Line 72**: total_spend_usd must be a number
- **Line 86**: longest_running must be an array
- **Line 94**: Failed to fetch worker timeline
- **Line 102**: Failed to parse timeline
- **Line 117**: Failed to spawn daemon
- **Line 127**: Failed to fetch dashboard
- **Line 150**: Failed to spawn daemon
- **Line 159**: Failed to fetch dashboard
- **Line 167**: Failed to parse response
- **Line 188**: Failed to spawn daemon
- **Line 197**: Failed to fetch dashboard
- **Line 199**: Failed to parse response
- **Line 209**: Failed to fetch dashboard
- **Line 211**: Failed to parse response
- **Line 229**: Failed to spawn daemon
- **Line 237**: Failed to fetch dashboard
- **Line 239**: Failed to parse response
- **Line 244**: total_spend_usd must be present
- **Line 254**: spend_by_project must be an array
- **Line 279**: Failed to spawn daemon
- **Line 287**: Failed to fetch dashboard
- **Line 289**: Failed to parse response
- **Line 293**: total_workers must be present
- **Line 297**: workers_by_project must be an array

#### hoop-daemon/tests/s2_transcript_archaeology.rs

- **Line 32**: Failed to spawn daemon
- **Line 41**: Failed to fetch beads
- **Line 49**: Failed to parse beads
- **Line 56**: Bead should have an id
- **Line 63**: Failed to fetch bead events
- **Line 73**: Failed to parse events
- **Line 90**: Failed to spawn daemon
- **Line 99**: Failed to fetch beads
- **Line 101**: Failed to parse beads
- **Line 107**: Bead should have an id
- **Line 116**: Failed to fetch bead events
- **Line 139**: Failed to spawn daemon
- **Line 148**: Failed to connect to stitch endpoint
- **Line 167**: Failed to spawn daemon
- **Line 184**: Failed to connect to endpoint
- **Line 203**: Failed to spawn daemon
- **Line 212**: Failed to fetch conversations
- **Line 220**: Failed to parse conversations
- **Line 238**: Failed to spawn daemon
- **Line 247**: Failed to fetch beads
- **Line 249**: Failed to parse beads
- **Line 272**: Failed to spawn daemon
- **Line 281**: Failed to fetch cost trends
- **Line 289**: Failed to parse cost data
- **Line 308**: Failed to spawn daemon
- **Line 317**: Failed to fetch beads
- **Line 319**: Failed to parse beads
- **Line 325**: Bead should have an id
- **Line 332**: Failed to fetch bead events
- **Line 335**: Failed to parse events

#### hoop-daemon/tests/s3_bead_creation_from_chat.rs

- **Line 41**: create temp dir
- **Line 56**: create br script
- **Line 57**: write br script
- **Line 62**: chmod br script
- **Line 107**: Failed to spawn daemon
- **Line 133**: Failed to create draft
- **Line 145**: Failed to parse draft response
- **Line 149**: draft_id should be present
- **Line 162**: Failed to list drafts
- **Line 169**: Failed to parse list response
- **Line 173**: drafts should be an array
- **Line 186**: Failed to get draft
- **Line 193**: Failed to parse draft
- **Line 217**: Failed to spawn daemon
- **Line 235**: Failed to create draft
- **Line 240**: Failed to parse draft response
- **Line 244**: draft_id should be present
- **Line 254**: Failed to approve draft
- **Line 266**: Failed to parse approve response
- **Line 270**: stitch_id should be present
- **Line 292**: Failed to get draft
- **Line 297**: Failed to parse draft
- **Line 318**: Failed to spawn daemon
- **Line 336**: Failed to create draft
- **Line 341**: Failed to parse draft response
- **Line 345**: draft_id should be present
- **Line 353**: Failed to approve draft
- **Line 358**: Failed to parse approve response
- **Line 362**: stitch_id should be present
- **Line 369**: Failed to query audit log
- **Line 376**: Failed to parse audit response
- **Line 380**: audit_rows should be an array
- **Line 392**: args should be an object
- **Line 404**: args should be an object
- **Line 412**: actor should be present
- **Line 434**: Failed to spawn daemon
- **Line 459**: Failed to create draft
- **Line 464**: Failed to parse response
- **Line 465**: draft_id present
- **Line 472**: Failed to list drafts
- **Line 474**: Failed to parse list
- **Line 475**: drafts array
- **Line 488**: Failed to approve draft
- **Line 493**: Failed to parse approve
- **Line 494**: stitch_id present
- **Line 509**: Failed to query audit
- **Line 511**: Failed to parse audit
- **Line 512**: audit_rows array
- **Line 526**: args object
- **Line 530**: args object
- **Line 534**: actor present
- **Line 556**: Failed to spawn daemon
- **Line 577**: Failed to create draft
- **Line 581**: Failed to parse
- **Line 582**: draft_id present
- **Line 589**: Failed to get draft
- **Line 593**: Failed to parse draft

#### hoop-daemon/tests/s4_daemon_restart.rs

- **Line 33**: workspace root is parent of hoop-daemon/
- **Line 107**: create temp dir for test HOOP home
- **Line 109**: create .hoop dir
- **Line 124**: write projects.yaml
- **Line 133**: write config.yml
- **Line 135**: create data dir
- **Line 155**: init fleet.db
- **Line 159**: write claim
- **Line 160**: write complete
- **Line 161**: write claim
- **Line 170**: Failed to spawn first daemon
- **Line 195**: Failed to fetch beads from first daemon
- **Line 203**: Failed to parse beads
- **Line 212**: write complete
- **Line 213**: write claim
- **Line 226**: Failed to spawn second daemon
- **Line 249**: Failed to fetch beads from second daemon
- **Line 257**: Failed to parse beads
- **Line 290**: init fleet.db
- **Line 296**: write claim
- **Line 298**: write complete
- **Line 305**: Failed to spawn first daemon
- **Line 332**: Failed to spawn second daemon
- **Line 364**: Failed to fetch beads
- **Line 387**: init fleet.db
- **Line 392**: Failed to spawn first daemon
- **Line 418**: write claim
- **Line 419**: write complete
- **Line 420**: write claim
- **Line 432**: Failed to spawn second daemon
- **Line 451**: write complete
- **Line 452**: write claim
- **Line 466**: Failed to fetch beads
- **Line 488**: init fleet.db
- **Line 495**: write claim
- **Line 496**: write complete
- **Line 502**: Failed to spawn daemon
- **Line 525**: Failed to fetch beads
- **Line 529**: Failed to parse beads
- **Line 552**: write claim

#### hoop-daemon/tests/s5_workspace_deleted.rs

- **Line 30**: Failed to create .beads dir
- **Line 32**: Failed to create issues.jsonl
- **Line 40**: Failed to create temp dir
- **Line 42**: Failed to create .hoop dir
- **Line 71**: Failed to write projects.yaml
- **Line 79**: Failed to write config.yml
- **Line 80**: Failed to create data dir
- **Line 120**: Failed to bind to random port
- **Line 121**: Failed to get local address
- **Line 166**: Failed to get readyz status
- **Line 173**: Failed to remove .beads from project A
- **Line 222**: Failed to bind to random port
- **Line 223**: Failed to get local address
- **Line 265**: Failed to remove .beads from project A
- **Line 275**: Failed to fetch projects
- **Line 279**: Failed to parse projects
- **Line 292**: Failed to check health
- **Line 323**: Failed to bind to random port
- **Line 324**: Failed to get local address
- **Line 367**: Failed to get readyz status
- **Line 372**: Failed to remove .beads from project A
- **Line 379**: Failed to get readyz status after deletion
- **Line 427**: Failed to bind to random port
- **Line 428**: Failed to get local address
- **Line 470**: Failed to remove .beads
- **Line 479**: Failed to check health

#### hoop-daemon/tests/secrets_scanner_parity.rs

- **Line 238**: Pattern should serialize to JSON
- **Line 239**: Serialized pattern should deserialize

#### hoop-daemon/tests/session_redaction.rs

- **Line 216**: valid JSON

#### hoop-daemon/tests/skills_integration.rs

- **Line 18**: Failed to create temp dir
- **Line 21**: Failed to create skill dir
- **Line 44**: Failed to write manifest
- **Line 56**: Failed to create temp dir
- **Line 59**: Failed to create skill dir
- **Line 77**: Failed to write manifest
- **Line 86**: Failed to write run script
- **Line 90**: Failed to get metadata
- **Line 94**: Failed to set permissions
- **Line 117**: Failed to create temp dir
- **Line 120**: Failed to create skill dir
- **Line 140**: Failed to write manifest
- **Line 162**: Failed to create temp dir
- **Line 165**: Failed to create skill dir
- **Line 187**: Failed to write manifest
- **Line 211**: Failed to create temp dir
- **Line 214**: Failed to create skill dir
- **Line 233**: Failed to write manifest
- **Line 252**: Failed to create temp dir
- **Line 255**: Failed to create skill dir
- **Line 275**: Failed to write manifest
- **Line 350**: Failed to create temp dir
- **Line 353**: Failed to create skill dir
- **Line 369**: Failed to write manifest
- **Line 379**: Failed to create temp dir
- **Line 382**: Failed to create skill dir
- **Line 396**: Failed to write manifest
- **Line 406**: Failed to create temp dir
- **Line 409**: Failed to create skill dir
- **Line 422**: Failed to write manifest

#### hoop-daemon/tests/skills_quarantine_integration.rs

- **Line 56**: Failed to create temp dir
- **Line 83**: Failed to create temp dir
- **Line 111**: Failed to create temp dir
- **Line 138**: Failed to create temp dir
- **Line 162**: Failed to create temp dir
- **Line 206**: Failed to create temp dir
- **Line 234**: Failed to create temp dir
- **Line 264**: Failed to create temp dir
- **Line 289**: Failed to create temp dir
- **Line 307**: Failed to create temp dir
- **Line 326**: Failed to create temp dir

#### hoop-daemon/tests/state_projections.rs

- **Line 153**: Failed to spawn daemon
- **Line 161**: Health check request failed
- **Line 171**: Failed to spawn daemon
- **Line 178**: Failed to connect to WebSocket
- **Line 184**: Timeout waiting for first message
- **Line 185**: WebSocket stream ended
- **Line 187**: Failed to receive first message
- **Line 191**: Failed to parse init event
- **Line 220**: Failed to spawn daemon
- **Line 224**: Failed to collect snapshots
- **Line 238**: Failed to spawn daemon
- **Line 243**: Failed to collect WS snapshots
- **Line 253**: REST workers request failed
- **Line 256**: Failed to parse REST workers response
- **Line 263**: REST beads request failed
- **Line 266**: Failed to parse REST beads response
- **Line 273**: REST projects request failed
- **Line 276**: Failed to parse REST projects response
- **Line 298**: Failed to spawn daemon
- **Line 305**: Failed to connect
- **Line 312**: Timeout waiting for init
- **Line 313**: Stream ended
- **Line 314**: Failed to receive init
- **Line 329**: Failed to send subscribe
- **Line 339**: Failed to send unsubscribe
- **Line 352**: Failed to spawn daemon
- **Line 360**: Config status request failed
- **Line 363**: Failed to parse config status
- **Line 376**: Failed to spawn daemon
- **Line 384**: Beads request failed
- **Line 387**: Failed to parse beads response
- **Line 405**: Failed to spawn daemon
- **Line 413**: Workers request failed
- **Line 416**: Failed to parse workers response
- **Line 427**: Failed to spawn daemon
- **Line 435**: Projects request failed
- **Line 438**: Failed to parse projects response
- **Line 449**: Failed to spawn daemon
- **Line 469**: Stream ended
- **Line 475**: Failed to parse
- **Line 488**: Task failed
- **Line 497**: Failed to spawn daemon
- **Line 505**: Failed to connect first time
- **Line 535**: Failed to reconnect
- **Line 590**: Failed to spawn daemon
- **Line 599**: Failed to connect
- **Line 679**: Failed to spawn first daemon
- **Line 683**: Failed to spawn second daemon
- **Line 698**: First daemon health check failed
- **Line 704**: Second daemon health check failed

#### hoop-daemon/tests/stdout_generation_test.rs

- **Line 150**: Failed to execute subprocess
- **Line 183**: Failed to execute test binary
- **Line 266**: Failed to execute multi-line subprocess

#### hoop-daemon/tests/stitch_percentile_index_integration.rs

- **Line 22**: Failed to open test DB
- **Line 41**: Failed to create stitches table
- **Line 57**: Failed to create stitch_messages table
- **Line 72**: Failed to create actions table
- **Line 76**: Failed to initialize percentile index
- **Line 113**: Failed to insert stitch
- **Line 131**: Failed to insert message
- **Line 150**: Failed to insert action
- **Line 156**: Failed to create temp dir
- **Line 166**: Failed to check table existence
- **Line 177**: Failed to check metadata table existence
- **Line 188**: Failed to get schema version
- **Line 198**: Failed to create temp dir
- **Line 204**: Failed to check schema version
- **Line 208**: Failed to check rebuild needed
- **Line 220**: Failed to corrupt schema version
- **Line 225**: Failed to check schema version
- **Line 229**: Failed to check rebuild needed
- **Line 324**: Failed to create temp dir
- **Line 360**: Failed to rebuild index
- **Line 369**: Failed to count index entries
- **Line 380**: Failed to query bucket
- **Line 390**: Failed to create temp dir
- **Line 421**: Failed to rebuild index
- **Line 430**: Failed to count buckets
- **Line 446**: Query should succeed
- **Line 467**: Failed to create temp dir
- **Line 482**: Failed to rebuild index
- **Line 492**: Query should succeed
- **Line 506**: Failed to create temp dir
- **Line 514**: Failed to rebuild index
- **Line 523**: Query should succeed
- **Line 557**: Failed to create temp dir
- **Line 572**: Failed to rebuild index
- **Line 582**: Query should succeed
- **Line 598**: Query should succeed
- **Line 609**: Failed to create temp dir
- **Line 617**: Failed to rebuild index
- **Line 625**: Failed to count
- **Line 642**: Failed to rebuild index
- **Line 650**: Failed to count
- **Line 658**: Failed to create temp dir
- **Line 678**: Failed to rebuild index
- **Line 687**: Failed to query bucket

#### hoop-daemon/tests/supervisor_health.rs

- **Line 57**: Failed to create CostAggregator
- **Line 141**: Reconcile should succeed
- **Line 178**: Reconcile should succeed
- **Line 216**: Reconcile should succeed
- **Line 257**: Reconcile should succeed
- **Line 284**: Reconcile should succeed
- **Line 429**: Reconcile should succeed
- **Line 467**: Reconcile should succeed
- **Line 508**: Reconcile should succeed
- **Line 579**: Reconcile should succeed

#### hoop-daemon/tests/supervisor_hotreload.rs

- **Line 115**: Empty reconcile should succeed
- **Line 129**: Reconcile with new project should succeed
- **Line 166**: Reconcile with multiple projects should succeed
- **Line 202**: Reconcile with two projects should succeed
- **Line 215**: Reconcile after removal should succeed
- **Line 242**: Initial reconcile should succeed
- **Line 253**: No-op reconcile should succeed
- **Line 317**: Reconcile should succeed

#### hoop-daemon/tests/supervisor_isolation.rs

- **Line 59**: CostAggregator creation should succeed
- **Line 138**: Reconcile should succeed
- **Line 181**: Reconcile should succeed
- **Line 210**: project-a should exist
- **Line 215**: project-b should exist
- **Line 260**: Reconcile should succeed
- **Line 288**: project-b should exist
- **Line 293**: project-c should exist
- **Line 383**: Reconcile should succeed
- **Line 428**: Reconcile should succeed
- **Line 475**: Reconcile should succeed
- **Line 507**: project-b should exist

#### hoop-daemon/tests/supervisor_restart.rs

- **Line 57**: Failed to create cost aggregator

#### hoop-daemon/tests/supervisor_shutdown.rs

- **Line 119**: Reconcile should succeed
- **Line 161**: Reconcile should succeed
- **Line 174**: Reconcile after removal should succeed
- **Line 210**: Reconcile should succeed
- **Line 223**: Reconcile to empty should succeed
- **Line 252**: Reconcile should succeed
- **Line 301**: Reconcile should succeed
- **Line 318**: Reconcile to empty should succeed

#### hoop-daemon/tests/testrepo_harness_integration.rs

- **Line 258**: Failed to spawn daemon
- **Line 260**: Failed to create test client
- **Line 263**: Health check failed
- **Line 267**: Ready check failed
- **Line 276**: Failed to spawn daemon
- **Line 278**: Failed to create test client
- **Line 285**: Failed to connect to WebSocket
- **Line 291**: Timeout waiting for first message
- **Line 292**: WebSocket stream ended
- **Line 294**: Failed to receive first message
- **Line 298**: Failed to parse init event
- **Line 309**: subscriptions should be array
- **Line 317**: subscriptions should be array
- **Line 336**: Failed to spawn daemon
- **Line 338**: Failed to create test client
- **Line 340**: Failed to collect snapshots
- **Line 370**: Failed to spawn daemon
- **Line 372**: Failed to create test client
- **Line 375**: Failed to fetch beads
- **Line 380**: Failed to fetch workers
- **Line 385**: Failed to fetch conversations
- **Line 390**: Failed to fetch projects
- **Line 395**: Failed to fetch config status
- **Line 399**: Failed to fetch capacity
- **Line 409**: Failed to spawn daemon
- **Line 411**: Failed to create test client
- **Line 413**: Failed to fetch metrics
- **Line 444**: Failed to spawn daemon
- **Line 451**: Failed to connect
- **Line 458**: Timeout waiting for init
- **Line 459**: Stream ended
- **Line 460**: Failed to receive init
- **Line 474**: Failed to send subscribe
- **Line 483**: Failed to send unsubscribe
- **Line 496**: Failed to spawn daemon
- **Line 516**: Stream ended
- **Line 522**: Failed to parse
- **Line 535**: Task failed
- **Line 544**: Failed to spawn daemon
- **Line 553**: Failed to connect first time
- **Line 560**: Timeout on first connection
- **Line 561**: Stream ended
- **Line 562**: No init on first connection
- **Line 573**: Failed to reconnect
- **Line 580**: Timeout on reconnection
- **Line 581**: Stream ended
- **Line 582**: No init on reconnection
- **Line 592**: Timeout waiting for snapshots after reconnect
- **Line 593**: Stream ended
- **Line 594**: No snapshots after reconnect
- **Line 613**: Failed to spawn daemon
- **Line 615**: Failed to create test client
- **Line 618**: Failed to fetch beads
- **Line 635**: Failed to fetch workers
- **Line 645**: Failed to fetch projects

#### hoop-daemon/tests/testrepo_integration.rs

- **Line 238**: Failed to spawn daemon
- **Line 240**: Failed to create test client
- **Line 243**: Health check failed
- **Line 247**: Ready check failed
- **Line 256**: Failed to spawn daemon
- **Line 258**: Failed to create test client
- **Line 265**: Failed to connect to WebSocket
- **Line 271**: Timeout waiting for first message
- **Line 272**: WebSocket stream ended
- **Line 274**: Failed to receive first message
- **Line 278**: Failed to parse init event
- **Line 289**: subscriptions should be array
- **Line 308**: Failed to spawn daemon
- **Line 310**: Failed to create test client
- **Line 312**: Failed to collect snapshots
- **Line 342**: Failed to spawn daemon
- **Line 344**: Failed to create test client
- **Line 347**: Failed to collect WS snapshots
- **Line 350**: Failed to fetch beads via REST
- **Line 351**: Failed to fetch workers via REST
- **Line 352**: Failed to fetch projects via REST
- **Line 353**: Failed to fetch config via REST
- **Line 390**: Failed to spawn daemon
- **Line 392**: Failed to create test client
- **Line 395**: Failed to fetch beads
- **Line 399**: Failed to fetch workers
- **Line 403**: Failed to fetch projects
- **Line 413**: Failed to fetch config status
- **Line 417**: Failed to fetch capacity
- **Line 426**: Failed to spawn daemon
- **Line 428**: Failed to create test client
- **Line 430**: Failed to fetch metrics
- **Line 461**: Failed to spawn daemon
- **Line 468**: Failed to connect
- **Line 475**: Timeout waiting for init
- **Line 476**: Stream ended
- **Line 477**: Failed to receive init
- **Line 491**: Failed to send subscribe
- **Line 500**: Failed to send unsubscribe
- **Line 513**: Failed to spawn daemon
- **Line 533**: Stream ended
- **Line 539**: Failed to parse
- **Line 552**: Task failed
- **Line 561**: Failed to spawn daemon
- **Line 570**: Failed to connect first time
- **Line 577**: Timeout on first connection
- **Line 578**: Stream ended
- **Line 579**: No init on first connection
- **Line 590**: Failed to reconnect
- **Line 597**: Timeout on reconnection
- **Line 598**: Stream ended
- **Line 599**: No init on reconnection
- **Line 609**: Timeout waiting for snapshots after reconnect
- **Line 610**: Stream ended
- **Line 611**: No snapshots after reconnect
- **Line 630**: Failed to spawn daemon
- **Line 632**: Failed to create test client
- **Line 635**: Failed to fetch beads
- **Line 652**: Failed to fetch workers
- **Line 665**: Failed to fetch projects

#### hoop-daemon/tests_phase5/adapter_failover_test.rs

- **Line 152**: Failed to spawn daemon
- **Line 154**: Failed to create client
- **Line 157**: Health check failed
- **Line 161**: Failed to spawn agent
- **Line 169**: Failed to get agent status
- **Line 173**: Health check failed
- **Line 184**: Failed to spawn daemon
- **Line 186**: Failed to create client
- **Line 189**: Failed to spawn agent
- **Line 194**: Should have session_db_id
- **Line 200**: Failed to get agent status
- **Line 212**: Failed to switch adapter
- **Line 217**: Should have new session_db_id
- **Line 229**: Failed to list sessions
- **Line 249**: Failed to get agent status
- **Line 262**: Failed to spawn daemon
- **Line 264**: Failed to create client
- **Line 267**: Failed to spawn agent
- **Line 272**: Should have session_db_id
- **Line 278**: Failed to switch adapter
- **Line 284**: Failed to list sessions
- **Line 290**: Should find archived session
- **Line 308**: Failed to query stitch from fleet.db
- **Line 341**: Failed to spawn daemon
- **Line 343**: Failed to create client
- **Line 364**: Failed to insert reflection entry
- **Line 367**: Failed to spawn agent
- **Line 371**: Failed to switch adapter
- **Line 375**: Failed to list reflection entries
- **Line 386**: Entry should exist
- **Line 399**: Failed to spawn daemon
- **Line 401**: Failed to create client
- **Line 404**: Failed to spawn agent
- **Line 408**: Should have session_db_id
- **Line 414**: Failed to switch adapter
- **Line 420**: Failed to switch adapter back
- **Line 424**: Should have second session_db_id
- **Line 430**: Failed to list sessions
- **Line 447**: Should find first archived session
- **Line 451**: Should find second archived session
- **Line 477**: Failed to spawn daemon
- **Line 479**: Failed to create client
- **Line 482**: Failed to spawn agent
- **Line 503**: Failed to insert reflection entry
- **Line 509**: Failed to switch adapter
- **Line 515**: Failed to get agent status
- **Line 521**: Failed to list reflection entries
- **Line 536**: Failed to spawn daemon
- **Line 538**: Failed to create client
- **Line 541**: Failed to spawn agent
- **Line 568**: Switch 1 should complete
- **Line 571**: Switch 2 should complete
- **Line 580**: Health check failed
- **Line 597**: Failed to spawn daemon
- **Line 599**: Failed to create client
- **Line 602**: Failed to spawn agent
- **Line 607**: Should have session_db_id
- **Line 613**: Failed to get agent status
- **Line 639**: Failed to write updated config.yml
- **Line 650**: Failed to get agent status after config reload
- **Line 663**: Failed to list sessions
- **Line 683**: Should find original archived session
- **Line 700**: Failed to query stitch from fleet.db
- **Line 722**: Health check failed
- **Line 808**: Failed to start mock Anthropic server
- **Line 815**: Failed to spawn daemon
- **Line 817**: Failed to create client
- **Line 820**: Health check failed
- **Line 840**: Failed to write config with mock server URL
- **Line 856**: Health check failed
- **Line 868**: Ready endpoint request failed
- **Line 885**: Health check failed
- **Line 895**: Health check failed
- **Line 913**: Failed to start mock Anthropic server
- **Line 920**: Failed to spawn daemon
- **Line 922**: Failed to create client
- **Line 925**: Health check failed
- **Line 940**: Failed to write config
- **Line 946**: Health check failed
- **Line 953**: Adapter switch should succeed
- **Line 961**: Failed to get agent status
- **Line 966**: Health check failed

#### hoop-mcp/tests/create_only_stub.rs

- **Line 20**: create temp dir
- **Line 37**: create br script
- **Line 38**: write br script
- **Line 43**: chmod br script
- **Line 96**: run fake br
- **Line 266**: run fake br

#### hoop-mcp/tests/forbidden_worker_steering.rs

- **Line 116**: Failed to create McpServerState for test
- **Line 149**: Failed to create McpServerState for test

#### hoop-mcp/tests/protocol_contract.rs

- **Line 22**: workspace root
- **Line 43**: JsonRpcRequest must deserialize from initialize fixture
- **Line 74**: JsonRpcRequest must deserialize from tools/list fixture
- **Line 145**: JsonRpcRequest must deserialize from prompts/list fixture
- **Line 197**: JsonRpcRequest must deserialize from resources/list fixture
- **Line 249**: JsonRpcRequest must deserialize from shutdown fixture
- **Line 376**: JsonRpcRequest must deserialize from tools_call fixture

#### hoop-mcp/tests/socket_permissions.rs

- **Line 14**: temp dir
- **Line 18**: bind socket
- **Line 21**: set permissions
- **Line 24**: metadata
- **Line 127**: temp dir
- **Line 130**: bind socket
- **Line 132**: set permissions
- **Line 135**: metadata

#### hoop-schema/tests/schema_drift.rs

- **Line 756**: Failed to create fixture directory
- **Line 778**: Failed to write index
- **Line 885**: Failed to read index.json
- **Line 887**: Failed to parse index.json

#### tests/acceptance/s1_morning_review.rs

- **Line 34**: workspace root
- **Line 103**: Failed to spawn daemon
- **Line 111**: Failed to fetch dashboard
- **Line 115**: Failed to parse dashboard
- **Line 123**: total_workers must be a number
- **Line 131**: total_spend_usd must be a number
- **Line 140**: longest_running must be an array
- **Line 146**: Failed to fetch worker timeline
- **Line 150**: Failed to parse timeline
- **Line 161**: Failed to spawn daemon
- **Line 171**: Failed to fetch dashboard
- **Line 190**: Failed to spawn daemon
- **Line 198**: Failed to fetch dashboard
- **Line 206**: Failed to parse response
- **Line 223**: Failed to spawn daemon
- **Line 231**: Failed to fetch dashboard
- **Line 233**: Failed to parse response
- **Line 241**: Failed to fetch dashboard
- **Line 243**: Failed to parse response
- **Line 254**: Failed to spawn daemon
- **Line 262**: Failed to fetch dashboard
- **Line 264**: Failed to parse response
- **Line 268**: total_spend_usd must be present
- **Line 274**: spend_by_project must be an array
- **Line 295**: Failed to spawn daemon
- **Line 303**: Failed to fetch dashboard
- **Line 305**: Failed to parse response
- **Line 309**: total_workers must be present
- **Line 313**: workers_by_project must be an array

#### tests/acceptance/s2_transcript_archaeology.rs

- **Line 35**: workspace root
- **Line 104**: Failed to spawn daemon
- **Line 112**: Failed to fetch beads
- **Line 116**: Failed to parse beads
- **Line 122**: Bead should have an id
- **Line 128**: Failed to fetch bead events
- **Line 136**: Failed to parse events
- **Line 152**: Failed to spawn daemon
- **Line 160**: Failed to fetch beads
- **Line 162**: Failed to parse beads
- **Line 168**: Bead should have an id
- **Line 176**: Failed to fetch bead events
- **Line 197**: Failed to spawn daemon
- **Line 205**: Failed to connect to stitch endpoint
- **Line 219**: Failed to spawn daemon
- **Line 235**: Failed to connect to endpoint
- **Line 251**: Failed to spawn daemon
- **Line 259**: Failed to fetch conversations
- **Line 263**: Failed to parse conversations
- **Line 274**: Failed to spawn daemon
- **Line 282**: Failed to fetch beads
- **Line 284**: Failed to parse beads
- **Line 300**: Failed to spawn daemon
- **Line 308**: Failed to fetch cost trends
- **Line 312**: Failed to parse cost data
- **Line 323**: Failed to spawn daemon
- **Line 331**: Failed to fetch beads
- **Line 333**: Failed to parse beads
- **Line 339**: Bead should have an id
- **Line 345**: Failed to fetch bead events
- **Line 348**: Failed to parse events

#### tests/acceptance/s3_bead_creation_from_chat.rs

- **Line 37**: workspace root
- **Line 106**: Failed to spawn daemon
- **Line 124**: Failed to create draft
- **Line 140**: Failed to spawn daemon
- **Line 148**: Failed to fetch drafts
- **Line 162**: Failed to spawn daemon
- **Line 170**: Failed to fetch audit log
- **Line 184**: Failed to spawn daemon
- **Line 192**: Failed to fetch beads
- **Line 205**: Failed to spawn daemon
- **Line 225**: Failed to create draft
- **Line 228**: Failed to parse draft
- **Line 248**: Failed to spawn daemon
- **Line 256**: Failed to fetch audit log
- **Line 259**: Failed to parse audit
- **Line 279**: Failed to spawn daemon
- **Line 298**: Failed to create draft
- **Line 302**: Failed to parse draft
- **Line 306**: draft_id should be present
- **Line 320**: Failed to list drafts
- **Line 323**: Failed to parse list

#### tests/acceptance/s4_daemon_restart.rs

- **Line 31**: workspace root
- **Line 105**: create temp dir
- **Line 107**: create .hoop dir
- **Line 122**: write projects.yaml
- **Line 131**: write config.yml
- **Line 132**: create data dir
- **Line 191**: write claim
- **Line 192**: write complete
- **Line 193**: write claim
- **Line 197**: Failed to spawn first daemon
- **Line 205**: Failed to fetch beads from first daemon
- **Line 209**: Failed to parse beads
- **Line 215**: write complete
- **Line 216**: write claim
- **Line 223**: Failed to spawn second daemon
- **Line 229**: Failed to fetch beads from second daemon
- **Line 233**: Failed to parse beads
- **Line 263**: write claim
- **Line 265**: write complete
- **Line 271**: Failed to spawn first daemon
- **Line 277**: Failed to spawn second daemon
- **Line 302**: Failed to spawn first daemon
- **Line 307**: write claim
- **Line 308**: write complete
- **Line 309**: write claim
- **Line 320**: Failed to spawn second daemon
- **Line 322**: write complete
- **Line 323**: write claim
- **Line 337**: Failed to fetch beads
- **Line 358**: write claim
- **Line 359**: write complete
- **Line 364**: Failed to spawn daemon
- **Line 385**: Failed to fetch beads
- **Line 389**: Failed to parse beads
- **Line 408**: write claim

#### tests/acceptance/s5_workspace_deleted.rs

- **Line 27**: Failed to create .beads dir
- **Line 29**: Failed to create issues.jsonl
- **Line 37**: Failed to create temp dir
- **Line 39**: Failed to create .hoop dir
- **Line 68**: Failed to write projects.yaml
- **Line 77**: Failed to write config.yml
- **Line 79**: Failed to create data dir
- **Line 166**: Failed to spawn daemon
- **Line 173**: Failed to get readyz status
- **Line 179**: Failed to remove .beads from project A
- **Line 228**: Failed to spawn daemon
- **Line 237**: Failed to remove .beads from project A
- **Line 247**: Failed to fetch projects
- **Line 251**: Failed to parse projects
- **Line 263**: Failed to check health
- **Line 291**: Failed to spawn daemon
- **Line 299**: Failed to get readyz status
- **Line 304**: Failed to remove .beads from project A
- **Line 311**: Failed to get readyz status after deletion
- **Line 364**: Failed to spawn daemon
- **Line 373**: Failed to remove .beads
- **Line 382**: Failed to check health

#### tests/acceptance/s6_machine_mode.rs

- **Line 36**: workspace root
- **Line 107**: Failed to spawn daemon
- **Line 116**: Failed to fetch status
- **Line 120**: Failed to parse status
- **Line 131**: Failed to spawn daemon
- **Line 139**: Failed to fetch projects
- **Line 152**: Failed to spawn daemon
- **Line 160**: Failed to fetch projects
- **Line 164**: Failed to parse projects
- **Line 175**: Failed to spawn daemon
- **Line 192**: Failed to fetch endpoint
- **Line 210**: Failed to spawn daemon
- **Line 218**: Failed to fetch projects
- **Line 220**: Failed to parse projects
- **Line 242**: Failed to spawn daemon
- **Line 250**: Failed to fetch healthz
- **Line 261**: Failed to spawn daemon
- **Line 269**: Failed to fetch readyz
- **Line 284**: Failed to spawn daemon
- **Line 293**: Failed to fetch bead
- **Line 310**: Failed to spawn daemon
- **Line 327**: Task panicked

#### tests/cli_test_helpers.rs

- **Line 398**: projects.rs must exist
- **Line 412**: main.rs must exist

### .unwrap_err() (25 instances)

#### hoop-cli/tests/cli_test_helpers.rs

- **Line 2606**: unwrap_err() call
- **Line 2613**: unwrap_err() call
- **Line 2620**: unwrap_err() call
- **Line 2651**: unwrap_err() call
- **Line 2665**: unwrap_err() call

#### hoop-cli/tests/cli_test_utils_examples.rs

- **Line 397**: unwrap_err() call
- **Line 423**: unwrap_err() call

#### hoop-daemon/tests/backup_restore_cycle.rs

- **Line 313**: unwrap_err() call

#### hoop-daemon/tests/claimed_at_parsing.rs

- **Line 191**: unwrap_err() call
- **Line 713**: unwrap_err() call

#### hoop-daemon/tests/config_reload_cycle.rs

- **Line 262**: unwrap_err() call
- **Line 275**: unwrap_err() call
- **Line 293**: unwrap_err() call

#### hoop-daemon/tests/create_only_stub.rs

- **Line 265**: unwrap_err() call

#### hoop-daemon/tests/disaster_recovery_runbook.rs

- **Line 198**: unwrap_err() call
- **Line 556**: unwrap_err() call

#### hoop-daemon/tests/mutation_handler_test.rs

- **Line 163**: unwrap_err() call
- **Line 205**: unwrap_err() call
- **Line 238**: unwrap_err() call

#### hoop-daemon/tests/per_project_redaction_integration.rs

- **Line 188**: unwrap_err() call
- **Line 266**: unwrap_err() call

#### hoop-daemon/tests/skills_integration.rs

- **Line 153**: unwrap_err() call
- **Line 200**: unwrap_err() call
- **Line 246**: unwrap_err() call

#### hoop-mcp/tests/forbidden_worker_steering.rs

- **Line 126**: unwrap_err() call

### anyhow::bail!() (41 instances)

#### hoop-cli/tests/remove_no_interactive_flag.rs

- **Line 744**: unparsed

#### hoop-daemon/src/integration_test_client.rs

- **Line 69**: unparsed
- **Line 99**: unparsed
- **Line 115**: unparsed
- **Line 158**: unparsed
- **Line 178**: unparsed
- **Line 193**: unparsed
- **Line 208**: unparsed
- **Line 234**: unparsed
- **Line 243**: unparsed
- **Line 252**: unparsed
- **Line 262**: unparsed
- **Line 271**: unparsed
- **Line 286**: unparsed
- **Line 352**: unparsed
- **Line 355**: unparsed
- **Line 361**: unparsed
- **Line 404**: unparsed

#### hoop-daemon/src/load_test.rs

- **Line 369**: unparsed

#### hoop-daemon/tests/adapter_failover_test.rs

- **Line 49**: unparsed

#### hoop-daemon/tests/hoop_dies_nothing_notices.rs

- **Line 168**: unparsed

#### hoop-daemon/tests/integration_harness.rs

- **Line 109**: unparsed
- **Line 115**: unparsed
- **Line 121**: unparsed
- **Line 127**: unparsed
- **Line 136**: unparsed
- **Line 143**: unparsed
- **Line 152**: unparsed
- **Line 217**: unparsed
- **Line 220**: unparsed
- **Line 223**: unparsed
- **Line 226**: unparsed
- **Line 247**: unparsed
- **Line 250**: unparsed
- **Line 359**: unparsed
- **Line 365**: unparsed
- **Line 371**: unparsed
- **Line 698**: unparsed

#### hoop-daemon/tests/testrepo_harness_integration.rs

- **Line 57**: unparsed

#### hoop-daemon/tests/testrepo_integration.rs

- **Line 67**: unparsed

#### hoop-daemon/tests_phase5/adapter_failover_test.rs

- **Line 46**: unparsed

### assert! (2,032 instances)

#### hoop-cli/tests/clap_test_utils.rs

- **Line 946**: /tmp
- **Line 954**: test-project
- **Line 962**: --from
- **Line 973**: scan
- **Line 979**: /tmp
- **Line 984**: /tmp
- **Line 989**: /tmp
- **Line 994**: /tmp
- **Line 999**: /tmp
- **Line 1046**: remove
- **Line 1049**: remove
- **Line 1055**: scan
- **Line 1058**: init
- **Line 1064**: scan
- **Line 1067**: --no-interactive
- **Line 1319**: /tmp
- **Line 1325**: scan
- **Line 1331**: test-project
- **Line 1337**: --from
- **Line 1346**: /tmp
- **Line 1381**: scan
- **Line 1384**: scan

#### hoop-cli/tests/cli_test_helpers.rs

- **Line 1971**: Failed to parse flag before subcommand for {}
- **Line 1977**: no_interactive should be true with flag before {}
- **Line 1990**: Failed to parse flag after subcommand for {}
- **Line 1996**: no_interactive should be true with flag after {}
- **Line 2017**: Short flag -y should set no_interactive=true for {}
- **Line 2050**: Failed to parse nested command for {} {}
- **Line 2079**: /tmp
- **Line 2087**: /tmp
- **Line 2088**: /tmp
- **Line 2108**: Failed to parse command without flag
- **Line 2111**: no_interactive should default to false when not specified
- **Line 2117**: Default flag value verification failed
- **Line 2162**: Failed to parse {} with flag before subcommand
- **Line 2168**: Flag before subcommand should set no_interactive=true for {}
- **Line 2183**: Failed to parse {} with flag after subcommand
- **Line 2189**: Flag after subcommand should set no_interactive=true for {}
- **Line 2204**: Short flag -y should set no_interactive=true for {}
- **Line 2217**: Flag position consistency check failed for {}
- **Line 2225**: Failed to parse {} without flag
- **Line 2231**: Default no_interactive should be false for {}
- **Line 2240**: Flag propagation check failed for {}
- **Line 2287**: Failed to parse {} with --no-interactive --confirm
- **Line 2293**: Args should include --confirm flag for {}
- **Line 2294**: Args should include --confirm flag for {}
- **Line 2307**: Should parse {} with --no-interactive (even without --confirm)
- **Line 2313**: Args should not include --confirm flag for {}
- **Line 2314**: Args should not include --confirm flag for {}
- **Line 2406**: projects
- **Line 2424**: status
- **Line 2435**: /tmp
- **Line 2445**: /tmp
- **Line 2455**: remove
- **Line 2472**: -y
- **Line 2483**: /tmp
- **Line 2493**: projects
- **Line 2509**: add
- **Line 2520**: /tmp
- **Line 2531**: --json
- **Line 2542**: /tmp
- **Line 2587**: remove
- **Line 2593**: No arguments provided
- **Line 2599**: No arguments provided
- **Line 2605**: No arguments provided
- **Line 2612**: No arguments provided
- **Line 2619**: No arguments provided
- **Line 2634**: scan
- **Line 2635**: scan
- **Line 2643**: /tmp
- **Line 2650**: /tmp
- **Line 2651**: /tmp
- **Line 2657**: scan
- **Line 2664**: scan
- **Line 2665**: scan
- **Line 2671**: /tmp
- **Line 2677**: scan
- **Line 2684**: /tmp
- **Line 2690**: remove
- **Line 2696**: --json
- **Line 2702**: /tmp
- **Line 2711**: /tmp
- **Line 2717**: scan
- **Line 2723**: remove
- **Line 2729**: /tmp
- **Line 2735**: remove
- **Line 2741**: --json
- **Line 2747**: new-project
- **Line 2753**: /tmp
- **Line 2760**: scan
- **Line 2766**: scan
- **Line 2774**: /tmp
- **Line 2790**: status
- **Line 2833**: /tmp
- **Line 2841**: status
- **Line 2870**: Flag should be consistent at both positions
- **Line 2874**: Flag should propagate correctly through handler chain
- **Line 2884**: --json
- **Line 2885**: --json
- **Line 2892**: --json
- **Line 2896**: All parsing levels should agree on flag value
- **Line 2912**: Empty args should error
- **Line 2913**: Empty args should error
- **Line 2919**: Empty args should error

#### hoop-cli/tests/cli_test_utils.rs

- **Line 504**: Failed to parse args: {:?}
- **Line 532**: Failed to parse args: {:?}
- **Line 557**: Failed to parse args: {:?}
- **Line 629**: Failed to parse args: {:?}
- **Line 671**: Failed to parse with flag before command
- **Line 682**: Failed to parse with flag after command
- **Line 693**: Failed to parse with -y flag
- **Line 711**: Failed to parse without flag
- **Line 776**: before
- **Line 777**: after
- **Line 790**: /tmp
- **Line 836**: --no-interactive
- **Line 839**: --no-interactive
- **Line 842**: --no-interactive
- **Line 894**: projects.yaml
- **Line 895**: projects.yaml
- **Line 899**: test-project
- **Line 900**: test-project
- **Line 904**: test-project
- **Line 905**: test-project
- **Line 929**: before
- **Line 936**: after
- **Line 962**: --no-interactive
- **Line 963**: --no-interactive
- **Line 964**: --no-interactive
- **Line 974**: --no-interactive
- **Line 980**: --no-interactive
- **Line 987**: --no-interactive
- **Line 991**: status
- **Line 999**: scan
- **Line 1020**: scan
- **Line 1031**: scan
- **Line 1042**: scan
- **Line 1053**: scan
- **Line 1064**: scan
- **Line 1074**: scan
- **Line 1084**: before
- **Line 1090**: after
- **Line 1096**: Continue?
- **Line 1106**: Remove project?
- **Line 1117**: Remove project?
- **Line 1128**: test-project
- **Line 1131**: test-project
- **Line 1134**: test-project
- **Line 1142**: --no-interactive
- **Line 1143**: --no-interactive
- **Line 1151**: --no-interactive
- **Line 1152**: --no-interactive

#### hoop-cli/tests/cli_test_utils_examples.rs

- **Line 17**: scan
- **Line 22**: scan
- **Line 29**: scan
- **Line 34**: --no-interactive
- **Line 48**: projects
- **Line 53**: /tmp
- **Line 54**: /tmp
- **Line 63**: scan
- **Line 74**: restore
- **Line 89**: Verification should succeed: {:?}
- **Line 99**: Verification should succeed: {:?}
- **Line 108**: Should verify no flag is present: {:?}
- **Line 117**: Should fail when flag is actually present
- **Line 134**: Prompt should be suppressed: {:?}
- **Line 146**: Prompt should be shown when no_interactive=false
- **Line 162**: Should require --confirm when no_interactive=true for destructive operations
- **Line 169**: Should pass with --confirm flag
- **Line 173**: Should not require --confirm in interactive mode
- **Line 249**: Workspace directory should exist
- **Line 250**: Workspace should have .beads/ directory
- **Line 262**: Registry file should exist
- **Line 263**: Registry should be in .hoop/ directory
- **Line 271**: Registry should have empty projects list
- **Line 285**: Should parse scan command successfully
- **Line 292**: --no-interactive
- **Line 316**: Should succeed with --confirm flag
- **Line 319**: Should succeed with --confirm flag
- **Line 323**: Should succeed with --confirm flag
- **Line 386**: All complex multi-command tests should pass
- **Line 395**: Should fail with empty args
- **Line 398**: Should have descriptive error message
- **Line 408**: Should fail with invalid expected_position
- **Line 421**: Should require --confirm flag
- **Line 424**: Error message should mention --confirm flag
- **Line 450**: before
- **Line 451**: after
- **Line 458**: Delete?

#### hoop-cli/tests/init_no_interactive_flag.rs

- **Line 24**: Should successfully parse flag before subcommand
- **Line 29**: Args should contain init command
- **Line 40**: Should successfully parse flag after subcommand
- **Line 45**: Args should contain init command
- **Line 56**: Should successfully parse short flag before subcommand
- **Line 68**: Should successfully parse short flag after subcommand
- **Line 80**: Should successfully parse command without flag
- **Line 95**: Flag extraction should verify for 'before' position
- **Line 108**: Flag extraction should verify for 'after' position
- **Line 121**: Should verify no flag is present
- **Line 137**: Flag should be extracted from parsed CLI structure
- **Line 143**: Flag should be passed to run_init_wizard handler function
- **Line 149**: Init command handler should exist in main.rs
- **Line 162**: run_init_wizard must accept no_interactive parameter
- **Line 168**: run_init_wizard must check no_interactive flag
- **Line 185**: Init must check no_interactive flag early in the handler
- **Line 191**: Init must exit with code 2 when no_interactive is true
- **Line 197**: Init must explain why it cannot run non-interactively
- **Line 202**: Init must state that it requires interactive input
- **Line 207**: Init must suggest manual configuration for automation
- **Line 226**: Init should have no_interactive check
- **Line 230**: Init should have wizard banner print
- **Line 234**: Init should have stage 1 dependency check
- **Line 240**: no_interactive check must come before wizard banner
- **Line 244**: Wizard banner must come before stage 1
- **Line 285**: Wizard should not run when no_interactive=true (explicit rejection)
- **Line 302**: Wizard should run when no_interactive=false (default interactive mode)
- **Line 319**: Error message should be clear and start with command name
- **Line 325**: Error message should go to stderr via eprintln!
- **Line 331**: Error message should provide automated setup alternative
- **Line 337**: Error message should reference config.yml
- **Line 341**: Error message should reference projects.yaml
- **Line 367**: Init must exit with code 2 (fatal/precondition error)
- **Line 381**: Should parse flag before command
- **Line 386**: Should parse flag after command
- **Line 421**: ✓ Flag accepted as parameter
- **Line 427**: ✓ Flag checked early in handler
- **Line 433**: ✓ Helpful error message provided
- **Line 439**: ✓ Manual configuration alternative suggested
- **Line 445**: ✓ Correct exit code (2) used
- **Line 451**: ✓ Wizard stage 1 exists
- **Line 455**: ✓ Wizard stage 2 exists
- **Line 461**: All Init command no_interactive tests verified

#### hoop-cli/tests/no_interactive_flag_behavior.rs

- **Line 50**: Test workspace should have .beads/
- **Line 67**: Interactive scan requires prompts (verified by code review)
- **Line 78**: Scan combines no_interactive || yes correctly
- **Line 96**: Remove must check for confirm flag in non-interactive mode
- **Line 100**: Remove must show helpful error when confirm is missing
- **Line 115**: Should successfully parse flag before subcommand
- **Line 120**: Should include 'remove' in args
- **Line 121**: Should include project name
- **Line 131**: Should successfully parse flag after subcommand
- **Line 136**: Should include 'remove' in args
- **Line 137**: Should include project name
- **Line 149**: Handler signature must include no_interactive parameter
- **Line 155**: Handler must check no_interactive flag in safety condition
- **Line 160**: Handler must check no_interactive flag for prompt suppression
- **Line 169**: main() must pass no_interactive flag to remove_project handler
- **Line 185**: Should have confirm requirement check
- **Line 189**: Should have prompt suppression check
- **Line 193**: Confirm check must come before prompt check (early exit on success)
- **Line 200**: After safety checks, handler should proceed with removal
- **Line 215**: Handler should have branch for interactive prompting
- **Line 221**: Handler should prompt for confirmation with clear message
- **Line 227**: Prompt should use eprint! to write to stderr
- **Line 233**: Handler should read user response from stdin
- **Line 239**: Handler should check for yes/y response
- **Line 244**: Handler should notify on cancellation
- **Line 249**: Handler should return false on cancellation
- **Line 261**: Should successfully parse short flag variant
- **Line 274**: Error message should clearly state the requirement
- **Line 280**: Error message should show correct command pattern
- **Line 285**: Error message should include both flags in example
- **Line 303**: Prompt should be suppressed when no_interactive=true AND confirm=true
- **Line 313**: Prompt should be shown when no_interactive=false (default)
- **Line 350**: Should parse flag before command
- **Line 355**: Should parse flag after command
- **Line 378**: Should successfully parse command without flag
- **Line 395**: Flag should be extracted from parsed CLI structure
- **Line 401**: Flag should be passed to remove_project handler function
- **Line 416**: Remove should have interactive prompting branch
- **Line 420**: Remove should prompt for confirmation in interactive mode
- **Line 438**: Remove must have both confirm check and prompt check
- **Line 444**: Confirm check must come before prompt check
- **Line 460**: Restore must check for confirm flag in non-interactive mode
- **Line 464**: Restore must show helpful error when confirm is missing
- **Line 478**: Restore should have interactive prompting branch
- **Line 482**: Restore should prompt for confirmation in interactive mode
- **Line 497**: Dry-run should check no_interactive flag
- **Line 501**: Should show non-interactive command format
- **Line 516**: Should successfully parse flag before subcommand
- **Line 521**: Should include 'restore' in args
- **Line 522**: Should include --from flag
- **Line 523**: Should include URI
- **Line 533**: Should successfully parse flag after subcommand
- **Line 538**: Should include 'restore' in args
- **Line 539**: Should include --from flag
- **Line 540**: Should include URI
- **Line 552**: Handler signature must include no_interactive parameter
- **Line 558**: Handler must check no_interactive flag in safety condition
- **Line 563**: Handler must check no_interactive flag for prompt suppression
- **Line 572**: main() must pass no_interactive flag to run_restore handler
- **Line 588**: Should have confirm requirement check
- **Line 592**: Should have prompt suppression check
- **Line 596**: Confirm check must come before prompt check (early exit on success)
- **Line 603**: After safety checks, handler should proceed with S3 URI parsing
- **Line 618**: Handler should have branch for interactive prompting
- **Line 624**: Handler should prompt for confirmation with clear message
- **Line 630**: Prompt should use eprint! to write to stderr
- **Line 636**: Handler should read user response from stdin
- **Line 642**: Handler should check for yes/y response
- **Line 647**: Handler should notify on cancellation
- **Line 652**: Handler should return Ok on cancellation
- **Line 671**: Should successfully parse short flag variant
- **Line 675**: Should include --confirm flag
- **Line 685**: Error message should clearly state the requirement
- **Line 691**: Error message should show correct command pattern
- **Line 696**: Error message should include both flags in example
- **Line 714**: Prompt should be suppressed when no_interactive=true AND confirm=true
- **Line 724**: Prompt should be shown when no_interactive=false (default)
- **Line 767**: Should parse flag before command
- **Line 778**: Should parse flag after command
- **Line 806**: Should successfully parse command without flag
- **Line 823**: Flag should be extracted from parsed CLI structure
- **Line 829**: Flag should be passed to run_restore handler function
- **Line 846**: Restore must have both confirm check and prompt check
- **Line 852**: Confirm check must come before prompt check
- **Line 869**: Init must check no_interactive flag
- **Line 873**: Init must explain why it cannot run non-interactively
- **Line 877**: Init must suggest manual configuration for automation
- **Line 895**: Init should have no_interactive check and wizard stages
- **Line 901**: no_interactive check must come before wizard stages
- **Line 916**: Flag must be extracted from parsed CLI
- **Line 922**: Flag must be passed to scan handler
- **Line 928**: Flag must be passed to remove handler
- **Line 934**: Flag must be passed to restore handler
- **Line 940**: Flag must be passed to init handler
- **Line 953**: Flag must have global = true attribute
- **Line 966**: scan_projects must accept no_interactive parameter
- **Line 972**: scan_projects must check no_interactive flag
- **Line 985**: remove_project must accept no_interactive parameter
- **Line 991**: remove_project must check no_interactive flag
- **Line 1004**: run_restore must accept no_interactive parameter
- **Line 1010**: run_restore must check no_interactive flag
- **Line 1023**: run_init_wizard must accept no_interactive parameter
- **Line 1029**: run_init_wizard must check no_interactive flag
- **Line 1053**: root: &str, no_interactive: bool)
- **Line 1057**: Scan should not have confirm parameter
- **Line 1061**: Scan should not check confirm flag in non-interactive mode
- **Line 1089**: Remove must require --confirm in non-interactive mode
- **Line 1095**: Restore must require --confirm in non-interactive mode
- **Line 1110**: Init must check no_interactive
- **Line 1114**: Init must exit when no_interactive is true

#### hoop-cli/tests/remove_no_interactive_flag.rs

- **Line 25**: Should successfully parse flag before subcommand
- **Line 30**: Args should contain remove command
- **Line 34**: Args should contain project name
- **Line 45**: Should successfully parse flag after subcommand
- **Line 50**: Args should contain remove command
- **Line 54**: Args should contain project name
- **Line 65**: Should successfully parse short flag before subcommand
- **Line 77**: Should successfully parse short flag after subcommand
- **Line 89**: Should successfully parse command without flag
- **Line 105**: Flag extraction should verify for 'before' position
- **Line 119**: Flag extraction should verify for 'after' position
- **Line 133**: Should verify no flag is present
- **Line 149**: Flag should be extracted from parsed CLI structure
- **Line 155**: Flag should be passed to remove_project handler
- **Line 161**: Remove command handler should exist in main.rs
- **Line 174**: remove_project must accept no_interactive parameter
- **Line 180**: remove_project must check no_interactive flag for confirm requirement
- **Line 185**: remove_project must check no_interactive flag for prompting logic
- **Line 209**: Should error when --confirm is missing in no-interactive mode
- **Line 215**: Error should suggest using --confirm flag
- **Line 248**: Should have confirmation prompt in interactive mode
- **Line 254**: Should read from stdin for confirmation
- **Line 260**: Should process user input
- **Line 284**: Should show project removal message
- **Line 289**: Should prompt for confirmation
- **Line 294**: Should flush stderr after prompt
- **Line 299**: Should read user input from stdin
- **Line 304**: Should process user input
- **Line 309**: Should check for yes/yes response
- **Line 314**: Should show cancellation message
- **Line 337**: Removal message should use eprintln! to write to stderr
- **Line 342**: Prompt should use eprint! to write to stderr
- **Line 348**: Should flush stderr after prompt to ensure it appears
- **Line 377**: Confirm requirement block should NOT contain confirmation prompt
- **Line 391**: Confirmation prompt should exist in !no_interactive branch
- **Line 441**: Prompt should not be shown when no_interactive=true
- **Line 457**: Prompt should be shown when no_interactive=false (default)
- **Line 475**: Confirmation prompt should be suppressed when no_interactive=true
- **Line 491**: Confirmation prompt should be shown when no_interactive=false (default)
- **Line 507**: --confirm should be required when no_interactive=true
- **Line 523**: --confirm should NOT be required when no_interactive=false (prompts instead)
- **Line 541**: When no_interactive=true: prompt suppressed AND --confirm required
- **Line 559**: When no_interactive=false: prompt shown AND --confirm not required
- **Line 573**: Should parse flag before command
- **Line 578**: Should parse flag after command
- **Line 743**: Should bail out when --confirm is missing in no-interactive mode
- **Line 749**: Error should mention --confirm requirement
- **Line 755**: Error should suggest the correct command with --confirm
- **Line 784**: Behavior: When no_interactive=false, should show removal message
- **Line 789**: Behavior: When no_interactive=false, should prompt for confirmation
- **Line 794**: Behavior: When no_interactive=false, should flush stderr after prompt
- **Line 799**: Behavior: When no_interactive=false, should read user input from stdin
- **Line 826**: Behavior: Removal message should go to stderr (eprintln!)
- **Line 831**: Behavior: Confirmation prompt should go to stderr (eprint!)
- **Line 837**: Behavior: Should flush stderr after prompts to ensure visibility
- **Line 869**: Behavior: Between confirm check and prompt check, should NOT read from stdin
- **Line 876**: Behavior: Stdin reading should only occur in interactive mode (no_interactive=false)
- **Line 899**: Code must have confirm requirement check
- **Line 904**: Code must have prompt check for interactive mode
- **Line 921**: Confirm requirement section must NOT have confirmation prompt (it's an error bail-out)
- **Line 930**: Interactive section must have confirmation prompt
- **Line 976**: Behavior: When checks pass, should proceed to removal
- **Line 981**: Behavior: After removal, should save the registry
- **Line 996**: Behavior: Successful removal should print confirmation message to stdout
- **Line 1016**: ✓ Remove command has confirm field
- **Line 1022**: ✓ Global flag extracted in main
- **Line 1028**: ✓ Flag passed to remove_project
- **Line 1034**: ✓ remove_project accepts both no_interactive and confirm parameters
- **Line 1040**: ✓ remove_project checks confirm requirement in no-interactive mode
- **Line 1046**: ✓ remove_project checks no_interactive flag for prompting
- **Line 1052**: ✓ Error message suggests --confirm in no-interactive mode
- **Line 1058**: ✓ Prompts for confirmation when no_interactive=false
- **Line 1064**: ✓ Prompts go to stderr (not stdout)
- **Line 1070**: ✓ Removal proceeds when checks pass
- **Line 1076**: All Remove command no_interactive tests verified

#### hoop-cli/tests/restore_no_interactive_flag.rs

- **Line 32**: Should successfully parse flag before subcommand
- **Line 37**: Args should contain restore command
- **Line 41**: Args should contain --from flag
- **Line 45**: Args should contain S3 URI
- **Line 63**: Should successfully parse flag after subcommand
- **Line 68**: Args should contain restore command
- **Line 72**: Args should contain S3 URI
- **Line 90**: Should successfully parse short flag before subcommand
- **Line 112**: Should successfully parse short flag after subcommand
- **Line 133**: Should successfully parse command without flag
- **Line 156**: Should successfully parse with --dry-run flag
- **Line 161**: Args should contain --dry-run flag
- **Line 177**: Flag extraction should verify for 'before' position
- **Line 195**: Flag extraction should verify for 'after' position
- **Line 218**: Should verify no flag is present
- **Line 233**: Flag should be extracted from parsed CLI structure
- **Line 239**: Flag should be passed to run_restore handler
- **Line 245**: Restore command handler should exist in main.rs
- **Line 258**: run_restore must accept no_interactive parameter
- **Line 265**: run_restore must check no_interactive flag for confirm requirement
- **Line 271**: run_restore must check no_interactive flag for prompting logic
- **Line 298**: Should error when --confirm is missing in no-interactive mode
- **Line 304**: Error should suggest using --confirm flag
- **Line 310**: Error should warn about destructive operation
- **Line 346**: Should have warning message in interactive mode
- **Line 351**: Should have confirmation prompt in interactive mode
- **Line 357**: Should read from stdin for confirmation
- **Line 363**: Should process user input
- **Line 390**: Should show warning message
- **Line 395**: Should show snapshot ID
- **Line 400**: Should show creation timestamp
- **Line 405**: Should prompt for confirmation
- **Line 410**: Should flush stderr after prompt
- **Line 415**: Should read user input from stdin
- **Line 420**: Should process user input
- **Line 425**: Should check for yes/yes response
- **Line 430**: Should show cancellation message
- **Line 456**: Warning message should use eprintln! to write to stderr
- **Line 461**: Prompt should use eprint! to write to stderr
- **Line 467**: Should flush stderr after prompt to ensure it appears
- **Line 500**: Confirm requirement block should NOT contain confirmation prompt
- **Line 516**: Confirmation prompt should exist in !no_interactive branch
- **Line 542**: Dry-run mode must show --no-interactive --confirm usage when no_interactive is true
- **Line 548**: Dry-run mode must show simple usage when no_interactive is false
- **Line 580**: manifest.validate() must be called before move_aside_for_rollback() \          (validate at offset {validate_pos}, move_aside at offset {move_aside_pos} from fn start)
- **Line 608**: Confirm requirement check must come before prompt check \          (confirm at offset {confirm_check}, prompt at offset {prompt_check} from fn start)
- **Line 629**: Error must clearly state --confirm is required
- **Line 634**: Error must warn about destructive operation
- **Line 639**: Error must explain what will be replaced
- **Line 644**: Error must show correct re-run command
- **Line 649**: Error should explicitly say 'Re-run with:'
- **Line 716**: ✓ Restore command has from, dry_run, and confirm fields
- **Line 722**: ✓ Global flag extracted in main
- **Line 728**: ✓ Flag passed to run_restore
- **Line 734**: ✓ run_restore accepts both no_interactive and confirm parameters
- **Line 740**: ✓ run_restore checks confirm requirement in no-interactive mode
- **Line 746**: ✓ run_restore checks no_interactive flag for prompting
- **Line 752**: ✓ Error message suggests --confirm in no-interactive mode
- **Line 758**: ✓ Prompts for confirmation when no_interactive=false
- **Line 764**: ✓ Prompts go to stderr (not stdout)
- **Line 770**: ✓ Dry-run mode shows --no-interactive --confirm usage
- **Line 792**: ✓ manifest.validate() called before move_aside_for_rollback()

#### hoop-cli/tests/scan_no_interactive_flag.rs

- **Line 25**: Should successfully parse flag before subcommand
- **Line 30**: Args should contain scan command
- **Line 34**: Args should contain scan path
- **Line 45**: Should successfully parse flag after subcommand
- **Line 50**: Args should contain scan command
- **Line 54**: Args should contain scan path
- **Line 65**: Should successfully parse short flag before subcommand
- **Line 77**: Should successfully parse short flag after subcommand
- **Line 89**: Should successfully parse command without flag
- **Line 102**: Should successfully parse local --yes flag
- **Line 107**: Args should contain local --yes flag
- **Line 119**: Should successfully parse both flags
- **Line 124**: Args should contain local --yes flag
- **Line 138**: Flag extraction should verify for 'before' position
- **Line 151**: Flag extraction should verify for 'after' position
- **Line 164**: Should verify no flag is present
- **Line 180**: Flag should be extracted from parsed CLI structure
- **Line 186**: Flag should be passed to scan_projects handler with || auto_confirm logic
- **Line 192**: Scan command handler should exist in main.rs
- **Line 205**: scan_projects must accept no_interactive parameter
- **Line 211**: scan_projects must check no_interactive flag
- **Line 225**: Scan should use || logic to combine global no_interactive with local auto_confirm
- **Line 240**: Logic should be: no_interactive || auto_confirm (OR logic)
- **Line 265**: When no_interactive=true, should print 'registering' message
- **Line 270**: When no_interactive=true, should call registry.add() without prompting
- **Line 276**: When no_interactive=true, should NOT prompt for confirmation
- **Line 297**: When no_interactive=false, should prompt with 'register? [y/N]'
- **Line 302**: When no_interactive=false, should read user input from stdin
- **Line 307**: When no_interactive=false, should process user input
- **Line 312**: When no_interactive=false, should check for yes/yes response
- **Line 332**: Prompt should use eprint! to write to stderr
- **Line 337**: Name prompt should also use eprint! to write to stderr
- **Line 343**: Should flush stderr after prompt to ensure it appears
- **Line 368**: When no_interactive=true, should call registry.add with None (use default name)
- **Line 375**: When no_interactive=true, should NOT prompt for custom name in the non-interactive branch
- **Line 383**: Rename prompt should exist in the function (in interactive mode)
- **Line 452**: Prompt should not be shown when no_interactive=true
- **Line 467**: Prompt should be shown when no_interactive=false (default)
- **Line 485**: Registration prompt should be suppressed when no_interactive=true
- **Line 501**: Registration prompt should be shown when no_interactive=false (default)
- **Line 517**: Rename prompt should be suppressed when no_interactive=true
- **Line 533**: Rename prompt should be shown when no_interactive=false (default)
- **Line 549**: Scan should auto-register when no_interactive=true
- **Line 565**: Scan should not auto-register when no_interactive=false (requires confirmation)
- **Line 584**: When no_interactive=true: both prompts suppressed AND auto-registration enabled
- **Line 603**: When no_interactive=false: both prompts shown AND auto-registration disabled
- **Line 652**: Should parse flag before command
- **Line 657**: Should parse flag after command
- **Line 690**: Scan command should have auto_confirm field for local --yes flag
- **Line 709**: Scan --yes flag should be documented as auto-confirming prompts
- **Line 715**: Scan should have local --yes flag defined with arg attribute
- **Line 738**: Handler should combine flags with OR logic: no_interactive || auto_confirm
- **Line 758**: ✓ Scan command has auto_confirm field
- **Line 764**: ✓ Global flag extracted in main
- **Line 770**: ✓ Flags combined with OR logic
- **Line 776**: ✓ Combined value passed to scan_projects
- **Line 782**: ✓ scan_projects accepts no_interactive parameter
- **Line 788**: ✓ scan_projects checks no_interactive flag
- **Line 794**: ✓ Auto-registers without prompting when no_interactive=true
- **Line 800**: ✓ Prompts for confirmation when no_interactive=false
- **Line 806**: ✓ Prompts go to stderr (not stdout)
- **Line 812**: ✓ Skips rename prompt and uses default name when no_interactive=true
- **Line 818**: All Scan command no_interactive tests verified
- **Line 1062**: Behavior: When no_interactive=true, should NOT prompt for registration
- **Line 1067**: Behavior: When no_interactive=true, should NOT prompt for custom name
- **Line 1072**: Behavior: When no_interactive=true, should NOT read from stdin
- **Line 1078**: Behavior: When no_interactive=true, should print 'registering' message
- **Line 1083**: Behavior: When no_interactive=true, should call registry.add() directly without prompting
- **Line 1112**: Behavior: When no_interactive=false, should prompt for registration confirmation
- **Line 1117**: Behavior: When no_interactive=false, should flush stderr after prompt
- **Line 1122**: Behavior: When no_interactive=false, should read user input from stdin
- **Line 1128**: Behavior: When no_interactive=false, should prompt for custom name
- **Line 1134**: Behavior: When no_interactive=false, should process user input
- **Line 1139**: Behavior: When no_interactive=false, should check for yes/yes response
- **Line 1162**: Behavior: Registration result should go to stdout (println!)
- **Line 1168**: Behavior: Registration prompt should go to stderr (eprint!)
- **Line 1173**: Behavior: Name prompt should go to stderr (eprint!)
- **Line 1179**: Behavior: Should flush stderr after prompts to ensure visibility
- **Line 1211**: Behavior: When no_interactive=true, should NOT read from stdin (non-blocking)
- **Line 1218**: Behavior: Stdin reading should only occur in interactive mode (no_interactive=false)
- **Line 1241**: Code must have if no_interactive branch
- **Line 1262**: Else branch must have registration prompt
- **Line 1268**: If branch must NOT have registration prompt
- **Line 1300**: Behavior: When no_interactive=true, should call registry.add with None (use default name)
- **Line 1306**: Behavior: When no_interactive=true, should NOT prompt for custom name

#### hoop-daemon/src/load_test.rs

- **Line 775**: beads.jsonl
- **Line 779**: beads.jsonl
- **Line 783**: beads.jsonl
- **Line 786**: beads.jsonl
- **Line 787**: beads.jsonl
- **Line 788**: beads.jsonl

#### hoop-daemon/tests/acceptance/s1_morning_review.rs

- **Line 52**: Dashboard must include total_workers count
- **Line 59**: total_workers must be numeric, got: {}
- **Line 66**: Dashboard must include total_spend_usd
- **Line 73**: total_spend_usd must be non-negative, got: {}
- **Line 80**: Dashboard must include longest_running array
- **Line 134**: Dashboard must render in under 3 seconds, took: {:?}
- **Line 174**: S1 PASS: All data derived from on-disk event files
- **Line 175**: Failed to spawn daemon
- **Line 176**: Failed to spawn daemon
- **Line 177**: Failed to spawn daemon
- **Line 178**: Failed to spawn daemon
- **Line 179**: Failed to spawn daemon
- **Line 255**: Total cost must be non-negative
- **Line 273**: Sum of project costs ({}) should equal total ({})

#### hoop-daemon/tests/acceptance/s2_transcript_archaeology.rs

- **Line 65**: Bead events endpoint should return 200 or 404, got: {}
- **Line 73**: Events should be an array
- **Line 123**: Visual debug panel must load in under 5 seconds, took: {:?}
- **Line 155**: Stitch read endpoint should return 200 or 404, got: {}
- **Line 193**: Endpoint {} should return 200 or 404, got: {}
- **Line 232**: Conversations should be an array
- **Line 307**: Cost data should be an object
- **Line 360**: Event should have timestamp
- **Line 364**: Event should have type

#### hoop-daemon/tests/acceptance/s3_bead_creation_from_chat.rs

- **Line 151**: Draft should be created within 3 seconds, took {:?}
- **Line 179**: Draft should appear in the draft queue
- **Line 272**: Bead should be created within 3 seconds of approval, took {:?}
- **Line 282**: stub br should record br create call with expected title
- **Line 388**: Audit log should contain DraftCreated entry
- **Line 401**: Audit log should contain DraftApproved entry
- **Line 413**: Operator identity should be present in audit log
- **Line 478**: Draft should be in queue
- **Line 499**: stub br should record the create call
- **Line 522**: Audit should have DraftCreated
- **Line 523**: Audit should have DraftApproved
- **Line 535**: operator identity should be present
- **Line 601**: chat

#### hoop-daemon/tests/acceptance/s4_daemon_restart.rs

- **Line 218**: Worker should have written more events while HOOP was down
- **Line 267**: Bead count should be stable across restart: before={}, after={}
- **Line 357**: UI state should rebuild in under 5 seconds, took: {:?}
- **Line 430**: Worker should continue writing events during HOOP downtime
- **Line 462**: Worker should continue after HOOP restart
- **Line 547**: Beads should not disappear across restarts in cycle {}

#### hoop-daemon/tests/acceptance/s5_workspace_deleted.rs

- **Line 194**: Error card should appear within 10s of workspace deletion
- **Line 285**: Other projects should still be accessible
- **Line 297**: Daemon should still be healthy
- **Line 385**: Should be in degraded state after workspace deletion
- **Line 411**: Auto-recovery should occur within 10s of workspace restore
- **Line 489**: Daemon should still be running after workspace deletion

#### hoop-daemon/tests/acceptance/s6_machine_mode.rs

- **Line 127**: JSON output should be an object
- **Line 128**: JSON output should have 'projects' field
- **Line 138**: Each project should be an object
- **Line 139**: Each project should have 'name' field
- **Line 143**: Each project should have 'workspaces' field
- **Line 147**: Each project should have 'total_beads' field
- **Line 266**: stdout should not contain interactive prompts, got: {}
- **Line 284**: Output should be concise without prompts, got {} lines
- **Line 367**: Error JSON should have 'error' field
- **Line 412**: stdout should not contain prompt '{}' for args {:?}, got: {}
- **Line 504**: JSON output should be pretty-printed
- **Line 512**: Each project should be an object
- **Line 547**: stdout should contain error JSON
- **Line 556**: Error JSON should have error field
- **Line 608**: JSON should be an object

#### hoop-daemon/tests/adapter_failover.rs

- **Line 99**: Adapter build should succeed
- **Line 120**: ZAI adapter build should succeed after Anthropic
- **Line 189**: Stitch title should reference the adapter
- **Line 271**: Archived timestamp should be set
- **Line 448**: Global rule should be preserved
- **Line 449**: Project rule should be preserved
- **Line 613**: Stitch title should reference the adapter
- **Line 617**: Stitch title should indicate it was archived
- **Line 641**: Tool name should be in content
- **Line 716**: Multi-line content should be preserved
- **Line 717**: Quotes should be preserved
- **Line 718**: Code blocks should be preserved
- **Line 795**: First rule should be present
- **Line 799**: Second rule should be present

#### hoop-daemon/tests/adapter_failover_integration.rs

- **Line 73**: Adapter build should succeed
- **Line 93**: ZAI adapter build should succeed after Anthropic
- **Line 186**: Stitch title should reference the old adapter
- **Line 546**: Global rule should be preserved
- **Line 550**: Project rule should be preserved
- **Line 557**: project:hoop
- **Line 558**: project:hoop
- **Line 615**: SELECT status, archived_reason FROM agent_sessions WHERE id = ?1
- **Line 674**: rejected rule
- **Line 675**: rejected rule
- **Line 676**: rejected rule
- **Line 722**: Stitch title should contain the session date
- **Line 726**: Stitch title should contain the session time
- **Line 730**: Stitch title should reference the adapter

#### hoop-daemon/tests/adapter_failover_test.rs

- **Line 235**: Should have at least 2 sessions, got {}
- **Line 304**: Archived session should have a stitch_id linking to the preserved Stitch
- **Line 313**: Stitch should exist in fleet.db
- **Line 323**: Stitch title should reference agent session
- **Line 380**: Reflection entry should persist after adapter switch
- **Line 436**: Should have at least 3 sessions, got {}
- **Line 456**: First archived session should have stitch_id
- **Line 460**: Second archived session should have stitch_id
- **Line 526**: Reflection Ledger entry should be preserved for continuity
- **Line 573**: At least one switch should succeed
- **Line 666**: Should have at least 2 sessions, got {}
- **Line 693**: Archived session should have a stitch_id linking to the preserved Stitch
- **Line 702**: Stitch should exist in fleet.db
- **Line 899**: Should have performed at least 6 health checks over 30s

#### hoop-daemon/tests/agent_turn_audit_trail.rs

- **Line 164**: System message should reference the turn_id

#### hoop-daemon/tests/backup_config_deserialization.rs

- **Line 59**: BackupFileConfig should deserialize
- **Line 85**: bucket

#### hoop-daemon/tests/backup_restore_cycle.rs

- **Line 67**: State should be deleted
- **Line 144**: Should return None when credentials missing
- **Line 153**: Should succeed when encryption disabled
- **Line 158**: age_key should be None when encryption disabled
- **Line 166**: Should succeed when age key provided
- **Line 169**: age_key should be Some when encryption enabled
- **Line 178**: Should return None when age key missing but encryption enabled
- **Line 223**: age encryption should succeed
- **Line 229**: Encrypted file should exist
- **Line 258**: age decryption should succeed with HOOP_BACKUP_AGE_IDENTITY
- **Line 311**: Backup should fail when encryption enabled but age key missing
- **Line 314**: Error should mention encryption or age failure: {}
- **Line 350**: Config should have encryption enabled
- **Line 351**: Credentials should have age key
- **Line 391**: Config should have encryption disabled
- **Line 392**: Credentials should not have age key

#### hoop-daemon/tests/bead_created_by_hoop_broadcast.rs

- **Line 78**: hoop-ttb.3.53
- **Line 79**: hoop-ttb.3.53
- **Line 88**: Notification should be received within 100ms, took {}ms
- **Line 148**: Fleet notification ring should contain bead_created_by_hoop event

#### hoop-daemon/tests/bead_real_line_deserialization.rs

- **Line 66**: title

#### hoop-daemon/tests/beads_deletion_http.rs

- **Line 139**: Daemon should become healthy initially within 10s
- **Line 173**: /readyz should report project-a as degraded within 30s
- **Line 186**: project-a state should not be Healthy, got: {}
- **Line 193**: project-b should not be in degraded list
- **Line 197**: project-c should not be in degraded list
- **Line 216**: project-b should be Healthy or Starting, got: {}
- **Line 223**: project-c should be Healthy or Starting, got: {}
- **Line 255**: Daemon should recover to healthy state after .beads/ restoration
- **Line 265**: Project {} should be Healthy or Starting after recovery, got: {}
- **Line 356**: project-a should be degraded
- **Line 370**: Metrics should still be collected during degradation
- **Line 379**: project-b should be operational, got: {}
- **Line 384**: project-c should be operational, got: {}
- **Line 416**: Should be healthy initially

#### hoop-daemon/tests/beads_deletion_isolation.rs

- **Line 20**: Missing .beads should be a permanent error
- **Line 24**: Missing workspace should be a permanent error
- **Line 28**: Connection errors should not be permanent
- **Line 32**: Timeouts should not be permanent

#### hoop-daemon/tests/beads_removal_recovery.rs

- **Line 192**: Project A should show error state within 30s after .beads/ removal
- **Line 223**: Project B should remain healthy during project A's degradation
- **Line 228**: Project C should remain healthy during project A's degradation
- **Line 256**: /readyz should list project_a as degraded
- **Line 262**: /readyz should NOT list project_b as degraded
- **Line 267**: /readyz should NOT list project_c as degraded
- **Line 282**: Config reload should succeed
- **Line 305**: Project A should recover after .beads/ is restored and config is reloaded
- **Line 410**: At least 2 projects should be degraded
- **Line 434**: Project C should remain healthy even when A and B are degraded

#### hoop-daemon/tests/claimed_at_parsing.rs

- **Line 79**: Valid RFC3339 timestamp should parse
- **Line 87**: Valid RFC3339 timestamp with milliseconds should parse
- **Line 95**: Valid RFC3339 timestamp with offset should parse
- **Line 103**: Empty timestamp should be invalid (reproduces 'premature end of input')
- **Line 111**: Partial timestamp (date only) should be invalid
- **Line 119**: Wrong format timestamp should be invalid
- **Line 127**: Garbage timestamp should be invalid
- **Line 183**: Timestamp '{}' should fail to parse
- **Line 193**: Empty timestamp should produce 'premature end of input' error, got: {}
- **Line 253**: Valid timestamp '{}' should parse successfully
- **Line 300**: Timestamp with {} decimal places should parse: '{}'
- **Line 328**: Timestamp with timezone offset should parse: '{}'
- **Line 367**: Round-tripped timestamp should still be parseable
- **Line 441**: Timestamp with invalid character '{}' should not parse
- **Line 473**: SQL injection attempt '{}' should not parse as valid timestamp
- **Line 485**: SQL injection string should fail to parse: '{}'
- **Line 528**: Negative timestamp (before epoch) '{}' should parse as valid RFC3339
- **Line 539**: Negative timestamp should still be parseable after storage
- **Line 559**: Extreme future date '{}' should parse as valid RFC3339
- **Line 588**: Invalid timezone offset '{}' should not parse
- **Line 671**: Timestamp with special character '{}' should not parse
- **Line 700**: Empty variant '{}' should not parse
- **Line 715**: Empty string should produce 'premature end of input' error, got: {}
- **Line 739**: Timestamp with extra text '{}' should not parse

#### hoop-daemon/tests/config_field_validation.rs

- **Line 43**: missing schema_version should fail
- **Line 45**: error should include field path
- **Line 46**: field path should mention schema_version: {:?}
- **Line 59**: integer schema_version should fail
- **Line 61**: expected should be string: {:?}
- **Line 66**: error should include field path
- **Line 75**: invalid schema_version format should fail
- **Line 77**: error should mention pattern/format: {:?}
- **Line 90**: invalid schema_version text should fail
- **Line 92**: error should mention pattern/format: {:?}
- **Line 109**: missing agent.adapter should fail
- **Line 111**: error should mention adapter: {:?}
- **Line 126**: integer adapter should fail
- **Line 128**: expected should be string: {:?}
- **Line 133**: field path should include adapter: {:?}
- **Line 148**: invalid adapter value should fail
- **Line 150**: error should mention adapter/variant: {:?}
- **Line 165**: null adapter should fail
- **Line 167**: expected should be string: {:?}
- **Line 185**: integer model should fail
- **Line 187**: expected should be string: {:?}
- **Line 192**: field path should include model: {:?}
- **Line 209**: object model should fail
- **Line 211**: expected should be string: {:?}
- **Line 228**: integer bind_addr should fail
- **Line 230**: expected should be string: {:?}
- **Line 235**: field path should include bind_addr: {:?}
- **Line 252**: object bind_addr should fail
- **Line 254**: expected should be string: {:?}
- **Line 271**: string metrics.enabled should fail
- **Line 273**: expected should be boolean: {:?}
- **Line 278**: field path should include enabled: {:?}
- **Line 293**: integer metrics.enabled should fail
- **Line 295**: expected should be boolean: {:?}
- **Line 312**: string metrics.port should fail
- **Line 314**: expected should be integer: {:?}
- **Line 319**: field path should include port: {:?}
- **Line 337**: error should mention port/range: {:?}
- **Line 355**: string retention_days should fail
- **Line 357**: expected should be integer: {:?}
- **Line 362**: field path should include retention_days: {:?}
- **Line 377**: boolean retention_days should fail
- **Line 379**: expected should be integer: {:?}
- **Line 396**: string hash_chain should fail
- **Line 398**: expected should be boolean: {:?}
- **Line 413**: integer hash_chain should fail
- **Line 415**: expected should be boolean: {:?}
- **Line 432**: integer ui.theme should fail
- **Line 434**: expected should be string: {:?}
- **Line 449**: invalid ui.theme value should fail
- **Line 451**: error should mention theme/variant: {:?}
- **Line 466**: boolean ui.theme should fail
- **Line 468**: expected should be string: {:?}
- **Line 485**: string archive_after_days should fail
- **Line 487**: expected should be integer: {:?}
- **Line 502**: boolean archive_after_days should fail
- **Line 504**: expected should be integer: {:?}
- **Line 521**: string reflection.enabled should fail
- **Line 523**: expected should be boolean: {:?}
- **Line 540**: string detection_threshold should fail
- **Line 542**: expected should be number: {:?}
- **Line 557**: boolean detection_threshold should fail
- **Line 559**: expected should be number: {:?}
- **Line 576**: string auto_archive_after_days should fail
- **Line 578**: expected should be integer: {:?}
- **Line 595**: string roles.viewers should fail (must be array)
- **Line 597**: expected should be array: {:?}
- **Line 614**: integer in viewers array should fail
- **Line 616**: expected should be string: {:?}
- **Line 633**: string roles.drafters should fail (must be array)
- **Line 635**: expected should be array: {:?}
- **Line 652**: integer agent_extensions.skills should fail
- **Line 654**: expected should be string: {:?}
- **Line 670**: array agent_extensions.scripts should fail
- **Line 672**: expected should be string: {:?}
- **Line 688**: missing project name should fail
- **Line 690**: error should mention name: field={:?}, message={:?}
- **Line 706**: integer project name should fail
- **Line 708**: expected should be string: {:?}
- **Line 722**: missing project path should fail
- **Line 724**: error should mention path: field={:?}, message={:?}
- **Line 740**: integer project path should fail
- **Line 742**: expected should be string: {:?}
- **Line 757**: boolean project path should fail
- **Line 759**: expected should be string: {:?}
- **Line 775**: integer project label should fail
- **Line 777**: expected should be string: {:?}
- **Line 793**: integer project color should fail
- **Line 795**: expected should be string: {:?}
- **Line 811**: string project disabled should fail
- **Line 813**: expected should be boolean: {:?}
- **Line 826**: non-array projects should fail
- **Line 828**: error should mention array: expected={:?}, message={:?}
- **Line 843**: string in projects array should fail
- **Line 845**: error should mention object: expected={:?}, message={:?}
- **Line 862**: unknown field should be rejected
- **Line 864**: error should mention unknown field: {:?}
- **Line 880**: unknown nested field should be rejected
- **Line 882**: error should mention unknown field: {:?}
- **Line 898**: unknown nested field in ui should be rejected
- **Line 900**: error should mention unknown field: {:?}
- **Line 916**: unknown field in project entry should be rejected
- **Line 918**: error should mention unknown field: {:?}
- **Line 933**: unclosed quote should fail
- **Line 935**: error should include line number: {:?}
- **Line 950**: unmatched bracket should fail
- **Line 952**: error should include line number: {:?}
- **Line 967**: invalid escape sequence should fail
- **Line 969**: error should include line number: {:?}
- **Line 986**: trailing comma should fail
- **Line 988**: error should include line number: {:?}
- **Line 1005**: error should include line number: {:?}
- **Line 1010**: error should include column number: {:?}
- **Line 1025**: error should include field path: {:?}
- **Line 1031**: field path should mention adapter: {:?}
- **Line 1046**: error should include expected type: {:?}
- **Line 1051**: error should include actual type: {:?}
- **Line 1066**: error message should not be empty
- **Line 1070**: error message should be concise (got {} chars)

#### hoop-daemon/tests/config_reload_audit.rs

- **Line 88**: delta should include +project:proj-two, got: {:?}
- **Line 118**: hash chain must advance
- **Line 119**: hash chain must advance
- **Line 120**: hash chain must advance
- **Line 134**: fetched delta_keys should contain +project:proj-two
- **Line 258**: should have -project:proj-two, got: {:?}
- **Line 263**: should have ~project:test-proj.paths (path changed repo1→repo2), got: {:?}
- **Line 342**: delta should reflect proj-two was added

#### hoop-daemon/tests/config_reload_cycle.rs

- **Line 106**: content hash must be set
- **Line 111**: truncated YAML must be rejected
- **Line 144**: delta should show proj-beta added, got: {:?}
- **Line 153**: missing field must be rejected
- **Line 172**: delta should show proj-beta removed, got: {:?}
- **Line 261**: missing name should fail
- **Line 264**: missing field error should have location info: line={}, field={:?}
- **Line 270**: error message should not be empty
- **Line 274**: integer name should fail
- **Line 277**: type error should report line number: line={}
- **Line 282**: type error should have structured details: expected={:?}, got={:?}, field={:?}
- **Line 292**: truncated YAML should fail
- **Line 295**: parse error should report line number: line={}
- **Line 367**: should detect at least 2 semantic errors (no .beads + missing path), got: {:?}
- **Line 377**: should detect missing .beads for no-beads-proj, got: {:?}
- **Line 383**: semantic error should have field path
- **Line 384**: expected should say what's needed
- **Line 392**: should detect nonexistent path for missing-path-proj, got: {:?}
- **Line 398**: missing path error should have field
- **Line 399**: expected should say 'existing directory'
- **Line 421**: valid config should pass validation
- **Line 432**: nonexistent path should fail validation
- **Line 450**: fixed config should pass validation
- **Line 458**: delta should show good-proj removed, got: {:?}
- **Line 463**: delta should show another-proj added, got: {:?}
- **Line 504**: rejected row should have error message

#### hoop-daemon/tests/create_only_stub.rs

- **Line 106**: fake br should succeed
- **Line 201**: expected invocation to start with '{}', got '{}'
- **Line 211**: read verb '{}' classified as write — this is a bug
- **Line 236**: '{}' missing from FORBIDDEN_WRITE_VERBS
- **Line 241**: '{}' not detected as forbidden
- **Line 249**: create
- **Line 250**: create
- **Line 260**: assert_create_only('{}') should have panicked
- **Line 271**: panic message should mention create-only invariant, got: {}
- **Line 308**: create
- **Line 337**: validate_br_subprocess_args should reject raw '{}' command
- **Line 371**: fake br should succeed for '{}'
- **Line 391**: first invocation should contain title
- **Line 395**: should contain stitch label

#### hoop-daemon/tests/create_stitch_no_auto_submit.rs

- **Line 299**: draft must NOT have stitch_id until approved (combo: {})
- **Line 306**: draft must NOT be in 'submitted' status immediately after creation (combo: {})
- **Line 312**: draft must NOT be in 'approved' status immediately after creation (combo: {})
- **Line 379**: stitch_id must be None before approval
- **Line 506**: create temp HOOP home
- **Line 507**: create temp HOOP home
- **Line 651**: Property violation for combo '{}': stitch_id must be None after creation, got {:?}

#### hoop-daemon/tests/cross_workspace_blockers.rs

- **Line 191**: workspace_from column should exist
- **Line 202**: workspace_to column should exist

#### hoop-daemon/tests/disaster_recovery_runbook.rs

- **Line 164**: fresh host has no ~/.hoop/
- **Line 180**: projects restored
- **Line 196**: newer version is rejected
- **Line 199**: error message mentions version mismatch
- **Line 203**: error suggests upgrading
- **Line 224**: corrupted database fails to open
- **Line 251**: corrupted database is preserved
- **Line 252**: filename indicates corruption
- **Line 273**: ~/.hoop/ is gone after deletion
- **Line 277**: ~/.hoop/ is gone after deletion
- **Line 285**: fleet.db restored
- **Line 286**: projects.yaml restored
- **Line 289**: projects.yaml
- **Line 312**: config.yml
- **Line 314**: config.yml
- **Line 368**: paths updated for new host
- **Line 389**: fleet.db
- **Line 412**: PRAGMA integrity_check
- **Line 413**: ok
- **Line 425**: ok
- **Line 426**: ok
- **Line 456**: projects.yaml
- **Line 457**: projects.yaml
- **Line 479**: local restore completes in seconds
- **Line 492**: corruption recovery is fast locally
- **Line 509**: AGE_IDENTITY
- **Line 510**: AGE_IDENTITY
- **Line 511**: AGE_IDENTITY
- **Line 522**: control.sock
- **Line 523**: control.sock
- **Line 540**: 99.0.0
- **Line 554**: mentions snapshot version
- **Line 557**: mentions snapshot version
- **Line 558**: mentions current version
- **Line 559**: explains the problem
- **Line 569**: older schema version accepted
- **Line 591**: {} has test coverage

#### hoop-daemon/tests/draft_queue_invariants.rs

- **Line 89**: draft must not have stitch_id until approved
- **Line 209**: fleet.db must persist on disk
- **Line 272**: draft-s6
- **Line 340**: audit row should be written successfully
- **Line 347**: hash_self must be populated
- **Line 351**: hash_prev must be populated (genesis or previous)
- **Line 426**: approved
- **Line 652**: first edit must store original_json
- **Line 783**: opened_at should be set
- **Line 814**: abandoned_at should be set
- **Line 825**: abandoned_at should be cleared on reopen
- **Line 826**: abandoned_at should be cleared on reopen
- **Line 862**: last_autosave_at should be set
- **Line 902**: abandoned
- **Line 913**: abandoned_at should be set
- **Line 1047**: old abandoned draft should be deleted
- **Line 1076**: frontend
- **Line 1094**: Updated description with more details
- **Line 1116**: abandoned draft should still exist

#### hoop-daemon/tests/epoch_sync_invariant.rs

- **Line 50**: init should contain subscriptions array
- **Line 63**: global should always be in subscriptions
- **Line 101**: Should receive at least one message
- **Line 128**: Should receive workers_snapshot after init
- **Line 132**: Should receive beads_snapshot after init
- **Line 136**: Should receive config_status after init
- **Line 226**: Reconnect should receive init event
- **Line 227**: Reconnect should receive beads_snapshot
- **Line 332**: Connection should receive init

#### hoop-daemon/tests/filesystem_failure_isolation.rs

- **Line 177**: No projects should be degraded initially
- **Line 206**: project-b should not be degraded
- **Line 210**: project-c should not be degraded
- **Line 222**: project-a should have an error message
- **Line 229**: Error message should mention .beads: {}
- **Line 245**: project-a should be detected as degraded within 30 seconds
- **Line 327**: No projects should be degraded initially
- **Line 361**: project-a should be detected as degraded within 30 seconds
- **Line 397**: project-a should recover after .beads/ is restored
- **Line 516**: project-a should be degraded
- **Line 568**: project-b should continue sending status events
- **Line 572**: project-c should continue sending status events

#### hoop-daemon/tests/fix_patterns_integration.rs

- **Line 56**: create should return non-empty ID
- **Line 101**: pattern should be deleted
- **Line 178**: first match similarity should be > 0.99
- **Line 179**: first match similarity should be > 0.99
- **Line 182**: first match similarity should be > 0.99
- **Line 186**: second match similarity should be > 0.99
- **Line 193**: different pattern should have similarity > 0.5
- **Line 296**: should find 1 pattern with 'bounds'
- **Line 297**: should find 1 pattern with 'bounds'
- **Line 364**: orthogonal vectors should have near-zero similarity

#### hoop-daemon/tests/fleet_notifications_integration.rs

- **Line 34**: Notification should be delivered within 5 seconds
- **Line 62**: Should deserialize from JSON
- **Line 63**: Should deserialize from JSON
- **Line 64**: Should deserialize from JSON
- **Line 65**: Should deserialize from JSON
- **Line 66**: Should deserialize from JSON
- **Line 67**: test-project
- **Line 181**: Fleet notifications in context should not exceed RING_SIZE

#### hoop-daemon/tests/golden_transcripts_regression.rs

- **Line 61**: testrepo/golden-transcripts/ must exist — create it with adapter subdirectories
- **Line 65**: testrepo/golden-transcripts/ must be a directory
- **Line 77**: testrepo/golden-transcripts/{adapter}/{VERSION}/ must exist — create adapter directory
- **Line 81**: testrepo/golden-transcripts/{adapter}/{VERSION}/ must be a directory
- **Line 95**: testrepo/golden-transcripts/{adapter}/{VERSION}/{scenario}/ must exist
- **Line 99**: testrepo/golden-transcripts/{adapter}/{VERSION}/{scenario}/ must be a directory
- **Line 119**: testrepo/golden-transcripts/{adapter}/{VERSION}/{scenario}/ must contain at least one .jsonl file
- **Line 149**: Golden transcripts corpus must be < 10MB, currently {} bytes
- **Line 217**: Golden transcript file {:?} must contain at least one non-empty JSON line
- **Line 280**: Simple turn scenario {:?} for adapter '{}' must contain at least one text event
- **Line 344**: Tool-heavy scenario {:?} for adapter '{}' must contain at least one tool event
- **Line 391**: Failure scenario {:?} for adapter '{}' must contain at least one error indication
- **Line 405**: AdapterKind::from_config must recognize adapter '{}': add it to the enum
- **Line 421**: Adapter path {:?} must be a directory
- **Line 431**: Version path {:?} must be a directory
- **Line 442**: Scenario path {:?} must be a directory
- **Line 467**: Corpus should only contain .jsonl files (plus README.md), found {:?}
- **Line 480**: testrepo/golden-transcripts/README.md must exist — document the fixture format
- **Line 509**: Failed to parse line {} of {:?}:\n  Line: {}\n  Error: {:?}
- **Line 561**: Simple turn scenario {:?} for adapter '{}' must parse to at least one TextDelta event
- **Line 613**: Tool-heavy scenario {:?} for adapter '{}' must parse to at least one ToolUse event
- **Line 619**: Tool-heavy scenario {:?} for adapter '{}' must parse to at least one ToolResult event
- **Line 665**: Failure scenario {:?} for adapter '{}' must parse to at least one Error event

#### hoop-daemon/tests/hoop_dies_nothing_notices.rs

- **Line 211**: worker should have written at least 2 events
- **Line 215**: events.jsonl should contain at least 2 events
- **Line 232**: should be able to parse at least 2 events from events.jsonl
- **Line 249**: worker should continue writing events during HOOP absence
- **Line 256**: events.jsonl should contain events written during HOOP absence
- **Line 320**: iteration {}: should have at least 6 events before HOOP starts
- **Line 339**: iteration {}: worker should write more events during HOOP absence
- **Line 346**: iteration {}: events.jsonl should grow during HOOP absence
- **Line 450**: Rebuild should complete in < 5s, took {:?}
- **Line 505**: events should accumulate across restarts
- **Line 590**: fleet.db should persist across restarts
- **Line 672**: should still parse all valid events

#### hoop-daemon/tests/integration_harness.rs

- **Line 445**: Events fixture should contain {} event
- **Line 470**: Heartbeats should contain idle state
- **Line 474**: Heartbeats should contain executing state
- **Line 522**: Event {}: bead_id should not be empty
- **Line 527**: Event {}: worker should not be empty
- **Line 544**: testrepo should exist within the repository
- **Line 551**: events.jsonl should be in the repository
- **Line 561**: Parsing events should be fast (< 1s), took {:?}
- **Line 773**: projects should be a list
- **Line 810**: init should contain subscriptions
- **Line 959**: testrepo should be in projects list
- **Line 986**: bead id should not be empty
- **Line 987**: bead title should not be empty
- **Line 988**: bead project should not be empty
- **Line 1012**: Metrics should contain hoop_ prefixed metrics
- **Line 1066**: Should receive init event
- **Line 1067**: Should receive workers_snapshot event
- **Line 1071**: Should receive beads_snapshot event
- **Line 1167**: Integration test should complete quickly, took {:?}
- **Line 1307**: Bead creation should succeed
- **Line 1328**: Should be able to fetch beads
- **Line 1410**: Non-existent bead should return error
- **Line 1421**: Invalid JSON should return error
- **Line 1439**: Metrics endpoint should return 200
- **Line 1444**: Metrics should not be empty
- **Line 1456**: Metrics should contain at least one valid metric line
- **Line 1476**: File listing should succeed
- **Line 1481**: Files should be an array or object
- **Line 1506**: Bead creation should succeed
- **Line 1518**: Getting bead should succeed
- **Line 1531**: Listing beads should succeed
- **Line 1535**: New bead should appear in list
- **Line 1553**: Capacity endpoint should return 200
- **Line 1558**: Capacity should be object or array
- **Line 1576**: Config status endpoint should return 200
- **Line 1581**: Config status should include 'valid' field

#### hoop-daemon/tests/lint_regex_global_state.rs

- **Line 142**: Synthetic violation should have been detected
- **Line 147**: test 123
- **Line 189**: Safe patterns should not trigger violations: {:?}

#### hoop-daemon/tests/load_test.rs

- **Line 66**: load-test-project-
- **Line 95**: beads.jsonl
- **Line 103**: beads.jsonl
- **Line 132**: Memory
- **Line 133**: Memory
- **Line 134**: Memory
- **Line 135**: Memory
- **Line 136**: Memory
- **Line 220**: Small-scale load test should pass performance budgets
- **Line 285**: Should process events
- **Line 286**: Should measure API latencies
- **Line 296**: P95 API latency {}ms should be within budget {}ms

#### hoop-daemon/tests/load_test_integration.rs

- **Line 130**: Endpoint {} returned unexpected status: {}
- **Line 144**: Max API latency {}ms exceeds budget {}ms (avg: {}ms)
- **Line 196**: Max memory {}MB exceeds ceiling {}MB
- **Line 264**: 95th percentile latency {}ms exceeds budget {}ms
- **Line 360**: Load test should pass all budgets
- **Line 492**: Load test should pass all budgets
- **Line 535**: Daemon startup took {:?}, expected < 30s
- **Line 577**: Memory usage {}MB exceeds ceiling

#### hoop-daemon/tests/multi_operator_concurrency.rs

- **Line 187**: last_autosave_at should be set
- **Line 236**: abandoned_at should be set
- **Line 287**: should detect similar existing draft
- **Line 331**: One incident of corruption
- **Line 332**: One incident of corruption
- **Line 333**: proposed
- **Line 372**: proposal should be approved
- **Line 385**: should have approved entries
- **Line 422**: proposal should be rejected
- **Line 430**: rejected proposal should not appear in proposed list

#### hoop-daemon/tests/mutation_handler_test.rs

- **Line 162**: draft-123
- **Line 166**: draft-123
- **Line 173**: Should include error in broadcast state
- **Line 180**: pending
- **Line 204**: draft-456
- **Line 207**: draft-456
- **Line 208**: draft-456
- **Line 214**: Some Title
- **Line 217**: Some Title
- **Line 218**: Some Title
- **Line 237**: draft-789
- **Line 240**: draft-789
- **Line 241**: draft-789
- **Line 248**: Client's state update path is the same whether the     /// update was accepted or rejected.
- **Line 251**: test-user
- **Line 252**: test-user
- **Line 282**: Title B
- **Line 289**: Should reject empty title
- **Line 294**: Error present for UI to display
- **Line 317**: pending
- **Line 319**: pending
- **Line 324**: pending
- **Line 326**: pending
- **Line 332**: pending
- **Line 334**: pending
- **Line 346**: permission
- **Line 368**: Final Title
- **Line 370**: Final Title
- **Line 374**: test-user
- **Line 376**: Title
- **Line 440**: Database

#### hoop-daemon/tests/needle_events_roundtrip.rs

- **Line 48**: testrepo/.beads/events.jsonl must exist — it is the canonical NEEDLE event schema reference
- **Line 52**: testrepo/.beads/heartbeats.jsonl must exist — it is the canonical NEEDLE heartbeat schema reference
- **Line 74**: fixture must contain at least one '{event_type}' event — add one to testrepo/.beads/events.jsonl
- **Line 91**: fixture must contain at least one '{state}' heartbeat — add one to testrepo/.beads/heartbeats.jsonl
- **Line 115**: claim: worker must be non-empty
- **Line 116**: claim: bead must start with 'bd-'
- **Line 117**: claim in fixture should include strand field
- **Line 142**: dispatch: worker must be non-empty
- **Line 143**: dispatch: bead must start with 'bd-'
- **Line 147**: dispatch in fixture should include adapter
- **Line 151**: dispatch in fixture should include model
- **Line 174**: complete: worker must be non-empty
- **Line 175**: complete: bead must start with 'bd-'
- **Line 179**: complete in fixture should include outcome
- **Line 183**: complete in fixture should include duration_ms
- **Line 187**: complete in fixture should include exit_code
- **Line 212**: fail: worker must be non-empty
- **Line 213**: fail: bead must start with 'bd-'
- **Line 214**: fail in fixture should include error
- **Line 215**: fail in fixture should include duration_ms
- **Line 234**: release: worker must be non-empty
- **Line 235**: release: bead must start with 'bd-'
- **Line 254**: timeout: worker must be non-empty
- **Line 255**: timeout: bead must start with 'bd-'
- **Line 279**: crash: worker must be non-empty
- **Line 280**: crash: bead must start with 'bd-'
- **Line 281**: crash in fixture should include exit_code
- **Line 300**: line {} parsed as Unknown — add the event type to NeedleEvent: {line}
- **Line 323**: from_event returned None for recognized event on line {}: {line}
- **Line 329**: BeadEventData must have bead_id (line {})
- **Line 334**: BeadEventData must have worker (line {})
- **Line 339**: BeadEventData must have event_type (line {})
- **Line 456**: heartbeat: worker must be non-empty
- **Line 459**: executing: bead must start with 'bd-'
- **Line 463**: executing: pid must be positive
- **Line 464**: executing: adapter must be non-empty
- **Line 481**: heartbeat: worker must be non-empty
- **Line 482**: heartbeat: worker must be non-empty
- **Line 496**: heartbeat: worker must be non-empty
- **Line 499**: knot: reason must be non-empty
- **Line 515**: heartbeat line {} failed to parse: {line}
- **Line 535**: heartbeat line {} has invalid/epoch timestamp
- **Line 585**: worker={worker}, bead={bead}: consecutive heartbeats must be ~10s apart (got {gap}s)

#### hoop-daemon/tests/orphans_integration.rs

- **Line 83**: open
- **Line 84**: open
- **Line 113**: fleet.db
- **Line 114**: fleet.db
- **Line 115**: fleet.db
- **Line 151**: attach_orphan_to_stitch should succeed
- **Line 162**: stitch_beads link should exist with relationship='referenced'
- **Line 171**: duplicate attach should succeed (idempotent)
- **Line 230**: attach should succeed when link already exists

#### hoop-daemon/tests/output_capture_helpers/mod.rs

- **Line 791**: Verification should pass when content matches
- **Line 793**: Verification should fail when content differs
- **Line 810**: Verification should fail when content differs
- **Line 811**: Short\nExtra\n
- **Line 812**: Verification should fail when lengths differ
- **Line 813**: Verification should fail when lengths differ
- **Line 814**: Verification should fail when lengths differ
- **Line 831**: Verification should fail when lengths differ
- **Line 850**: Should handle unicode and special characters
- **Line 893**: Large output verification should pass
- **Line 923**: context...
- **Line 936**: column 10
- **Line 937**: column 10
- **Line 938**: column 10
- **Line 939**: column 10

#### hoop-daemon/tests/panic_isolation.rs

- **Line 73**: test error
- **Line 80**: test error
- **Line 89**: Workspace path does not exist: /test
- **Line 90**: Workspace path does not exist: /test
- **Line 96**: Workspace path does not exist: /test
- **Line 104**: Panic: synthetic panic
- **Line 105**: Panic: synthetic panic
- **Line 106**: Panic: synthetic panic

#### hoop-daemon/tests/path_traversal_hardening.rs

- **Line 33**: dotdot-slash traversal must be rejected at the ID validator
- **Line 45**: multi-segment dotdot traversal must be rejected at the ID validator
- **Line 57**: null-byte injection must be rejected at the ID validator
- **Line 68**: absolute-path bead ID must be rejected at the ID validator
- **Line 79**: tilde-expansion attempt must be rejected at the ID validator
- **Line 91**: percent-encoded traversal must be rejected at the ID validator
- **Line 95**: mixed percent/slash traversal must be rejected at the ID validator
- **Line 107**: Unicode lookalike dot must be rejected at the ID validator
- **Line 118**: backslash separator must be rejected at the ID validator
- **Line 129**: traversal after valid-looking prefix must be rejected at the ID validator
- **Line 162**: symlink escaping the allowlist must be rejected by canonicalize_and_check
- **Line 172**: dangling symlink must also be rejected (canonicalize fails)
- **Line 185**: worker.name
- **Line 186**: worker.name
- **Line 187**: worker.name
- **Line 188**: worker.name
- **Line 195**: ../../etc/shadow
- **Line 196**: ../../etc/shadow
- **Line 197**: ../../etc/shadow
- **Line 223**: response body must not echo attack input {:?}: body was {:?}
- **Line 231**: response body must not contain path separators
- **Line 250**: upload dir inside allowlist must be accepted
- **Line 269**: symlink escaping uploads dir must be rejected
- **Line 303**: symlink-escaped upload directory must be rejected

#### hoop-daemon/tests/pattern_query_evaluator_integration.rs

- **Line 172**: query should match the stitch title
- **Line 173**: query should not be slow
- **Line 181**: first insert should succeed
- **Line 189**: second insert should return false (idempotent)
- **Line 339**: p0
- **Line 340**: p0
- **Line 341**: p0
- **Line 367**: should parse query '{}': {:?}
- **Line 375**: AND query should match
- **Line 382**: NOT query should match
- **Line 389**: OR query should match
- **Line 396**: non-matching query should not match
- **Line 420**: kind:operator should match operator stitch
- **Line 423**: kind:operator should not match worker stitch
- **Line 439**: standalone word should match as label
- **Line 444**: non-matching standalone word should not match

#### hoop-daemon/tests/per_project_redaction_integration.rs

- **Line 93**: customer-data should have redaction policy
- **Line 108**: internal-tools should have redaction policy
- **Line 123**: legacy-project should not have redaction override
- **Line 184**: customer-data with reject policy should block attachment with secret
- **Line 190**: internal-tools
- **Line 198**: internal-tools with warn policy should allow attachment with secret
- **Line 209**: legacy-project with global warn policy should allow attachment with secret
- **Line 261**: customer-data should block content with {}
- **Line 277**: customer-data should allow clean content
- **Line 307**: customer-data should block Anthropic keys
- **Line 316**: customer-data should allow GitHub tokens when not in pattern list
- **Line 363**: customer-data
- **Line 364**: customer-data
- **Line 384**: Initial: customer-data should block secrets
- **Line 407**: After hot-reload: customer-data should allow secrets with warn policy

#### hoop-daemon/tests/performance_budget.rs

- **Line 129**: /healthz took {}ms, budget is {}ms
- **Line 145**: /readyz took {}ms, budget is {}ms
- **Line 161**: /api/projects took {}ms, budget is {}ms
- **Line 185**: /metrics took {}ms, budget is {}ms
- **Line 194**: Daemon using {}MB RAM, budget is {}MB
- **Line 199**: Daemon using {}MB RAM, budget is {}MB
- **Line 292**: /readyz took {}ms, budget is {}ms

#### hoop-daemon/tests/phase2_exit_gate.rs

- **Line 415**: Phase 2 exit gate FAILED: {} of 13 core deliverables lack passing tests. \         Marquee features (14-17) cannot merge until all core deliverables are verified.
- **Line 439**: Phase 2 must have exactly 13 core deliverables
- **Line 440**: Phase 2 must have exactly 13 core deliverables
- **Line 441**: Phase 2 must have exactly 13 core deliverables

#### hoop-daemon/tests/privacy_surface_audit.rs

- **Line 44**: screen-capture frame with Anthropic key must be flagged; got: {findings:?}
- **Line 48**: anthropic_api_key
- **Line 60**: screen-capture frame with GitHub token must be flagged; got: {findings:?}
- **Line 64**: github_token_ghp
- **Line 75**: clean frame text must produce no findings; got: {findings:?}
- **Line 86**: voice transcript with Anthropic key must be flagged; got: {findings:?}
- **Line 90**: anthropic_api_key
- **Line 101**: voice transcript with JWT must be flagged; got: {findings:?}
- **Line 105**: jwt
- **Line 117**: voice transcript with env-var secret must be flagged; got: {findings:?}
- **Line 129**: clean voice transcript must produce no findings; got: {findings:?}
- **Line 141**: should find secrets
- **Line 145**: at least one finding should overlap the key portion
- **Line 152**: finding match_len must be > 0; got: {f:?}
- **Line 165**: draft title with embedded key must be flagged; got: {findings:?}
- **Line 180**: draft body with AWS key must be flagged; got: {findings:?}
- **Line 184**: aws_access_key
- **Line 236**: JSON-style secret field in draft body must be flagged; got: {findings:?}
- **Line 240**: json_secret_field
- **Line 253**: clean draft must produce no findings; got: {findings:?}
- **Line 265**: draft body with Bearer token must be flagged; got: {findings:?}
- **Line 287**: morning brief with embedded API key must be flagged; got: {findings:?}
- **Line 291**: anthropic_api_key
- **Line 307**: morning brief with GitHub token must be flagged; got: {findings:?}
- **Line 311**: github_token_ghp
- **Line 326**: clean morning brief must produce no findings; got: {findings:?}
- **Line 338**: propagation draft with key in title must be flagged; got: {findings:?}
- **Line 353**: propagation draft with AWS key in body must be flagged; got: {findings:?}
- **Line 357**: aws_access_key
- **Line 370**: clean propagation draft must produce no findings; got: {findings:?}
- **Line 383**: JWT in propagation draft must be flagged to prevent lateral leak; got: {findings:?}
- **Line 387**: jwt
- **Line 459**: Phase
- **Line 460**: Phase
- **Line 461**: Phase
- **Line 462**: Phase
- **Line 463**: Phase

#### hoop-daemon/tests/projection_file_audit.rs

- **Line 287**: scanner failed to detect worker_state.json write — scanner is broken
- **Line 291**: violation must be on the worker_state line, got: {:?}
- **Line 307**: fleet_status.json write must be detected
- **Line 320**: live-workers.json write must be detected
- **Line 333**: fleet_state.json create must be detected
- **Line 346**: fleet_state.yaml write must be detected
- **Line 359**: live-fleet.json write must be detected
- **Line 380**: innocent filenames must not trigger; violations on: {:?}
- **Line 404**: allowlist must suppress the matched violation; got: {:?}
- **Line 425**: allowlist must not suppress violations in non-matching files

#### hoop-daemon/tests/property_invariants.rs

- **Line 119**: Event timestamp out of order: {} < {}
- **Line 168**: Claim while already dispatched: claimed={:?}, dispatched={:?}
- **Line 177**: Dispatch without Claim
- **Line 181**: Double dispatch: claimed={:?}, dispatched={:?}
- **Line 190**: Terminal event {:?} without Dispatch: claimed={:?}, dispatched={:?}
- **Line 467**: Expected Quiet status, got {:?}
- **Line 540**: Quiet days decreased: {} -> {}
- **Line 543**: Quiet days decreased: {} -> {}
- **Line 776**: Should have at most 1 event when split in middle, got {}
- **Line 894**: ts
- **Line 895**: ts

#### hoop-daemon/tests/protocol_contract.rs

- **Line 154**: AggregatedStitchResponse must serialize '{}' (declared in fixture)
- **Line 164**: StitchRow must serialize '{}' (declared in fixture)
- **Line 174**: StitchMessage must serialize '{}' (declared in fixture)
- **Line 187**: CostDuration must serialize '{}' (declared in fixture)
- **Line 228**: running
- **Line 256**: test-project
- **Line 266**: daemon not running
- **Line 311**: init event must have 'subscriptions'
- **Line 315**: subscriptions must be an array
- **Line 370**: worker_update must have 'worker'
- **Line 406**: workers_snapshot must have 'workers'
- **Line 435**: beads_snapshot must have 'beads'
- **Line 458**: config_status must have 'config_status'
- **Line 485**: stitch_created must have 'stitch_created'
- **Line 510**: bead_created_by_hoop must have 'bead_created_by_hoop'
- **Line 540**: draft_update must have 'draft_update'
- **Line 567**: collision_alert must have 'collision_alert'
- **Line 592**: morning_brief must have 'morning_brief'
- **Line 625**: projects_snapshot must have 'projects'
- **Line 698**: fixture {} must be a JSON object

#### hoop-daemon/tests/pure_functions.rs

- **Line 148**: alpha
- **Line 183**: world
- **Line 205**: subdir
- **Line 218**: http://www.w3.org/2000/svg
- **Line 219**: sanitize should not fail
- **Line 228**: sanitize should not fail
- **Line 239**: onclick
- **Line 241**: sanitize should not fail
- **Line 248**: sanitize should not fail
- **Line 250**: sanitize should not fail
- **Line 261**: Working on {{project}}
- **Line 263**: Working on myproject
- **Line 270**: Working on myproject
- **Line 288**: file
- **Line 334**: ANSI strip too slow: {:?}
- **Line 351**: ANSI strip too slow: {:?}
- **Line 359**: Cost functions too slow: {:?}
- **Line 368**: Embedding too slow: {:?}
- **Line 376**: Similarity too slow: {:?}
- **Line 398**: Status derivation too slow: {:?}
- **Line 406**: Tag join too slow: {:?}
- **Line 415**: Prompt substitute too slow: {:?}
- **Line 447**: hello
- **Line 469**: p p p
- **Line 483**: deep/nested/path
- **Line 487**: deep/nested/path
- **Line 492**: onclick
- **Line 493**: onclick
- **Line 500**: onclick
- **Line 504**: onclick
- **Line 509**: onclick
- **Line 510**: onclick

#### hoop-daemon/tests/quarantine_integration.rs

- **Line 62**: quarantine dir should exist
- **Line 79**: test.jsonl
- **Line 80**: test.jsonl
- **Line 126**: HOOP_QUARANTINE_DIR

#### hoop-daemon/tests/reflection_detector_integration.rs

- **Line 168**: run_detection should succeed
- **Line 189**: Rule should mention unwrap or don't: {}
- **Line 232**: Should propose 1 preference pattern
- **Line 270**: Should propose 1 correction pattern
- **Line 323**: Should not propose patterns: worker stitches ignored, operator below threshold
- **Line 381**: Should detect at least 1 pattern from synthetic fixtures, got {}
- **Line 384**: Should detect at least 1 pattern from synthetic fixtures, got {}
- **Line 405**: At least one pattern should span 3+ Stitches
- **Line 443**: Should not propose patterns: old stitches outside window
- **Line 468**: operator+operator should be true
- **Line 472**: worker+fleet should be false
- **Line 476**: operator+fleet should be false
- **Line 480**: nonexistent stitch should be false
- **Line 554**: build_reflection_rules_with_audit should succeed
- **Line 556**: SELECT id, kind, target, args_json FROM actions WHERE kind = 'reflection_injected'
- **Line 557**: SELECT id, kind, target, args_json FROM actions WHERE kind = 'reflection_injected'
- **Line 583**: SELECT id, last_applied, applied_count FROM reflection_ledger WHERE status = 'approved'
- **Line 605**: last_applied should be set
- **Line 611**: applied_count should be 2 after second injection

#### hoop-daemon/tests/risk_patterns_standalone.rs

- **Line 12**: Library should contain all patterns passed to from_patterns()
- **Line 24**: Library should contain expected pattern IDs
- **Line 33**: missing_test_coverage
- **Line 37**: Need to generate lots of code
- **Line 48**: codegen
- **Line 64**: Test fix
- **Line 95**: large_codegen_stack_overflow
- **Line 102**: large_codegen_stack_overflow
- **Line 105**: large_codegen_stack_overflow
- **Line 108**: missing_test_coverage
- **Line 197**: Combined match should have higher confidence

#### hoop-daemon/tests/s1_morning_review.rs

- **Line 52**: Dashboard must include total_workers count
- **Line 59**: total_workers must be numeric, got: {}
- **Line 66**: Dashboard must include total_spend_usd
- **Line 73**: total_spend_usd must be non-negative, got: {}
- **Line 80**: Dashboard must include longest_running array
- **Line 133**: Dashboard must render in under 3 seconds, took: {:?}
- **Line 171**: Failed to spawn daemon
- **Line 172**: Failed to spawn daemon
- **Line 173**: Failed to spawn daemon
- **Line 174**: Failed to spawn daemon
- **Line 175**: Failed to spawn daemon
- **Line 176**: Failed to spawn daemon
- **Line 246**: Total cost must be non-negative
- **Line 264**: Sum of project costs ({}) should equal total ({})

#### hoop-daemon/tests/s2_transcript_archaeology.rs

- **Line 66**: Bead events endpoint should return 200 or 404, got: {}
- **Line 74**: Events should be an array
- **Line 121**: Visual debug panel must load in under 5 seconds, took: {:?}
- **Line 151**: Stitch read endpoint should return 200 or 404, got: {}
- **Line 187**: Endpoint {} should return 200 or 404, got: {}
- **Line 223**: Conversations should be an array
- **Line 292**: Cost data should be an object
- **Line 342**: Event should have timestamp
- **Line 346**: Event should have type

#### hoop-daemon/tests/s3_bead_creation_from_chat.rs

- **Line 151**: Draft should be created within 3 seconds, took {:?}
- **Line 179**: Draft should appear in the draft queue
- **Line 272**: Bead should be created within 3 seconds of approval, took {:?}
- **Line 282**: stub br should record br create call with expected title
- **Line 388**: Audit log should contain DraftCreated entry
- **Line 401**: Audit log should contain DraftApproved entry
- **Line 413**: Operator identity should be present in audit log
- **Line 478**: Draft should be in queue
- **Line 499**: stub br should record the create call
- **Line 522**: Audit should have DraftCreated
- **Line 523**: Audit should have DraftApproved
- **Line 535**: operator identity should be present
- **Line 601**: chat

#### hoop-daemon/tests/s4_daemon_restart.rs

- **Line 216**: Worker should have written more events while HOOP was down
- **Line 265**: Bead count should be stable across restart: before={}, after={}
- **Line 353**: UI state should rebuild in under 5 seconds, took: {:?}
- **Line 424**: Worker should continue writing events during HOOP downtime
- **Line 456**: Worker should continue after HOOP restart
- **Line 539**: Beads should not disappear across restarts in cycle {}

#### hoop-daemon/tests/s5_workspace_deleted.rs

- **Line 193**: Error card should appear within 10s of workspace deletion
- **Line 282**: Other projects should still be accessible
- **Line 294**: Daemon should still be healthy
- **Line 380**: Should be in degraded state after workspace deletion
- **Line 406**: Auto-recovery should occur within 10s of workspace restore
- **Line 481**: Daemon should still be running after workspace deletion

#### hoop-daemon/tests/secrets_scanner_integration.rs

- **Line 198**: Fixture '{}' should have been detected but got no findings. Content: {}
- **Line 207**: Fixture '{}' should match pattern '{}' but got: {:?}
- **Line 216**: Fixture '{}' should NOT have been detected but got {} findings: {:?}
- **Line 235**: Git SHA in 'commit' context should not be flagged as high entropy
- **Line 243**: UUID should not be flagged as high entropy
- **Line 252**: High-entropy string should be detected
- **Line 264**: Email should not be detected when disabled
- **Line 273**: Email should be detected when enabled
- **Line 277**: Should detect email pattern
- **Line 294**: high
- **Line 297**: high
- **Line 298**: high
- **Line 299**: high
- **Line 300**: high
- **Line 302**: high
- **Line 320**: Should detect at least 3 secrets, got {}
- **Line 330**: API key should have high entropy: {}
- **Line 331**: API key should have high entropy: {}
- **Line 332**: API key should have high entropy: {}
- **Line 341**: API key should have high entropy: {}
- **Line 346**: Normal text should have low entropy: {}
- **Line 356**: Very short strings should not be flagged
- **Line 360**: Very short strings should not be flagged
- **Line 364**: API_KEY=sk-ant-api03-TEST1234567890ABCDEFGHIJ
- **Line 368**: API_KEY=sk-ant-api03-TEST1234567890ABCDEFGHIJ
- **Line 379**: ANTHROPIC_API_KEY=sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444
- **Line 383**: ANTHROPIC_API_KEY=sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444
- **Line 402**: Should detect with either anthropic_api_key or env_var_secret pattern
- **Line 408**: Should detect with either anthropic_api_key or env_var_secret pattern
- **Line 434**: Benign content should not produce findings: '{}'. Got: {:?}
- **Line 528**: False positive rate ({:.1}%) exceeds 5% threshold
- **Line 535**: Should scan at least 3 files for meaningful test

#### hoop-daemon/tests/secrets_scanner_parity.rs

- **Line 177**: Fixture '{}' (pattern: {}) should have been detected but wasn't. Content: {}
- **Line 187**: Fixture '{}' matched but not by expected pattern '{}'. Found patterns: {:?}
- **Line 196**: Fixture '{}' should NOT have been detected but got {} findings: {:?}
- **Line 212**: Default patterns should not be empty
- **Line 216**: Pattern ID should not be empty
- **Line 217**: Pattern name should not be empty
- **Line 218**: Pattern '{}' has invalid severity: {}
- **Line 224**: Pattern '{}' should have at least one regex
- **Line 256**: Fixture '{}' references pattern_id '{}' which doesn't exist in default patterns
- **Line 279**: Should detect Anthropic API key
- **Line 280**: Should match anthropic_api_key pattern from config_resolver
- **Line 308**: Custom pattern should detect test secret
- **Line 309**: Should match custom pattern
- **Line 318**: Default anthropic_api_key pattern should not work after replacing with custom patterns

#### hoop-daemon/tests/session_redaction.rs

- **Line 92**: Raw file must still contain original key
- **Line 103**: expected [REDACTED], got: {out}
- **Line 107**: raw key must not appear: {out}
- **Line 111**: surrounding text must be preserved: {out}
- **Line 121**: expected [REDACTED], got: {out}
- **Line 125**: raw key must not appear: {out}
- **Line 140**: token in block 0 must be redacted: {text0}
- **Line 144**: raw token must not appear in block 0: {text0}
- **Line 172**: must be redacted: {r1}
- **Line 173**: raw key must not appear: {r1}
- **Line 186**: old content must be redacted: {r_old}
- **Line 190**: new content must be redacted: {r_new}
- **Line 194**: old key must not appear: {r_old}
- **Line 198**: new key must not appear: {r_new}
- **Line 226**: line {i}: raw Anthropic key must not appear after redaction
- **Line 230**: line {i}: raw GitHub token must not appear after redaction
- **Line 234**: line {i}: raw fake key fragment must not appear after redaction
- **Line 245**: JWT must be redacted: {out}
- **Line 246**: raw JWT must not appear: {out}
- **Line 263**: tool result secret must be redacted: {serialised}
- **Line 267**: raw key fragment must not appear: {serialised}

#### hoop-daemon/tests/skills_integration.rs

- **Line 51**: Failed to create skill dir
- **Line 98**: hello world
- **Line 108**: Failed to create skill dir
- **Line 111**: Failed to create skill dir
- **Line 112**: Failed to create skill dir
- **Line 152**: Failed to create skill dir
- **Line 154**: Failed to create skill dir
- **Line 157**: Failed to create skill dir
- **Line 199**: integer
- **Line 201**: integer
- **Line 204**: integer
- **Line 245**: Failed to create skill dir
- **Line 247**: Failed to create skill dir
- **Line 286**: count
- **Line 294**: properties

#### hoop-daemon/tests/skills_quarantine_integration.rs

- **Line 72**: enable-test
- **Line 73**: enable-test
- **Line 74**: enable-test
- **Line 78**: enable-test
- **Line 103**: disable-test
- **Line 104**: disable-test
- **Line 105**: disable-test
- **Line 106**: disable-test
- **Line 132**: bad-name
- **Line 133**: bad-name
- **Line 191**: #!/usr/bin/env python3
- **Line 192**: #!/usr/bin/env python3
- **Line 196**: #!/usr/bin/env python3
- **Line 227**: remove-pending
- **Line 229**: remove-pending
- **Line 249**: remove-active
- **Line 251**: remove-active
- **Line 257**: duplicate-test
- **Line 259**: duplicate-test
- **Line 280**: nonexistent
- **Line 284**: nonexistent
- **Line 299**: yaml-show-test
- **Line 300**: yaml-show-test
- **Line 315**: run
- **Line 316**: run
- **Line 317**: run
- **Line 318**: run

#### hoop-daemon/tests/state_projections.rs

- **Line 163**: Health check should return 200
- **Line 194**: init must contain subscriptions array
- **Line 206**: global must always be in subscriptions
- **Line 226**: Must receive workers_snapshot
- **Line 227**: Must receive beads_snapshot
- **Line 228**: Must receive conversations_snapshot
- **Line 229**: Must receive projects_snapshot
- **Line 230**: Must receive config_status
- **Line 344**: Should receive messages after subscribe/unsubscribe
- **Line 365**: Config status must include 'valid' field
- **Line 390**: Beads must be an array
- **Line 394**: Each bead must have an id
- **Line 395**: Each bead must have a title
- **Line 396**: Each bead must have a status
- **Line 419**: Workers response is valid array
- **Line 441**: Projects response is valid array
- **Line 488**: Connection should receive init
- **Line 572**: Reconnect should receive init event
- **Line 573**: Reconnect should receive beads_snapshot
- **Line 635**: Connection should establish in <1s, took {:?}
- **Line 641**: Snapshots should arrive in <5s, took {:?}
- **Line 647**: Full test should complete in <10s, took {:?}
- **Line 653**: Should receive all snapshot events
- **Line 662**: global should be valid
- **Line 663**: project:testrepo should be valid
- **Line 664**: project with colons should be valid
- **Line 667**: empty project name should be invalid
- **Line 668**: fleet: prefix should be invalid
- **Line 669**: empty string should be invalid
- **Line 670**: GLOBAL (uppercase) should be invalid

#### hoop-daemon/tests/stderr_stdout_capture.rs

- **Line 141**: Generated stdout must be at least 10KB, got {} bytes
- **Line 157**: Default config must generate >10KB
- **Line 165**: Large config must generate >20KB
- **Line 166**: Large config should generate more output than default

#### hoop-daemon/tests/stdout_generation_test.rs

- **Line 283**: Subprocess should succeed
- **Line 284**: Should have stdout output
- **Line 296**: Stderr subprocess should succeed
- **Line 308**: Mixed subprocess should succeed
- **Line 309**: Should have stdout output
- **Line 310**: Should have stderr output
- **Line 322**: Multi-line subprocess should succeed
- **Line 323**: Should have stdout output
- **Line 324**: Should have stderr output
- **Line 351**: Configured subprocess should succeed
- **Line 365**: Should have exit code
- **Line 366**: Should succeed
- **Line 367**: Should have stdout
- **Line 380**: Path should be in target directory
- **Line 417**: stdout

#### hoop-daemon/tests/stdout_verification.rs

- **Line 88**: In-memory verification should pass
- **Line 117**: Verification should fail for mismatched content
- **Line 118**: Hello, 世界! 🌍\nTab\there\tand\nnewlines\n
- **Line 119**: test_unicode_verify.log
- **Line 149**: Unicode verification should pass

#### hoop-daemon/tests/stitch_percentile_index_integration.rs

- **Line 168**: stitch_percentile_index table should exist
- **Line 179**: stitch_percentile_index_meta table should exist
- **Line 202**: Failed to check schema version
- **Line 206**: Failed to check rebuild needed
- **Line 223**: Failed to check schema version
- **Line 227**: Failed to check rebuild needed
- **Line 383**: Cost p50 should be positive
- **Line 384**: Cost p90 should be >= p50
- **Line 432**: Should have multiple buckets for diverse stitches
- **Line 450**: Query should find a matching bucket
- **Line 456**: Query should complete in <50ms, took {}ms
- **Line 495**: Fuzzy fallback should find a match based on title hash and body length
- **Line 527**: Sample count should be below threshold
- **Line 585**: Should find match with similar title tokens and same body length
- **Line 601**: Should not match with completely different title tokens
- **Line 697**: p50 should be ~3.0, got {}
- **Line 702**: p90 should be ~5.0, got {}

#### hoop-daemon/tests/supervisor_health.rs

- **Line 152**: test-project
- **Line 186**: Should receive status update
- **Line 230**: Should not be ready with no runtimes
- **Line 246**: Should not be ready with no runtimes
- **Line 263**: Should be ready with healthy runtime
- **Line 309**: Should not be ready when all failed
- **Line 323**: Should not be ready when all in error state
- **Line 337**: Should not be ready when all abandoned
- **Line 378**: Should be ready with at least one healthy
- **Line 405**: Should be ready with at least one starting
- **Line 483**: Should receive at least one status update, got {}

#### hoop-daemon/tests/supervisor_hotreload.rs

- **Line 175**: project-3
- **Line 176**: project-1
- **Line 177**: project-2
- **Line 292**: test-project

#### hoop-daemon/tests/supervisor_isolation.rs

- **Line 148**: Project {} should be running
- **Line 188**: Both runtimes should still exist
- **Line 189**: Both runtimes should still exist
- **Line 218**: project-a should not be running after .beads corruption
- **Line 224**: project-b should still be running despite project-a failure
- **Line 269**: project-b should exist
- **Line 295**: Iteration {}: project-b should still be running
- **Line 301**: Iteration {}: project-c should still be running
- **Line 352**: panic metric should have incremented
- **Line 392**: {} should have its own bead reader running
- **Line 437**: {} should have its own session tailer running
- **Line 509**: project-b should still be running

#### hoop-daemon/tests/supervisor_restart.rs

- **Line 143**: Should be permanent: {}
- **Line 160**: Should NOT be permanent: {}
- **Line 173**: test error
- **Line 174**: test error
- **Line 183**: Test error message
- **Line 189**: Test error message
- **Line 195**: Test error message
- **Line 224**: starting
- **Line 225**: starting

#### hoop-daemon/tests/supervisor_shutdown.rs

- **Line 130**: Runtime should be running
- **Line 264**: Runtime should be in running state, got: {:?}
- **Line 274**: Runtime should be in running state, got: {:?}

#### hoop-daemon/tests/testrepo_harness_integration.rs

- **Line 301**: init must contain subscriptions array
- **Line 322**: Global subscription should be present
- **Line 343**: workers_snapshot should be received
- **Line 347**: beads_snapshot should be received
- **Line 351**: conversations_snapshot should be received
- **Line 355**: projects_snapshot should be received
- **Line 359**: config_status should be received
- **Line 377**: Beads response should be an array
- **Line 382**: Workers response should be an array
- **Line 387**: Conversations response should be an array
- **Line 392**: Projects response should be an array
- **Line 396**: Config status must include 'valid' field
- **Line 401**: Capacity should be object or array
- **Line 416**: Metrics should contain hoop_ prefixed metrics or be non-empty
- **Line 430**: Metric line {} should have whitespace separator: {}
- **Line 488**: Should receive messages after subscribe/unsubscribe
- **Line 535**: Connection should receive init
- **Line 600**: After reconnect, should receive snapshot events, got {}
- **Line 620**: Each bead must have an 'id' field
- **Line 624**: Each bead must have a 'title' field
- **Line 628**: Each bead must have a 'status' field
- **Line 637**: Each worker must have a 'name' field
- **Line 647**: Each project must have a 'name' field
- **Line 651**: Each project must have a 'path' field

#### hoop-daemon/tests/testrepo_integration.rs

- **Line 281**: init must contain subscriptions array
- **Line 294**: Global subscription should be present
- **Line 315**: workers_snapshot should be received
- **Line 319**: beads_snapshot should be received
- **Line 323**: conversations_snapshot should be received
- **Line 327**: projects_snapshot should be received
- **Line 331**: config_status should be received
- **Line 396**: Beads response should not be empty
- **Line 400**: Workers response should not be empty
- **Line 404**: Projects response should not be empty
- **Line 410**: testrepo should be in projects list
- **Line 414**: Config status must include 'valid' field
- **Line 418**: Capacity should be object or array
- **Line 433**: Metrics should contain hoop_ prefixed metrics or be non-empty
- **Line 447**: Metric line {} should have whitespace separator: {}
- **Line 505**: Should receive messages after subscribe/unsubscribe
- **Line 552**: Connection should receive init
- **Line 617**: After reconnect, should receive snapshot events, got {}
- **Line 637**: Each bead must have an 'id' field
- **Line 641**: Each bead must have a 'title' field
- **Line 645**: Each bead must have a 'status' field
- **Line 654**: Each worker must have a 'name' field
- **Line 658**: Each worker must have a 'state' field
- **Line 667**: Each project must have a 'name' field
- **Line 671**: Each project must have a 'path' field

#### hoop-daemon/tests/upload_secrets_scan.rs

- **Line 42**: Should detect secret in attachment
- **Line 43**: Should detect anthropic_api_key pattern
- **Line 67**: Should detect at least 3 secrets
- **Line 72**: README.md
- **Line 73**: README.md
- **Line 74**: README.md
- **Line 94**: Clean attachment should have no findings
- **Line 114**: Should detect secret in .{} file
- **Line 138**: Binary files should not be scanned
- **Line 165**: Should detect secrets in JSON
- **Line 171**: Should detect anthropic_api_key in nested JSON
- **Line 175**: Should detect github_token in nested JSON
- **Line 198**: Should detect at least 2 env var secrets
- **Line 218**: Large files should be skipped
- **Line 233**: test_pattern
- **Line 236**: test_pattern
- **Line 237**: attachment
- **Line 238**: attachment
- **Line 287**: Benign file '{}' should have no findings, got: {:?}

#### hoop-daemon/tests/zero_write_invariant.rs

- **Line 44**: read verb '{}' classified as write
- **Line 55**: write verb '{}' not classified as write
- **Line 65**: 'create' must not be forbidden
- **Line 70**: verb '{}' should be forbidden
- **Line 196**: br
- **Line 197**: br
- **Line 205**: {}
- **Line 206**: {}
- **Line 362**: validate_br_subprocess_args should reject '{}'

#### hoop-daemon/tests_phase5/adapter_failover.rs

- **Line 99**: Adapter build should succeed
- **Line 120**: ZAI adapter build should succeed after Anthropic
- **Line 190**: Stitch title should reference the adapter
- **Line 273**: Archived timestamp should be set
- **Line 452**: Global rule should be preserved
- **Line 453**: Project rule should be preserved
- **Line 619**: Stitch title should reference the adapter
- **Line 623**: Stitch title should indicate it's an agent session
- **Line 647**: Tool name should be in content
- **Line 723**: Multi-line content should be preserved
- **Line 724**: Quotes should be preserved
- **Line 725**: Code blocks should be preserved
- **Line 803**: First rule should be present
- **Line 807**: Second rule should be present

#### hoop-daemon/tests_phase5/adapter_failover_integration.rs

- **Line 74**: Adapter build should succeed
- **Line 95**: ZAI adapter build should succeed after Anthropic
- **Line 189**: Stitch title should reference the old adapter
- **Line 550**: Global rule should be preserved
- **Line 554**: Project rule should be preserved
- **Line 561**: project:hoop
- **Line 562**: project:hoop
- **Line 619**: SELECT status, archived_reason FROM agent_sessions WHERE id = ?1
- **Line 678**: rejected rule
- **Line 679**: rejected rule
- **Line 680**: rejected rule
- **Line 726**: Stitch title should contain the session date
- **Line 730**: Stitch title should contain the session time
- **Line 734**: Stitch title should reference the adapter

#### hoop-daemon/tests_phase5/adapter_failover_test.rs

- **Line 232**: Should have at least 2 sessions, got {}
- **Line 301**: Archived session should have a stitch_id linking to the preserved Stitch
- **Line 310**: Stitch should exist in fleet.db
- **Line 320**: Stitch title should reference agent session
- **Line 377**: Reflection entry should persist after adapter switch
- **Line 433**: Should have at least 3 sessions, got {}
- **Line 453**: First archived session should have stitch_id
- **Line 457**: Second archived session should have stitch_id
- **Line 523**: Reflection Ledger entry should be preserved for continuity
- **Line 574**: At least one switch should succeed
- **Line 666**: Should have at least 2 sessions, got {}
- **Line 693**: Archived session should have a stitch_id linking to the preserved Stitch
- **Line 702**: Stitch should exist in fleet.db
- **Line 902**: Should have performed at least 6 health checks over 30s

#### hoop-daemon/tests_phase5/agent_turn_audit_trail.rs

- **Line 167**: System message should reference the turn_id

#### hoop-daemon/tests_phase5/reflection_detector_integration.rs

- **Line 168**: run_detection should succeed
- **Line 189**: Rule should mention unwrap or don't: {}
- **Line 232**: Should propose 1 preference pattern
- **Line 270**: Should propose 1 correction pattern
- **Line 323**: Should not propose patterns: worker stitches ignored, operator below threshold
- **Line 381**: Should detect at least 1 pattern from synthetic fixtures, got {}
- **Line 384**: Should detect at least 1 pattern from synthetic fixtures, got {}
- **Line 405**: At least one pattern should span 3+ Stitches
- **Line 443**: Should not propose patterns: old stitches outside window
- **Line 468**: operator+operator should be true
- **Line 472**: worker+fleet should be false
- **Line 476**: operator+fleet should be false
- **Line 480**: nonexistent stitch should be false
- **Line 554**: build_reflection_rules_with_audit should succeed
- **Line 556**: SELECT id, kind, target, args_json FROM actions WHERE kind = 'reflection_injected'
- **Line 557**: SELECT id, kind, target, args_json FROM actions WHERE kind = 'reflection_injected'
- **Line 583**: SELECT id, last_applied, applied_count FROM reflection_ledger WHERE status = 'approved'
- **Line 605**: last_applied should be set
- **Line 611**: applied_count should be 2 after second injection

#### hoop-mcp/tests/create_only_stub.rs

- **Line 97**: fake br should succeed
- **Line 174**: read verb '{}' classified as write — this is a bug
- **Line 195**: '{}' missing from FORBIDDEN_WRITE_VERBS
- **Line 200**: '{}' not detected as forbidden
- **Line 207**: create
- **Line 208**: create
- **Line 217**: assert_create_only('{}') should have panicked
- **Line 239**: validate_br_subprocess_args should reject raw '{}' command
- **Line 267**: fake br should succeed for '{}'
- **Line 285**: first invocation should contain title
- **Line 289**: should contain stitch label

#### hoop-mcp/tests/forbidden_worker_steering.rs

- **Line 34**: '{}' missing from FORBIDDEN_WORKER_STEERING_VERBS
- **Line 39**: '{}' not detected as forbidden
- **Line 51**: '{}' should be detected as forbidden
- **Line 74**: '{}' should NOT be detected as forbidden
- **Line 87**: Error message for '{}' should mention the tool name
- **Line 88**: Error message for '{}' should mention the tool name
- **Line 89**: Error message for '{}' should mention the tool name
- **Line 90**: Error message for '{}' should mention the tool name
- **Line 97**: Error message for '{}' should mention the tool name
- **Line 102**: Error should mention 'worker-steering'
- **Line 120**: call_tool should reject forbidden verb '{}'
- **Line 127**: Error for '{}' should mention 'worker-steering', got: {}
- **Line 133**: Error for '{}' should mention the tool name, got: {}
- **Line 158**: Legitimate tool should not trigger worker-steering error, got: {}
- **Line 172**: Unknown tool should not be classified as worker-steering verb

#### hoop-mcp/tests/protocol_contract.rs

- **Line 123**: response must have 'result'
- **Line 127**: result must have 'tools' array
- **Line 175**: response must have 'result'
- **Line 179**: result must have 'prompts' array
- **Line 227**: response must have 'result'
- **Line 231**: result must have 'resources' array
- **Line 279**: response must have 'result'
- **Line 334**: response must have 'result'
- **Line 341**: InitializeResult must serialize '{}' (declared in fixture)
- **Line 352**: ServerInfo must serialize '{}' (declared in fixture)
- **Line 389**: argument '{}' from fixture must be in ToolCallParams.arguments (flattened wire format)
- **Line 432**: ToolCallResult must serialize 'content'
- **Line 435**: ToolCallResult must serialize 'content'
- **Line 441**: 'content' must not be empty
- **Line 451**: Text content must have 'text' field
- **Line 536**: read_stitch fixture must have 'messages' — hoop-mcp's redact_stitch_response requires it
- **Line 540**: 'messages' must be an array
- **Line 544**: read_stitch fixture must have 'stitch' top-level object
- **Line 552**: stitch message must have '{}' field (declared in fixture)
- **Line 586**: fixture {} must be a JSON object

#### hoop-mcp/tests/socket_permissions.rs

- **Line 37**: MCP endpoint must be a Unix socket, not a regular file
- **Line 55**: socket.rs must not contain TcpListener (TCP exposure violates §13)
- **Line 61**: socket.rs must use UnixListener for same-user security
- **Line 67**: socket.rs must document 0600 permissions (same-user only)
- **Line 83**: default socket path should be ~/.hoop/mcp.sock, got {}
- **Line 90**: socket path must not be in /tmp (security risk)

#### testrepo/tests/integration/test_01.rs

- **Line 5**: Integration test 01 passed

#### testrepo/tests/integration/test_02.rs

- **Line 5**: Integration test 02 passed

#### testrepo/tests/integration/test_03.rs

- **Line 5**: Integration test 03 passed

#### testrepo/tests/integration/test_04.rs

- **Line 5**: Integration test 04 passed

#### testrepo/tests/integration/test_05.rs

- **Line 5**: Integration test 05 passed

#### testrepo/tests/integration/test_06.rs

- **Line 5**: Integration test 06 passed

#### testrepo/tests/integration/test_07.rs

- **Line 5**: Integration test 07 passed

#### testrepo/tests/integration/test_08.rs

- **Line 5**: Integration test 08 passed

#### testrepo/tests/integration/test_09.rs

- **Line 5**: Integration test 09 passed

#### testrepo/tests/integration/test_10.rs

- **Line 5**: Integration test 10 passed

#### testrepo/tests/integration/test_11.rs

- **Line 5**: Integration test 11 passed

#### testrepo/tests/integration/test_12.rs

- **Line 5**: Integration test 12 passed

#### testrepo/tests/integration/test_13.rs

- **Line 5**: Integration test 13 passed

#### testrepo/tests/integration/test_14.rs

- **Line 5**: Integration test 14 passed

#### testrepo/tests/integration/test_15.rs

- **Line 5**: Integration test 15 passed

#### testrepo/tests/integration/test_16.rs

- **Line 5**: Integration test 16 passed

#### testrepo/tests/integration/test_17.rs

- **Line 5**: Integration test 17 passed

#### testrepo/tests/integration/test_18.rs

- **Line 5**: Integration test 18 passed

#### testrepo/tests/integration/test_19.rs

- **Line 5**: Integration test 19 passed

#### testrepo/tests/integration/test_20.rs

- **Line 5**: Integration test 20 passed

#### tests/acceptance/s1_morning_review.rs

- **Line 117**: Dashboard must include total_workers count
- **Line 125**: Dashboard must include total_spend_usd
- **Line 132**: total_spend_usd must be non-negative
- **Line 134**: Dashboard must include longest_running array
- **Line 177**: Dashboard must render in under 3 seconds, took: {:?}
- **Line 209**: Failed to spawn daemon
- **Line 210**: Failed to spawn daemon
- **Line 211**: Failed to spawn daemon
- **Line 212**: Failed to fetch dashboard
- **Line 213**: Failed to fetch dashboard
- **Line 214**: Failed to parse response
- **Line 270**: Total cost must be non-negative
- **Line 283**: Sum of project costs should equal total

#### tests/acceptance/s2_transcript_archaeology.rs

- **Line 130**: Bead events endpoint should return 200 or 404
- **Line 137**: Events should be an array
- **Line 180**: Visual debug panel must load in under 5 seconds, took: {:?}
- **Line 207**: Stitch read endpoint should return 200 or 404
- **Line 237**: Endpoint {} should return 200 or 404
- **Line 265**: Conversations should be an array
- **Line 314**: Cost data should be an object
- **Line 353**: Event should have timestamp
- **Line 357**: Event should have type

#### tests/acceptance/s3_bead_creation_from_chat.rs

- **Line 127**: Draft endpoint should respond, got: {}
- **Line 150**: Draft queue endpoint should return 200 or 404
- **Line 172**: Audit log endpoint should return 200 or 404
- **Line 231**: Draft response should have an id field
- **Line 262**: Audit log should have rows structure
- **Line 309**: Draft should be created within 3 seconds, took {:?}
- **Line 328**: Draft should appear in queue

#### tests/acceptance/s4_daemon_restart.rs

- **Line 219**: Worker should have written events
- **Line 239**: Bead count should be stable across restart: before={}, after={}
- **Line 281**: UI state should rebuild in under 5 seconds, took: {:?}
- **Line 313**: Worker should continue writing events during HOOP downtime
- **Line 327**: Worker should continue after HOOP restart
- **Line 398**: Beads should not disappear across restarts in cycle {}

#### tests/acceptance/s5_workspace_deleted.rs

- **Line 199**: Error card should appear within 10s of workspace deletion
- **Line 253**: Other projects should still be accessible
- **Line 265**: Daemon should still be healthy
- **Line 335**: Auto-recovery should occur within 10s of workspace restore
- **Line 384**: Daemon should still be running after workspace deletion

#### tests/acceptance/s6_machine_mode.rs

- **Line 122**: Status should be a JSON object
- **Line 166**: Projects should be an array
- **Line 223**: Should be parseable by jq
- **Line 227**: Each project should be an object
- **Line 228**: Project should have 'name' field for jq queries
- **Line 272**: Readyz endpoint should return 200 or 503
- **Line 295**: Non-existent resource should return 404, got: {}

#### tests/cli_test_helpers.rs

- **Line 338**: Flag must be true after extraction
- **Line 356**: Handler must accept no_interactive parameter
- **Line 364**: main() must extract flag from CLI
- **Line 367**: main() must pass flag to handler
- **Line 390**: CLI must parse flag as true
- **Line 399**: Handler must accept no_interactive parameter
- **Line 405**: Handler must check the flag value
- **Line 413**: main() must pass extracted flag to handler
- **Line 455**: Child process must receive no_interactive flag
- **Line 464**: --no-interactive
- **Line 476**: --no-interactive
- **Line 498**: Parent must have flag set
- **Line 510**: Child CLI must parse no_interactive=true from passed args
- **Line 514**: Flag must appear in child's argument vector
- **Line 571**: Top level must have flag
- **Line 577**: Flag accessible at Projects level
- **Line 583**: Confirm flag must be true
- **Line 585**: Flag accessible at Remove level
- **Line 615**: non-interactive mode
- **Line 646**: Level 0: Global flag must be true
- **Line 652**: Level 1: Flag accessible in Projects
- **Line 658**: Remove's --confirm flag must be true
- **Line 661**: Level 2: Flag accessible in Remove
- **Line 720**: 1
- **Line 760**: Flag must be parsed as true
- **Line 784**: Child environment check must succeed for HOOP_NO_INTERACTIVE=1
- **Line 790**: Flag must be false when not specified
- **Line 822**: Flag must be true at top level
- **Line 827**: Flag accessible at Projects level
- **Line 836**: Child must receive no_interactive flag
- **Line 844**: Handler must accept no_interactive parameter
- **Line 867**: --no-interactive
- **Line 893**: --no-interactive
- **Line 917**: Child process must receive no_interactive flag
- **Line 926**: --no-interactive
- **Line 938**: --no-interactive
- **Line 960**: Parent must have flag set
- **Line 971**: Child CLI must parse no_interactive=true from passed args
- **Line 975**: Flag must appear in child's argument vector
- **Line 995**: Top level must have flag
- **Line 1001**: Flag accessible at Projects level
- **Line 1008**: Flag accessible at Remove level
- **Line 1038**: non-interactive mode
- **Line 1069**: Level 0: Global flag must be true
- **Line 1075**: Level 1: Flag accessible in Projects
- **Line 1081**: Remove's --confirm flag must be true
- **Line 1084**: Level 2: Flag accessible in Remove
- **Line 1128**: 1
- **Line 1168**: Flag must be parsed as true
- **Line 1192**: Child environment check must succeed for HOOP_NO_INTERACTIVE=1
- **Line 1198**: Flag must be false when not specified
- **Line 1230**: Flag must be true at top level
- **Line 1235**: Flag accessible at Projects level
- **Line 1244**: Child must receive no_interactive flag
- **Line 1268**: --no-interactive
- **Line 1481**: Expected {} command, but command parsing failed
- **Line 1596**: Child process should receive --no-interactive flag in args: {:?}
- **Line 1602**: Child process should NOT receive --no-interactive flag when parent value is false: {:?}
- **Line 1678**: Child process should receive {} flag in args: {:?}
- **Line 1745**: Flag must remain accessible through all {} nesting levels
- **Line 2098**: /tmp
- **Line 2104**: /tmp
- **Line 2338**: Child process must receive no_interactive flag when parent has it
- **Line 2344**: --no-interactive
- **Line 2365**: Flag must remain accessible at Projects level

### assert_eq! (1,501 instances)

#### hoop-cli/tests/clap_test_utils.rs

- **Line 682**: no_interactive should be true with flag before command
- **Line 704**: no_interactive should be true with flag after command
- **Line 726**: no_interactive should be true with -y flag
- **Line 772**: no_interactive should default to false
- **Line 804**: Should parse with flag after command
- **Line 812**: Should parse with -y flag
- **Line 820**: --no-interactive
- **Line 921**: scan
- **Line 927**: -y
- **Line 933**: scan
- **Line 939**: /tmp
- **Line 1007**: /tmp
- **Line 1013**: /tmp
- **Line 1019**: --no-interactive
- **Line 1025**: hoop
- **Line 1026**: scan
- **Line 1037**: hoop
- **Line 1105**: --no-interactive
- **Line 1106**: scan
- **Line 1119**: /tmp
- **Line 1122**: --no-interactive
- **Line 1142**: scan
- **Line 1143**: /tmp
- **Line 1154**: /tmp
- **Line 1157**: scan
- **Line 1171**: /tmp
- **Line 1174**: Expected Scan command
- **Line 1175**: hoop
- **Line 1188**: Expected Scan command
- **Line 1189**: hoop
- **Line 1202**: Expected Remove command
- **Line 1203**: hoop
- **Line 1216**: --no-interactive
- **Line 1235**: hoop
- **Line 1236**: hoop
- **Line 1245**: /tmp
- **Line 1249**: Expected Scan command
- **Line 1250**: hoop
- **Line 1259**: /tmp
- **Line 1264**: Expected Projects command
- **Line 1276**: scan
- **Line 1283**: --no-interactive
- **Line 1292**: scan
- **Line 1299**: scan
- **Line 1306**: --no-interactive
- **Line 1313**: /tmp
- **Line 1367**: hoop
- **Line 1368**: hoop
- **Line 1369**: --no-interactive

#### hoop-cli/tests/cli_test_helpers.rs

- **Line 2003**: Flag position should not affect value for {}
- **Line 2058**: Primary subcommand should be {}
- **Line 2064**: Nested subcommand should be {}
- **Line 2211**: Flag position must not affect value for {}
- **Line 2361**: -y
- **Line 2362**: scan
- **Line 2363**: scan
- **Line 2368**: init
- **Line 2369**: remove
- **Line 2370**: remove
- **Line 2409**: scan
- **Line 2410**: --no-interactive
- **Line 2411**: --no-interactive
- **Line 2412**: projects
- **Line 2427**: projects
- **Line 2428**: remove
- **Line 2429**: status
- **Line 2438**: status
- **Line 2439**: /tmp
- **Line 2448**: scan
- **Line 2449**: /tmp
- **Line 2458**: scan
- **Line 2459**: projects
- **Line 2460**: remove
- **Line 2475**: projects
- **Line 2476**: remove
- **Line 2477**: -y
- **Line 2486**: status
- **Line 2487**: /tmp
- **Line 2496**: scan
- **Line 2497**: projects
- **Line 2512**: projects
- **Line 2513**: remove
- **Line 2514**: add
- **Line 2523**: patterns
- **Line 2524**: add
- **Line 2525**: /tmp
- **Line 2534**: scan
- **Line 2535**: status
- **Line 2536**: --json
- **Line 2545**: status
- **Line 2546**: scan
- **Line 2547**: /tmp
- **Line 2552**: --no-interactive
- **Line 2553**: /tmp
- **Line 2554**: /tmp
- **Line 2559**: status
- **Line 2560**: scan
- **Line 2561**: /tmp
- **Line 2566**: --no-interactive
- **Line 2570**: my-project
- **Line 2574**: status
- **Line 2579**: --verbose
- **Line 2580**: scan
- **Line 2581**: /tmp
- **Line 2606**: No arguments provided
- **Line 2613**: No arguments provided
- **Line 2620**: scan
- **Line 2629**: {:?}
- **Line 2630**: FlagParseResult
- **Line 2778**: /tmp
- **Line 2786**: status
- **Line 2831**: scan
- **Line 2832**: /tmp
- **Line 2839**: scan
- **Line 2840**: status
- **Line 2847**: projects
- **Line 2848**: remove
- **Line 2856**: remove
- **Line 2857**: Flag should be consistent at both positions
- **Line 2867**: Flag should be consistent at both positions
- **Line 2883**: status
- **Line 2891**: --json
- **Line 2903**: Direct extraction should work
- **Line 2907**: scan
- **Line 2924**: -y
- **Line 2925**: scan
- **Line 2929**: Should detect flag presence regardless of count

#### hoop-cli/tests/cli_test_utils.rs

- **Line 506**: no_interactive should be true
- **Line 534**: no_interactive should be true
- **Line 559**: no_interactive should be true with -y
- **Line 597**: no_interactive value must be consistent regardless of flag position
- **Line 602**: no_interactive should be true
- **Line 631**: no_interactive should be false when not specified
- **Line 673**: no_interactive should be true before command
- **Line 684**: no_interactive should be true after command
- **Line 695**: no_interactive should be true with -y
- **Line 698**: no_interactive value must be consistent regardless of flag position
- **Line 713**: no_interactive should be false when not specified
- **Line 758**: status
- **Line 759**: status
- **Line 765**: status
- **Line 766**: Flag value must be consistent regardless of position
- **Line 769**: Flag value must be consistent regardless of position
- **Line 783**: status
- **Line 789**: /tmp
- **Line 801**: /tmp
- **Line 880**: All 4 test cases should succeed
- **Line 881**: No test cases should fail
- **Line 927**: remove
- **Line 928**: before
- **Line 934**: remove
- **Line 935**: after
- **Line 939**: Flag must be consistent regardless of position
- **Line 982**: --no-interactive
- **Line 983**: --no-interactive
- **Line 993**: status
- **Line 994**: status
- **Line 1001**: scan
- **Line 1023**: scan
- **Line 1024**: scan
- **Line 1025**: /tmp
- **Line 1034**: scan
- **Line 1035**: scan
- **Line 1036**: /tmp
- **Line 1045**: scan
- **Line 1046**: scan
- **Line 1047**: /tmp
- **Line 1056**: scan
- **Line 1057**: scan
- **Line 1058**: /tmp
- **Line 1067**: scan
- **Line 1068**: /tmp
- **Line 1077**: scan
- **Line 1078**: test

#### hoop-cli/tests/cli_test_utils_examples.rs

- **Line 20**: scan
- **Line 21**: scan
- **Line 32**: scan
- **Line 33**: --no-interactive
- **Line 51**: projects
- **Line 52**: /tmp
- **Line 66**: scan
- **Line 67**: --from
- **Line 77**: restore
- **Line 78**: test
- **Line 235**: All test cases should succeed
- **Line 236**: No test cases should fail
- **Line 288**: scan
- **Line 289**: hoop
- **Line 315**: Should succeed with --confirm flag
- **Line 441**: Parse with flag after should succeed
- **Line 447**: before

#### hoop-cli/tests/init_no_interactive_flag.rs

- **Line 27**: no_interactive should be true
- **Line 28**: Command should be 'init'
- **Line 43**: no_interactive should be true
- **Line 44**: Command should be 'init'
- **Line 59**: no_interactive should be true with -y
- **Line 60**: Command should be 'init'
- **Line 71**: no_interactive should be true with -y
- **Line 72**: Command should be 'init'
- **Line 83**: no_interactive should default to false
- **Line 84**: Command should be 'init'
- **Line 98**: init
- **Line 99**: after
- **Line 111**: init
- **Line 112**: init
- **Line 123**: Flag should be extracted from parsed CLI structure
- **Line 390**: Flag position should not affect the extracted value
- **Line 396**: Both positions should extract no_interactive as true
- **Line 402**: Both positions should extract the same command

#### hoop-cli/tests/no_interactive_flag_behavior.rs

- **Line 118**: Flag should be extracted as true
- **Line 119**: Should identify 'projects' as command
- **Line 134**: Flag should be extracted as true
- **Line 135**: Should identify 'projects' as command
- **Line 264**: Short flag -y should set no_interactive to true
- **Line 359**: Flag position should not affect the extracted value
- **Line 365**: Both positions should extract no_interactive as true
- **Line 381**: no_interactive should default to false when flag is not provided
- **Line 519**: Flag should be extracted as true
- **Line 520**: Should identify 'restore' as command
- **Line 536**: Flag should be extracted as true
- **Line 537**: Should identify 'restore' as command
- **Line 674**: Short flag -y should set no_interactive to true
- **Line 782**: Flag position should not affect the extracted value
- **Line 788**: Both positions should extract no_interactive as true
- **Line 809**: no_interactive should default to false when flag is not provided

#### hoop-cli/tests/remove_no_interactive_flag.rs

- **Line 28**: no_interactive should be true
- **Line 29**: Command should be 'remove'
- **Line 48**: no_interactive should be true
- **Line 49**: Command should be 'remove'
- **Line 68**: no_interactive should be true with -y
- **Line 69**: Command should be 'remove'
- **Line 80**: no_interactive should be true with -y
- **Line 81**: Command should be 'remove'
- **Line 92**: no_interactive should default to false
- **Line 93**: Command should be 'remove'
- **Line 108**: remove
- **Line 109**: my-project
- **Line 122**: remove
- **Line 123**: remove
- **Line 135**: Flag should be extracted from parsed CLI structure
- **Line 582**: Flag position should not affect the extracted value
- **Line 588**: Both positions should extract no_interactive as true
- **Line 594**: Both positions should extract the same command
- **Line 611**: Handler should receive true when global --no-interactive is set
- **Line 626**: Handler should receive false when no flag is set
- **Line 642**: Global flag should produce true
- **Line 648**: No flag should produce false
- **Line 658**: Short -y flag should set no_interactive to true
- **Line 665**: Handler should receive true when short -y flag is used
- **Line 687**: Flag position should not affect the handler value
- **Line 691**: Both should produce true

#### hoop-cli/tests/restore_no_interactive_flag.rs

- **Line 35**: no_interactive should be true
- **Line 36**: Command should be 'restore'
- **Line 66**: no_interactive should be true
- **Line 67**: Command should be 'restore'
- **Line 96**: no_interactive should be true with -y
- **Line 97**: Command should be 'restore'
- **Line 118**: no_interactive should be true with -y
- **Line 119**: Command should be 'restore'
- **Line 136**: no_interactive should default to false
- **Line 141**: Command should be 'restore'
- **Line 159**: no_interactive should be true
- **Line 160**: Command should be 'restore'
- **Line 183**: restore
- **Line 184**: --from
- **Line 201**: restore
- **Line 202**: restore
- **Line 220**: Flag should be extracted from parsed CLI structure
- **Line 671**: Flag position should not affect the extracted value
- **Line 677**: Both positions should extract no_interactive as true
- **Line 683**: Both positions should extract the same command
- **Line 697**: Short -y flag should set no_interactive to true
- **Line 702**: Command should be 'restore'

#### hoop-cli/tests/scan_no_interactive_flag.rs

- **Line 28**: no_interactive should be true
- **Line 29**: Command should be 'scan'
- **Line 48**: no_interactive should be true
- **Line 49**: Command should be 'scan'
- **Line 68**: no_interactive should be true with -y
- **Line 69**: Command should be 'scan'
- **Line 80**: no_interactive should be true with -y
- **Line 81**: Command should be 'scan'
- **Line 92**: no_interactive should default to false
- **Line 93**: Command should be 'scan'
- **Line 105**: Global no_interactive should remain false with local --yes
- **Line 106**: Command should be 'scan'
- **Line 122**: Global no_interactive should be true
- **Line 123**: Command should be 'scan'
- **Line 141**: scan
- **Line 142**: /tmp
- **Line 154**: scan
- **Line 155**: scan
- **Line 166**: Flag should be extracted from parsed CLI structure
- **Line 629**: {}: registration prompt mismatch
- **Line 633**: {}: rename prompt mismatch
- **Line 637**: {}: auto-registration mismatch
- **Line 661**: Flag position should not affect the extracted value
- **Line 667**: Both positions should extract no_interactive as true
- **Line 673**: Both positions should extract the same command
- **Line 831**: Handler should receive true when global --no-interactive is set
- **Line 846**: Handler should receive true when local --yes is set (auto_confirm=true)
- **Line 861**: Handler should receive true when both flags are set (true || true = true)
- **Line 876**: Handler should receive false when neither flag is set (false || false = false)
- **Line 898**: OR logic failed for case: {} ({} || {} should be {})
- **Line 915**: Global flag should produce true
- **Line 921**: Local flag should produce true
- **Line 927**: Both flags should produce true
- **Line 933**: No flags should produce false
- **Line 943**: Short -y flag should set no_interactive to true
- **Line 950**: Handler should receive true when short -y flag is used
- **Line 965**: Global flag should cause non-interactive mode even without local flag (true || false = true)
- **Line 979**: Local flag should work without global flag (false || true = true)
- **Line 1001**: Flag position should not affect the handler value
- **Line 1005**: Both should produce true

#### hoop-daemon/tests/acceptance/s1_morning_review.rs

- **Line 40**: Dashboard endpoint should return 200
- **Line 96**: Worker timeline endpoint should return 200
- **Line 132**: Dashboard should return 200
- **Line 164**: Dashboard should work without external services
- **Line 173**: S1 PASS: All data derived from on-disk event files
- **Line 220**: Failed to spawn daemon
- **Line 319**: Sum of project worker counts ({}) should equal total ({})

#### hoop-daemon/tests/acceptance/s2_transcript_archaeology.rs

- **Line 42**: Beads endpoint should return 200
- **Line 223**: Conversations endpoint should return 200
- **Line 298**: Cost trends endpoint should return 200

#### hoop-daemon/tests/acceptance/s3_bead_creation_from_chat.rs

- **Line 137**: Draft creation should return 200
- **Line 164**: List drafts should return 200
- **Line 188**: Get draft should return 200
- **Line 195**: Draft title should match chat input
- **Line 196**: Draft kind should be fix
- **Line 197**: Draft source should be chat
- **Line 198**: Draft project should be testrepo
- **Line 199**: Draft status should be pending
- **Line 258**: Draft approval should return 200
- **Line 299**: Draft status should be submitted
- **Line 300**: Draft should have stitch_id
- **Line 371**: Audit query should return 200
- **Line 393**: DraftCreated source should be chat
- **Line 406**: DraftApproved args should contain stitch_id
- **Line 461**: Failed to list drafts
- **Line 490**: stub br should record the create call
- **Line 527**: source should be chat
- **Line 531**: stitch_id should match
- **Line 579**: Failed to get draft
- **Line 591**: title
- **Line 596**: Test draft with all fields
- **Line 597**: feature
- **Line 598**: Full description
- **Line 599**: labels
- **Line 600**: chat
- **Line 602**: testrepo
- **Line 603**: pending
- **Line 604**: S3 PASS: Draft queue exposes all required fields

#### hoop-daemon/tests/acceptance/s4_daemon_restart.rs

- **Line 199**: First daemon should return beads
- **Line 253**: Second daemon should return beads
- **Line 370**: Should be able to fetch beads after rebuild
- **Line 474**: Should see all beads including those created during restart
- **Line 535**: Should fetch beads in cycle {}

#### hoop-daemon/tests/acceptance/s5_workspace_deleted.rs

- **Line 169**: Initial readyz should return 200
- **Line 170**: Initial readyz status should be ok
- **Line 280**: Projects endpoint should still work
- **Line 373**: Failed to get readyz status after deletion

#### hoop-daemon/tests/acceptance/s6_machine_mode.rs

- **Line 114**: hoop status --json should exit with code 0, got: {:?}
- **Line 134**: Should have 3 projects
- **Line 194**: hoop status --json should exit with code 0
- **Line 219**: jq should successfully parse hoop status --json output
- **Line 274**: hoop projects scan --yes should exit with code 0, stderr: {}
- **Line 324**: Successful operation should exit with code 0
- **Line 356**: Fatal error (project not found) should exit with code 2
- **Line 597**: Command should succeed without TTY in machine mode
- **Line 662**: All concurrent commands should succeed

#### hoop-daemon/tests/adapter_failover.rs

- **Line 102**: glm-5
- **Line 123**: anthropic-session-123
- **Line 187**: Stitch should be in hoop-agent project
- **Line 188**: Stitch should be kind=operator
- **Line 203**: All history messages should be stored
- **Line 214**: Agent session should be linked to the archived stitch
- **Line 266**: Session should be marked as switched
- **Line 267**: Archived reason should be 'switched'
- **Line 333**: Only one session should be active
- **Line 343**: Active adapter should be zai
- **Line 445**: Both Reflection Ledger entries should be preserved
- **Line 521**: Should have exactly one active session
- **Line 524**: Active adapter should be zai
- **Line 525**: Active model should be glm-5
- **Line 526**: New session should have 0 turns
- **Line 535**: Should have one archived session
- **Line 536**: Archived adapter should be anthropic
- **Line 540**: Archived session should preserve turn count
- **Line 544**: Archived session should preserve cost
- **Line 621**: Created by should be hoop:agent
- **Line 632**: All 4 messages should be stored
- **Line 640**: Tool message should be preserved
- **Line 708**: Message count should match
- **Line 711**: Role mismatch at message {}
- **Line 712**: Content mismatch at message {}
- **Line 792**: Only approved entries should appear

#### hoop-daemon/tests/adapter_failover_integration.rs

- **Line 76**: glm-5
- **Line 173**: Stitch should be created
- **Line 184**: Stitch should be in hoop-agent project
- **Line 185**: Stitch should be kind=operator
- **Line 199**: All conversation messages should be preserved
- **Line 213**: Session should be marked as switched
- **Line 214**: Archived reason should be adapter_switch
- **Line 228**: Agent session should be linked to the Stitch
- **Line 242**: Reflection Ledger entries should be preserved
- **Line 300**: Only one session should be active after switch
- **Line 314**: zai
- **Line 315**: SELECT status FROM agent_sessions WHERE id = ?1
- **Line 326**: switched
- **Line 367**: Cost should be preserved
- **Line 368**: Input tokens should be preserved
- **Line 369**: Output tokens should be preserved
- **Line 370**: Turn count should be preserved
- **Line 459**: SELECT COUNT(*) FROM agent_sessions WHERE status = 'switched' AND stitch_id IS NOT NULL
- **Line 465**: SELECT COUNT(*) FROM agent_sessions WHERE status = 'switched' AND stitch_id IS NOT NULL
- **Line 475**: SELECT COUNT(*) FROM stitches WHERE project = 'hoop-agent' AND kind = 'operator'
- **Line 543**: All approved rules should be preserved
- **Line 611**: zai
- **Line 612**: glm-5
- **Line 613**: active
- **Line 614**: SELECT status, archived_reason FROM agent_sessions WHERE id = ?1
- **Line 626**: adapter_switch
- **Line 627**: adapter_switch
- **Line 671**: Only approved rules should be returned

#### hoop-daemon/tests/adapter_failover_test.rs

- **Line 161**: Daemon should be healthy
- **Line 165**: Agent spawn should succeed
- **Line 173**: Agent should be active
- **Line 177**: Daemon should remain healthy after 5xx
- **Line 193**: Agent spawn should succeed
- **Line 204**: Agent should be active
- **Line 205**: Initial adapter should be claude
- **Line 216**: Adapter switch should succeed
- **Line 245**: Should have exactly 1 active session
- **Line 246**: Should have 1 switched (archived) session
- **Line 253**: Agent should still be active
- **Line 254**: Adapter should be zai
- **Line 255**: Model should be glm-5
- **Line 271**: glm-5
- **Line 296**: Old session should be switched (archived)
- **Line 319**: Stitch kind should be 'operator'
- **Line 327**: Stitch should belong to hoop-agent project
- **Line 331**: Stitch should be created by hoop:agent
- **Line 390**: global
- **Line 391**: approved
- **Line 392**: approved
- **Line 444**: Should have 2 switched sessions
- **Line 519**: zai
- **Line 520**: Reflection Ledger entry should be preserved for continuity
- **Line 580**: Daemon should remain healthy
- **Line 603**: Agent spawn should succeed
- **Line 614**: Agent should be active
- **Line 615**: Initial adapter should be claude
- **Line 651**: Agent should still be active
- **Line 652**: Adapter should be zai after config reload
- **Line 657**: Model should be glm-5
- **Line 676**: Should have exactly 1 active session
- **Line 677**: Should have 1 switched (archived) session
- **Line 685**: Original session should be switched (archived)
- **Line 708**: Stitch kind should be 'operator'
- **Line 712**: Stitch should belong to hoop-agent project
- **Line 716**: Stitch should be created by hoop:agent
- **Line 723**: Daemon should remain healthy after hot-reload
- **Line 818**: Daemon should be healthy initially
- **Line 854**: Daemon must remain healthy after Anthropic 5xx error
- **Line 867**: /readyz should return 200 after 5xx error
- **Line 883**: Daemon should stay healthy during 503 outage (check {})
- **Line 893**: Daemon must still be healthy after 30s of Anthropic 5xx errors
- **Line 923**: schema_version: 
- **Line 944**: glm-5
- **Line 952**: Switch to ZAI should succeed
- **Line 959**: Agent should be active after switch
- **Line 960**: Should be using ZAI adapter
- **Line 964**: Daemon should be healthy after recovery

#### hoop-daemon/tests/agent_turn_audit_trail.rs

- **Line 90**: stitch-audit-test
- **Line 91**: hoop:agent:{}
- **Line 138**: created_by_actor should be set
- **Line 139**: created_by_actor should be set
- **Line 140**: created_by_session_id should be set
- **Line 141**: created_by_adapter should be set
- **Line 142**: created_by_model should be set
- **Line 143**: turn_id should be set
- **Line 154**: Should have one system note with turn reference
- **Line 219**: args_json should be valid JSON
- **Line 220**: agent_adapter
- **Line 229**: agent_model
- **Line 230**: turn_id
- **Line 231**: hoop:agent:<session-id>
- **Line 232**: hoop
- **Line 245**: hoop
- **Line 246**: agent
- **Line 247**: hoop:agent:
- **Line 248**: hoop:agent:
- **Line 252**: agent-session-reconstruct
- **Line 314**: /agent?session={}&turn={}
- **Line 315**: /agent?session={}&turn={}
- **Line 316**: /agent?session={}&turn={}
- **Line 317**: /agent?session={}&turn={}

#### hoop-daemon/tests/backup_config_deserialization.rs

- **Line 54**: my-bucket
- **Line 55**: backups/
- **Line 56**: 0 4 * * *
- **Line 57**: endpoint: https://s3.example.com\n\                       bucket: my-bucket\n\                       prefix: backups/\n\                       schedule: '*/30 * * * *'\n\                       retention_days: 14\n\                       encryption: true
- **Line 58**: YAML→JSON conversion should succeed
- **Line 80**: my-bucket
- **Line 81**: backups/
- **Line 82**: */30 * * * *
- **Line 83**: {         
- **Line 84**: bucket
- **Line 99**: my-bucket
- **Line 100**: backups/
- **Line 101**: 0 4 * * *

#### hoop-daemon/tests/backup_restore_cycle.rs

- **Line 93**: fleet.db checksum should match after restore
- **Line 99**: config.yml checksum should match after restore
- **Line 105**: projects.yaml checksum should match after restore
- **Line 118**: Attachment {} size should match after restore
- **Line 156**: test-secret-key
- **Line 157**: age_key should be None when encryption disabled
- **Line 170**: Should return None when age key missing but encryption enabled
- **Line 265**: Decrypted data should match original
- **Line 445**: Cron schedule should have 5 fields

#### hoop-daemon/tests/bead_created_by_hoop_broadcast.rs

- **Line 76**: test-project
- **Line 77**: hoop-ttb.3.53
- **Line 82**: test-project
- **Line 83**: tailscale:test@example.com
- **Line 84**: form
- **Line 85**: Notification should be received within 100ms, took {}ms
- **Line 113**: bd-123
- **Line 114**: os:testuser
- **Line 115**: chat
- **Line 116**: 2026-04-26T12:00:00Z
- **Line 117**: test-project
- **Line 142**: Fleet notification ring should contain bead_created_by_hoop event

#### hoop-daemon/tests/bead_real_line_deserialization.rs

- **Line 42**: {         
- **Line 43**: {         
- **Line 44**: {         
- **Line 45**: {         
- **Line 64**: ); // Default to empty string     assert!(bead.dependencies.is_empty()); // Default to empty Vec     assert_eq!(bead.status, BeadStatus::Open);     assert_eq!(bead.issue_type, BeadType::Bug); }  /// Each BeadStatus lowercase wire value deserializes correctly. #[test] fn all_bead_status_lowercase_values_deserialize() {     let statuses = [         (
- **Line 65**: open
- **Line 67**: open
- **Line 68**: closed
- **Line 91**: Status '{}' should deserialize to {:?}
- **Line 125**: Issue type '{}' should deserialize to {:?}
- **Line 147**: Unrecognized status '{}' should become Unknown
- **Line 169**: Unrecognized issue type '{}' should become Unknown
- **Line 196**: {         
- **Line 197**: {         
- **Line 198**: title

#### hoop-daemon/tests/bead_status_deserialization.rs

- **Line 14**: open\
- **Line 18**: closed\
- **Line 22**: blocked\
- **Line 26**: completed\
- **Line 30**: done\
- **Line 40**: cancelled\
- **Line 44**: in-progress\

#### hoop-daemon/tests/beads_deletion_http.rs

- **Line 205**: Healthy
- **Line 361**: API should still be accessible
- **Line 393**: Should be able to query beads during degradation
- **Line 419**: ok

#### hoop-daemon/tests/beads_removal_recovery.rs

- **Line 149**: All projects should be healthy initially
- **Line 240**: /readyz should return 503 when any project is degraded
- **Line 251**: Readiness status should be degraded
- **Line 317**: All projects should be healthy after recovery

#### hoop-daemon/tests/claimed_at_parsing.rs

- **Line 138**: bd-test001
- **Line 139**: test-project
- **Line 140**: worker-alpha
- **Line 141**: bd-test001
- **Line 150**: bd-test001
- **Line 151**: bd-test001
- **Line 159**: bd-test001
- **Line 160**: bd-test001
- **Line 179**: Timestamp '{}' should fail to parse
- **Line 218**: Timestamp '{}' parse result mismatch: expected {}, got {}
- **Line 275**: Timestamp string should be preserved exactly in CollisionIndexEntry
- **Line 309**: 2026-04-21T18:42:10+00:00
- **Line 361**: Timestamp should round-trip through CollisionIndexEntry unchanged
- **Line 392**: Timestamp with whitespace '{}' should parse: {}
- **Line 400**: 2026-04-21T18:42:10z
- **Line 415**: Timestamp '{}' case sensitivity check failed
- **Line 449**: 2026-04-21T18:42:10'; DROP TABLE collision_index; --
- **Line 481**: SQL injection string should fail to parse: '{}'
- **Line 505**: 1969-12-31T23:59:59Z
- **Line 536**: Negative timestamp should still be parseable after storage
- **Line 567**: 2026-04-21T18:42:10+25:00
- **Line 596**: 2017-01-01T00:00:00Z
- **Line 619**: 2026-13-01T00:00:00Z
- **Line 640**: Boundary value '{}' should parse: {}
- **Line 648**: 2026-04-21T18:42:10🔥Z
- **Line 679**:  
- **Line 708**: Empty string should produce 'premature end of input' error, got: {}

#### hoop-daemon/tests/config_reload_audit.rs

- **Line 117**: hash chain must advance
- **Line 125**: should find exactly one config_reloaded row
- **Line 129**: delta_keys
- **Line 130**: fetched delta_keys should contain +project:proj-two
- **Line 204**: should find exactly one config_reload_rejected row
- **Line 212**: hash chain should be valid
- **Line 213**: projects.yaml
- **Line 245**: should have exactly one delta: +project:proj-two
- **Line 250**: -project:proj-two
- **Line 323**: prev_hash mismatch
- **Line 328**: prev_hash mismatch
- **Line 332**: new_hash mismatch
- **Line 338**: delta_keys in audit row must match computed delta

#### hoop-daemon/tests/config_reload_cycle.rs

- **Line 103**: v1: one project
- **Line 104**: content hash must be set
- **Line 115**: previous config preserved
- **Line 120**: hash unchanged after rejection
- **Line 121**: hash unchanged after rejection
- **Line 136**: v2: two projects
- **Line 137**: proj-beta
- **Line 138**: content hash must change on valid edit
- **Line 156**: v2 config preserved after second rejection
- **Line 161**: v2 hash unchanged
- **Line 168**: v3: back to one project
- **Line 231**: one rejected audit row
- **Line 240**: one success audit row
- **Line 311**: rejection metric should increment by 1
- **Line 326**: success metric should increment by 1
- **Line 438**: previous config preserved
- **Line 443**: hash unchanged
- **Line 444**: hash unchanged
- **Line 454**: -project:good-proj
- **Line 499**: one rejected audit row for semantic validation

#### hoop-daemon/tests/create_only_stub.rs

- **Line 109**: expected exactly one invocation, got {:?}
- **Line 115**: only 'create' verb should be called, got '{}'
- **Line 159**: expected 3 invocations, got {:?}
- **Line 161**: only 'create' verb should be called, got '{}'
- **Line 227**: FORBIDDEN_WRITE_VERBS has {} entries, expected {}
- **Line 312**: create
- **Line 313**: create
- **Line 318**: invoke_br_create must produce 'create' as first arg
- **Line 380**: expected 3 invocations, got {:?}
- **Line 382**: invocation {} should be 'create', got '{}'

#### hoop-daemon/tests/create_stitch_no_auto_submit.rs

- **Line 276**: draft ID should match
- **Line 277**: draft status should be pending
- **Line 278**: draft title should match
- **Line 279**: source should match combo
- **Line 280**: agent_session_id should match combo
- **Line 284**: priority should match combo
- **Line 288**: labels should match combo
- **Line 292**: has_acceptance_criteria should match combo
- **Line 378**: stitch_id must be None before approval
- **Line 400**: status should be submitted after approval
- **Line 401**: stitch_id must be set after approval
- **Line 406**: fleet.db
- **Line 504**: pending
- **Line 505**: pending
- **Line 571**: pending
- **Line 645**: Property violation for combo '{}': status must be 'pending' after creation, got '{}'

#### hoop-daemon/tests/cross_workspace_blockers.rs

- **Line 119**: Should find 2 child stitches
- **Line 125**: Workspace B should match
- **Line 131**: Workspace C should match
- **Line 152**: Should find 2 child beads
- **Line 158**: Bead B workspace should match
- **Line 164**: Bead C workspace should match
- **Line 219**: /ws/b

#### hoop-daemon/tests/disaster_recovery_runbook.rs

- **Line 176**: restored stitch data present
- **Line 186**: database integrity verified
- **Line 235**: corrupted
- **Line 432**: original database intact after rollback

#### hoop-daemon/tests/draft_queue_invariants.rs

- **Line 87**: agent
- **Line 88**: draft must not have stitch_id until approved
- **Line 135**: sess-worker3
- **Line 136**: os:agent-worker-3
- **Line 137**: draft-persist-1
- **Line 214**: pending
- **Line 215**: Second draft
- **Line 220**: edited
- **Line 221**: draft-s1
- **Line 222**: draft-s1
- **Line 271**: rejected
- **Line 276**: draft-s6
- **Line 277**: pending
- **Line 285**: draft-audit-1
- **Line 343**: draft-audit-1
- **Line 344**: draft-audit-1
- **Line 345**: test-project
- **Line 346**: hash_self must be populated
- **Line 431**: approved
- **Line 432**: approved
- **Line 433**: draft-reject-reason
- **Line 494**: draft-reject-noreason
- **Line 495**: draft-reject-noreason
- **Line 496**: draft-reject-noreason
- **Line 497**: draft-reject-noreason
- **Line 548**: rejection reason is optional
- **Line 549**: rejection reason is optional
- **Line 553**: Already tracked in stitch-xyz
- **Line 590**: draft-edit-ver
- **Line 647**: Updated description
- **Line 648**: edit must increment version
- **Line 649**: edit must increment version
- **Line 650**: edit must increment version
- **Line 651**: edit must set status to 'edited'
- **Line 712**: submitted
- **Line 748**: hash_prev must match previous row's hash_self
- **Line 780**: opened_at should be set
- **Line 781**: opened_at should be set
- **Line 782**: opened_at should be set
- **Line 784**: form
- **Line 785**: draft-open-existing
- **Line 805**: abandon should succeed
- **Line 824**: abandoned_at should be cleared on reopen
- **Line 857**: Updated Description
- **Line 858**: investigation
- **Line 859**: urgent
- **Line 860**: urgent
- **Line 861**: security
- **Line 881**: autosave should not increment version
- **Line 901**: abandoned
- **Line 912**: abandoned_at should be set
- **Line 1041**: should delete exactly one old draft
- **Line 1054**: draft-lifecycle
- **Line 1075**: My Stitch Title
- **Line 1093**: Updated description with more details
- **Line 1115**: abandoned draft should still exist

#### hoop-daemon/tests/epoch_sync_invariant.rs

- **Line 49**: First message should be init event
- **Line 106**: First message must be init
- **Line 233**: Bead count should be consistent across reconnects
- **Line 274**: First message must be init (iteration {})
- **Line 321**: init

#### hoop-daemon/tests/filesystem_failure_isolation.rs

- **Line 175**: Initial readyz should return 200
- **Line 176**: Initial readyz status should be ok
- **Line 325**: Initial readyz should return 200
- **Line 326**: Initial readyz status should be ok
- **Line 480**: ok
- **Line 481**: ws://
- **Line 537**: project-b should remain healthy
- **Line 545**: project-c should remain healthy

#### hoop-daemon/tests/fix_patterns_integration.rs

- **Line 62**: unwrap,option,panic,null
- **Line 63**: );     assert_eq!(pattern.applied_count, 0);      // Test LIST     let patterns = hoop_daemon::fix_patterns::FixPatternService::list().unwrap();     assert_eq!(patterns.len(), 1, 
- **Line 64**: should have 1 pattern
- **Line 68**: should have 1 pattern
- **Line 69**: Unwrap Option (Fixed)
- **Line 86**: unwrap,option,pattern-matching
- **Line 87**: );     // Signature should remain unchanged     assert_eq!(updated.signature_vector, vec![0.8, 0.2, 0.0, 0.5]);      // Test record_application     hoop_daemon::fix_patterns::FixPatternService::record_application(&id).unwrap();     let applied = hoop_daemon::fix_patterns::FixPatternService::get(&id)         .unwrap()         .expect(
- **Line 89**: pattern should exist
- **Line 96**: pattern should be deleted
- **Line 170**: should match all 3 patterns above threshold 0.5
- **Line 192**: different pattern should have similarity > 0.5
- **Line 205**: should match 2 patterns with threshold 0.99
- **Line 218**: should match all patterns with zero threshold
- **Line 230**: should limit results
- **Line 294**: should find 2 patterns with 'panic'
- **Line 303**: should find 1 pattern with 'bounds'
- **Line 304**: case-insensitive search should work
- **Line 309**: case-insensitive search should work
- **Line 310**: journal_mode

#### hoop-daemon/tests/fleet_notifications_integration.rs

- **Line 40**: Test notification
- **Line 41**: test-project
- **Line 42**: test-project
- **Line 72**: test-project
- **Line 73**: test-project
- **Line 74**: test-project
- **Line 94**: Snapshot should contain exactly RING_SIZE notifications
- **Line 101**: Oldest retained notification should be index 5
- **Line 102**: Newest notification should be index 24
- **Line 154**:  projects:   - name: test-project     path: /tmp/test 
- **Line 157**:  projects:   - name: test-project     path: /tmp/test 
- **Line 210**: st-xyz
- **Line 213**: metadata
- **Line 214**: test-project
- **Line 215**: multi-sub-test
- **Line 241**: Broadcast to all subscribers
- **Line 242**: Broadcast to all subscribers
- **Line 243**: Broadcast to all subscribers

#### hoop-daemon/tests/hoop_dies_nothing_notices.rs

- **Line 275**: HOOP should see all events after restart
- **Line 354**: iteration {}: all events should persist across HOOP restart
- **Line 374**: iteration {}: all events should be parseable after restart
- **Line 463**: testrepo should exist
- **Line 516**: all events should persist across multiple restarts
- **Line 580**: fleet.db should persist across restarts
- **Line 603**: pending
- **Line 604**: pending
- **Line 605**: testrepo should exist
- **Line 671**: should detect exactly one corrupted line
- **Line 707**: empty events.jsonl should have 0 events

#### hoop-daemon/tests/integration_harness.rs

- **Line 338**: Should have 2 open beads
- **Line 339**: Should have 1 closed bead
- **Line 343**: All beads should belong to testrepo
- **Line 497**: Event {} Claim: bead_id should match
- **Line 502**: Event {} Claim: worker should match
- **Line 509**: Event {} Complete: bead_id should match
- **Line 514**: Event {} Complete: worker should match
- **Line 721**: healthz should return 200
- **Line 725**: healthz status should be ok
- **Line 734**: readyz should return 200
- **Line 752**: GET /api/beads should return 200
- **Line 766**: GET /api/projects should return 200
- **Line 809**: First message should be init event
- **Line 830**: Should receive workers_snapshot
- **Line 848**: Should receive beads_snapshot
- **Line 894**: Daemon should be healthy after boot
- **Line 903**: Should be able to read beads
- **Line 912**: Should be able to get projects
- **Line 944**: testrepo should be in projects list
- **Line 980**: bead id should not be empty
- **Line 1007**: Metrics should contain hoop_ prefixed metrics
- **Line 1162**: Integration test should complete quickly, took {:?}
- **Line 1234**: Daemon should still be healthy after malformed messages
- **Line 1281**: All concurrent requests should succeed
- **Line 1382**: All WebSocket connections should receive init
- **Line 1401**: Non-existent endpoint should return 404
- **Line 1521**: Fetched bead ID should match
- **Line 1522**: Fetched bead title should match
- **Line 1603**: Failed to GET /api/projects

#### hoop-daemon/tests/lint_regex_global_state.rs

- **Line 146**:  // Safe: local regex with captures_iter() fn safe_local() {{     let re = Regex::new(r

#### hoop-daemon/tests/load_test.rs

- **Line 24**: HOOP_LOAD_PROJECTS
- **Line 25**: HOOP_LOAD_PROJECTS
- **Line 26**: HOOP_LOAD_PROJECTS
- **Line 27**: HOOP_LOAD_PROJECTS
- **Line 28**: 5
- **Line 39**: HOOP_LOAD_PROJECTS
- **Line 40**: HOOP_LOAD_PROJECTS
- **Line 41**: HOOP_LOAD_PROJECTS
- **Line 42**: HOOP_LOAD_BEADS
- **Line 63**: load-test-project-
- **Line 203**: Failed to spawn test daemon
- **Line 204**: Failed to spawn test daemon
- **Line 217**: Small-scale load test should pass performance budgets

#### hoop-daemon/tests/load_test_integration.rs

- **Line 83**: Daemon should be healthy
- **Line 312**: All WebSocket clients should connect

#### hoop-daemon/tests/multi_operator_concurrency.rs

- **Line 121**: tailscale:operator-b@example.com
- **Line 126**: draft-autosave-test
- **Line 182**: Updated description
- **Line 183**: urgent
- **Line 184**: urgent
- **Line 185**: version should NOT increment on autosave
- **Line 186**: version should NOT increment on autosave
- **Line 235**: abandoned_at should be set
- **Line 288**: Always run tests before closing beads
- **Line 318**: duplicate proposal should return the same ID
- **Line 324**: should have only one proposal
- **Line 325**: proposal ID should match first proposal
- **Line 330**: should have 3 merged source stitches
- **Line 386**: global
- **Line 459**: tailscale:operator-a@example.com
- **Line 460**: test-project
- **Line 461**: visible
- **Line 462**: tailscale:operator-a@example.com
- **Line 485**: hidden presence should not be returned
- **Line 520**: stale presence should be filtered out
- **Line 542**: test-project
- **Line 556**: claude-session-a
- **Line 625**: both operator sessions should coexist
- **Line 671**: tailscale:operator-a@example.com
- **Line 672**: tailscale:operator-a@example.com
- **Line 724**: tailscale:operator-b@example.com
- **Line 725**: tailscale:operator-b@example.com

#### hoop-daemon/tests/mutation_handler_test.rs

- **Line 164**: title
- **Line 165**: draft-123
- **Line 170**: pending
- **Line 171**: Should include error in broadcast state
- **Line 172**: Should include error in broadcast state
- **Line 179**: pending
- **Line 184**: unauthorized_user
- **Line 185**: Valid Title
- **Line 206**: draft-456
- **Line 212**: pending
- **Line 213**: Some Title
- **Line 239**: draft-789
- **Line 245**: approved
- **Line 246**: contention:
- **Line 247**: Client's state update path is the same whether the     /// update was accepted or rejected.
- **Line 281**: Title B
- **Line 293**: Error present for UI to display
- **Line 320**:    
- **Line 327**: Valid Title
- **Line 335**: Title cannot be empty
- **Line 338**: Title cannot be empty
- **Line 342**: Title cannot be empty
- **Line 379**: test-user
- **Line 380**: Title
- **Line 413**: priority
- **Line 414**: positive integer
- **Line 415**: -5
- **Line 416**: Title
- **Line 439**: Database

#### hoop-daemon/tests/needle_events_roundtrip.rs

- **Line 357**: pluck
- **Line 358**: bd-abc123
- **Line 359**: alpha
- **Line 373**: claude
- **Line 374**: claude-opus-4-6
- **Line 375**: 2026-04-21T18:52:01Z
- **Line 390**: success
- **Line 391**: 2026-04-21T18:53:00Z
- **Line 392**: 2026-04-21T18:53:00Z
- **Line 393**: bravo
- **Line 408**: context limit exceeded
- **Line 409**: 2026-04-21T18:53:00Z
- **Line 410**: 2026-04-21T18:53:00Z
- **Line 411**: bravo
- **Line 426**: abc123def456
- **Line 427**: heartbeats_test

#### hoop-daemon/tests/observer_mode_integration.rs

- **Line 22**: 127.0.0.1:3001
- **Line 23**: /tmp/test-observer.sock
- **Line 92**: 127.0.0.1:3000

#### hoop-daemon/tests/orphans_integration.rs

- **Line 67**: hoop-ttb.1
- **Line 68**: hoop-ttb.1
- **Line 69**: hoop-ttb.1
- **Line 70**: hoop-ttb.2
- **Line 71**: urgent
- **Line 93**: Test orphan
- **Line 182**: should have exactly one stitch_beads row
- **Line 240**: existing relationship should be preserved

#### hoop-daemon/tests/output_capture_helpers/mod.rs

- **Line 792**: Line 1\nLine 2\n
- **Line 832**: [STDOUT] Hello, 世界!\n[STDOUT] Tab\there\n
- **Line 851**: Line 1\nLine 2\nLine 3\n
- **Line 866**: [STDOUT] {}
- **Line 906**: Line 1\n
- **Line 907**: ✅
- **Line 911**: ✅
- **Line 912**: ✅
- **Line 916**: ✅
- **Line 917**: ✅

#### hoop-daemon/tests/path_traversal_hardening.rs

- **Line 217**: safe_rejection must return 400

#### hoop-daemon/tests/pattern_query_evaluator_integration.rs

- **Line 170**: should have 1 pattern query result
- **Line 171**: query should match the stitch title
- **Line 199**: should have exactly 1 pattern member
- **Line 332**: should have 3 pattern query results
- **Line 336**: should match 2 patterns

#### hoop-daemon/tests/per_project_redaction_integration.rs

- **Line 142**: project:customer-data
- **Line 143**: internal-tools
- **Line 144**: project:internal-tools
- **Line 148**: project:internal-tools
- **Line 149**: legacy-project
- **Line 150**: built-in default
- **Line 154**: built-in default
- **Line 155**: unknown-project
- **Line 156**: built-in default
- **Line 160**: built-in default
- **Line 161**: Test: same attachment, different policy → different outcomes
- **Line 162**:      let config = make_minimal_config();     let projects = make_mixed_policy_projects();     let state = RedactionPolicyState::new(&config, projects);      let rt = tokio::runtime::Runtime::new().unwrap();      // Same content with Anthropic API key     let content_with_secret =         
- **Line 189**: internal-tools
- **Line 267**: customer-data
- **Line 360**: project:multi-workspace-project
- **Line 361**: aws_access_key
- **Line 362**: ANTHROPIC_API_KEY=sk-ant-FAKE-KEY-TESTING-ONLY-XYZ

#### hoop-daemon/tests/performance_budget.rs

- **Line 128**: /healthz took {}ms, budget is {}ms
- **Line 144**: /readyz took {}ms, budget is {}ms
- **Line 160**: /api/projects took {}ms, budget is {}ms
- **Line 171**: Expected {} projects
- **Line 184**: /metrics took {}ms, budget is {}ms

#### hoop-daemon/tests/phase2_exit_gate.rs

- **Line 448**: Phase 2 must have exactly 13 core deliverables

#### hoop-daemon/tests/privacy_surface_audit.rs

- **Line 223**: only item 1 (with Slack token) should be flagged; got: {flagged_items:?}
- **Line 480**: update this count when adding new surfaces

#### hoop-daemon/tests/property_invariants.rs

- **Line 266**: Event {} timestamp mismatch
- **Line 270**: Event {} timestamp mismatch
- **Line 273**: Event {} timestamp mismatch
- **Line 276**: Event {} timestamp mismatch
- **Line 379**: First and second calls differ
- **Line 380**: Second and third calls differ
- **Line 452**: Expected InProgress when claimed={} or streaming={}
- **Line 460**: Expected AwaitingReview when has_open_review=true
- **Line 579**: Status derivation is non-deterministic
- **Line 688**: Event count mismatch: live={}, replay={}
- **Line 697**: Event {} mismatch: live={}, replay={}
- **Line 768**: Should have parsed exactly 1 event when split at boundary (split_pos={})
- **Line 853**: First and second replays differ
- **Line 854**: Second and third replays differ
- **Line 893**: {{

#### hoop-daemon/tests/protocol_contract.rs

- **Line 49**: title
- **Line 50**: kind
- **Line 51**: source
- **Line 52**: description
- **Line 53**: description
- **Line 57**: status
- **Line 87**: field '{}' value mismatch
- **Line 212**: test-project
- **Line 257**: test-project
- **Line 258**: test-project
- **Line 259**: running
- **Line 260**: running
- **Line 261**: running
- **Line 262**: running
- **Line 286**: ws_events/init.json
- **Line 310**: init event must have 'subscriptions'
- **Line 369**: worker_update must have 'worker'
- **Line 374**: worker
- **Line 405**: workers_snapshot must have 'workers'
- **Line 434**: beads_snapshot must have 'beads'
- **Line 457**: config_status must have 'config_status'
- **Line 484**: stitch_created must have 'stitch_created'
- **Line 509**: bead_created_by_hoop must have 'bead_created_by_hoop'
- **Line 539**: draft_update must have 'draft_update'
- **Line 566**: collision_alert must have 'collision_alert'
- **Line 591**: morning_brief must have 'morning_brief'
- **Line 624**: projects_snapshot must have 'projects'
- **Line 658**: fixture {} event_type must round-trip

#### hoop-daemon/tests/pure_functions.rs

- **Line 42**: Red
- **Line 43**: );         assert_eq!(ansi_strip::strip_ansi(
- **Line 48**: );     }      #[test]     fn test_strip_rgb_color() {         assert_eq!(ansi_strip::strip_ansi(
- **Line 49**: );         assert_eq!(ansi_strip::strip_ansi(
- **Line 54**: );     }      #[test]     fn test_preserve_normal_text() {         assert_eq!(ansi_strip::strip_ansi(
- **Line 55**: Just normal text
- **Line 60**: Text with 🎉 emoji
- **Line 61**: codex
- **Line 70**: codex
- **Line 81**: codex
- **Line 113**: bd-1
- **Line 138**: alpha
- **Line 149**: [dictated] Voice note transcript
- **Line 176**: authentication
- **Line 177**: hello
- **Line 193**: world
- **Line 199**: world
- **Line 206**: subdir
- **Line 281**: {{project}} and {{file}} and {{custom}}
- **Line 294**: file
- **Line 307**: {\
- **Line 320**: test
- **Line 434**: );         // Unicode with ANSI         assert_eq!(ansi_strip::strip_ansi(
- **Line 436**: 你好
- **Line 438**: );     }      #[test]     fn test_cost_edge_cases() {         // Cost edge case tests would require making private methods public         // These are skipped for now - the public API works correctly         assert!(true);     }      #[test]     fn test_similarity_edge_cases() {         // Empty strings         let sim = similarity::text_similarity(
- **Line 440**: );         assert_eq!(sim.jaccard, 1.0);         // Single word         let sim = similarity::text_similarity(
- **Line 454**: hello
- **Line 457**: hello
- **Line 460**: {{project}} {{project}} {{project}}
- **Line 473**: deep/nested/path
- **Line 542**: bd-1

#### hoop-daemon/tests/quarantine_integration.rs

- **Line 57**: should parse 3 good lines
- **Line 58**: should quarantine 1 bad line
- **Line 59**: should skip 1 empty line
- **Line 67**: should have one date directory
- **Line 73**: should have one quarantined entry
- **Line 78**: test.jsonl
- **Line 84**: custom_parser
- **Line 114**: tag
- **Line 120**: custom_parser
- **Line 124**: reason
- **Line 125**: HOOP_QUARANTINE_DIR
- **Line 218**: Codex should parse 4 good lines
- **Line 219**: Codex should quarantine 1 bad line
- **Line 220**: Gemini should parse 3 good lines
- **Line 221**: Gemini should quarantine 1 bad line
- **Line 228**: should have one date directory
- **Line 234**: should have two quarantined entries (one per adapter)

#### hoop-daemon/tests/reflection_detector_integration.rs

- **Line 171**: Should propose 1 pattern from 3 similar negatives
- **Line 186**: Should have 1 reflection ledger entry
- **Line 188**: Rule should mention unwrap or don't: {}
- **Line 196**: Should have 3 source stitches
- **Line 235**: Should propose 1 preference pattern
- **Line 273**: Should propose 1 correction pattern
- **Line 326**: Should not propose patterns: worker stitches ignored, operator below threshold
- **Line 446**: Should not propose patterns: old stitches outside window
- **Line 572**: Should have 2 audit rows, one per injected rule
- **Line 576**: turn_index
- **Line 579**: rule_id
- **Line 580**: SELECT id, last_applied, applied_count FROM reflection_ledger WHERE status = 'approved'
- **Line 586**: SELECT id, last_applied, applied_count FROM reflection_ledger WHERE status = 'approved'
- **Line 602**: last_applied should be set
- **Line 606**: applied_count should be 1 after injection
- **Line 624**: applied_count should be 2 after second injection
- **Line 633**: Should have 4 audit rows total (2 per injection)

#### hoop-daemon/tests/risk_patterns_standalone.rs

- **Line 21**: Library should contain all patterns passed to from_patterns()
- **Line 84**: Should find exactly one match for 'test' keyword
- **Line 85**: Matched pattern should have the expected ID
- **Line 139**: Library should contain exactly 2 patterns
- **Line 142**: Should find exactly one match for keyword1
- **Line 143**: Should find exactly one match for keyword2
- **Line 146**: Should find exactly one match for keyword2
- **Line 147**: Label Pattern
- **Line 167**: Should find match via label keyword
- **Line 168**: Mixed Keywords
- **Line 188**: Should find match via title keyword
- **Line 192**: Should find match via label keyword
- **Line 196**: Should find match with both keywords

#### hoop-daemon/tests/s1_morning_review.rs

- **Line 40**: Dashboard endpoint should return 200
- **Line 96**: Worker timeline endpoint should return 200
- **Line 131**: Dashboard should return 200
- **Line 161**: Dashboard should work without external services
- **Line 170**: Failed to spawn daemon
- **Line 214**: Failed to spawn daemon
- **Line 307**: Sum of project worker counts ({}) should equal total ({})

#### hoop-daemon/tests/s2_transcript_archaeology.rs

- **Line 43**: Beads endpoint should return 200
- **Line 214**: Conversations endpoint should return 200
- **Line 283**: Cost trends endpoint should return 200

#### hoop-daemon/tests/s3_bead_creation_from_chat.rs

- **Line 137**: Draft creation should return 200
- **Line 164**: List drafts should return 200
- **Line 188**: Get draft should return 200
- **Line 195**: Draft title should match chat input
- **Line 196**: Draft kind should be fix
- **Line 197**: Draft source should be chat
- **Line 198**: Draft project should be testrepo
- **Line 199**: Draft status should be pending
- **Line 258**: Draft approval should return 200
- **Line 299**: Draft status should be submitted
- **Line 300**: Draft should have stitch_id
- **Line 371**: Audit query should return 200
- **Line 393**: DraftCreated source should be chat
- **Line 406**: DraftApproved args should contain stitch_id
- **Line 461**: Failed to list drafts
- **Line 490**: stub br should record the create call
- **Line 527**: source should be chat
- **Line 531**: stitch_id should match
- **Line 579**: Failed to get draft
- **Line 591**: title
- **Line 596**: Test draft with all fields
- **Line 597**: feature
- **Line 598**: Full description
- **Line 599**: labels
- **Line 600**: chat
- **Line 602**: testrepo
- **Line 603**: pending
- **Line 604**: S3 PASS: Draft queue exposes all required fields

#### hoop-daemon/tests/s4_daemon_restart.rs

- **Line 197**: First daemon should return beads
- **Line 251**: Second daemon should return beads
- **Line 366**: Should be able to fetch beads after rebuild
- **Line 468**: Should see all beads including those created during restart
- **Line 527**: Should fetch beads in cycle {}

#### hoop-daemon/tests/s5_workspace_deleted.rs

- **Line 168**: Initial readyz should return 200
- **Line 169**: Initial readyz status should be ok
- **Line 277**: Projects endpoint should still work
- **Line 368**: Failed to get readyz status after deletion

#### hoop-daemon/tests/secrets_scanner_integration.rs

- **Line 301**: high
- **Line 388**: ANTHROPIC_API_KEY=sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444
- **Line 389**: anthropic_api_key

#### hoop-daemon/tests/secrets_scanner_parity.rs

- **Line 241**: none
- **Line 242**: none
- **Line 243**: none
- **Line 244**: Fixture '{}' references pattern_id '{}' which doesn't exist in default patterns

#### hoop-daemon/tests/session_redaction.rs

- **Line 88**: Raw session file must not be modified by redaction
- **Line 150**: clean block must be unchanged
- **Line 160**: clean content must pass through unchanged
- **Line 170**: cache must return same result
- **Line 171**: cache must return same result
- **Line 203**: both rotated-file variants should produce identical redacted form

#### hoop-daemon/tests/skills_integration.rs

- **Line 47**: test-skill
- **Line 48**: A test skill for manifest parsing
- **Line 49**: Failed to create temp dir
- **Line 50**: Failed to create skill dir
- **Line 97**: hello world
- **Line 110**: Failed to create skill dir
- **Line 143**: count
- **Line 190**: not a number
- **Line 236**: not-a-valid-uri
- **Line 278**: count
- **Line 341**: skill_fetch
- **Line 342**: Fetch a URL
- **Line 343**: object
- **Line 344**: url
- **Line 345**: Failed to create skill dir
- **Line 372**: project-a
- **Line 373**: project-a
- **Line 374**: project-b
- **Line 399**: fix-*
- **Line 400**: fix-*
- **Line 401**: Failed to create skill dir

#### hoop-daemon/tests/skills_quarantine_integration.rs

- **Line 201**: active-skill

#### hoop-daemon/tests/state_projections.rs

- **Line 193**: First message must be init
- **Line 279**: WS and REST worker counts must match
- **Line 283**: WS and REST bead counts must match
- **Line 287**: WS and REST project counts must match
- **Line 318**: topic
- **Line 477**: init
- **Line 576**: Bead count should be consistent across reconnects

#### hoop-daemon/tests/stderr_stdout_capture.rs

- **Line 171**: Same configuration should produce identical output size

#### hoop-daemon/tests/stdout_generation_test.rs

- **Line 330**: Should have 5 stdout lines
- **Line 331**: Should have 5 stderr lines
- **Line 396**: stdout
- **Line 408**: stdout
- **Line 409**: stdout
- **Line 418**: stdout
- **Line 419**: stderr
- **Line 420**: stderr

#### hoop-daemon/tests/stitch_percentile_index_integration.rs

- **Line 190**: Schema version should be 1.0.0
- **Line 250**: Same features should produce same bucket key
- **Line 371**: Should have one bucket for 3 similar stitches
- **Line 385**: Should have 3 samples
- **Line 504**: stitch_1
- **Line 537**: Should take first 5 tokens
- **Line 546**: Should take first 5 tokens
- **Line 627**: Should have one bucket
- **Line 653**: Should have two buckets after rebuild
- **Line 689**: Should have 5 samples

#### hoop-daemon/tests/supervisor_health.rs

- **Line 129**: project-2
- **Line 153**: test-project
- **Line 190**: test-project
- **Line 289**: test
- **Line 434**: my-test-project
- **Line 437**: my-test-project
- **Line 513**: workspace1
- **Line 517**: multi-workspace-project

#### hoop-daemon/tests/supervisor_hotreload.rs

- **Line 118**: Should have no runtimes initially
- **Line 135**: Should have one runtime
- **Line 136**: test-project
- **Line 172**: Should have three runtimes
- **Line 207**: Should have two runtimes initially
- **Line 220**: Should have one runtime after removal
- **Line 221**: test-project
- **Line 247**: No-op reconcile should succeed
- **Line 258**: invalid-project
- **Line 259**: invalid-project

#### hoop-daemon/tests/supervisor_isolation.rs

- **Line 144**: Should have two runtimes
- **Line 187**: Both runtimes should still exist
- **Line 204**: Both runtimes should still exist
- **Line 388**: {} should have its own bead reader running
- **Line 433**: {} should have its own session tailer running

#### hoop-daemon/tests/supervisor_restart.rs

- **Line 121**: Workspace path does not exist: /nonexistent
- **Line 129**: .beads directory not found at: /path
- **Line 215**: starting
- **Line 221**: starting
- **Line 231**: healthy
- **Line 232**: failed
- **Line 234**: failed
- **Line 245**: error
- **Line 254**: abandoned

#### hoop-daemon/tests/supervisor_shutdown.rs

- **Line 125**: Should have one runtime
- **Line 136**: Runtime should still exist
- **Line 166**: Should have two runtimes
- **Line 179**: Should have one runtime after removal
- **Line 180**: project-1
- **Line 215**: Should have three runtimes
- **Line 228**: Should have no runtimes after shutdown
- **Line 270**: Runtime should be in running state, got: {:?}
- **Line 306**: Cycle {}: Should have one runtime
- **Line 323**: Cycle {}: Should have no runtimes

#### hoop-daemon/tests/testrepo_harness_integration.rs

- **Line 264**: Health check should return ok
- **Line 268**: Ready check should return ok
- **Line 300**: First message must be init
- **Line 464**: topic
- **Line 524**: init
- **Line 566**: Failed to reconnect
- **Line 586**: Timeout waiting for snapshots after reconnect

#### hoop-daemon/tests/testrepo_integration.rs

- **Line 244**: Health check should return ok
- **Line 248**: Ready check should return ok
- **Line 280**: First message must be init
- **Line 356**: WS and REST bead counts should match
- **Line 363**: WS and REST worker counts should match
- **Line 370**: WS and REST project counts should match
- **Line 379**: WS and REST config valid status should match
- **Line 481**: topic
- **Line 541**: init
- **Line 583**: Failed to reconnect
- **Line 603**: Timeout waiting for snapshots after reconnect

#### hoop-daemon/tests/upload_secrets_scan.rs

- **Line 264**: Should write one audit entry

#### hoop-daemon/tests/zero_write_invariant.rs

- **Line 194**: bd-abc123
- **Line 203**: create-only-write
- **Line 251**: list
- **Line 267**: zero-write-v01
- **Line 277**: ZERO_WRITE_ACTIVE should match cfg!(feature = \
- **Line 286**: CREATE_ONLY_ACTIVE should match cfg!(feature = \
- **Line 295**: WRITE_RESTRICTED should be true when any write restriction is active

#### hoop-daemon/tests_phase5/adapter_failover.rs

- **Line 102**: glm-5
- **Line 123**: anthropic-session-123
- **Line 188**: Stitch should be in hoop-agent project
- **Line 189**: Stitch should be kind=operator
- **Line 204**: All history messages should be stored
- **Line 215**: Agent session should be linked to the archived stitch
- **Line 268**: Session should be marked as switched
- **Line 269**: Archived reason should be 'switched'
- **Line 336**: Only one session should be active
- **Line 346**: Active adapter should be zai
- **Line 449**: Both Reflection Ledger entries should be preserved
- **Line 526**: Should have exactly one active session
- **Line 529**: Active adapter should be zai
- **Line 530**: Active model should be glm-5
- **Line 531**: New session should have 0 turns
- **Line 540**: Should have one archived session
- **Line 541**: Archived adapter should be anthropic
- **Line 545**: Archived session should preserve turn count
- **Line 549**: Archived session should preserve cost
- **Line 627**: Created by should be hoop:agent
- **Line 638**: All 4 messages should be stored
- **Line 646**: Tool message should be preserved
- **Line 715**: Message count should match
- **Line 718**: Role mismatch at message {}
- **Line 719**: Content mismatch at message {}
- **Line 800**: Only approved entries should appear

#### hoop-daemon/tests_phase5/adapter_failover_integration.rs

- **Line 77**: glm-5
- **Line 98**: Phase 5 test - temporarily disabled for Phase 1 CI gate (bf-5mpcl)
- **Line 176**: Stitch should be created
- **Line 187**: Stitch should be in hoop-agent project
- **Line 188**: Stitch should be kind=operator
- **Line 202**: All conversation messages should be preserved
- **Line 216**: Session should be marked as switched
- **Line 217**: Archived reason should be adapter_switch
- **Line 231**: Agent session should be linked to the Stitch
- **Line 245**: Reflection Ledger entries should be preserved
- **Line 303**: Only one session should be active after switch
- **Line 317**: zai
- **Line 318**: SELECT status FROM agent_sessions WHERE id = ?1
- **Line 329**: switched
- **Line 370**: Cost should be preserved
- **Line 371**: Input tokens should be preserved
- **Line 372**: Output tokens should be preserved
- **Line 373**: Turn count should be preserved
- **Line 462**: SELECT COUNT(*) FROM agent_sessions WHERE status = 'switched' AND stitch_id IS NOT NULL
- **Line 468**: SELECT COUNT(*) FROM agent_sessions WHERE status = 'switched' AND stitch_id IS NOT NULL
- **Line 478**: SELECT COUNT(*) FROM stitches WHERE project = 'hoop-agent' AND kind = 'operator'
- **Line 488**: Phase 5 test - temporarily disabled for Phase 1 CI gate (bf-5mpcl)
- **Line 547**: All approved rules should be preserved
- **Line 615**: zai
- **Line 616**: glm-5
- **Line 617**: active
- **Line 618**: SELECT status, archived_reason FROM agent_sessions WHERE id = ?1
- **Line 630**: adapter_switch
- **Line 631**: adapter_switch
- **Line 675**: Only approved rules should be returned

#### hoop-daemon/tests_phase5/adapter_failover_test.rs

- **Line 158**: Daemon should be healthy
- **Line 162**: Agent spawn should succeed
- **Line 170**: Agent should be active
- **Line 174**: Daemon should remain healthy after 5xx
- **Line 190**: Agent spawn should succeed
- **Line 201**: Agent should be active
- **Line 202**: Initial adapter should be claude
- **Line 213**: Adapter switch should succeed
- **Line 242**: Should have exactly 1 active session
- **Line 243**: Should have 1 switched (archived) session
- **Line 250**: Agent should still be active
- **Line 251**: Adapter should be zai
- **Line 252**: Model should be glm-5
- **Line 268**: glm-5
- **Line 293**: Old session should be switched (archived)
- **Line 316**: Stitch kind should be 'operator'
- **Line 324**: Stitch should belong to hoop-agent project
- **Line 328**: Stitch should be created by hoop:agent
- **Line 387**: global
- **Line 388**: approved
- **Line 389**: approved
- **Line 441**: Should have 2 switched sessions
- **Line 516**: zai
- **Line 517**: Reflection Ledger entry should be preserved for continuity
- **Line 581**: Daemon should remain healthy
- **Line 603**: Agent spawn should succeed
- **Line 614**: Agent should be active
- **Line 615**: Initial adapter should be claude
- **Line 651**: Agent should still be active
- **Line 652**: Adapter should be zai after config reload
- **Line 657**: Model should be glm-5
- **Line 676**: Should have exactly 1 active session
- **Line 677**: Should have 1 switched (archived) session
- **Line 685**: Original session should be switched (archived)
- **Line 708**: Stitch kind should be 'operator'
- **Line 712**: Stitch should belong to hoop-agent project
- **Line 716**: Stitch should be created by hoop:agent
- **Line 723**: Daemon should remain healthy after hot-reload
- **Line 821**: Daemon should be healthy initially
- **Line 857**: Daemon must remain healthy after Anthropic 5xx error
- **Line 870**: /readyz should return 200 after 5xx error
- **Line 886**: Daemon should stay healthy during 503 outage (check {})
- **Line 896**: Daemon must still be healthy after 30s of Anthropic 5xx errors
- **Line 926**: schema_version: 
- **Line 947**: glm-5
- **Line 955**: Switch to ZAI should succeed
- **Line 962**: Agent should be active after switch
- **Line 963**: Should be using ZAI adapter
- **Line 967**: Daemon should be healthy after recovery

#### hoop-daemon/tests_phase5/agent_turn_audit_trail.rs

- **Line 90**: stitch-audit-test
- **Line 91**: hoop:agent:{}
- **Line 141**: created_by_actor should be set
- **Line 142**: created_by_actor should be set
- **Line 143**: created_by_session_id should be set
- **Line 144**: created_by_adapter should be set
- **Line 145**: created_by_model should be set
- **Line 146**: turn_id should be set
- **Line 157**: Should have one system note with turn reference
- **Line 222**: args_json should be valid JSON
- **Line 223**: agent_adapter
- **Line 232**: agent_model
- **Line 233**: turn_id
- **Line 234**: hoop:agent:<session-id>
- **Line 235**: hoop
- **Line 248**: hoop
- **Line 249**: agent
- **Line 250**: hoop:agent:
- **Line 251**: hoop:agent:
- **Line 255**: agent-session-reconstruct
- **Line 320**: /agent?session={}&turn={}
- **Line 321**: /agent?session={}&turn={}
- **Line 322**: /agent?session={}&turn={}
- **Line 323**: /agent?session={}&turn={}

#### hoop-daemon/tests_phase5/reflection_detector_integration.rs

- **Line 171**: Should propose 1 pattern from 3 similar negatives
- **Line 186**: Should have 1 reflection ledger entry
- **Line 188**: Rule should mention unwrap or don't: {}
- **Line 196**: Should have 3 source stitches
- **Line 235**: Should propose 1 preference pattern
- **Line 273**: Should propose 1 correction pattern
- **Line 326**: Should not propose patterns: worker stitches ignored, operator below threshold
- **Line 446**: Should not propose patterns: old stitches outside window
- **Line 572**: Should have 2 audit rows, one per injected rule
- **Line 576**: turn_index
- **Line 579**: rule_id
- **Line 580**: SELECT id, last_applied, applied_count FROM reflection_ledger WHERE status = 'approved'
- **Line 586**: SELECT id, last_applied, applied_count FROM reflection_ledger WHERE status = 'approved'
- **Line 602**: last_applied should be set
- **Line 606**: applied_count should be 1 after injection
- **Line 624**: applied_count should be 2 after second injection
- **Line 633**: Should have 4 audit rows total (2 per injection)

#### hoop-mcp/tests/create_only_stub.rs

- **Line 100**: expected exactly one invocation, got {:?}
- **Line 106**: zero-write-v01
- **Line 137**: expected 3 invocations, got {:?}
- **Line 139**: only 'create' verb should be called, got '{}'
- **Line 186**: FORBIDDEN_WRITE_VERBS has {} entries, expected {}
- **Line 275**: expected 3 invocations, got {:?}
- **Line 277**: invocation {} should be 'create', got '{}'

#### hoop-mcp/tests/forbidden_worker_steering.rs

- **Line 25**: FORBIDDEN_WORKER_STEERING_VERBS has {} entries, expected {}

#### hoop-mcp/tests/protocol_contract.rs

- **Line 48**: params
- **Line 51**: expected Method::Initialize
- **Line 56**: mcp_socket/tools_list_request.json
- **Line 122**: response must have 'result'
- **Line 174**: response must have 'result'
- **Line 226**: response must have 'result'
- **Line 278**: response must have 'result'
- **Line 333**: response must have 'result'
- **Line 381**: name
- **Line 431**: ToolCallResult must serialize 'content'
- **Line 447**: Content type must match fixture
- **Line 508**: field '{}' value mismatch between MCP body and fixture
- **Line 517**: source must always be 'agent' (protocol invariant)
- **Line 521**: has_acceptance_criteria must always be false (protocol invariant)

#### hoop-mcp/tests/socket_permissions.rs

- **Line 27**: socket must have mode 0600 (user read/write only), got 0{:o}
- **Line 108**: /tmp/test.sock
- **Line 109**: temp dir
- **Line 140**: kernel enforces same-user-only when mode is 0600

#### hoop-schema/tests/schema_drift.rs

- **Line 875**: Round-trip failed for {}: serialized value differs after round-trip

#### testrepo/src/async/runtime.rs

- **Line 16**: async/runtime

#### testrepo/src/async/task.rs

- **Line 16**: async/task

#### testrepo/src/crypto/aes.rs

- **Line 16**: crypto/aes

#### testrepo/src/crypto/hash.rs

- **Line 16**: crypto/hash

#### testrepo/src/network/http.rs

- **Line 16**: network/http

#### testrepo/src/network/tcp.rs

- **Line 16**: network/tcp

#### testrepo/src/parsing/csv.rs

- **Line 16**: parsing/csv

#### testrepo/src/parsing/json.rs

- **Line 16**: parsing/json

#### testrepo/src/storage/memory.rs

- **Line 16**: storage/memory

#### testrepo/src/storage/sql.rs

- **Line 16**: storage/sql

#### tests/acceptance/s1_morning_review.rs

- **Line 113**: Dashboard endpoint should return 200
- **Line 148**: Worker timeline endpoint should return 200
- **Line 175**: Dashboard should return 200
- **Line 200**: Dashboard should work without external services
- **Line 208**: Failed to spawn daemon
- **Line 245**: Failed to spawn daemon
- **Line 322**: Sum of project worker counts should equal total

#### tests/acceptance/s2_transcript_archaeology.rs

- **Line 114**: Beads endpoint should return 200
- **Line 261**: Conversations endpoint should return 200
- **Line 310**: Cost trends endpoint should return 200

#### tests/acceptance/s3_bead_creation_from_chat.rs

- **Line 194**: Bead list endpoint should return 200

#### tests/acceptance/s4_daemon_restart.rs

- **Line 207**: First daemon should return beads
- **Line 231**: Second daemon should return beads
- **Line 339**: Should see all beads
- **Line 387**: Should fetch beads in cycle {}

#### tests/acceptance/s5_workspace_deleted.rs

- **Line 175**: Initial readyz should return 200
- **Line 249**: Projects endpoint should still work
- **Line 300**: Failed to get readyz status after deletion

#### tests/acceptance/s6_machine_mode.rs

- **Line 118**: Status endpoint should return 200
- **Line 141**: Projects endpoint should return 200
- **Line 162**: Projects should be an array
- **Line 194**: Read endpoint {} should work without interaction
- **Line 252**: Healthz endpoint should return 200
- **Line 337**: All concurrent requests should succeed

#### tests/cli_test_helpers.rs

- **Line 347**: ),        
- **Line 394**: Extracted value must match CLI value
- **Line 470**: --no-interactive
- **Line 520**: Flag position in child args must not affect value
- **Line 582**: Confirm flag must be true
- **Line 657**: Remove's --confirm flag must be true
- **Line 670**: Global no_interactive flag must persist through entire command chain
- **Line 711**: Environment variable must be set when flag is true
- **Line 727**: 1
- **Line 738**: 1
- **Line 773**: HOOP_NO_INTERACTIVE must be '1' when no_interactive flag is true
- **Line 799**: HOOP_NO_INTERACTIVE must be '0' when no_interactive flag is false
- **Line 840**: Environment variable must be '1'
- **Line 932**: --no-interactive
- **Line 1006**: Flag accessible at Remove level
- **Line 1080**: Remove's --confirm flag must be true
- **Line 1093**: Global no_interactive flag must persist through entire command chain
- **Line 1119**: Environment variable must be set when flag is true
- **Line 1135**: 1
- **Line 1146**: 1
- **Line 1181**: HOOP_NO_INTERACTIVE must be '1' when no_interactive flag is true
- **Line 1207**: HOOP_NO_INTERACTIVE must be '0' when no_interactive flag is false
- **Line 1248**: Environment variable must be '1'
- **Line 1368**: no_interactive should be true
- **Line 1378**: no_interactive should be false
- **Line 1389**: no_interactive value must be consistent regardless of flag position
- **Line 1493**: Flag value must be position-independent for command: {} {}
- **Line 1501**: no_interactive should be true in both positions
- **Line 1522**: Flag value must be position-independent for command: {}
- **Line 1529**: no_interactive should be true in both positions
- **Line 1580**: Parent CLI should extract no_interactive={}
- **Line 1613**: Environment variable {} should be set to {} when no_interactive={}
- **Line 1624**: Flag presence in child args should match expected value: expected={}, found={}
- **Line 1687**: Environment variable {} should be set to {} when no_interactive={}
- **Line 1728**: Top-level flag should be {} for full command: {:?}
- **Line 1779**: no_interactive value must be consistent
- **Line 1780**: no_interactive should be true
- **Line 1837**: no_interactive value must be position-independent
- **Line 1838**: no_interactive should be true
- **Line 1896**: Global flag should be true when specified before command: {} {}
- **Line 1906**: Global flag should be true when specified before command: {}
- **Line 1957**: Subcommand flag should be true when specified after command: {} {}
- **Line 1967**: Subcommand flag should be true when specified after command: {}
- **Line 2032**: Global flag should propagate through command chain: {} {:?}
- **Line 2051**: Local flag should override global flag: global={}, local={}
- **Line 2077**: Flag value must be consistent across positions for command: {:?}
- **Line 2082**: Expected consistency check failed: expected {}
- **Line 2113**: hoop
- **Line 2114**: --no-interactive
- **Line 2216**: Values must match
- **Line 2217**: --no-interactive
- **Line 2228**: scan
- **Line 2229**: scan
- **Line 2357**: Global no_interactive must be true for command chain

### assert_ne! (13 instances)

#### hoop-daemon/tests/adapter_failover_test.rs

- **Line 223**: New session ID should differ from initial
- **Line 466**: Each archived session should create a distinct Stitch

#### hoop-daemon/tests/backup_restore_cycle.rs

- **Line 233**: Encrypted data should differ from original

#### hoop-daemon/tests/beads_deletion_http.rs

- **Line 210**: project-a should not be Healthy via API

#### hoop-daemon/tests/config_reload_cycle.rs

- **Line 140**: content hash must change on valid edit

#### hoop-daemon/tests/observer_mode_integration.rs

- **Line 21**: 127.0.0.1:3001

#### hoop-daemon/tests/output_capture_helpers/mod.rs

- **Line 833**: ;         let temp_dir = std::env::temp_dir();         let log_path = temp_dir.join(

#### hoop-daemon/tests/state_projections.rs

- **Line 686**: Concurrent daemons must use different ports

#### hoop-daemon/tests/stdout_generation_test.rs

- **Line 407**: stdout

#### hoop-daemon/tests/stitch_percentile_index_integration.rs

- **Line 264**: Different body length should produce different bucket
- **Line 278**: Different labels should produce different bucket

#### hoop-daemon/tests_phase5/adapter_failover_test.rs

- **Line 220**: New session ID should differ from initial
- **Line 463**: Each archived session should create a distinct Stitch

### panic! (93 instances)

#### hoop-cli/tests/clap_test_utils.rs

- **Line 1039**: scan
- **Line 1124**: /tmp
- **Line 1159**: /tmp
- **Line 1177**: scan
- **Line 1191**: remove
- **Line 1205**: projects
- **Line 1218**: /tmp
- **Line 1252**: --no-interactive
- **Line 1266**: --no-interactive
- **Line 1268**: --no-interactive

#### hoop-daemon/tests/acceptance/s6_machine_mode.rs

- **Line 404**: y/n

#### hoop-daemon/tests/backup_restore_cycle.rs

- **Line 641**: age-keygen output should contain public key

#### hoop-daemon/tests/bead_real_line_deserialization.rs

- **Line 89**: Status '{}' should deserialize to {:?}
- **Line 123**: Issue type '{}' should deserialize to {:?}
- **Line 145**: Unrecognized status '{}' should become Unknown
- **Line 167**: Unrecognized issue type '{}' should become Unknown

#### hoop-daemon/tests/epoch_sync_invariant.rs

- **Line 68**: ws://
- **Line 280**: ws://

#### hoop-daemon/tests/golden_transcripts_regression.rs

- **Line 107**: Failed to read scenario directory {scenario_path:?}: {e}
- **Line 170**: Invalid JSON on line {} of {:?}: {}\n  Line: {}
- **Line 177**: Invalid JSON on line {} of {:?}: {}\n  Line: {}
- **Line 199**: jsonl
- **Line 212**: Golden transcript file {:?} must contain at least one non-empty JSON line
- **Line 235**: jsonl
- **Line 248**: parts
- **Line 297**: jsonl
- **Line 310**: name
- **Line 361**: jsonl
- **Line 374**: failed
- **Line 502**: Failed to parse line {} of {:?}:\n  Line: {}\n  Error: {:?}
- **Line 530**: jsonl
- **Line 543**: Failed to read {:?}: {}
- **Line 579**: jsonl
- **Line 592**: Failed to read {:?}: {}
- **Line 637**: jsonl
- **Line 650**: Failure scenario {:?} for adapter '{}' must parse to at least one Error event

#### hoop-daemon/tests/integration_harness.rs

- **Line 815**: workers_snapshot

#### hoop-daemon/tests/lint_regex_global_state.rs

- **Line 102**: Found {} violation(s) of regex_global_state lint

#### hoop-daemon/tests/load_test_integration.rs

- **Line 306**: All WebSocket clients should connect

#### hoop-daemon/tests/needle_events_roundtrip.rs

- **Line 41**: testrepo/.beads/events.jsonl must exist — it is the canonical NEEDLE event schema reference
- **Line 122**: fixture must have a dispatch event
- **Line 153**: fixture must have a complete event
- **Line 192**: fixture must have a fail event
- **Line 220**: release: worker must be non-empty
- **Line 240**: timeout: worker must be non-empty
- **Line 260**: crash: worker must be non-empty
- **Line 286**: line {} parsed as Unknown — add the event type to NeedleEvent: {line}
- **Line 466**: heartbeat: worker must be non-empty
- **Line 501**: heartbeat line {} failed to parse: {line}

#### hoop-daemon/tests/per_project_redaction_integration.rs

- **Line 103**: internal-tools should have redaction policy
- **Line 118**: legacy-project should not have redaction override
- **Line 128**: project:customer-data

#### hoop-daemon/tests/projection_file_audit.rs

- **Line 229**: failed to read {}: {}
- **Line 262**: worker_state.json

#### hoop-daemon/tests/property_invariants.rs

- **Line 278**: Event type mismatch at index {}

#### hoop-daemon/tests/protocol_contract.rs

- **Line 28**: CreateDraftRequest must deserialize from fixture (daemon side)
- **Line 30**: project
- **Line 82**: CreateDraftResponse missing field '{}' (fixture declares it)
- **Line 268**: daemon not running
- **Line 288**: project:test-project
- **Line 654**: fixture {} event_type must round-trip

#### hoop-daemon/tests/pure_functions.rs

- **Line 544**: worker-2

#### hoop-daemon/tests/state_projections.rs

- **Line 211**: Must receive workers_snapshot

#### hoop-daemon/tests/testrepo_harness_integration.rs

- **Line 327**: workers_snapshot should be received

#### hoop-daemon/tests/testrepo_integration.rs

- **Line 299**: workers_snapshot should be received

#### hoop-daemon/tests/zero_write_invariant.rs

- **Line 221**: bd-abc123

#### hoop-mcp/tests/protocol_contract.rs

- **Line 26**: JsonRpcRequest must deserialize from initialize fixture
- **Line 28**: protocol_version
- **Line 58**: JsonRpcRequest must deserialize from tools/list fixture
- **Line 80**: create_stitch
- **Line 151**: prompts
- **Line 203**: resources
- **Line 255**: mcp_socket/shutdown_response.json
- **Line 396**: text
- **Line 498**: hoop-mcp must send '{}' to daemon (declared in fixture). \                  If the field was intentionally renamed, update both \                  create_stitch_via_daemon and the fixture.

#### hoop-schema/tests/schema_drift.rs

- **Line 860**: Failed to parse normalized JSON for {}: {}
- **Line 864**: Failed to parse normalized JSON for {}: {}
- **Line 868**: Failed to parse normalized JSON for {}: {}
- **Line 872**: Failed to parse normalized JSON for {}: {}

#### tests/cli_test_helpers.rs

- **Line 587**: Expected Projects command
- **Line 590**: Expected Projects command
- **Line 663**: Global no_interactive flag must persist through entire command chain
- **Line 666**: Global no_interactive flag must persist through entire command chain
- **Line 830**: create
- **Line 1010**: Expected Projects command
- **Line 1013**: Expected Projects command
- **Line 1086**: Global no_interactive flag must persist through entire command chain
- **Line 1089**: Global no_interactive flag must persist through entire command chain
- **Line 1238**: create
- **Line 2178**: remove
- **Line 2192**: --no-interactive
- **Line 2206**: /tmp
- **Line 2368**: Expected Projects command

---

## Methodology

This inventory combines two extraction sources:

1. **Error/anyhow patterns** (extracted via `extract_error_messages.sh`)
   - `.expect()`, `.expect_err()`, `.unwrap_err()`
   - `anyhow!()`, `anyhow::bail!()`, `.context()`

2. **Assertion patterns** (extracted via `extract_assertion_messages_v2.py`)
   - `assert!()`, `assert_eq!()`, `assert_ne!()`
   - `panic!()`, `unwrap()`, `unwrap_err()`

All duplicates removed based on: file path + line number + pattern type + message text.

---

## Raw Data

Complete structured data available in JSON format at:
`docs/error_messages_complete_inventory.json`
