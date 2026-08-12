# HOOP Error Message Inventory

**Generated:** 2026-08-12T14:03:18.306059Z
**Bead:** bf-5hyh6
**Total Messages:** 4757

## Sources

- error-extraction (bf-5z50g): Error/anyhow patterns from 1527 messages
- test-analysis (bf-3ysoc): Comprehensive test assertion patterns from 3230 messages

## Statistics

### By Pattern Type

| Pattern Type | Count | Percentage |
|--------------|-------|------------|
| .expect() | 1987 | 41.8% |
| expect | 754 | 15.9% |
| assert | 704 | 14.8% |
| assert! | 516 | 10.8% |
| assert_* | 404 | 8.5% |
| assertion message | 123 | 2.6% |
| panic! | 94 | 2.0% |
| panic | 47 | 1.0% |
| anyhow::bail! | 37 | 0.8% |
| unwrap_or_else panic | 30 | 0.6% |
| unwrap_or_else panic with args | 30 | 0.6% |
| anyhow | 22 | 0.5% |
| anyhow::anyhow! | 9 | 0.2% |

### By Source

| Source | Count |
|--------|-------|
| error-extraction | 1527 |
| test-analysis | 3230 |

### Top Files by Error Count

| File | Count |
|------|-------|
| hoop-daemon/tests/integration_harness.rs | 332 |
| hoop-daemon/tests/draft_queue_invariants.rs | 239 |
| hoop-cli/tests/scan_no_interactive_flag.rs | 182 |
| hoop-cli/tests/remove_no_interactive_flag.rs | 165 |
| hoop-daemon/tests/acceptance/s3_bead_creation_from_chat.rs | 165 |
| hoop-cli/tests/no_interactive_flag_behavior.rs | 155 |
| tests/cli_test_helpers.rs | 152 |
| hoop-daemon/tests/config_field_validation.rs | 148 |
| hoop-cli/tests/cli_test_helpers.rs | 136 |
| hoop-cli/tests/restore_no_interactive_flag.rs | 122 |
| hoop-daemon/tests_phase5/adapter_failover_test.rs | 110 |
| hoop-daemon/tests/adapter_failover_test.rs | 110 |
| hoop-daemon/tests/acceptance/s6_machine_mode.rs | 107 |
| hoop-cli/tests/cli_test_utils.rs | 105 |
| hoop-daemon/tests/acceptance/s4_daemon_restart.rs | 86 |
| hoop-cli/tests/clap_test_utils.rs | 79 |
| tests/acceptance/s1_morning_review.rs | 78 |
| hoop-daemon/tests/s3_bead_creation_from_chat.rs | 78 |
| hoop-daemon/tests/state_projections.rs | 76 |
| hoop-daemon/tests/multi_operator_concurrency.rs | 74 |

## Detailed Catalog

### hoop-cli/tests/clap_test_utils.rs

**Total messages in file:** 79

| Line | Pattern Type | Message |
|------|--------------|---------|
| 192 | assert | /tmp |
| 477 | assert | scan |
| 477 | assert! | /tmp |
| 480 | assert | projects |
| 480 | assert! | scan |
| 635 | assert | /tmp |
| 681 | expect | Should parse with flag before command |
| 681 | .expect() | Should parse with flag before command |
| 703 | expect | Should parse with flag after command |
| 703 | .expect() | Should parse with flag after command |
| 725 | expect | Should parse with -y flag |
| 725 | .expect() | Should parse with -y flag |
| 771 | expect | Should parse without flag |
| 771 | .expect() | Should parse without flag |
| 803 | expect | Should parse with flag before command |
| 803 | .expect() | Should parse with flag before command |
| 811 | expect | Should parse with flag after command |
| 811 | .expect() | Should parse with flag after command |
| 819 | expect | Should parse with -y flag |
| 819 | .expect() | Should parse with -y flag |
| 840 | expect | Should parse without flag |
| 840 | .expect() | Should parse without flag |
| 984 | assert | scan |
| 984 | assert! | /tmp |
| 989 | assert | scan |
| 989 | assert! | /tmp |
| 994 | assert | scan |
| 994 | assert! | /tmp |
| 999 | assert | scan |
| 999 | assert! | /tmp |
| 1037 | assert | /tmp |
| 1039 | panic | Expected Scan command |
| 1039 | panic! | Expected Scan command |
| 1122 | assert | /tmp |
| 1124 | panic | Expected Scan command |
| 1124 | panic! | Expected Scan command |
| 1157 | assert | /tmp |
| 1159 | panic | Expected Scan command |
| 1159 | panic! | Expected Scan command |
| 1174 | assert | /tmp |
| 1177 | panic | Expected Scan command |
| 1177 | panic! | Expected Scan command |
| 1188 | assert | /tmp |
| 1191 | panic | Expected Scan command |
| 1191 | panic! | Expected Scan command |
| 1202 | assert | test-project |
| 1205 | panic | Expected Remove command |
| 1205 | panic! | Expected Remove command |
| 1216 | assert | /tmp |
| 1218 | panic | Expected Projects::Scan command |
| 1218 | panic! | Expected Projects::Scan command |
| 1249 | assert | /tmp |
| 1252 | panic | Expected Scan command |
| 1252 | panic! | Expected Scan command |
| 1264 | assert | /tmp |
| 1266 | panic | Expected Projects::Scan command |
| 1266 | panic! | Expected Projects::Scan command |
| 1268 | panic | Expected Projects command |
| 1268 | panic! | Expected Projects command |
| 1319 | assert | scan |
| 1319 | assert! | /tmp |
| 1325 | assert | projects |
| 1325 | assert! | scan |
| 1331 | assert | remove |
| 1331 | assert! | test-project |
| 1346 | assert | init |
| 1384 | assert | projects |
| 1384 | assert! | scan |
| 1450 | assert | Some tests failed: {:?} |
| 1450 | assert_* | Some tests failed: {:?} |
| 1469 | assert | /tmp |
| 1472 | panic | Expected Scan command |
| 1472 | panic! | Expected Scan command |
| 1501 | assert | /tmp |
| 1503 | panic | Expected Scan command |
| 1503 | panic! | Expected Scan command |
| 1509 | assert | test-project |
| 1512 | panic | Expected Remove command |
| 1512 | panic! | Expected Remove command |

### hoop-cli/tests/cli_test_helpers.rs

**Total messages in file:** 136

| Line | Pattern Type | Message |
|------|--------------|---------|
| 75 | assert | scan |
| 75 | assert! | /tmp |
| 127 | assert | scan |
| 127 | assert! | /tmp |
| 161 | assert | scan |
| 161 | assert! | /tmp |
| 229 | expect | Failed to read main.rs |
| 229 | .expect() | Failed to read main.rs |
| 281 | assert | Parsing should succeed |
| 281 | assert! | Parsing should succeed |
| 287 | assert | Flag should be true |
| 287 | assert_* | Flag should be true |
| 290 | assert | scan |
| 317 | assert | scan |
| 339 | assert | Parsing should succeed even with flag at end |
| 339 | assert! | Parsing should succeed even with flag at end |
| 345 | assert | Flag should be true at any position |
| 345 | assert_* | Flag should be true at any position |
| 348 | assert | scan |
| 376 | assert | scan |
| 403 | assert | scan |
| 410 | assert | scan |
| 410 | assert! | /tmp |
| 437 | assert | scan |
| 442 | assert | scan |
| 445 | assert | scan |
| 445 | assert! | /tmp |
| 753 | expect | Failed to read mycommand.rs |
| 753 | .expect() | Failed to read mycommand.rs |
| 770 | expect | Failed to read main.rs |
| 770 | .expect() | Failed to read main.rs |
| 804 | expect | Failed to read projects.rs |
| 804 | .expect() | Failed to read projects.rs |
| 835 | expect | Failed to read init.rs |
| 835 | .expect() | Failed to read init.rs |
| 851 | assertion message | cannot run in non-interactive mode |
| 876 | expect | Failed to read main.rs |
| 876 | .expect() | Failed to read main.rs |
| 891 | expect | Failed to read projects.rs |
| 891 | .expect() | Failed to read projects.rs |
| 927 | assert | scan |
| 927 | assert! | /tmp |
| 930 | assert | scan |
| 930 | assert! | /tmp |
| 933 | assert | scan |
| 933 | assert! | /tmp |
| 1066 | assert | scan |
| 1073 | assert | projects |
| 1074 | assert | remove |
| 1161 | assert | scan |
| 1168 | assert | projects |
| 1169 | assert | remove |
| 1255 | assert | projects |
| 1256 | assert | remove |
| 1263 | assert | patterns |
| 1264 | assert | add |
| 1271 | assert | scan |
| 1377 | assert | scan |
| 1380 | assert | projects |
| 2108 | assert | Failed to parse command without flag |
| 2108 | assert! | Failed to parse command without flag |
| 2174 | expect | Flag before subcommand assertion failed |
| 2174 | .expect() | Flag before subcommand assertion failed |
| 2195 | expect | Flag after subcommand assertion failed |
| 2195 | .expect() | Flag after subcommand assertion failed |
| 2237 | expect | Default flag assertion failed |
| 2237 | .expect() | Default flag assertion failed |
| 2361 | assert | --no-interactive |
| 2362 | assert | -y |
| 2368 | assert | scan |
| 2369 | assert | init |
| 2370 | assert | remove |
| 2410 | assert | scan |
| 2428 | assert | projects |
| 2429 | assert | remove |
| 2439 | assert | status |
| 2449 | assert | scan |
| 2459 | assert | scan |
| 2476 | assert | projects |
| 2477 | assert | remove |
| 2487 | assert | status |
| 2497 | assert | scan |
| 2513 | assert | projects |
| 2514 | assert | remove |
| 2524 | assert | patterns |
| 2525 | assert | add |
| 2535 | assert | scan |
| 2546 | assert | status |
| 2552 | assert | scan |
| 2552 | assert_* | --no-interactive |
| 2553 | assert | -y |
| 2553 | assert_* | /tmp |
| 2554 | assert | scan |
| 2554 | assert_* | /tmp |
| 2559 | assert | scan |
| 2560 | assert | status |
| 2574 | assert | --no-interactive |
| 2579 | assert | --no-interactive |
| 2580 | assert | --json |
| 2606 | assert | No arguments provided |
| 2613 | assert | No arguments provided |
| 2620 | assert | No arguments provided |
| 2634 | assert | FlagParseResult |
| 2635 | assert | no_interactive |
| 2651 | assert | Expected no_interactive flag to be true |
| 2665 | assert | Expected no_interactive flag to be false |
| 2830 | expect | Should parse flag before subcommand |
| 2830 | .expect() | Should parse flag before subcommand |
| 2832 | assert | scan |
| 2838 | expect | Should parse flag after subcommand |
| 2838 | .expect() | Should parse flag after subcommand |
| 2840 | assert | scan |
| 2846 | expect | Should parse short flag |
| 2846 | .expect() | Should parse short flag |
| 2855 | expect | Should parse nested command |
| 2855 | .expect() | Should parse nested command |
| 2856 | assert | projects |
| 2857 | assert | remove |
| 2866 | expect | Should parse nested command with flag |
| 2866 | .expect() | Should parse nested command with flag |
| 2882 | expect | Should parse command with multiple flags |
| 2882 | .expect() | Should parse command with multiple flags |
| 2884 | assert | --verbose |
| 2885 | assert | --json |
| 2890 | expect | Should parse command without flag |
| 2890 | .expect() | Should parse command without flag |
| 2903 | assert | Direct extraction should work |
| 2903 | assert_* | Direct extraction should work |
| 2907 | assert | projects |
| 2911 | expect | Should parse successfully |
| 2911 | .expect() | Should parse successfully |
| 2919 | assert | Empty args should error |
| 2919 | assert! | Empty args should error |
| 2923 | expect | Should parse flag-only args |
| 2923 | .expect() | Should parse flag-only args |
| 2974 | assert | newcommand::run_newcommand(no_interactive) |

### hoop-cli/tests/cli_test_utils.rs

**Total messages in file:** 105

| Line | Pattern Type | Message |
|------|--------------|---------|
| 407 | expect | Failed to create .beads/ directory |
| 407 | .expect() | Failed to create .beads/ directory |
| 414 | expect | Failed to create .hoop/ directory |
| 414 | .expect() | Failed to create .hoop/ directory |
| 423 | expect | Failed to write projects.yaml |
| 423 | .expect() | Failed to write projects.yaml |
| 504 | assert | Failed to parse args: {:?} |
| 504 | assert! | Failed to parse args: {:?} |
| 506 | assert | no_interactive should be true |
| 506 | assert_* | no_interactive should be true |
| 532 | assert | Failed to parse args: {:?} |
| 532 | assert! | Failed to parse args: {:?} |
| 534 | assert | no_interactive should be true |
| 534 | assert_* | no_interactive should be true |
| 557 | assert | Failed to parse args: {:?} |
| 557 | assert! | Failed to parse args: {:?} |
| 559 | assert | no_interactive should be true with -y |
| 559 | assert_* | no_interactive should be true with -y |
| 586 | expect | Failed to parse with flag before command |
| 586 | .expect() | Failed to parse with flag before command |
| 595 | expect | Failed to parse with flag after command |
| 595 | .expect() | Failed to parse with flag after command |
| 629 | assert | Failed to parse args: {:?} |
| 629 | assert! | Failed to parse args: {:?} |
| 671 | assert | Failed to parse with flag before command |
| 671 | assert! | Failed to parse with flag before command |
| 673 | assert | no_interactive should be true before command |
| 673 | assert_* | no_interactive should be true before command |
| 682 | assert | Failed to parse with flag after command |
| 682 | assert! | Failed to parse with flag after command |
| 684 | assert | no_interactive should be true after command |
| 684 | assert_* | no_interactive should be true after command |
| 693 | assert | Failed to parse with -y flag |
| 693 | assert! | Failed to parse with -y flag |
| 695 | assert | no_interactive should be true with -y |
| 695 | assert_* | no_interactive should be true with -y |
| 711 | assert | Failed to parse without flag |
| 711 | assert! | Failed to parse without flag |
| 757 | expect | Failed to parse with flag before subcommand |
| 757 | .expect() | Failed to parse with flag before subcommand |
| 759 | assert | status |
| 764 | expect | Failed to parse with flag after subcommand |
| 764 | .expect() | Failed to parse with flag after subcommand |
| 766 | assert | status |
| 776 | assert | before |
| 776 | assert! | before |
| 777 | assert | after |
| 777 | assert! | after |
| 782 | expect | Failed to parse with -y flag |
| 782 | .expect() | Failed to parse with -y flag |
| 788 | expect | Failed to parse without flag |
| 788 | .expect() | Failed to parse without flag |
| 800 | expect | Failed to parse with flag before subcommand |
| 800 | .expect() | Failed to parse with flag before subcommand |
| 805 | expect | Failed to parse with flag after subcommand |
| 805 | .expect() | Failed to parse with flag after subcommand |
| 880 | assert | All 4 test cases should succeed |
| 880 | assert_* | All 4 test cases should succeed |
| 881 | assert | No test cases should fail |
| 881 | assert_* | No test cases should fail |
| 890 | expect | Failed to create temp dir |
| 890 | .expect() | Failed to create temp dir |
| 895 | assert | .beads |
| 900 | assert | .hoop |
| 905 | assert | projects.yaml |
| 917 | expect | Failed to create temp dir |
| 917 | .expect() | Failed to create temp dir |
| 926 | expect | Failed to parse remove with flag before |
| 926 | .expect() | Failed to parse remove with flag before |
| 928 | assert | remove |
| 929 | assert | before |
| 929 | assert! | before |
| 933 | expect | Failed to parse remove with flag after |
| 933 | .expect() | Failed to parse remove with flag after |
| 935 | assert | remove |
| 936 | assert | after |
| 936 | assert! | after |
| 964 | assert | .beads |
| 980 | assert | --no-interactive |
| 983 | assert | --no-interactive |
| 994 | assert | status |
| 1024 | assert | scan |
| 1025 | assert | scan |
| 1025 | assert_* | /tmp |
| 1035 | assert | scan |
| 1036 | assert | scan |
| 1036 | assert_* | /tmp |
| 1046 | assert | scan |
| 1047 | assert | scan |
| 1047 | assert_* | /tmp |
| 1057 | assert | scan |
| 1058 | assert | scan |
| 1058 | assert_* | /tmp |
| 1068 | assert | scan |
| 1078 | assert | scan |
| 1084 | assert | before |
| 1084 | assert! | before |
| 1090 | assert | after |
| 1090 | assert! | after |
| 1139 | expect | Failed to create temp dir |
| 1139 | .expect() | Failed to create temp dir |
| 1143 | assert | .beads |
| 1148 | expect | Failed to create temp dir |
| 1148 | .expect() | Failed to create temp dir |
| 1152 | assert | .hoop |

### hoop-cli/tests/cli_test_utils_examples.rs

**Total messages in file:** 68

| Line | Pattern Type | Message |
|------|--------------|---------|
| 21 | assert | scan |
| 22 | assert | /tmp |
| 33 | assert | scan |
| 34 | assert | /tmp |
| 52 | assert | projects |
| 53 | assert | test-project |
| 54 | assert | --confirm |
| 67 | assert | scan |
| 78 | assert | restore |
| 89 | assert | Verification should succeed: {:?} |
| 89 | assert! | Verification should succeed: {:?} |
| 99 | assert | Verification should succeed: {:?} |
| 99 | assert! | Verification should succeed: {:?} |
| 108 | assert | Should verify no flag is present: {:?} |
| 108 | assert! | Should verify no flag is present: {:?} |
| 134 | assert | Prompt should be suppressed: {:?} |
| 134 | assert! | Prompt should be suppressed: {:?} |
| 169 | assert | Should pass with --confirm flag |
| 169 | assert! | Should pass with --confirm flag |
| 235 | assert | All test cases should succeed |
| 235 | assert_* | All test cases should succeed |
| 236 | assert | No test cases should fail |
| 236 | assert_* | No test cases should fail |
| 246 | expect | Failed to create temp dir |
| 246 | .expect() | Failed to create temp dir |
| 249 | assert | Workspace directory should exist |
| 249 | assert! | Workspace directory should exist |
| 259 | expect | Failed to create temp dir |
| 259 | .expect() | Failed to create temp dir |
| 262 | assert | Registry file should exist |
| 262 | assert! | Registry file should exist |
| 270 | expect | Failed to read registry file |
| 270 | .expect() | Failed to read registry file |
| 271 | assert | projects: [] |
| 271 | assert! | Registry should have empty projects list |
| 279 | expect | Failed to create temp dir |
| 279 | .expect() | Failed to create temp dir |
| 285 | assert | Should parse scan command successfully |
| 285 | assert! | Should parse scan command successfully |
| 289 | assert | scan |
| 292 | assert | .beads |
| 313 | expect | Should parse remove command successfully |
| 313 | .expect() | Should parse remove command successfully |
| 316 | assert | --confirm |
| 323 | assert | Should succeed with --confirm flag |
| 323 | assert! | Should succeed with --confirm flag |
| 386 | assert | All complex multi-command tests should pass |
| 386 | assert! | All complex multi-command tests should pass |
| 395 | assert | Should fail with empty args |
| 395 | assert! | Should fail with empty args |
| 398 | assert | No arguments provided |
| 398 | assert! | Should have descriptive error message |
| 405 | expect | Should parse successfully |
| 405 | .expect() | Should parse successfully |
| 408 | assert | Should fail with invalid expected_position |
| 408 | assert! | Should fail with invalid expected_position |
| 421 | assert | Should require --confirm flag |
| 421 | assert! | Should require --confirm flag |
| 433 | expect | Failed to create temp dir |
| 433 | .expect() | Failed to create temp dir |
| 440 | expect | Parse with flag before should succeed |
| 440 | .expect() | Parse with flag before should succeed |
| 446 | expect | Parse with flag after should succeed |
| 446 | .expect() | Parse with flag after should succeed |
| 450 | assert | before |
| 450 | assert! | before |
| 451 | assert | after |
| 451 | assert! | after |

### hoop-cli/tests/init_no_interactive_flag.rs

**Total messages in file:** 70

| Line | Pattern Type | Message |
|------|--------------|---------|
| 24 | assert | Should successfully parse flag before subcommand |
| 24 | assert! | Should successfully parse flag before subcommand |
| 27 | assert | no_interactive should be true |
| 27 | assert_* | no_interactive should be true |
| 28 | assert | init |
| 28 | assert_* | Command should be 'init' |
| 40 | assert | Should successfully parse flag after subcommand |
| 40 | assert! | Should successfully parse flag after subcommand |
| 43 | assert | no_interactive should be true |
| 43 | assert_* | no_interactive should be true |
| 44 | assert | init |
| 44 | assert_* | Command should be 'init' |
| 56 | assert | Should successfully parse short flag before subcommand |
| 56 | assert! | Should successfully parse short flag before subcommand |
| 59 | assert | no_interactive should be true with -y |
| 59 | assert_* | no_interactive should be true with -y |
| 60 | assert | init |
| 60 | assert_* | Command should be 'init' |
| 68 | assert | Should successfully parse short flag after subcommand |
| 68 | assert! | Should successfully parse short flag after subcommand |
| 71 | assert | no_interactive should be true with -y |
| 71 | assert_* | no_interactive should be true with -y |
| 72 | assert | init |
| 72 | assert_* | Command should be 'init' |
| 80 | assert | Should successfully parse command without flag |
| 80 | assert! | Should successfully parse command without flag |
| 83 | assert | no_interactive should default to false |
| 83 | assert_* | no_interactive should default to false |
| 84 | assert | init |
| 84 | assert_* | Command should be 'init' |
| 92 | expect | init |
| 92 | .expect() | Parse should succeed |
| 95 | assert | Flag extraction should verify for 'before' position |
| 95 | assert! | Flag extraction should verify for 'before' position |
| 99 | assert | init |
| 105 | expect | init |
| 105 | .expect() | Parse should succeed |
| 108 | assert | Flag extraction should verify for 'after' position |
| 108 | assert! | Flag extraction should verify for 'after' position |
| 112 | assert | init |
| 118 | expect | hoop |
| 118 | .expect() | Parse should succeed |
| 121 | assert | Should verify no flag is present |
| 121 | assert! | Should verify no flag is present |
| 134 | expect | Failed to read main.rs |
| 134 | .expect() | Failed to read main.rs |
| 159 | expect | Failed to read init.rs |
| 159 | .expect() | Failed to read init.rs |
| 182 | expect | Failed to read init.rs |
| 182 | .expect() | Failed to read init.rs |
| 198 | assertion message | cannot run in non-interactive mode |
| 219 | expect | Failed to read init.rs |
| 219 | .expect() | Failed to read init.rs |
| 316 | expect | Failed to read init.rs |
| 316 | .expect() | Failed to read init.rs |
| 353 | expect | Failed to read init.rs |
| 353 | .expect() | Failed to read init.rs |
| 358 | expect | Should find no_interactive check |
| 358 | .expect() | Should find no_interactive check |
| 364 | expect | Should find exit(2) in no_interactive section |
| 364 | .expect() | Should find exit(2) in no_interactive section |
| 381 | assert | Should parse flag before command |
| 381 | assert! | Should parse flag before command |
| 386 | assert | Should parse flag after command |
| 386 | assert! | Should parse flag after command |
| 417 | expect | Failed to read init.rs |
| 417 | .expect() | Failed to read init.rs |
| 434 | assertion message | cannot run in non-interactive mode |
| 461 | assert | All Init command no_interactive tests verified |
| 461 | assert! | All Init command no_interactive tests verified |

### hoop-cli/tests/no_interactive_flag_behavior.rs

**Total messages in file:** 155

| Line | Pattern Type | Message |
|------|--------------|---------|
| 24 | expect | .beads |
| 24 | .expect() | Failed to create .beads/ |
| 31 | expect | Failed to create .hoop/ |
| 31 | .expect() | Failed to create .hoop/ |
| 33 | expect | projects: [] |
| 33 | .expect() | Failed to write registry |
| 42 | expect | Failed to create temp dir |
| 42 | .expect() | Failed to create temp dir |
| 50 | assert | .beads |
| 50 | assert! | Test workspace should have .beads/ |
| 60 | expect | Failed to create temp dir |
| 60 | .expect() | Failed to create temp dir |
| 67 | assert | Interactive scan requires prompts (verified by code review) |
| 67 | assert! | Interactive scan requires prompts (verified by code review) |
| 78 | assert | Scan combines no_interactive \|\| yes correctly |
| 78 | assert! | Scan combines no_interactive \|\| yes correctly |
| 94 | expect | Failed to read projects.rs |
| 94 | .expect() | Failed to read projects.rs |
| 115 | assert | Should successfully parse flag before subcommand |
| 115 | assert! | Should successfully parse flag before subcommand |
| 118 | assert | Flag should be extracted as true |
| 118 | assert_* | Flag should be extracted as true |
| 119 | assert | projects |
| 119 | assert_* | Should identify 'projects' as command |
| 120 | assert | remove |
| 120 | assert! | Should include 'remove' in args |
| 121 | assert | my-project |
| 121 | assert! | Should include project name |
| 131 | assert | Should successfully parse flag after subcommand |
| 131 | assert! | Should successfully parse flag after subcommand |
| 134 | assert | Flag should be extracted as true |
| 134 | assert_* | Flag should be extracted as true |
| 135 | assert | projects |
| 135 | assert_* | Should identify 'projects' as command |
| 136 | assert | remove |
| 136 | assert! | Should include 'remove' in args |
| 137 | assert | my-project |
| 137 | assert! | Should include project name |
| 146 | expect | Failed to read projects.rs |
| 146 | .expect() | Failed to read projects.rs |
| 167 | expect | Failed to read main.rs |
| 167 | .expect() | Failed to read main.rs |
| 181 | expect | Failed to read projects.rs |
| 181 | .expect() | Failed to read projects.rs |
| 185 | assert | Should have confirm requirement check |
| 185 | assert! | Should have confirm requirement check |
| 189 | assert | Should have prompt suppression check |
| 189 | assert! | Should have prompt suppression check |
| 212 | expect | Failed to read projects.rs |
| 212 | .expect() | Failed to read projects.rs |
| 261 | assert | Should successfully parse short flag variant |
| 261 | assert! | Should successfully parse short flag variant |
| 264 | assert | Short flag -y should set no_interactive to true |
| 264 | assert_* | Short flag -y should set no_interactive to true |
| 271 | expect | Failed to read projects.rs |
| 271 | .expect() | Failed to read projects.rs |
| 350 | assert | Should parse flag before command |
| 350 | assert! | Should parse flag before command |
| 355 | assert | Should parse flag after command |
| 355 | assert! | Should parse flag after command |
| 378 | assert | Should successfully parse command without flag |
| 378 | assert! | Should successfully parse command without flag |
| 392 | expect | Failed to read main.rs |
| 392 | .expect() | Failed to read main.rs |
| 413 | expect | Failed to read projects.rs |
| 413 | .expect() | Failed to read projects.rs |
| 432 | expect | Failed to read projects.rs |
| 432 | .expect() | Failed to read projects.rs |
| 458 | expect | Failed to read restore.rs |
| 458 | .expect() | Failed to read restore.rs |
| 476 | expect | Failed to read restore.rs |
| 476 | .expect() | Failed to read restore.rs |
| 494 | expect | Failed to read restore.rs |
| 494 | .expect() | Failed to read restore.rs |
| 516 | assert | Should successfully parse flag before subcommand |
| 516 | assert! | Should successfully parse flag before subcommand |
| 519 | assert | Flag should be extracted as true |
| 519 | assert_* | Flag should be extracted as true |
| 520 | assert | restore |
| 520 | assert_* | Should identify 'restore' as command |
| 521 | assert | restore |
| 521 | assert! | Should include 'restore' in args |
| 522 | assert | --from |
| 522 | assert! | Should include --from flag |
| 523 | assert | s3://my-bucket/backups/snap-001 |
| 523 | assert! | Should include URI |
| 533 | assert | Should successfully parse flag after subcommand |
| 533 | assert! | Should successfully parse flag after subcommand |
| 536 | assert | Flag should be extracted as true |
| 536 | assert_* | Flag should be extracted as true |
| 537 | assert | restore |
| 537 | assert_* | Should identify 'restore' as command |
| 538 | assert | restore |
| 538 | assert! | Should include 'restore' in args |
| 539 | assert | --from |
| 539 | assert! | Should include --from flag |
| 540 | assert | s3://my-bucket/backups/snap-001 |
| 540 | assert! | Should include URI |
| 549 | expect | Failed to read restore.rs |
| 549 | .expect() | Failed to read restore.rs |
| 570 | expect | Failed to read main.rs |
| 570 | .expect() | Failed to read main.rs |
| 584 | expect | Failed to read restore.rs |
| 584 | .expect() | Failed to read restore.rs |
| 588 | assert | Should have confirm requirement check |
| 588 | assert! | Should have confirm requirement check |
| 592 | assert | Should have prompt suppression check |
| 592 | assert! | Should have prompt suppression check |
| 615 | expect | Failed to read restore.rs |
| 615 | .expect() | Failed to read restore.rs |
| 671 | assert | Should successfully parse short flag variant |
| 671 | assert! | Should successfully parse short flag variant |
| 674 | assert | Short flag -y should set no_interactive to true |
| 674 | assert_* | Short flag -y should set no_interactive to true |
| 675 | assert | --confirm |
| 675 | assert! | Should include --confirm flag |
| 682 | expect | Failed to read restore.rs |
| 682 | .expect() | Failed to read restore.rs |
| 767 | assert | Should parse flag before command |
| 767 | assert! | Should parse flag before command |
| 778 | assert | Should parse flag after command |
| 778 | assert! | Should parse flag after command |
| 806 | assert | Should successfully parse command without flag |
| 806 | assert! | Should successfully parse command without flag |
| 820 | expect | Failed to read main.rs |
| 820 | .expect() | Failed to read main.rs |
| 840 | expect | Failed to read restore.rs |
| 840 | .expect() | Failed to read restore.rs |
| 866 | expect | Failed to read init.rs |
| 866 | .expect() | Failed to read init.rs |
| 874 | assertion message | cannot run in non-interactive mode |
| 889 | expect | Failed to read init.rs |
| 889 | .expect() | Failed to read init.rs |
| 913 | expect | Failed to read main.rs |
| 913 | .expect() | Failed to read main.rs |
| 950 | expect | Failed to read main.rs |
| 950 | .expect() | Failed to read main.rs |
| 963 | expect | Failed to read projects.rs |
| 963 | .expect() | Failed to read projects.rs |
| 982 | expect | Failed to read projects.rs |
| 982 | .expect() | Failed to read projects.rs |
| 1001 | expect | Failed to read restore.rs |
| 1001 | .expect() | Failed to read restore.rs |
| 1020 | expect | Failed to read init.rs |
| 1020 | .expect() | Failed to read init.rs |
| 1043 | expect | Failed to read projects.rs |
| 1043 | .expect() | Failed to read projects.rs |
| 1046 | expect | pub fn scan_projects |
| 1046 | .expect() | Should find scan_projects function |
| 1084 | expect | Failed to read projects.rs |
| 1084 | .expect() | Failed to read projects.rs |
| 1086 | expect | Failed to read restore.rs |
| 1086 | .expect() | Failed to read restore.rs |
| 1107 | expect | Failed to read init.rs |
| 1107 | .expect() | Failed to read init.rs |

### hoop-cli/tests/remove_no_interactive_flag.rs

**Total messages in file:** 165

| Line | Pattern Type | Message |
|------|--------------|---------|
| 25 | assert | Should successfully parse flag before subcommand |
| 25 | assert! | Should successfully parse flag before subcommand |
| 28 | assert | no_interactive should be true |
| 28 | assert_* | no_interactive should be true |
| 29 | assert | remove |
| 29 | assert_* | Command should be 'remove' |
| 45 | assert | Should successfully parse flag after subcommand |
| 45 | assert! | Should successfully parse flag after subcommand |
| 48 | assert | no_interactive should be true |
| 48 | assert_* | no_interactive should be true |
| 49 | assert | remove |
| 49 | assert_* | Command should be 'remove' |
| 65 | assert | Should successfully parse short flag before subcommand |
| 65 | assert! | Should successfully parse short flag before subcommand |
| 68 | assert | no_interactive should be true with -y |
| 68 | assert_* | no_interactive should be true with -y |
| 69 | assert | remove |
| 69 | assert_* | Command should be 'remove' |
| 77 | assert | Should successfully parse short flag after subcommand |
| 77 | assert! | Should successfully parse short flag after subcommand |
| 80 | assert | no_interactive should be true with -y |
| 80 | assert_* | no_interactive should be true with -y |
| 81 | assert | remove |
| 81 | assert_* | Command should be 'remove' |
| 89 | assert | Should successfully parse command without flag |
| 89 | assert! | Should successfully parse command without flag |
| 92 | assert | no_interactive should default to false |
| 92 | assert_* | no_interactive should default to false |
| 93 | assert | remove |
| 93 | assert_* | Command should be 'remove' |
| 102 | expect | Parse should succeed |
| 102 | .expect() | Parse should succeed |
| 105 | assert | Flag extraction should verify for 'before' position |
| 105 | assert! | Flag extraction should verify for 'before' position |
| 109 | assert | remove |
| 116 | expect | Parse should succeed |
| 116 | .expect() | Parse should succeed |
| 119 | assert | Flag extraction should verify for 'after' position |
| 119 | assert! | Flag extraction should verify for 'after' position |
| 123 | assert | remove |
| 130 | expect | Parse should succeed |
| 130 | .expect() | Parse should succeed |
| 133 | assert | Should verify no flag is present |
| 133 | assert! | Should verify no flag is present |
| 146 | expect | Failed to read main.rs |
| 146 | .expect() | Failed to read main.rs |
| 171 | expect | Failed to read projects.rs |
| 171 | .expect() | Failed to read projects.rs |
| 195 | expect | Failed to read projects.rs |
| 195 | .expect() | Failed to read projects.rs |
| 199 | expect | Should find remove_project function |
| 199 | .expect() | Should find remove_project function |
| 203 | expect | Should find confirm requirement check |
| 203 | .expect() | Should find confirm requirement check |
| 227 | expect | Failed to read projects.rs |
| 227 | .expect() | Failed to read projects.rs |
| 231 | expect | Should find remove_project function |
| 231 | .expect() | Should find remove_project function |
| 235 | expect | Should find confirm requirement check |
| 235 | .expect() | Should find confirm requirement check |
| 242 | expect | Should have prompt check after confirm requirement |
| 242 | .expect() | Should have prompt check after confirm requirement |
| 270 | expect | Failed to read projects.rs |
| 270 | .expect() | Failed to read projects.rs |
| 274 | expect | Should find remove_project function |
| 274 | .expect() | Should find remove_project function |
| 278 | expect | Should find prompt check |
| 278 | .expect() | Should find prompt check |
| 324 | expect | Failed to read projects.rs |
| 324 | .expect() | Failed to read projects.rs |
| 328 | expect | Should find remove_project function |
| 328 | .expect() | Should find remove_project function |
| 332 | expect | Should find prompt check |
| 332 | .expect() | Should find prompt check |
| 359 | expect | Failed to read projects.rs |
| 359 | .expect() | Failed to read projects.rs |
| 363 | expect | Should find remove_project function |
| 363 | .expect() | Should find remove_project function |
| 367 | expect | Should find confirm requirement check |
| 367 | .expect() | Should find confirm requirement check |
| 371 | expect | Should find end of confirm requirement block |
| 371 | .expect() | Should find end of confirm requirement block |
| 385 | expect | Should find prompt check after confirm requirement |
| 385 | .expect() | Should find prompt check after confirm requirement |
| 573 | assert | Should parse flag before command |
| 573 | assert! | Should parse flag before command |
| 578 | assert | Should parse flag after command |
| 578 | assert! | Should parse flag after command |
| 607 | expect | Should parse global --no-interactive flag |
| 607 | .expect() | Should parse global --no-interactive flag |
| 622 | expect | Should parse remove command without flags |
| 622 | .expect() | Should parse remove command without flags |
| 640 | expect | Parse with global flag |
| 640 | .expect() | Parse with global flag |
| 642 | assert | Global flag should produce true |
| 642 | assert_* | Global flag should produce true |
| 646 | expect | Parse without flags |
| 646 | .expect() | Parse without flags |
| 648 | assert | No flag should produce false |
| 648 | assert_* | No flag should produce false |
| 655 | expect | Should parse short -y flag |
| 655 | .expect() | Should parse short -y flag |
| 678 | expect | Parse flag before subcommand |
| 678 | .expect() | Parse flag before subcommand |
| 683 | expect | Parse flag after subcommand |
| 683 | .expect() | Parse flag after subcommand |
| 691 | assert | Both should produce true |
| 691 | assert_* | Both should produce true |
| 729 | expect | Failed to read projects.rs |
| 729 | .expect() | Failed to read projects.rs |
| 733 | expect | Should find remove_project function |
| 733 | .expect() | Should find remove_project function |
| 737 | expect | Should find confirm requirement check |
| 737 | .expect() | Should find confirm requirement check |
| 744 | anyhow | anyhow::bail! |
| 770 | expect | Failed to read projects.rs |
| 770 | .expect() | Failed to read projects.rs |
| 774 | expect | Should find remove_project function |
| 774 | .expect() | Should find remove_project function |
| 778 | expect | Should find prompt check for interactive mode |
| 778 | .expect() | Should find prompt check for interactive mode |
| 813 | expect | Failed to read projects.rs |
| 813 | .expect() | Failed to read projects.rs |
| 817 | expect | Should find remove_project function |
| 817 | .expect() | Should find remove_project function |
| 821 | expect | Should find prompt check |
| 821 | .expect() | Should find prompt check |
| 851 | expect | Failed to read projects.rs |
| 851 | .expect() | Failed to read projects.rs |
| 855 | expect | Should find remove_project function |
| 855 | .expect() | Should find remove_project function |
| 859 | expect | Should find confirm requirement check |
| 859 | .expect() | Should find confirm requirement check |
| 863 | expect | Should find prompt check after confirm requirement |
| 863 | .expect() | Should find prompt check after confirm requirement |
| 890 | expect | Failed to read projects.rs |
| 890 | .expect() | Failed to read projects.rs |
| 894 | expect | Should find remove_project function |
| 894 | .expect() | Should find remove_project function |
| 911 | expect | Should find confirm requirement check |
| 911 | .expect() | Should find confirm requirement check |
| 915 | expect | Should find prompt check after confirm requirement |
| 915 | .expect() | Should find prompt check after confirm requirement |
| 949 | expect | Failed to read projects.rs |
| 949 | .expect() | Failed to read projects.rs |
| 953 | expect | Should find remove_project function |
| 953 | .expect() | Should find remove_project function |
| 957 | expect | Should find confirm requirement check |
| 957 | .expect() | Should find confirm requirement check |
| 961 | expect | Should find end of confirm requirement block |
| 961 | .expect() | Should find end of confirm requirement block |
| 965 | expect | Should find prompt check after confirm requirement |
| 965 | .expect() | Should find prompt check after confirm requirement |
| 970 | expect | Should find removal call after checks |
| 970 | .expect() | Should find removal call after checks |
| 988 | expect | Failed to read main.rs |
| 988 | .expect() | Failed to read main.rs |
| 992 | expect | Should find Remove command handler in main.rs |
| 992 | .expect() | Should find Remove command handler in main.rs |
| 1010 | expect | Failed to read main.rs |
| 1010 | .expect() | Failed to read main.rs |
| 1012 | expect | Failed to read projects.rs |
| 1012 | .expect() | Failed to read projects.rs |
| 1076 | assert | All Remove command no_interactive tests verified |
| 1076 | assert! | All Remove command no_interactive tests verified |

### hoop-cli/tests/restore_no_interactive_flag.rs

**Total messages in file:** 122

| Line | Pattern Type | Message |
|------|--------------|---------|
| 32 | assert | Should successfully parse flag before subcommand |
| 32 | assert! | Should successfully parse flag before subcommand |
| 35 | assert | no_interactive should be true |
| 35 | assert_* | no_interactive should be true |
| 36 | assert | restore |
| 36 | assert_* | Command should be 'restore' |
| 63 | assert | Should successfully parse flag after subcommand |
| 63 | assert! | Should successfully parse flag after subcommand |
| 66 | assert | no_interactive should be true |
| 66 | assert_* | no_interactive should be true |
| 67 | assert | restore |
| 67 | assert_* | Command should be 'restore' |
| 96 | assert | no_interactive should be true with -y |
| 96 | assert_* | no_interactive should be true with -y |
| 97 | assert | restore |
| 97 | assert_* | Command should be 'restore' |
| 118 | assert | no_interactive should be true with -y |
| 118 | assert_* | no_interactive should be true with -y |
| 119 | assert | restore |
| 119 | assert_* | Command should be 'restore' |
| 133 | assert | Should successfully parse command without flag |
| 133 | assert! | Should successfully parse command without flag |
| 141 | assert | restore |
| 141 | assert_* | Command should be 'restore' |
| 156 | assert | Should successfully parse with --dry-run flag |
| 156 | assert! | Should successfully parse with --dry-run flag |
| 159 | assert | no_interactive should be true |
| 159 | assert_* | no_interactive should be true |
| 160 | assert | restore |
| 160 | assert_* | Command should be 'restore' |
| 174 | expect | Parse should succeed |
| 174 | .expect() | Parse should succeed |
| 184 | assert | restore |
| 192 | expect | Parse should succeed |
| 192 | .expect() | Parse should succeed |
| 202 | assert | restore |
| 215 | expect | Parse should succeed |
| 215 | .expect() | Parse should succeed |
| 218 | assert | Should verify no flag is present |
| 218 | assert! | Should verify no flag is present |
| 230 | expect | src/main.rs |
| 230 | .expect() | Failed to read main.rs |
| 255 | expect | Failed to read restore.rs |
| 255 | .expect() | Failed to read restore.rs |
| 281 | expect | Failed to read restore.rs |
| 281 | .expect() | Failed to read restore.rs |
| 286 | expect | Should find run_restore function |
| 286 | .expect() | Should find run_restore function |
| 291 | expect | Should find confirm requirement check |
| 291 | .expect() | Should find confirm requirement check |
| 322 | expect | Failed to read restore.rs |
| 322 | .expect() | Failed to read restore.rs |
| 327 | expect | Should find run_restore function |
| 327 | .expect() | Should find run_restore function |
| 332 | expect | Should find confirm requirement check |
| 332 | .expect() | Should find confirm requirement check |
| 340 | expect | Should have prompt check after confirm requirement |
| 340 | .expect() | Should have prompt check after confirm requirement |
| 373 | expect | Failed to read restore.rs |
| 373 | .expect() | Failed to read restore.rs |
| 378 | expect | Should find run_restore function |
| 378 | .expect() | Should find run_restore function |
| 383 | expect | Should find prompt check |
| 383 | .expect() | Should find prompt check |
| 440 | expect | Failed to read restore.rs |
| 440 | .expect() | Failed to read restore.rs |
| 445 | expect | Should find run_restore function |
| 445 | .expect() | Should find run_restore function |
| 450 | expect | Should find prompt check |
| 450 | .expect() | Should find prompt check |
| 478 | expect | Failed to read restore.rs |
| 478 | .expect() | Failed to read restore.rs |
| 483 | expect | Should find run_restore function |
| 483 | .expect() | Should find run_restore function |
| 488 | expect | Should find confirm requirement check |
| 488 | .expect() | Should find confirm requirement check |
| 493 | expect | Should find end of confirm requirement block |
| 493 | .expect() | Should find end of confirm requirement block |
| 510 | expect | Should find prompt check after confirm requirement |
| 510 | .expect() | Should find prompt check after confirm requirement |
| 531 | expect | Failed to read restore.rs |
| 531 | .expect() | Failed to read restore.rs |
| 536 | expect | run_restore must have dry_run mode |
| 536 | .expect() | run_restore must have dry_run mode |
| 565 | expect | Failed to read restore.rs |
| 565 | .expect() | Failed to read restore.rs |
| 570 | expect | restore.rs must define run_restore() |
| 570 | .expect() | restore.rs must define run_restore() |
| 575 | expect | run_restore must call manifest.validate(current) |
| 575 | .expect() | run_restore must call manifest.validate(current) |
| 578 | expect | run_restore must call move_aside_for_rollback() |
| 578 | .expect() | run_restore must call move_aside_for_rollback() |
| 593 | expect | Failed to read restore.rs |
| 593 | .expect() | Failed to read restore.rs |
| 598 | expect | restore.rs must define run_restore() |
| 598 | .expect() | restore.rs must define run_restore() |
| 603 | expect | run_restore must check no_interactive && !confirm |
| 603 | .expect() | run_restore must check no_interactive && !confirm |
| 606 | expect | run_restore must check !no_interactive for prompting |
| 606 | .expect() | run_restore must check !no_interactive for prompting |
| 619 | expect | Failed to read restore.rs |
| 619 | .expect() | Failed to read restore.rs |
| 624 | expect | Must have --confirm requirement check |
| 624 | .expect() | Must have --confirm requirement check |
| 663 | expect | Should parse flag before command |
| 663 | .expect() | Should parse flag before command |
| 668 | expect | Should parse flag after command |
| 668 | .expect() | Should parse flag after command |
| 694 | expect | Should parse -y flag |
| 694 | .expect() | Should parse -y flag |
| 702 | assert | restore |
| 702 | assert_* | Command should be 'restore' |
| 710 | expect | src/main.rs |
| 710 | .expect() | Failed to read main.rs |
| 712 | expect | Failed to read restore.rs |
| 712 | .expect() | Failed to read restore.rs |
| 779 | expect | run_restore function must exist |
| 779 | .expect() | run_restore function must exist |
| 788 | expect | manifest.validate() must be called in function body |
| 788 | .expect() | manifest.validate() must be called in function body |
| 791 | expect | move_aside_for_rollback() must be called in function body |
| 791 | .expect() | move_aside_for_rollback() must be called in function body |

### hoop-cli/tests/scan_no_interactive_flag.rs

**Total messages in file:** 182

| Line | Pattern Type | Message |
|------|--------------|---------|
| 25 | assert | Should successfully parse flag before subcommand |
| 25 | assert! | Should successfully parse flag before subcommand |
| 28 | assert | no_interactive should be true |
| 28 | assert_* | no_interactive should be true |
| 29 | assert | scan |
| 29 | assert_* | Command should be 'scan' |
| 45 | assert | Should successfully parse flag after subcommand |
| 45 | assert! | Should successfully parse flag after subcommand |
| 48 | assert | no_interactive should be true |
| 48 | assert_* | no_interactive should be true |
| 49 | assert | scan |
| 49 | assert_* | Command should be 'scan' |
| 65 | assert | Should successfully parse short flag before subcommand |
| 65 | assert! | Should successfully parse short flag before subcommand |
| 68 | assert | no_interactive should be true with -y |
| 68 | assert_* | no_interactive should be true with -y |
| 69 | assert | scan |
| 69 | assert_* | Command should be 'scan' |
| 77 | assert | Should successfully parse short flag after subcommand |
| 77 | assert! | Should successfully parse short flag after subcommand |
| 80 | assert | no_interactive should be true with -y |
| 80 | assert_* | no_interactive should be true with -y |
| 81 | assert | scan |
| 81 | assert_* | Command should be 'scan' |
| 89 | assert | Should successfully parse command without flag |
| 89 | assert! | Should successfully parse command without flag |
| 92 | assert | no_interactive should default to false |
| 92 | assert_* | no_interactive should default to false |
| 93 | assert | scan |
| 93 | assert_* | Command should be 'scan' |
| 102 | assert | Should successfully parse local --yes flag |
| 102 | assert! | Should successfully parse local --yes flag |
| 105 | assert | Global no_interactive should remain false with local --yes |
| 105 | assert_* | Global no_interactive should remain false with local --yes |
| 106 | assert | scan |
| 106 | assert_* | Command should be 'scan' |
| 119 | assert | Should successfully parse both flags |
| 119 | assert! | Should successfully parse both flags |
| 122 | assert | Global no_interactive should be true |
| 122 | assert_* | Global no_interactive should be true |
| 123 | assert | scan |
| 123 | assert_* | Command should be 'scan' |
| 135 | expect | scan |
| 135 | .expect() | Parse should succeed |
| 138 | assert | Flag extraction should verify for 'before' position |
| 138 | assert! | Flag extraction should verify for 'before' position |
| 142 | assert | scan |
| 148 | expect | scan |
| 148 | .expect() | Parse should succeed |
| 151 | assert | Flag extraction should verify for 'after' position |
| 151 | assert! | Flag extraction should verify for 'after' position |
| 155 | assert | scan |
| 161 | expect | hoop |
| 161 | .expect() | Parse should succeed |
| 164 | assert | Should verify no flag is present |
| 164 | assert! | Should verify no flag is present |
| 177 | expect | Failed to read main.rs |
| 177 | .expect() | Failed to read main.rs |
| 202 | expect | Failed to read projects.rs |
| 202 | .expect() | Failed to read projects.rs |
| 222 | expect | Failed to read main.rs |
| 222 | .expect() | Failed to read main.rs |
| 232 | expect | Should find Scan command handler |
| 232 | .expect() | Should find Scan command handler |
| 236 | expect | Should find scan_projects call with \|\| logic |
| 236 | .expect() | Should find scan_projects call with \|\| logic |
| 252 | expect | Failed to read projects.rs |
| 252 | .expect() | Failed to read projects.rs |
| 256 | expect | Should find scan_projects function |
| 256 | .expect() | Should find scan_projects function |
| 260 | expect | Should find no_interactive check in scan_projects |
| 260 | .expect() | Should find no_interactive check in scan_projects |
| 286 | expect | Failed to read projects.rs |
| 286 | .expect() | Failed to read projects.rs |
| 290 | expect | Should find scan_projects function |
| 290 | .expect() | Should find scan_projects function |
| 322 | expect | Failed to read projects.rs |
| 322 | .expect() | Failed to read projects.rs |
| 326 | expect | Should find scan_projects function |
| 326 | .expect() | Should find scan_projects function |
| 354 | expect | Failed to read projects.rs |
| 354 | .expect() | Failed to read projects.rs |
| 358 | expect | Should find scan_projects function |
| 358 | .expect() | Should find scan_projects function |
| 362 | expect | Should find no_interactive check in scan_projects |
| 362 | .expect() | Should find no_interactive check in scan_projects |
| 652 | assert | Should parse flag before command |
| 652 | assert! | Should parse flag before command |
| 657 | assert | Should parse flag after command |
| 657 | assert! | Should parse flag after command |
| 687 | expect | Failed to read main.rs |
| 687 | .expect() | Failed to read main.rs |
| 700 | expect | Failed to read main.rs |
| 700 | .expect() | Failed to read main.rs |
| 704 | expect | Should find Scan command documentation |
| 704 | .expect() | Should find Scan command documentation |
| 726 | expect | Failed to read main.rs |
| 726 | .expect() | Failed to read main.rs |
| 730 | expect | Should find Scan command handler |
| 730 | .expect() | Should find Scan command handler |
| 733 | expect | Should find scan_projects call |
| 733 | .expect() | Should find scan_projects call |
| 752 | expect | Failed to read main.rs |
| 752 | .expect() | Failed to read main.rs |
| 754 | expect | Failed to read projects.rs |
| 754 | .expect() | Failed to read projects.rs |
| 818 | assert | All Scan command no_interactive tests verified |
| 818 | assert! | All Scan command no_interactive tests verified |
| 827 | expect | Should parse global --no-interactive flag |
| 827 | .expect() | Should parse global --no-interactive flag |
| 842 | expect | Should parse local --yes flag |
| 842 | .expect() | Should parse local --yes flag |
| 857 | expect | Should parse both global --no-interactive and local --yes flags |
| 857 | .expect() | Should parse both global --no-interactive and local --yes flags |
| 872 | expect | Should parse scan command without flags |
| 872 | .expect() | Should parse scan command without flags |
| 913 | expect | Parse with global flag |
| 913 | .expect() | Parse with global flag |
| 915 | assert | Global flag should produce true |
| 915 | assert_* | Global flag should produce true |
| 919 | expect | Parse with local flag |
| 919 | .expect() | Parse with local flag |
| 921 | assert | Local flag should produce true |
| 921 | assert_* | Local flag should produce true |
| 925 | expect | Parse with both flags |
| 925 | .expect() | Parse with both flags |
| 927 | assert | Both flags should produce true |
| 927 | assert_* | Both flags should produce true |
| 931 | expect | Parse without flags |
| 931 | .expect() | Parse without flags |
| 933 | assert | No flags should produce false |
| 933 | assert_* | No flags should produce false |
| 940 | expect | Should parse short -y flag |
| 940 | .expect() | Should parse short -y flag |
| 961 | expect | Should parse with global flag only |
| 961 | .expect() | Should parse with global flag only |
| 975 | expect | Should parse with local flag only |
| 975 | .expect() | Should parse with local flag only |
| 992 | expect | Parse flag before subcommand |
| 992 | .expect() | Parse flag before subcommand |
| 997 | expect | Parse flag after subcommand |
| 997 | .expect() | Parse flag after subcommand |
| 1005 | assert | Both should produce true |
| 1005 | assert_* | Both should produce true |
| 1048 | expect | Failed to read projects.rs |
| 1048 | .expect() | Failed to read projects.rs |
| 1052 | expect | Should find scan_projects function |
| 1052 | .expect() | Should find scan_projects function |
| 1056 | expect | Should find no_interactive check |
| 1056 | .expect() | Should find no_interactive check |
| 1098 | expect | Failed to read projects.rs |
| 1098 | .expect() | Failed to read projects.rs |
| 1102 | expect | Should find scan_projects function |
| 1102 | .expect() | Should find scan_projects function |
| 1106 | expect | Should find else branch with interactive prompts |
| 1106 | .expect() | Should find else branch with interactive prompts |
| 1153 | expect | Failed to read projects.rs |
| 1153 | .expect() | Failed to read projects.rs |
| 1157 | expect | Should find scan_projects function |
| 1157 | .expect() | Should find scan_projects function |
| 1193 | expect | Failed to read projects.rs |
| 1193 | .expect() | Failed to read projects.rs |
| 1197 | expect | Should find scan_projects function |
| 1197 | .expect() | Should find scan_projects function |
| 1201 | expect | Should find no_interactive check |
| 1201 | .expect() | Should find no_interactive check |
| 1205 | expect | Should find else branch for interactive mode |
| 1205 | .expect() | Should find else branch for interactive mode |
| 1232 | expect | Failed to read projects.rs |
| 1232 | .expect() | Failed to read projects.rs |
| 1236 | expect | Should find scan_projects function |
| 1236 | .expect() | Should find scan_projects function |
| 1248 | expect | Should find no_interactive check |
| 1248 | .expect() | Should find no_interactive check |
| 1253 | expect | Should find else branch after no_interactive check |
| 1253 | .expect() | Should find else branch after no_interactive check |
| 1286 | expect | Failed to read projects.rs |
| 1286 | .expect() | Failed to read projects.rs |
| 1290 | expect | Should find scan_projects function |
| 1290 | .expect() | Should find scan_projects function |
| 1294 | expect | Should find no_interactive check |
| 1294 | .expect() | Should find no_interactive check |

### hoop-daemon/examples/populate-testrepo.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 37 | .expect() | workspace root is parent of hoop-daemon/ |

### hoop-daemon/src/integration_test_client.rs

**Total messages in file:** 15

| Line | Pattern Type | Message |
|------|--------------|---------|
| 69 | anyhow::bail! | Daemon did not become ready within {:?} |
| 99 | anyhow::bail! | GET /api/beads failed: {} |
| 115 | anyhow::bail! | GET /api/beads/{} failed: {} |
| 178 | anyhow::bail! | GET /api/capacity failed: {} |
| 193 | anyhow::bail! | GET /metrics failed: {} |
| 208 | anyhow::bail! | GET /api/workers/timeline failed: {} |
| 234 | anyhow::bail! | Health check failed: {} |
| 243 | anyhow::bail! | Readiness check failed: {} |
| 252 | anyhow::bail! | Bead ID mismatch: expected {}, got {} |
| 262 | anyhow::bail! | No bead with title '{}' found |
| 286 | anyhow::bail! | Capacity response is not an object |
| 322 | anyhow::anyhow! | WebSocket error: {} |
| 352 | anyhow::bail! | WebSocket connection closed |
| 355 | anyhow::bail! | WebSocket connection terminated |
| 361 | anyhow::bail! | Timeout waiting for bead event |

### hoop-daemon/src/load_test.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 369 | anyhow::bail! | Performance budget violations:\n{} |

### hoop-daemon/tests/acceptance/s1_morning_review.rs

**Total messages in file:** 66

| Line | Pattern Type | Message |
|------|--------------|---------|
| 29 | expect | Failed to spawn daemon |
| 29 | .expect() | Failed to spawn daemon |
| 38 | expect | Failed to fetch dashboard |
| 38 | .expect() | Failed to fetch dashboard |
| 49 | expect | Failed to parse dashboard response |
| 49 | .expect() | Failed to parse dashboard response |
| 58 | expect | total_workers must be a number |
| 58 | .expect() | total_workers must be a number |
| 72 | expect | total_spend_usd must be a number |
| 72 | .expect() | total_spend_usd must be a number |
| 86 | expect | longest_running must be an array |
| 86 | .expect() | longest_running must be an array |
| 94 | expect | Failed to fetch worker timeline |
| 94 | .expect() | Failed to fetch worker timeline |
| 102 | expect | Failed to parse timeline |
| 102 | .expect() | Failed to parse timeline |
| 118 | expect | Failed to spawn daemon |
| 118 | .expect() | Failed to spawn daemon |
| 128 | expect | Failed to fetch dashboard |
| 128 | .expect() | Failed to fetch dashboard |
| 132 | assert | Dashboard should return 200 |
| 132 | assert_* | Dashboard should return 200 |
| 153 | expect | Failed to spawn daemon |
| 153 | .expect() | Failed to spawn daemon |
| 162 | expect | Failed to fetch dashboard |
| 162 | .expect() | Failed to fetch dashboard |
| 170 | expect | Failed to parse response |
| 170 | .expect() | Failed to parse response |
| 173 | assert | range |
| 174 | assert | total_workers |
| 175 | assert | total_spend_usd |
| 176 | assert | spend_by_project |
| 177 | assert | spend_by_adapter |
| 178 | assert | workers_by_project |
| 179 | assert | longest_running |
| 194 | expect | Failed to spawn daemon |
| 194 | .expect() | Failed to spawn daemon |
| 203 | expect | Failed to fetch dashboard |
| 203 | .expect() | Failed to fetch dashboard |
| 205 | expect | Failed to parse response |
| 205 | .expect() | Failed to parse response |
| 215 | expect | Failed to fetch dashboard |
| 215 | .expect() | Failed to fetch dashboard |
| 217 | expect | Failed to parse response |
| 217 | .expect() | Failed to parse response |
| 220 | assert | range |
| 238 | expect | Failed to spawn daemon |
| 238 | .expect() | Failed to spawn daemon |
| 246 | expect | Failed to fetch dashboard |
| 246 | .expect() | Failed to fetch dashboard |
| 248 | expect | Failed to parse response |
| 248 | .expect() | Failed to parse response |
| 253 | expect | total_spend_usd must be present |
| 253 | .expect() | total_spend_usd must be present |
| 263 | expect | spend_by_project must be an array |
| 263 | .expect() | spend_by_project must be an array |
| 291 | expect | Failed to spawn daemon |
| 291 | .expect() | Failed to spawn daemon |
| 299 | expect | Failed to fetch dashboard |
| 299 | .expect() | Failed to fetch dashboard |
| 301 | expect | Failed to parse response |
| 301 | .expect() | Failed to parse response |
| 305 | expect | total_workers must be present |
| 305 | .expect() | total_workers must be present |
| 309 | expect | workers_by_project must be an array |
| 309 | .expect() | workers_by_project must be an array |

### hoop-daemon/tests/acceptance/s2_transcript_archaeology.rs

**Total messages in file:** 62

| Line | Pattern Type | Message |
|------|--------------|---------|
| 31 | expect | Failed to spawn daemon |
| 31 | .expect() | Failed to spawn daemon |
| 40 | expect | Failed to fetch beads |
| 40 | .expect() | Failed to fetch beads |
| 48 | expect | Failed to parse beads |
| 48 | .expect() | Failed to parse beads |
| 55 | expect | Bead should have an id |
| 55 | .expect() | Bead should have an id |
| 62 | expect | Failed to fetch bead events |
| 62 | .expect() | Failed to fetch bead events |
| 72 | expect | Failed to parse events |
| 72 | .expect() | Failed to parse events |
| 73 | assert | Events should be an array |
| 73 | assert! | Events should be an array |
| 92 | expect | Failed to spawn daemon |
| 92 | .expect() | Failed to spawn daemon |
| 101 | expect | Failed to fetch beads |
| 101 | .expect() | Failed to fetch beads |
| 103 | expect | Failed to parse beads |
| 103 | .expect() | Failed to parse beads |
| 109 | expect | Bead should have an id |
| 109 | .expect() | Bead should have an id |
| 118 | expect | Failed to fetch bead events |
| 118 | .expect() | Failed to fetch bead events |
| 143 | expect | Failed to spawn daemon |
| 143 | .expect() | Failed to spawn daemon |
| 152 | expect | Failed to connect to stitch endpoint |
| 152 | .expect() | Failed to connect to stitch endpoint |
| 173 | expect | Failed to spawn daemon |
| 173 | .expect() | Failed to spawn daemon |
| 190 | expect | Failed to connect to endpoint |
| 190 | .expect() | Failed to connect to endpoint |
| 212 | expect | Failed to spawn daemon |
| 212 | .expect() | Failed to spawn daemon |
| 221 | expect | Failed to fetch conversations |
| 221 | .expect() | Failed to fetch conversations |
| 229 | expect | Failed to parse conversations |
| 229 | .expect() | Failed to parse conversations |
| 250 | expect | Failed to spawn daemon |
| 250 | .expect() | Failed to spawn daemon |
| 259 | expect | Failed to fetch beads |
| 259 | .expect() | Failed to fetch beads |
| 261 | expect | Failed to parse beads |
| 261 | .expect() | Failed to parse beads |
| 287 | expect | Failed to spawn daemon |
| 287 | .expect() | Failed to spawn daemon |
| 296 | expect | Failed to fetch cost trends |
| 296 | .expect() | Failed to fetch cost trends |
| 304 | expect | Failed to parse cost data |
| 304 | .expect() | Failed to parse cost data |
| 326 | expect | Failed to spawn daemon |
| 326 | .expect() | Failed to spawn daemon |
| 335 | expect | Failed to fetch beads |
| 335 | .expect() | Failed to fetch beads |
| 337 | expect | Failed to parse beads |
| 337 | .expect() | Failed to parse beads |
| 343 | expect | Bead should have an id |
| 343 | .expect() | Bead should have an id |
| 350 | expect | Failed to fetch bead events |
| 350 | .expect() | Failed to fetch bead events |
| 353 | expect | Failed to parse events |
| 353 | .expect() | Failed to parse events |

### hoop-daemon/tests/acceptance/s3_bead_creation_from_chat.rs

**Total messages in file:** 165

| Line | Pattern Type | Message |
|------|--------------|---------|
| 41 | expect | create temp dir |
| 41 | .expect() | create temp dir |
| 56 | expect | create br script |
| 56 | .expect() | create br script |
| 57 | expect | write br script |
| 57 | .expect() | write br script |
| 62 | expect | chmod br script |
| 62 | .expect() | chmod br script |
| 107 | expect | Failed to spawn daemon |
| 107 | .expect() | Failed to spawn daemon |
| 133 | expect | Failed to create draft |
| 133 | .expect() | Failed to create draft |
| 145 | expect | Failed to parse draft response |
| 145 | .expect() | Failed to parse draft response |
| 149 | expect | draft_id should be present |
| 149 | .expect() | draft_id should be present |
| 162 | expect | Failed to list drafts |
| 162 | .expect() | Failed to list drafts |
| 164 | assert | List drafts should return 200 |
| 164 | assert_* | List drafts should return 200 |
| 169 | expect | Failed to parse list response |
| 169 | .expect() | Failed to parse list response |
| 173 | expect | drafts should be an array |
| 173 | .expect() | drafts should be an array |
| 179 | assert | Draft should appear in the draft queue |
| 179 | assert! | Draft should appear in the draft queue |
| 186 | expect | Failed to get draft |
| 186 | .expect() | Failed to get draft |
| 188 | assert | Get draft should return 200 |
| 188 | assert_* | Get draft should return 200 |
| 193 | expect | Failed to parse draft |
| 193 | .expect() | Failed to parse draft |
| 195 | assert | title |
| 195 | assert_* | Draft title should match chat input |
| 196 | assert | kind |
| 196 | assert_* | Draft kind should be fix |
| 197 | assert | source |
| 197 | assert_* | Draft source should be chat |
| 198 | assert | project |
| 198 | assert_* | Draft project should be testrepo |
| 199 | assert | status |
| 199 | assert_* | Draft status should be pending |
| 217 | expect | Failed to spawn daemon |
| 217 | .expect() | Failed to spawn daemon |
| 235 | expect | Failed to create draft |
| 235 | .expect() | Failed to create draft |
| 240 | expect | Failed to parse draft response |
| 240 | .expect() | Failed to parse draft response |
| 244 | expect | draft_id should be present |
| 244 | .expect() | draft_id should be present |
| 254 | expect | Failed to approve draft |
| 254 | .expect() | Failed to approve draft |
| 266 | expect | Failed to parse approve response |
| 266 | .expect() | Failed to parse approve response |
| 270 | expect | stitch_id should be present |
| 270 | .expect() | stitch_id should be present |
| 292 | expect | Failed to get draft |
| 292 | .expect() | Failed to get draft |
| 297 | expect | Failed to parse draft |
| 297 | .expect() | Failed to parse draft |
| 299 | assert | status |
| 299 | assert_* | Draft status should be submitted |
| 300 | assert | stitch_id |
| 300 | assert_* | Draft should have stitch_id |
| 318 | expect | Failed to spawn daemon |
| 318 | .expect() | Failed to spawn daemon |
| 336 | expect | Failed to create draft |
| 336 | .expect() | Failed to create draft |
| 341 | expect | Failed to parse draft response |
| 341 | .expect() | Failed to parse draft response |
| 345 | expect | draft_id should be present |
| 345 | .expect() | draft_id should be present |
| 353 | expect | Failed to approve draft |
| 353 | .expect() | Failed to approve draft |
| 358 | expect | Failed to parse approve response |
| 358 | .expect() | Failed to parse approve response |
| 362 | expect | stitch_id should be present |
| 362 | .expect() | stitch_id should be present |
| 369 | expect | Failed to query audit log |
| 369 | .expect() | Failed to query audit log |
| 371 | assert | Audit query should return 200 |
| 371 | assert_* | Audit query should return 200 |
| 376 | expect | Failed to parse audit response |
| 376 | .expect() | Failed to parse audit response |
| 380 | expect | audit_rows should be an array |
| 380 | .expect() | audit_rows should be an array |
| 388 | assert | Audit log should contain DraftCreated entry |
| 388 | assert! | Audit log should contain DraftCreated entry |
| 392 | expect | args |
| 392 | .expect() | args should be an object |
| 393 | assert | source |
| 393 | assert_* | DraftCreated source should be chat |
| 401 | assert | Audit log should contain DraftApproved entry |
| 401 | assert! | Audit log should contain DraftApproved entry |
| 404 | expect | args |
| 404 | .expect() | args should be an object |
| 412 | expect | actor |
| 412 | .expect() | actor should be present |
| 413 | assert | Operator identity should be present in audit log |
| 413 | assert! | Operator identity should be present in audit log |
| 434 | expect | Failed to spawn daemon |
| 434 | .expect() | Failed to spawn daemon |
| 459 | expect | Failed to create draft |
| 459 | .expect() | Failed to create draft |
| 464 | expect | Failed to parse response |
| 464 | .expect() | Failed to parse response |
| 465 | expect | draft_id |
| 465 | .expect() | draft_id present |
| 472 | expect | Failed to list drafts |
| 472 | .expect() | Failed to list drafts |
| 474 | expect | Failed to parse list |
| 474 | .expect() | Failed to parse list |
| 475 | expect | drafts |
| 475 | .expect() | drafts array |
| 478 | assert | Draft should be in queue |
| 478 | assert! | Draft should be in queue |
| 488 | expect | Failed to approve draft |
| 488 | .expect() | Failed to approve draft |
| 493 | expect | Failed to parse approve |
| 493 | .expect() | Failed to parse approve |
| 494 | expect | stitch_id |
| 494 | .expect() | stitch_id present |
| 509 | expect | Failed to query audit |
| 509 | .expect() | Failed to query audit |
| 511 | expect | Failed to parse audit |
| 511 | .expect() | Failed to parse audit |
| 512 | expect | audit_rows |
| 512 | .expect() | audit_rows array |
| 522 | assert | Audit should have DraftCreated |
| 522 | assert! | Audit should have DraftCreated |
| 523 | assert | Audit should have DraftApproved |
| 523 | assert! | Audit should have DraftApproved |
| 526 | expect | args |
| 526 | .expect() | args object |
| 527 | assert | source |
| 527 | assert_* | source should be chat |
| 530 | expect | args |
| 530 | .expect() | args object |
| 531 | assert | stitch_id |
| 531 | assert_* | stitch_id should match |
| 534 | expect | actor |
| 534 | .expect() | actor present |
| 535 | assert | operator identity should be present |
| 535 | assert! | operator identity should be present |
| 556 | expect | Failed to spawn daemon |
| 556 | .expect() | Failed to spawn daemon |
| 577 | expect | Failed to create draft |
| 577 | .expect() | Failed to create draft |
| 581 | expect | Failed to parse |
| 581 | .expect() | Failed to parse |
| 582 | expect | draft_id |
| 582 | .expect() | draft_id present |
| 589 | expect | Failed to get draft |
| 589 | .expect() | Failed to get draft |
| 593 | expect | Failed to parse draft |
| 593 | .expect() | Failed to parse draft |
| 596 | assert | id |
| 597 | assert | title |
| 598 | assert | kind |
| 599 | assert | description |
| 600 | assert | priority |
| 601 | assert | labels |
| 602 | assert | source |
| 603 | assert | project |
| 604 | assert | status |

### hoop-daemon/tests/acceptance/s4_daemon_restart.rs

**Total messages in file:** 86

| Line | Pattern Type | Message |
|------|--------------|---------|
| 32 | expect | workspace root is parent of hoop-daemon/ |
| 32 | .expect() | workspace root is parent of hoop-daemon/ |
| 106 | expect | create temp dir for test HOOP home |
| 106 | .expect() | create temp dir for test HOOP home |
| 108 | expect | create .hoop dir |
| 108 | .expect() | create .hoop dir |
| 123 | expect | write projects.yaml |
| 123 | .expect() | write projects.yaml |
| 132 | expect | write config.yml |
| 132 | .expect() | write config.yml |
| 134 | expect | data |
| 134 | .expect() | create data dir |
| 157 | expect | init fleet.db |
| 157 | .expect() | init fleet.db |
| 161 | expect | bd-001 |
| 161 | .expect() | write claim |
| 162 | expect | bd-001 |
| 162 | .expect() | write complete |
| 163 | expect | bd-002 |
| 163 | .expect() | write claim |
| 172 | expect | Failed to spawn first daemon |
| 172 | .expect() | Failed to spawn first daemon |
| 197 | expect | Failed to fetch beads from first daemon |
| 197 | .expect() | Failed to fetch beads from first daemon |
| 205 | expect | Failed to parse beads |
| 205 | .expect() | Failed to parse beads |
| 214 | expect | bd-002 |
| 214 | .expect() | write complete |
| 215 | expect | bd-003 |
| 215 | .expect() | write claim |
| 228 | expect | Failed to spawn second daemon |
| 228 | .expect() | Failed to spawn second daemon |
| 251 | expect | Failed to fetch beads from second daemon |
| 251 | .expect() | Failed to fetch beads from second daemon |
| 259 | expect | Failed to parse beads |
| 259 | .expect() | Failed to parse beads |
| 294 | expect | init fleet.db |
| 294 | .expect() | init fleet.db |
| 300 | expect | write claim |
| 300 | .expect() | write claim |
| 302 | expect | write complete |
| 302 | .expect() | write complete |
| 309 | expect | Failed to spawn first daemon |
| 309 | .expect() | Failed to spawn first daemon |
| 336 | expect | Failed to spawn second daemon |
| 336 | .expect() | Failed to spawn second daemon |
| 368 | expect | Failed to fetch beads |
| 368 | .expect() | Failed to fetch beads |
| 370 | assert | Should be able to fetch beads after rebuild |
| 370 | assert_* | Should be able to fetch beads after rebuild |
| 393 | expect | init fleet.db |
| 393 | .expect() | init fleet.db |
| 398 | expect | Failed to spawn first daemon |
| 398 | .expect() | Failed to spawn first daemon |
| 424 | expect | bd-restart-1 |
| 424 | .expect() | write claim |
| 425 | expect | bd-restart-1 |
| 425 | .expect() | write complete |
| 426 | expect | bd-restart-2 |
| 426 | .expect() | write claim |
| 438 | expect | Failed to spawn second daemon |
| 438 | .expect() | Failed to spawn second daemon |
| 457 | expect | bd-restart-2 |
| 457 | .expect() | write complete |
| 458 | expect | bd-restart-3 |
| 458 | .expect() | write claim |
| 472 | expect | Failed to fetch beads |
| 472 | .expect() | Failed to fetch beads |
| 474 | assert | Should see all beads including those created during restart |
| 474 | assert_* | Should see all beads including those created during restart |
| 496 | expect | init fleet.db |
| 496 | .expect() | init fleet.db |
| 503 | expect | bd-s4-1 |
| 503 | .expect() | write claim |
| 504 | expect | bd-s4-1 |
| 504 | .expect() | write complete |
| 510 | expect | Failed to spawn daemon |
| 510 | .expect() | Failed to spawn daemon |
| 533 | expect | Failed to fetch beads |
| 533 | .expect() | Failed to fetch beads |
| 535 | assert | Should fetch beads in cycle {} |
| 535 | assert_* | Should fetch beads in cycle {} |
| 537 | expect | Failed to parse beads |
| 537 | .expect() | Failed to parse beads |
| 560 | expect | bd-s4-{} |
| 560 | .expect() | write claim |

### hoop-daemon/tests/acceptance/s5_workspace_deleted.rs

**Total messages in file:** 57

| Line | Pattern Type | Message |
|------|--------------|---------|
| 29 | expect | Failed to create .beads dir |
| 29 | .expect() | Failed to create .beads dir |
| 31 | .expect() | Failed to create issues.jsonl |
| 39 | expect | Failed to create temp dir |
| 39 | .expect() | Failed to create temp dir |
| 41 | expect | Failed to create .hoop dir |
| 41 | .expect() | Failed to create .hoop dir |
| 70 | expect | Failed to write projects.yaml |
| 70 | .expect() | Failed to write projects.yaml |
| 78 | expect | config.yml |
| 78 | .expect() | Failed to write config.yml |
| 79 | expect | data |
| 79 | .expect() | Failed to create data dir |
| 121 | expect | Failed to bind to random port |
| 121 | .expect() | Failed to bind to random port |
| 122 | expect | Failed to get local address |
| 122 | .expect() | Failed to get local address |
| 167 | expect | Failed to get readyz status |
| 167 | .expect() | Failed to get readyz status |
| 169 | assert | Initial readyz should return 200 |
| 169 | assert_* | Initial readyz should return 200 |
| 170 | assert | ok |
| 170 | assert_* | Initial readyz status should be ok |
| 174 | expect | Failed to remove .beads from project A |
| 174 | .expect() | Failed to remove .beads from project A |
| 225 | expect | Failed to bind to random port |
| 225 | .expect() | Failed to bind to random port |
| 226 | expect | Failed to get local address |
| 226 | .expect() | Failed to get local address |
| 268 | expect | Failed to remove .beads from project A |
| 268 | .expect() | Failed to remove .beads from project A |
| 278 | expect | Failed to fetch projects |
| 278 | .expect() | Failed to fetch projects |
| 280 | assert | Projects endpoint should still work |
| 280 | assert_* | Projects endpoint should still work |
| 282 | expect | Failed to parse projects |
| 282 | .expect() | Failed to parse projects |
| 295 | expect | Failed to check health |
| 295 | .expect() | Failed to check health |
| 328 | expect | Failed to bind to random port |
| 328 | .expect() | Failed to bind to random port |
| 329 | expect | Failed to get local address |
| 329 | .expect() | Failed to get local address |
| 372 | expect | Failed to get readyz status |
| 372 | .expect() | Failed to get readyz status |
| 377 | expect | Failed to remove .beads from project A |
| 377 | .expect() | Failed to remove .beads from project A |
| 384 | expect | Failed to get readyz status after deletion |
| 384 | .expect() | Failed to get readyz status after deletion |
| 435 | expect | Failed to bind to random port |
| 435 | .expect() | Failed to bind to random port |
| 436 | expect | Failed to get local address |
| 436 | .expect() | Failed to get local address |
| 478 | expect | Failed to remove .beads |
| 478 | .expect() | Failed to remove .beads |
| 487 | expect | Failed to check health |
| 487 | .expect() | Failed to check health |

### hoop-daemon/tests/acceptance/s6_machine_mode.rs

**Total messages in file:** 107

| Line | Pattern Type | Message |
|------|--------------|---------|
| 32 | expect | Failed to create temp dir |
| 32 | .expect() | Failed to create temp dir |
| 34 | expect | Failed to create .hoop dir |
| 34 | .expect() | Failed to create .hoop dir |
| 42 | expect | Failed to write config.yml |
| 42 | .expect() | Failed to write config.yml |
| 47 | expect | Failed to write projects.yaml |
| 47 | .expect() | Failed to write projects.yaml |
| 61 | expect | Failed to create project dir |
| 61 | .expect() | Failed to create project dir |
| 64 | expect | Failed to create .beads dir |
| 64 | .expect() | Failed to create .beads dir |
| 68 | .expect() | Failed to create issues.jsonl |
| 102 | expect | Failed to write projects.yaml |
| 102 | .expect() | Failed to write projects.yaml |
| 111 | expect | Failed to run hoop status --json |
| 111 | .expect() | Failed to run hoop status --json |
| 120 | expect | Invalid UTF-8 in stdout |
| 120 | .expect() | Invalid UTF-8 in stdout |
| 124 | expect | hoop status --json should produce valid JSON |
| 124 | .expect() | hoop status --json should produce valid JSON |
| 127 | assert | JSON output should be an object |
| 127 | assert! | JSON output should be an object |
| 133 | expect | projects |
| 133 | .expect() | projects should be an array |
| 134 | assert | Should have 3 projects |
| 134 | assert_* | Should have 3 projects |
| 138 | assert | Each project should be an object |
| 138 | assert! | Each project should be an object |
| 176 | expect | Failed to write projects.yaml |
| 176 | .expect() | Failed to write projects.yaml |
| 192 | expect | Failed to run hoop status --json |
| 192 | .expect() | Failed to run hoop status --json |
| 206 | expect | Failed to spawn jq |
| 206 | .expect() | Failed to spawn jq |
| 209 | expect | Failed to open jq stdin |
| 209 | .expect() | Failed to open jq stdin |
| 212 | expect | Failed to write to jq stdin |
| 212 | .expect() | Failed to write to jq stdin |
| 217 | expect | Failed to read jq output |
| 217 | .expect() | Failed to read jq output |
| 236 | expect | Failed to create root dir |
| 236 | .expect() | Failed to create root dir |
| 241 | expect | Failed to move project |
| 241 | .expect() | Failed to move project |
| 260 | expect | Failed to run hoop projects scan --yes |
| 260 | .expect() | Failed to run hoop projects scan --yes |
| 262 | expect | Invalid UTF-8 in stdout |
| 262 | .expect() | Invalid UTF-8 in stdout |
| 263 | expect | Invalid UTF-8 in stderr |
| 263 | .expect() | Invalid UTF-8 in stderr |
| 313 | expect | Failed to write projects.yaml |
| 313 | .expect() | Failed to write projects.yaml |
| 322 | expect | Failed to run hoop status |
| 322 | .expect() | Failed to run hoop status |
| 353 | expect | Failed to run hoop status |
| 353 | .expect() | Failed to run hoop status |
| 361 | expect | Invalid UTF-8 in stdout |
| 361 | .expect() | Invalid UTF-8 in stdout |
| 365 | expect | Error output should still be valid JSON |
| 365 | .expect() | Error output should still be valid JSON |
| 384 | expect | Failed to create root dir |
| 384 | .expect() | Failed to create root dir |
| 387 | expect | Failed to move project |
| 387 | .expect() | Failed to move project |
| 404 | panic | Failed to run hoop with args: {:?} |
| 404 | panic! | Failed to run hoop with args: {:?} |
| 404 | unwrap_or_else panic | Failed to run hoop with args: {:?} |
| 404 | unwrap_or_else panic with args | Failed to run hoop with args: {:?} |
| 406 | expect | Invalid UTF-8 in stdout |
| 406 | .expect() | Invalid UTF-8 in stdout |
| 446 | expect | Failed to run hoop restore |
| 446 | .expect() | Failed to run hoop restore |
| 451 | expect | Invalid UTF-8 in stderr |
| 451 | .expect() | Invalid UTF-8 in stderr |
| 486 | expect | Failed to write projects.yaml |
| 486 | .expect() | Failed to write projects.yaml |
| 495 | expect | Failed to run hoop status --json |
| 495 | .expect() | Failed to run hoop status --json |
| 497 | expect | Invalid UTF-8 in stdout |
| 497 | .expect() | Invalid UTF-8 in stdout |
| 501 | expect | Output should be valid JSON |
| 501 | .expect() | Output should be valid JSON |
| 512 | assert | Each project should be an object |
| 512 | assert! | Each project should be an object |
| 540 | expect | Failed to run hoop status |
| 540 | .expect() | Failed to run hoop status |
| 542 | expect | Invalid UTF-8 in stdout |
| 542 | .expect() | Invalid UTF-8 in stdout |
| 543 | expect | Invalid UTF-8 in stderr |
| 543 | .expect() | Invalid UTF-8 in stderr |
| 554 | expect | Error output should be valid JSON |
| 554 | .expect() | Error output should be valid JSON |
| 584 | expect | Failed to write projects.yaml |
| 584 | .expect() | Failed to write projects.yaml |
| 594 | expect | Failed to run hoop status without TTY |
| 594 | .expect() | Failed to run hoop status without TTY |
| 602 | expect | Invalid UTF-8 in stdout |
| 602 | .expect() | Invalid UTF-8 in stdout |
| 606 | expect | Machine mode should produce valid JSON |
| 606 | .expect() | Machine mode should produce valid JSON |
| 608 | assert | JSON should be an object |
| 608 | assert! | JSON should be an object |
| 636 | expect | Failed to write projects.yaml |
| 636 | .expect() | Failed to write projects.yaml |
| 655 | expect | Thread panicked |
| 655 | .expect() | Thread panicked |

### hoop-daemon/tests/adapter_failover.rs

**Total messages in file:** 69

| Line | Pattern Type | Message |
|------|--------------|---------|
| 26 | .expect() | create temp dir |
| 28 | .expect() | create .hoop dir |
| 34 | .expect() | init fleet.db |
| 73 | .expect() | write config.yml |
| 99 | assert! | Adapter build should succeed |
| 120 | assert! | ZAI adapter build should succeed after Anthropic |
| 156 | .expect() | insert session |
| 174 | .expect() | archive session as stitch |
| 177 | .expect() | open db |
| 185 | .expect() | query stitch |
| 187 | assert_* | Stitch should be in hoop-agent project |
| 188 | assert_* | Stitch should be kind=operator |
| 201 | .expect() | count messages |
| 203 | assert_* | All history messages should be stored |
| 212 | .expect() | query linked stitch |
| 249 | .expect() | insert session |
| 253 | .expect() | archive session |
| 256 | .expect() | open db |
| 264 | .expect() | query archived session |
| 266 | assert_* | Session should be marked as switched |
| 311 | .expect() | insert session |
| 323 | .expect() | open db |
| 331 | .expect() | count active |
| 333 | assert_* | Only one session should be active |
| 341 | .expect() | get active adapter |
| 343 | assert_* | Active adapter should be zai |
| 392 | .expect() | insert entry 1 |
| 393 | .expect() | insert entry 2 |
| 415 | .expect() | insert session |
| 417 | .expect() | archive session |
| 439 | .expect() | insert new session |
| 443 | .expect() | list approved entries |
| 445 | assert_* | Both Reflection Ledger entries should be preserved |
| 448 | assert! | Global rule should be preserved |
| 484 | .expect() | insert old session |
| 488 | .expect() | archive old session |
| 510 | .expect() | insert new session |
| 514 | .expect() | list sessions |
| 521 | assert_* | Should have exactly one active session |
| 524 | assert_* | Active adapter should be zai |
| 525 | assert_* | Active model should be glm-5 |
| 526 | assert_* | New session should have 0 turns |
| 530 | .expect() | list sessions |
| 535 | assert_* | Should have one archived session |
| 579 | .expect() | insert session |
| 600 | .expect() | archive as stitch |
| 603 | .expect() | open db |
| 611 | .expect() | query stitch metadata |
| 621 | assert_* | Created by should be hoop:agent |
| 626 | .expect() | prepare query |
| 628 | .expect() | query messages |
| 632 | assert_* | All 4 messages should be stored |
| 640 | assert_* | Tool message should be preserved |
| 676 | .expect() | insert session |
| 694 | .expect() | archive as stitch |
| 697 | .expect() | open db |
| 701 | .expect() | prepare query |
| 703 | .expect() | query messages |
| 708 | assert_* | Message count should match |
| 711 | assert_* | Role mismatch at message {} |
| 712 | assert_* | Content mismatch at message {} |
| 716 | assert! | Multi-line content should be preserved |
| 717 | assert! | Quotes should be preserved |
| 718 | assert! | Code blocks should be preserved |
| 765 | .expect() | insert entry 1 |
| 766 | .expect() | insert entry 2 |
| 786 | .expect() | insert rejected |
| 790 | .expect() | list approved |
| 792 | assert_* | Only approved entries should appear |

### hoop-daemon/tests/adapter_failover_integration.rs

**Total messages in file:** 30

| Line | Pattern Type | Message |
|------|--------------|---------|
| 27 | .expect() | create temp dir |
| 29 | .expect() | create .hoop dir |
| 35 | .expect() | init fleet.db |
| 73 | assert! | Adapter build should succeed |
| 93 | assert! | ZAI adapter build should succeed after Anthropic |
| 140 | .expect() | load active session |
| 141 | .expect() | should have active session |
| 141 | assertion message | should have active session |
| 161 | .expect() | archive session as stitch |
| 165 | .expect() | archive agent session |
| 173 | assert_* | Stitch should be created |
| 184 | assert_* | Stitch should be in hoop-agent project |
| 185 | assert_* | Stitch should be kind=operator |
| 213 | assert_* | Session should be marked as switched |
| 356 | .expect() | archive session |
| 367 | assert_* | Cost should be preserved |
| 368 | assert_* | Input tokens should be preserved |
| 369 | assert_* | Output tokens should be preserved |
| 370 | assert_* | Turn count should be preserved |
| 540 | .expect() | list approved entries |
| 543 | assert_* | All approved rules should be preserved |
| 607 | .expect() | load active session should succeed |
| 608 | .expect() | should have an active session |
| 608 | assertion message | should have an active session |
| 668 | .expect() | list approved entries |
| 671 | assert_* | Only approved rules should be returned |
| 706 | .expect() | load active session |
| 707 | .expect() | should have active session |
| 707 | assertion message | should have active session |
| 711 | .expect() | archive as stitch |

### hoop-daemon/tests/adapter_failover_test.rs

**Total messages in file:** 110

| Line | Pattern Type | Message |
|------|--------------|---------|
| 49 | anyhow::bail! | Daemon did not become ready |
| 155 | .expect() | Failed to spawn daemon |
| 157 | .expect() | Failed to create client |
| 160 | .expect() | Health check failed |
| 161 | assert_* | Daemon should be healthy |
| 164 | .expect() | Failed to spawn agent |
| 165 | assert_* | Agent spawn should succeed |
| 172 | .expect() | Failed to get agent status |
| 173 | assert_* | Agent should be active |
| 176 | .expect() | Health check failed |
| 177 | assert_* | Daemon should remain healthy after 5xx |
| 187 | .expect() | Failed to spawn daemon |
| 189 | .expect() | Failed to create client |
| 192 | .expect() | Failed to spawn agent |
| 193 | assert_* | Agent spawn should succeed |
| 197 | .expect() | Should have session_db_id |
| 203 | .expect() | Failed to get agent status |
| 204 | assert_* | Agent should be active |
| 215 | .expect() | Failed to switch adapter |
| 216 | assert_* | Adapter switch should succeed |
| 220 | .expect() | Should have new session_db_id |
| 232 | .expect() | Failed to list sessions |
| 245 | assert_* | Should have exactly 1 active session |
| 246 | assert_* | Should have 1 switched (archived) session |
| 252 | .expect() | Failed to get agent status |
| 253 | assert_* | Agent should still be active |
| 254 | assert_* | Adapter should be zai |
| 255 | assert_* | Model should be glm-5 |
| 265 | .expect() | Failed to spawn daemon |
| 267 | .expect() | Failed to create client |
| 270 | .expect() | Failed to spawn agent |
| 275 | .expect() | Should have session_db_id |
| 281 | .expect() | Failed to switch adapter |
| 287 | .expect() | Failed to list sessions |
| 293 | .expect() | Should find archived session |
| 311 | .expect() | Failed to query stitch from fleet.db |
| 344 | .expect() | Failed to spawn daemon |
| 346 | .expect() | Failed to create client |
| 367 | .expect() | Failed to insert reflection entry |
| 370 | .expect() | Failed to spawn agent |
| 374 | .expect() | Failed to switch adapter |
| 378 | .expect() | Failed to list reflection entries |
| 389 | .expect() | Entry should exist |
| 402 | .expect() | Failed to spawn daemon |
| 404 | .expect() | Failed to create client |
| 407 | .expect() | Failed to spawn agent |
| 411 | .expect() | Should have session_db_id |
| 417 | .expect() | Failed to switch adapter |
| 423 | .expect() | Failed to switch adapter back |
| 427 | .expect() | Should have second session_db_id |
| 433 | .expect() | Failed to list sessions |
| 444 | assert_* | Should have 2 switched sessions |
| 450 | .expect() | Should find first archived session |
| 454 | .expect() | Should find second archived session |
| 480 | .expect() | Failed to spawn daemon |
| 482 | .expect() | Failed to create client |
| 485 | .expect() | Failed to spawn agent |
| 506 | .expect() | Failed to insert reflection entry |
| 512 | .expect() | Failed to switch adapter |
| 518 | .expect() | Failed to get agent status |
| 524 | .expect() | Failed to list reflection entries |
| 539 | .expect() | Failed to spawn daemon |
| 541 | .expect() | Failed to create client |
| 544 | .expect() | Failed to spawn agent |
| 567 | .expect() | Switch 1 should complete |
| 570 | .expect() | Switch 2 should complete |
| 579 | .expect() | Health check failed |
| 580 | assert_* | Daemon should remain healthy |
| 597 | .expect() | Failed to spawn daemon |
| 599 | .expect() | Failed to create client |
| 602 | .expect() | Failed to spawn agent |
| 603 | assert_* | Agent spawn should succeed |
| 607 | .expect() | Should have session_db_id |
| 613 | .expect() | Failed to get agent status |
| 614 | assert_* | Agent should be active |
| 639 | .expect() | Failed to write updated config.yml |
| 650 | .expect() | Failed to get agent status after config reload |
| 651 | assert_* | Agent should still be active |
| 657 | assert_* | Model should be glm-5 |
| 663 | .expect() | Failed to list sessions |
| 676 | assert_* | Should have exactly 1 active session |
| 677 | assert_* | Should have 1 switched (archived) session |
| 683 | .expect() | Should find original archived session |
| 700 | .expect() | Failed to query stitch from fleet.db |
| 722 | .expect() | Health check failed |
| 723 | assert_* | Daemon should remain healthy after hot-reload |
| 805 | .expect() | Failed to start mock Anthropic server |
| 812 | .expect() | Failed to spawn daemon |
| 814 | .expect() | Failed to create client |
| 817 | .expect() | Health check failed |
| 818 | assert_* | Daemon should be healthy initially |
| 837 | .expect() | Failed to write config with mock server URL |
| 853 | .expect() | Health check failed |
| 865 | .expect() | Ready endpoint request failed |
| 882 | .expect() | Health check failed |
| 892 | .expect() | Health check failed |
| 899 | assert! | Should have performed at least 6 health checks over 30s |
| 910 | .expect() | Failed to start mock Anthropic server |
| 917 | .expect() | Failed to spawn daemon |
| 919 | .expect() | Failed to create client |
| 922 | .expect() | Health check failed |
| 937 | .expect() | Failed to write config |
| 943 | .expect() | Health check failed |
| 950 | .expect() | Adapter switch should succeed |
| 952 | assert_* | Switch to ZAI should succeed |
| 958 | .expect() | Failed to get agent status |
| 959 | assert_* | Agent should be active after switch |
| 960 | assert_* | Should be using ZAI adapter |
| 963 | .expect() | Health check failed |
| 964 | assert_* | Daemon should be healthy after recovery |

### hoop-daemon/tests/agent_turn_audit_trail.rs

**Total messages in file:** 26

| Line | Pattern Type | Message |
|------|--------------|---------|
| 25 | .expect() | create temp dir |
| 27 | .expect() | create .hoop dir |
| 33 | .expect() | init fleet.db |
| 83 | .expect() | insert draft |
| 87 | .expect() | get draft |
| 88 | .expect() | draft exists |
| 114 | .expect() | create stitch with audit |
| 118 | .expect() | open fleet.db |
| 136 | .expect() | query stitch |
| 139 | assert_* | created_by_actor should be set |
| 140 | assert_* | created_by_session_id should be set |
| 141 | assert_* | created_by_adapter should be set |
| 142 | assert_* | created_by_model should be set |
| 143 | assert_* | turn_id should be set |
| 152 | .expect() | count system messages |
| 154 | assert_* | Should have one system note with turn reference |
| 162 | .expect() | get system message content |
| 164 | assert! | System message should reference the turn_id |
| 208 | .expect() | write audit row |
| 212 | .expect() | query audit rows |
| 217 | .expect() | should find audit row for our stitch |
| 217 | assertion message | should find audit row for our stitch |
| 227 | .expect() | args_json should be valid JSON |
| 292 | .expect() | create stitch for reconstruction |
| 296 | .expect() | open fleet.db |
| 311 | .expect() | query stitch for reconstruction |

### hoop-daemon/tests/backup_config_deserialization.rs

**Total messages in file:** 7

| Line | Pattern Type | Message |
|------|--------------|---------|
| 46 | .expect() | YAML should parse |
| 49 | .expect() | YAML→JSON conversion should succeed |
| 52 | .expect() | BackupFileConfig should deserialize |
| 72 | .expect() | YAML should parse |
| 75 | .expect() | YAML→JSON conversion should succeed |
| 78 | .expect() | BackupFileConfig should deserialize |
| 97 | .expect() | Should deserialize from JSON directly |

### hoop-daemon/tests/backup_restore_cycle.rs

**Total messages in file:** 38

| Line | Pattern Type | Message |
|------|--------------|---------|
| 67 | assert | State should be deleted |
| 67 | assert! | State should be deleted |
| 117 | expect | Attachment {} should exist after restore |
| 144 | assert | Should return None when credentials missing |
| 144 | assert! | Should return None when credentials missing |
| 153 | assert | Should succeed when encryption disabled |
| 153 | assert! | Should succeed when encryption disabled |
| 156 | assert | test-access-key |
| 157 | assert | test-secret-key |
| 158 | assert | age_key should be None when encryption disabled |
| 158 | assert! | age_key should be None when encryption disabled |
| 166 | assert | Should succeed when age key provided |
| 166 | assert! | Should succeed when age key provided |
| 169 | assert | age_key should be Some when encryption enabled |
| 169 | assert! | age_key should be Some when encryption enabled |
| 170 | assert | age1test-key-for-encryption |
| 178 | assert | Should return None when age key missing but encryption enabled |
| 178 | assert! | Should return None when age key missing but encryption enabled |
| 229 | assert | Encrypted file should exist |
| 229 | assert! | Encrypted file should exist |
| 311 | assert | Backup should fail when encryption enabled but age key missing |
| 311 | assert! | Backup should fail when encryption enabled but age key missing |
| 350 | assert | Config should have encryption enabled |
| 350 | assert! | Config should have encryption enabled |
| 351 | assert | Credentials should have age key |
| 351 | assert! | Credentials should have age key |
| 391 | assert | Config should have encryption disabled |
| 391 | assert! | Config should have encryption disabled |
| 392 | assert | Credentials should not have age key |
| 392 | assert! | Credentials should not have age key |
| 445 | assert | Cron schedule should have 5 fields |
| 445 | assert_* | Cron schedule should have 5 fields |
| 638 | expect | age-keygen should be installed for this test |
| 638 | .expect() | age-keygen should be installed for this test |
| 641 | panic | age-keygen failed: {} |
| 641 | panic! | age-keygen failed: {} |
| 651 | expect | age-keygen output should contain public key |
| 651 | .expect() | age-keygen output should contain public key |

### hoop-daemon/tests/bead_created_by_hoop_broadcast.rs

**Total messages in file:** 4

| Line | Pattern Type | Message |
|------|--------------|---------|
| 70 | .expect() | Fleet notification should be received within 200ms |
| 71 | .expect() | Fleet notification channel should not be closed |
| 110 | .expect() | Should serialize |
| 111 | .expect() | Should deserialize |

### hoop-daemon/tests/bead_real_line_deserialization.rs

**Total messages in file:** 16

| Line | Pattern Type | Message |
|------|--------------|---------|
| 40 | .expect() | Real br line must deserialize successfully |
| 62 | .expect() | Minimal bead line (without created_by/dependencies) must deserialize |
| 89 | panic! | Status '{}' must deserialize |
| 89 | unwrap_or_else panic | Status '{}' must deserialize |
| 89 | unwrap_or_else panic with args | Status '{}' must deserialize |
| 123 | panic! | Issue type '{}' must deserialize |
| 123 | unwrap_or_else panic | Issue type '{}' must deserialize |
| 123 | unwrap_or_else panic with args | Issue type '{}' must deserialize |
| 145 | panic! | Unrecognized status '{}' must deserialize as Unknown |
| 145 | unwrap_or_else panic | Unrecognized status '{}' must deserialize as Unknown |
| 145 | unwrap_or_else panic with args | Unrecognized status '{}' must deserialize as Unknown |
| 167 | panic! | Unrecognized issue type '{}' must deserialize as Unknown |
| 167 | unwrap_or_else panic | Unrecognized issue type '{}' must deserialize as Unknown |
| 167 | unwrap_or_else panic with args | Unrecognized issue type '{}' must deserialize as Unknown |
| 194 | .expect() | Bead line with extra unknown keys must deserialize |
| 216 | .expect() | Bead line with null description must deserialize |

### hoop-daemon/tests/beads_deletion_http.rs

**Total messages in file:** 9

| Line | Pattern Type | Message |
|------|--------------|---------|
| 111 | .expect() | Failed to write projects.yaml |
| 114 | .expect() | Failed to spawn daemon |
| 184 | .expect() | project-a should be in degraded list |
| 313 | .expect() | Failed to write projects.yaml |
| 316 | .expect() | Failed to spawn daemon |
| 356 | assert! | project-a should be degraded |
| 361 | assert_* | API should still be accessible |
| 410 | .expect() | Failed to spawn daemon |
| 416 | assert! | Should be healthy initially |

### hoop-daemon/tests/beads_removal_recovery.rs

**Total messages in file:** 29

| Line | Pattern Type | Message |
|------|--------------|---------|
| 26 | .expect() | Failed to create temp dir |
| 31 | .expect() | Failed to create .beads dir |
| 35 | .expect() | Failed to create issues.jsonl |
| 39 | .expect() | Failed to create events.jsonl |
| 47 | .expect() | Failed to remove .beads dir |
| 53 | .expect() | Failed to recreate .beads dir |
| 56 | .expect() | Failed to recreate issues.jsonl |
| 59 | .expect() | Failed to recreate events.jsonl |
| 104 | .expect() | Failed to write projects.yaml |
| 107 | .expect() | Failed to spawn test daemon |
| 118 | .expect() | Failed to GET /api/projects |
| 124 | .expect() | Failed to parse projects response |
| 147 | .expect() | Failed to GET /readyz |
| 167 | .expect() | Failed to GET /api/projects |
| 173 | .expect() | Failed to parse projects response |
| 202 | .expect() | Failed to GET /api/projects |
| 207 | .expect() | Failed to parse projects response |
| 238 | .expect() | Failed to GET /readyz |
| 249 | .expect() | Failed to parse readiness response |
| 280 | .expect() | Failed to POST /api/config/reload |
| 296 | .expect() | Failed to GET /readyz |
| 315 | .expect() | Failed to GET /readyz |
| 364 | .expect() | Failed to write projects.yaml |
| 367 | .expect() | Failed to spawn test daemon |
| 374 | .expect() | Failed to GET /readyz |
| 394 | .expect() | Failed to GET /readyz |
| 400 | .expect() | Failed to parse readiness response |
| 420 | .expect() | Failed to GET /api/projects |
| 425 | .expect() | Failed to parse projects response |

### hoop-daemon/tests/config_field_validation.rs

**Total messages in file:** 148

| Line | Pattern Type | Message |
|------|--------------|---------|
| 43 | assert | missing schema_version should fail |
| 43 | assert! | missing schema_version should fail |
| 45 | assert | error should include field path |
| 45 | assert! | error should include field path |
| 59 | assert | integer schema_version should fail |
| 59 | assert! | integer schema_version should fail |
| 63 | assertion message | expected should be string: {:?} |
| 66 | assert | error should include field path |
| 66 | assert! | error should include field path |
| 75 | assert | invalid schema_version format should fail |
| 75 | assert! | invalid schema_version format should fail |
| 75 | assertion message | invalid schema_version format should fail |
| 90 | assert | invalid schema_version text should fail |
| 90 | assert! | invalid schema_version text should fail |
| 90 | assertion message | invalid schema_version text should fail |
| 109 | assert | missing agent.adapter should fail |
| 109 | assert! | missing agent.adapter should fail |
| 126 | assert | integer adapter should fail |
| 126 | assert! | integer adapter should fail |
| 130 | assertion message | expected should be string: {:?} |
| 148 | assert | invalid adapter value should fail |
| 148 | assert! | invalid adapter value should fail |
| 148 | assertion message | invalid adapter value should fail |
| 165 | assert | null adapter should fail |
| 165 | assert! | null adapter should fail |
| 169 | assertion message | expected should be string: {:?} |
| 185 | assert | integer model should fail |
| 185 | assert! | integer model should fail |
| 189 | assertion message | expected should be string: {:?} |
| 209 | assert | object model should fail |
| 209 | assert! | object model should fail |
| 213 | assertion message | expected should be string: {:?} |
| 228 | assert | integer bind_addr should fail |
| 228 | assert! | integer bind_addr should fail |
| 232 | assertion message | expected should be string: {:?} |
| 252 | assert | object bind_addr should fail |
| 252 | assert! | object bind_addr should fail |
| 256 | assertion message | expected should be string: {:?} |
| 271 | assert | string metrics.enabled should fail |
| 271 | assert! | string metrics.enabled should fail |
| 275 | assertion message | expected should be boolean: {:?} |
| 293 | assert | integer metrics.enabled should fail |
| 293 | assert! | integer metrics.enabled should fail |
| 297 | assertion message | expected should be boolean: {:?} |
| 312 | assert | string metrics.port should fail |
| 312 | assert! | string metrics.port should fail |
| 316 | assertion message | expected should be integer: {:?} |
| 355 | assert | string retention_days should fail |
| 355 | assert! | string retention_days should fail |
| 359 | assertion message | expected should be integer: {:?} |
| 377 | assert | boolean retention_days should fail |
| 377 | assert! | boolean retention_days should fail |
| 381 | assertion message | expected should be integer: {:?} |
| 396 | assert | string hash_chain should fail |
| 396 | assert! | string hash_chain should fail |
| 400 | assertion message | expected should be boolean: {:?} |
| 413 | assert | integer hash_chain should fail |
| 413 | assert! | integer hash_chain should fail |
| 417 | assertion message | expected should be boolean: {:?} |
| 432 | assert | integer ui.theme should fail |
| 432 | assert! | integer ui.theme should fail |
| 436 | assertion message | expected should be string: {:?} |
| 449 | assert | invalid ui.theme value should fail |
| 449 | assert! | invalid ui.theme value should fail |
| 449 | assertion message | invalid ui.theme value should fail |
| 466 | assert | boolean ui.theme should fail |
| 466 | assert! | boolean ui.theme should fail |
| 470 | assertion message | expected should be string: {:?} |
| 485 | assert | string archive_after_days should fail |
| 485 | assert! | string archive_after_days should fail |
| 489 | assertion message | expected should be integer: {:?} |
| 502 | assert | boolean archive_after_days should fail |
| 502 | assert! | boolean archive_after_days should fail |
| 506 | assertion message | expected should be integer: {:?} |
| 521 | assert | string reflection.enabled should fail |
| 521 | assert! | string reflection.enabled should fail |
| 525 | assertion message | expected should be boolean: {:?} |
| 540 | assert | string detection_threshold should fail |
| 540 | assert! | string detection_threshold should fail |
| 544 | assertion message | expected should be number: {:?} |
| 557 | assert | boolean detection_threshold should fail |
| 557 | assert! | boolean detection_threshold should fail |
| 561 | assertion message | expected should be number: {:?} |
| 576 | assert | string auto_archive_after_days should fail |
| 576 | assert! | string auto_archive_after_days should fail |
| 580 | assertion message | expected should be integer: {:?} |
| 595 | assert | string roles.viewers should fail (must be array) |
| 595 | assert! | string roles.viewers should fail (must be array) |
| 599 | assertion message | expected should be array: {:?} |
| 614 | assert | integer in viewers array should fail |
| 614 | assert! | integer in viewers array should fail |
| 618 | assertion message | expected should be string: {:?} |
| 633 | assert | string roles.drafters should fail (must be array) |
| 633 | assert! | string roles.drafters should fail (must be array) |
| 637 | assertion message | expected should be array: {:?} |
| 652 | assert | integer agent_extensions.skills should fail |
| 652 | assert! | integer agent_extensions.skills should fail |
| 656 | assertion message | expected should be string: {:?} |
| 670 | assert | array agent_extensions.scripts should fail |
| 670 | assert! | array agent_extensions.scripts should fail |
| 674 | assertion message | expected should be string: {:?} |
| 688 | assert | missing project name should fail |
| 688 | assert! | missing project name should fail |
| 706 | assert | integer project name should fail |
| 706 | assert! | integer project name should fail |
| 710 | assertion message | expected should be string: {:?} |
| 722 | assert | missing project path should fail |
| 722 | assert! | missing project path should fail |
| 740 | assert | integer project path should fail |
| 740 | assert! | integer project path should fail |
| 744 | assertion message | expected should be string: {:?} |
| 757 | assert | boolean project path should fail |
| 757 | assert! | boolean project path should fail |
| 761 | assertion message | expected should be string: {:?} |
| 775 | assert | integer project label should fail |
| 775 | assert! | integer project label should fail |
| 779 | assertion message | expected should be string: {:?} |
| 793 | assert | integer project color should fail |
| 793 | assert! | integer project color should fail |
| 797 | assertion message | expected should be string: {:?} |
| 811 | assert | string project disabled should fail |
| 811 | assert! | string project disabled should fail |
| 815 | assertion message | expected should be boolean: {:?} |
| 826 | assert | non-array projects should fail |
| 826 | assert! | non-array projects should fail |
| 843 | assert | string in projects array should fail |
| 843 | assert! | string in projects array should fail |
| 859 | assertion message | should be rejected |
| 862 | assert | unknown field should be rejected |
| 862 | assert! | unknown field should be rejected |
| 877 | assertion message | should be rejected |
| 880 | assert | unknown nested field should be rejected |
| 880 | assert! | unknown nested field should be rejected |
| 895 | assertion message | should be rejected |
| 898 | assert | unknown nested field in ui should be rejected |
| 898 | assert! | unknown nested field in ui should be rejected |
| 913 | assertion message | should be rejected |
| 916 | assert | unknown field in project entry should be rejected |
| 916 | assert! | unknown field in project entry should be rejected |
| 933 | assert | unclosed quote should fail |
| 933 | assert! | unclosed quote should fail |
| 950 | assert | unmatched bracket should fail |
| 950 | assert! | unmatched bracket should fail |
| 967 | assert | invalid escape sequence should fail |
| 967 | assert! | invalid escape sequence should fail |
| 967 | assertion message | invalid escape sequence should fail |
| 986 | assert | trailing comma should fail |
| 986 | assert! | trailing comma should fail |

### hoop-daemon/tests/config_reload_audit.rs

**Total messages in file:** 23

| Line | Pattern Type | Message |
|------|--------------|---------|
| 48 | .expect() | tempdir |
| 51 | .expect() | init fleet db |
| 66 | .expect() | tempdir for projects |
| 115 | .expect() | write audit row |
| 120 | assert! | hash chain must advance |
| 124 | .expect() | query |
| 125 | assert_* | should find exactly one config_reloaded row |
| 125 | assertion message | should find exactly one config_reloaded row |
| 133 | .expect() | delta_keys should be array |
| 142 | .expect() | hash chain should be valid |
| 154 | .expect() | tempdir for projects |
| 189 | .expect() | write audit row |
| 205 | .expect() | query |
| 209 | assertion message | should find exactly one config_reload_rejected row |
| 218 | .expect() | hash chain should be valid |
| 226 | .expect() | tempdir |
| 250 | assertion message | should have exactly one delta: +project:proj-two |
| 262 | assertion message | should have -project:proj-two, got: {:?} |
| 267 | assertion message | should have ~project:test-proj.paths (path changed repo1→repo2), got: {:?} |
| 279 | .expect() | tempdir for projects |
| 320 | .expect() | write audit row |
| 324 | .expect() | query |
| 353 | .expect() | hash chain should be valid after round-trip |

### hoop-daemon/tests/config_reload_cycle.rs

**Total messages in file:** 42

| Line | Pattern Type | Message |
|------|--------------|---------|
| 68 | .expect() | tempdir |
| 71 | .expect() | init fleet db |
| 90 | .expect() | tempdir for projects |
| 102 | .expect() | v1 should parse successfully |
| 103 | assert_* | v1: one project |
| 106 | assert! | content hash must be set |
| 111 | assert! | truncated YAML must be rejected |
| 135 | .expect() | v2 should parse successfully |
| 136 | assert_* | v2: two projects |
| 140 | assert_* | content hash must change on valid edit |
| 153 | assert! | missing field must be rejected |
| 161 | assert_* | v2 hash unchanged |
| 167 | .expect() | v3 should parse successfully |
| 168 | assert_* | v3: back to one project |
| 199 | .expect() | write rejected audit row |
| 221 | .expect() | write success audit row |
| 230 | .expect() | query rejected |
| 231 | assert_* | one rejected audit row |
| 239 | .expect() | query success |
| 240 | assert_* | one success audit row |
| 247 | .expect() | hash chain intact after full cycle |
| 261 | assert! | missing name should fail |
| 270 | assert! | error message should not be empty |
| 274 | assert! | integer name should fail |
| 292 | assert! | truncated YAML should fail |
| 336 | .expect() | tempdir |
| 364 | .expect() | YAML should parse fine |
| 369 | assertion message | should detect at least 2 semantic errors (no .beads + missing path), got: {:?} |
| 379 | assertion message | should detect missing .beads for no-beads-proj, got: {:?} |
| 383 | assert! | semantic error should have field path |
| 386 | assertion message | expected should say what's needed |
| 394 | assertion message | should detect nonexistent path for missing-path-proj, got: {:?} |
| 398 | assert! | missing path error should have field |
| 401 | assertion message | expected should say 'existing directory' |
| 411 | .expect() | tempdir |
| 420 | .expect() | valid config should load |
| 430 | .expect() | YAML should still parse |
| 444 | assert_* | hash unchanged |
| 449 | .expect() | fixed config should load |
| 490 | .expect() | write rejected audit |
| 498 | .expect() | query |
| 509 | .expect() | hash chain intact |

### hoop-daemon/tests/create_only_stub.rs

**Total messages in file:** 15

| Line | Pattern Type | Message |
|------|--------------|---------|
| 25 | .expect() | create temp dir |
| 40 | .expect() | create br script |
| 41 | .expect() | write br script |
| 46 | .expect() | chmod br script |
| 105 | .expect() | run fake br |
| 106 | assert! | fake br should succeed |
| 112 | assertion message | expected exactly one invocation, got {:?} |
| 159 | assert_* | expected 3 invocations, got {:?} |
| 159 | assertion message | expected 3 invocations, got {:?} |
| 203 | assertion message | expected invocation to start with '{}', got '{}' |
| 307 | .expect() | run fake br |
| 370 | .expect() | run fake br |
| 380 | assert_* | expected 3 invocations, got {:?} |
| 380 | assertion message | expected 3 invocations, got {:?} |
| 397 | assertion message | should contain stitch label |

### hoop-daemon/tests/create_stitch_no_auto_submit.rs

**Total messages in file:** 42

| Line | Pattern Type | Message |
|------|--------------|---------|
| 143 | .expect() | create temp dir for test project |
| 145 | .expect() | create project dir |
| 148 | .expect() | create .beads dir |
| 152 | .expect() | create beads.db |
| 192 | .expect() | create temp HOOP home |
| 194 | .expect() | create .hoop dir |
| 207 | .expect() | write projects.yaml |
| 217 | .expect() | write config.yml |
| 227 | .expect() | init fleet.db |
| 276 | assert_* | draft ID should match |
| 277 | assert_* | draft status should be pending |
| 278 | assert_* | draft title should match |
| 279 | assert_* | source should match combo |
| 332 | .expect() | create temp HOOP home |
| 334 | .expect() | create .hoop dir |
| 337 | .expect() | init fleet.db |
| 371 | .expect() | insert draft |
| 375 | .expect() | get draft |
| 376 | .expect() | draft exists |
| 379 | assert! | stitch_id must be None before approval |
| 393 | .expect() | update draft status |
| 397 | .expect() | get approved draft |
| 398 | .expect() | approved draft exists |
| 400 | assert_* | status should be submitted after approval |
| 419 | .expect() | create temp HOOP home |
| 421 | .expect() | create .hoop dir |
| 424 | .expect() | init fleet.db |
| 458 | .expect() | insert first draft |
| 492 | .expect() | insert second draft with force_create bypass |
| 496 | .expect() | get first draft |
| 497 | .expect() | first draft exists |
| 500 | .expect() | get second draft |
| 501 | .expect() | second draft exists |
| 525 | .expect() | create temp HOOP home |
| 527 | .expect() | create .hoop dir |
| 530 | .expect() | init fleet.db |
| 564 | .expect() | insert draft |
| 568 | .expect() | get draft |
| 569 | .expect() | draft exists |
| 599 | .expect() | create temp HOOP home |
| 601 | .expect() | create .hoop dir |
| 604 | .expect() | init fleet.db |

### hoop-daemon/tests/cross_workspace_blockers.rs

**Total messages in file:** 38

| Line | Pattern Type | Message |
|------|--------------|---------|
| 26 | .expect() | Failed to create temp dir |
| 30 | .expect() | Failed to open fleet.db |
| 42 | .expect() | Failed to insert parent stitch |
| 50 | .expect() | Failed to insert parent bead |
| 59 | .expect() | Failed to insert child stitch B |
| 67 | .expect() | Failed to insert child bead B |
| 76 | .expect() | Failed to insert child stitch C |
| 84 | .expect() | Failed to insert child bead C |
| 91 | .expect() | Failed to insert link to child B |
| 97 | .expect() | Failed to insert link to child C |
| 105 | .expect() | Failed to prepare stitch_links query |
| 114 | .expect() | Failed to query child stitches |
| 119 | assert_* | Should find 2 child stitches |
| 124 | .expect() | Should find child stitch B |
| 125 | assert_* | Workspace B should match |
| 130 | .expect() | Should find child stitch C |
| 131 | assert_* | Workspace C should match |
| 138 | .expect() | Failed to prepare stitch_beads query |
| 142 | .expect() | Failed to query child beads |
| 152 | assert_* | Should find 2 child beads |
| 157 | .expect() | Should find child bead B |
| 158 | assert_* | Bead B workspace should match |
| 163 | .expect() | Should find child bead C |
| 164 | assert_* | Bead C workspace should match |
| 174 | .expect() | Failed to create temp dir |
| 178 | .expect() | Failed to open fleet.db |
| 189 | .expect() | Failed to query workspace_from column |
| 191 | assert! | workspace_from column should exist |
| 200 | .expect() | Failed to query workspace_to column |
| 202 | assert! | workspace_to column should exist |
| 209 | .expect() | Failed to insert stitch link with workspaces |
| 217 | .expect() | Failed to query workspace columns |
| 243 | .expect() | Failed to create stitches table |
| 256 | .expect() | Failed to create stitch_beads table |
| 271 | .expect() | Failed to create stitch_links table |
| 277 | .expect() | Failed to create idx_stitch_links_from |
| 282 | .expect() | Failed to create idx_stitch_links_to |
| 287 | .expect() | Failed to create idx_stitch_beads_project |

### hoop-daemon/tests/disaster_recovery_runbook.rs

**Total messages in file:** 21

| Line | Pattern Type | Message |
|------|--------------|---------|
| 164 | assert! | fresh host has no ~/.hoop/ |
| 176 | assert_* | restored stitch data present |
| 180 | assert! | projects restored |
| 186 | assert_* | database integrity verified |
| 196 | assert! | newer version is rejected |
| 203 | assert! | error suggests upgrading |
| 224 | assert! | corrupted database fails to open |
| 251 | assert! | corrupted database is preserved |
| 252 | assert! | filename indicates corruption |
| 277 | assert! | ~/.hoop/ is gone after deletion |
| 285 | assert! | fleet.db restored |
| 286 | assert! | projects.yaml restored |
| 368 | assert! | paths updated for new host |
| 432 | assert_* | original database intact after rollback |
| 479 | assert! | local restore completes in seconds |
| 492 | assert! | corruption recovery is fast locally |
| 557 | assert! | mentions snapshot version |
| 558 | assert! | mentions current version |
| 559 | assert! | explains the problem |
| 569 | assert! | older schema version accepted |
| 591 | assert! | {} has test coverage |

### hoop-daemon/tests/draft_queue_invariants.rs

**Total messages in file:** 239

| Line | Pattern Type | Message |
|------|--------------|---------|
| 27 | expect | create temp dir |
| 27 | .expect() | create temp dir |
| 29 | expect | create .hoop dir |
| 29 | .expect() | create .hoop dir |
| 35 | expect | init fleet.db |
| 35 | .expect() | init fleet.db |
| 82 | expect | insert draft |
| 82 | .expect() | insert draft |
| 85 | expect | get draft |
| 85 | .expect() | get draft |
| 86 | expect | draft exists |
| 86 | .expect() | draft exists |
| 87 | assert | pending |
| 88 | assert | agent |
| 129 | expect | insert draft |
| 129 | .expect() | insert draft |
| 132 | expect | get draft |
| 132 | .expect() | get draft |
| 133 | expect | draft exists |
| 133 | .expect() | draft exists |
| 135 | assert | agent |
| 136 | assert | sess-worker3 |
| 137 | assert | os:agent-worker-3 |
| 206 | expect | insert draft1 |
| 206 | .expect() | insert draft1 |
| 207 | expect | insert draft2 |
| 207 | .expect() | insert draft2 |
| 209 | assert | fleet.db must persist on disk |
| 209 | assert! | fleet.db must persist on disk |
| 212 | expect | get draft1 |
| 212 | .expect() | get draft1 |
| 213 | expect | draft1 exists |
| 213 | .expect() | draft1 exists |
| 214 | assert | First draft |
| 215 | assert | pending |
| 218 | expect | get draft2 |
| 218 | .expect() | get draft2 |
| 219 | expect | draft2 exists |
| 219 | .expect() | draft2 exists |
| 220 | assert | Second draft |
| 221 | assert | edited |
| 266 | expect | insert draft |
| 266 | .expect() | insert draft |
| 270 | expect | pending |
| 270 | .expect() | list pending |
| 272 | assert | pending |
| 275 | expect | rejected |
| 275 | .expect() | list rejected |
| 277 | assert | draft-s6 |
| 281 | expect | pending |
| 281 | .expect() | list pending |
| 282 | expect | edited |
| 282 | .expect() | list edited |
| 326 | expect | insert draft |
| 326 | .expect() | insert draft |
| 340 | assert | audit row should be written successfully |
| 340 | assert! | audit row should be written successfully |
| 343 | assert | os:test-agent |
| 345 | assert | draft-audit-1 |
| 346 | assert | test-project |
| 386 | expect | insert draft |
| 386 | .expect() | insert draft |
| 399 | expect | update draft status |
| 399 | .expect() | update draft status |
| 419 | expect | write audit row |
| 419 | .expect() | write audit row |
| 426 | assert | operator |
| 429 | expect | get draft |
| 429 | .expect() | get draft |
| 430 | expect | draft exists |
| 430 | .expect() | draft exists |
| 433 | assert | approved |
| 474 | expect | insert draft |
| 474 | .expect() | insert draft |
| 488 | expect | reject draft |
| 488 | .expect() | reject draft |
| 491 | expect | get draft |
| 491 | .expect() | get draft |
| 492 | expect | draft exists |
| 492 | .expect() | draft exists |
| 494 | assert | rejected |
| 529 | expect | insert draft |
| 529 | .expect() | insert draft |
| 542 | expect | reject draft |
| 542 | .expect() | reject draft |
| 545 | expect | get draft |
| 545 | .expect() | get draft |
| 546 | expect | draft exists |
| 546 | .expect() | draft exists |
| 548 | assert | rejected |
| 581 | expect | write audit row |
| 581 | .expect() | write audit row |
| 590 | assert | rejection_reason |
| 631 | expect | insert draft |
| 631 | .expect() | insert draft |
| 641 | expect | edit draft |
| 641 | .expect() | edit draft |
| 644 | expect | get draft |
| 644 | .expect() | get draft |
| 645 | expect | draft exists |
| 645 | .expect() | draft exists |
| 647 | assert | Updated title |
| 648 | assert | Updated description |
| 650 | assert | edit must increment version |
| 650 | assert_* | edit must increment version |
| 651 | assert | edited |
| 651 | assert_* | edit must set status to 'edited' |
| 692 | expect | insert draft |
| 692 | .expect() | insert draft |
| 706 | expect | approve and submit draft |
| 706 | .expect() | approve and submit draft |
| 709 | expect | get draft |
| 709 | .expect() | get draft |
| 710 | expect | draft exists |
| 710 | .expect() | draft exists |
| 712 | assert | submitted |
| 745 | expect | write audit row |
| 745 | .expect() | write audit row |
| 756 | expect | hash chain must be valid after draft actions |
| 756 | .expect() | hash chain must be valid after draft actions |
| 774 | expect | open_draft should succeed |
| 774 | .expect() | open_draft should succeed |
| 777 | expect | get draft should succeed |
| 777 | .expect() | get draft should succeed |
| 778 | expect | draft should exist |
| 778 | .expect() | draft should exist |
| 783 | assert | opened_at should be set |
| 783 | assert! | opened_at should be set |
| 784 | assert | pending |
| 785 | assert | form |
| 799 | expect | first open should succeed |
| 799 | .expect() | first open should succeed |
| 802 | expect | get draft should succeed |
| 802 | .expect() | get draft should succeed |
| 803 | expect | draft should exist |
| 803 | .expect() | draft should exist |
| 805 | assert | os:operator-a |
| 808 | expect | abandon should succeed |
| 808 | .expect() | abandon should succeed |
| 811 | expect | get draft should succeed |
| 811 | .expect() | get draft should succeed |
| 812 | expect | draft should exist |
| 812 | .expect() | draft should exist |
| 814 | assert | abandoned_at should be set |
| 814 | assert! | abandoned_at should be set |
| 818 | expect | second open should succeed |
| 818 | .expect() | second open should succeed |
| 821 | expect | get draft should succeed |
| 821 | .expect() | get draft should succeed |
| 822 | expect | draft should exist |
| 822 | .expect() | draft should exist |
| 824 | assert | os:operator-b |
| 826 | assert | abandoned_at should be cleared on reopen |
| 826 | assert! | abandoned_at should be cleared on reopen |
| 840 | expect | open should succeed |
| 840 | .expect() | open should succeed |
| 851 | expect | autosave should succeed |
| 851 | .expect() | autosave should succeed |
| 854 | expect | get draft should succeed |
| 854 | .expect() | get draft should succeed |
| 855 | expect | draft should exist |
| 855 | .expect() | draft should exist |
| 857 | assert | Updated Title |
| 858 | assert | Updated Description |
| 859 | assert | investigation |
| 861 | assert | urgent |
| 861 | assert_* | security |
| 862 | assert | last_autosave_at should be set |
| 862 | assert! | last_autosave_at should be set |
| 875 | expect | second autosave should succeed |
| 875 | .expect() | second autosave should succeed |
| 878 | expect | get draft should succeed |
| 878 | .expect() | get draft should succeed |
| 879 | expect | draft should exist |
| 879 | .expect() | draft should exist |
| 881 | assert | autosave should not increment version |
| 881 | assert_* | autosave should not increment version |
| 895 | expect | open should succeed |
| 895 | .expect() | open should succeed |
| 898 | expect | get draft should succeed |
| 898 | .expect() | get draft should succeed |
| 899 | expect | draft should exist |
| 899 | .expect() | draft should exist |
| 901 | assert | pending |
| 906 | expect | abandon should succeed |
| 906 | .expect() | abandon should succeed |
| 909 | expect | get draft should succeed |
| 909 | .expect() | get draft should succeed |
| 910 | expect | draft should exist |
| 910 | .expect() | draft should exist |
| 912 | assert | abandoned |
| 913 | assert | abandoned_at should be set |
| 913 | assert! | abandoned_at should be set |
| 953 | expect | insert draft |
| 953 | .expect() | insert draft |
| 1002 | expect | insert old draft |
| 1002 | .expect() | insert old draft |
| 1035 | expect | insert recent draft |
| 1035 | .expect() | insert recent draft |
| 1039 | expect | cleanup should succeed |
| 1039 | .expect() | cleanup should succeed |
| 1041 | assert | should delete exactly one old draft |
| 1041 | assert_* | should delete exactly one old draft |
| 1041 | assertion message | should delete exactly one old draft |
| 1045 | expect | get draft should succeed |
| 1045 | .expect() | get draft should succeed |
| 1047 | assert | old abandoned draft should be deleted |
| 1047 | assert! | old abandoned draft should be deleted |
| 1051 | expect | get draft should succeed |
| 1051 | .expect() | get draft should succeed |
| 1052 | expect | recent abandoned draft should still exist |
| 1052 | .expect() | recent abandoned draft should still exist |
| 1069 | expect | open should succeed |
| 1069 | .expect() | open should succeed |
| 1072 | expect | get draft should succeed |
| 1072 | .expect() | get draft should succeed |
| 1073 | expect | draft should exist |
| 1073 | .expect() | draft should exist |
| 1075 | assert | pending |
| 1087 | expect | autosave should succeed |
| 1087 | .expect() | autosave should succeed |
| 1090 | expect | get draft should succeed |
| 1090 | .expect() | get draft should succeed |
| 1091 | expect | draft should exist |
| 1091 | .expect() | draft should exist |
| 1093 | assert | My Stitch Title |
| 1105 | expect | second autosave should succeed |
| 1105 | .expect() | second autosave should succeed |
| 1109 | expect | abandon should succeed |
| 1109 | .expect() | abandon should succeed |
| 1112 | expect | get draft should succeed |
| 1112 | .expect() | get draft should succeed |
| 1113 | expect | draft should exist |
| 1113 | .expect() | draft should exist |
| 1115 | assert | abandoned |
| 1120 | expect | get draft should succeed |
| 1120 | .expect() | get draft should succeed |
| 1121 | expect | abandoned draft should still exist |
| 1121 | .expect() | abandoned draft should still exist |

### hoop-daemon/tests/epoch_sync_invariant.rs

**Total messages in file:** 28

| Line | Pattern Type | Message |
|------|--------------|---------|
| 26 | .expect() | Failed to spawn test daemon |
| 33 | .expect() | Failed to connect to WebSocket |
| 40 | .expect() | Timeout waiting for init message |
| 41 | .expect() | WebSocket stream ended |
| 43 | .expect() | Failed to receive init message |
| 47 | .expect() | Failed to parse init event as JSON |
| 49 | assert_* | First message should be init event |
| 68 | panic! | Expected text message for init, got {:?} |
| 77 | .expect() | Failed to spawn test daemon |
| 84 | .expect() | Failed to connect to WebSocket |
| 101 | assert! | Should receive at least one message |
| 105 | .expect() | Failed to parse first message |
| 106 | assert_* | First message must be init |
| 147 | .expect() | Failed to spawn test daemon |
| 155 | .expect() | Failed to connect to WebSocket |
| 188 | .expect() | Failed to reconnect to WebSocket |
| 226 | assert! | Reconnect should receive init event |
| 244 | .expect() | Failed to spawn test daemon |
| 253 | .expect() | Failed to connect to WebSocket (iteration {}) |
| 263 | .expect() | WebSocket stream ended |
| 272 | .expect() | Failed to parse message as JSON |
| 280 | panic! | Expected text message (iteration {}) |
| 292 | .expect() | Failed to spawn test daemon |
| 305 | .expect() | Failed to connect |
| 313 | .expect() | Stream ended |
| 319 | .expect() | Failed to parse |
| 332 | assert! | Connection should receive init |
| 332 | .expect() | Task failed |

### hoop-daemon/tests/filesystem_failure_isolation.rs

**Total messages in file:** 30

| Line | Pattern Type | Message |
|------|--------------|---------|
| 28 | .expect() | Failed to create .beads dir |
| 30 | .expect() | Failed to create issues.jsonl |
| 39 | .expect() | Failed to create temp dir |
| 41 | .expect() | Failed to create .hoop dir |
| 71 | .expect() | Failed to write projects.yaml |
| 80 | .expect() | Failed to write config.yml |
| 83 | .expect() | Failed to create data dir |
| 125 | .expect() | Failed to bind to random port |
| 126 | .expect() | Failed to get local address |
| 173 | .expect() | Failed to get readyz status |
| 175 | assert_* | Initial readyz should return 200 |
| 176 | assert_* | Initial readyz status should be ok |
| 184 | .expect() | Failed to remove .beads from project A |
| 188 | .expect() | Failed to get projects.yaml metadata |
| 189 | .expect() | Failed to get modified time |
| 220 | .expect() | project-a should be in degraded list |
| 275 | .expect() | Failed to bind to random port |
| 276 | .expect() | Failed to get local address |
| 323 | .expect() | Failed to get readyz status |
| 325 | assert_* | Initial readyz should return 200 |
| 326 | assert_* | Initial readyz status should be ok |
| 334 | .expect() | Failed to remove .beads from project A |
| 371 | .expect() | Failed to read projects.yaml |
| 372 | .expect() | Failed to write projects.yaml |
| 430 | .expect() | Failed to bind to random port |
| 431 | .expect() | Failed to get local address |
| 479 | .expect() | Failed to get readyz status |
| 488 | .expect() | Failed to connect to WebSocket |
| 493 | .expect() | Failed to remove .beads from project A |
| 516 | assert! | project-a should be degraded |

### hoop-daemon/tests/fix_patterns_integration.rs

**Total messages in file:** 17

| Line | Pattern Type | Message |
|------|--------------|---------|
| 56 | assert! | create should return non-empty ID |
| 61 | .expect() | pattern should exist |
| 68 | assert_* | should have 1 pattern |
| 68 | assertion message | should have 1 pattern |
| 85 | .expect() | pattern should exist after update |
| 95 | .expect() | pattern should exist |
| 101 | assert! | pattern should be deleted |
| 173 | assertion message | should match all 3 patterns above threshold 0.5 |
| 208 | assertion message | should match 2 patterns with threshold 0.99 |
| 221 | assertion message | should match all patterns with zero threshold |
| 230 | assert_* | should limit results |
| 230 | assertion message | should limit results |
| 294 | assert_* | should find 2 patterns with 'panic' |
| 294 | assertion message | should find 2 patterns with 'panic' |
| 303 | assert_* | should find 1 pattern with 'bounds' |
| 303 | assertion message | should find 1 pattern with 'bounds' |
| 309 | assert_* | case-insensitive search should work |

### hoop-daemon/tests/fleet_notifications_integration.rs

**Total messages in file:** 4

| Line | Pattern Type | Message |
|------|--------------|---------|
| 59 | .expect() | Should serialize to JSON |
| 71 | .expect() | Should deserialize from JSON |
| 101 | assert_* | Oldest retained notification should be index 5 |
| 102 | assert_* | Newest notification should be index 24 |

### hoop-daemon/tests/golden_transcripts_regression.rs

**Total messages in file:** 50

| Line | Pattern Type | Message |
|------|--------------|---------|
| 39 | .expect() | workspace root is parent of hoop-daemon/ |
| 107 | panic! | Failed to read scenario directory {scenario_path:?}: {e} |
| 170 | panic! | Failed to read {:?}: {} |
| 170 | unwrap_or_else panic | Failed to read {:?}: {} |
| 170 | unwrap_or_else panic with args | Failed to read {:?}: {} |
| 199 | panic! | Failed to read {:?}: {} |
| 199 | unwrap_or_else panic | Failed to read {:?}: {} |
| 199 | unwrap_or_else panic with args | Failed to read {:?}: {} |
| 212 | panic! | Failed to read {:?}: {} |
| 212 | unwrap_or_else panic | Failed to read {:?}: {} |
| 212 | unwrap_or_else panic with args | Failed to read {:?}: {} |
| 235 | panic! | Failed to read {:?}: {} |
| 235 | unwrap_or_else panic | Failed to read {:?}: {} |
| 235 | unwrap_or_else panic with args | Failed to read {:?}: {} |
| 248 | panic! | Failed to read {:?}: {} |
| 248 | unwrap_or_else panic | Failed to read {:?}: {} |
| 248 | unwrap_or_else panic with args | Failed to read {:?}: {} |
| 297 | panic! | Failed to read {:?}: {} |
| 297 | unwrap_or_else panic | Failed to read {:?}: {} |
| 297 | unwrap_or_else panic with args | Failed to read {:?}: {} |
| 310 | panic! | Failed to read {:?}: {} |
| 310 | unwrap_or_else panic | Failed to read {:?}: {} |
| 310 | unwrap_or_else panic with args | Failed to read {:?}: {} |
| 361 | panic! | Failed to read {:?}: {} |
| 361 | unwrap_or_else panic | Failed to read {:?}: {} |
| 361 | unwrap_or_else panic with args | Failed to read {:?}: {} |
| 374 | panic! | Failed to read {:?}: {} |
| 374 | unwrap_or_else panic | Failed to read {:?}: {} |
| 374 | unwrap_or_else panic with args | Failed to read {:?}: {} |
| 502 | panic! | Failed to read {:?}: {} |
| 502 | unwrap_or_else panic | Failed to read {:?}: {} |
| 502 | unwrap_or_else panic with args | Failed to read {:?}: {} |
| 530 | panic! | Failed to read {:?}: {} |
| 530 | unwrap_or_else panic | Failed to read {:?}: {} |
| 530 | unwrap_or_else panic with args | Failed to read {:?}: {} |
| 543 | panic! | Failed to read {:?}: {} |
| 543 | unwrap_or_else panic | Failed to read {:?}: {} |
| 543 | unwrap_or_else panic with args | Failed to read {:?}: {} |
| 579 | panic! | Failed to read {:?}: {} |
| 579 | unwrap_or_else panic | Failed to read {:?}: {} |
| 579 | unwrap_or_else panic with args | Failed to read {:?}: {} |
| 592 | panic! | Failed to read {:?}: {} |
| 592 | unwrap_or_else panic | Failed to read {:?}: {} |
| 592 | unwrap_or_else panic with args | Failed to read {:?}: {} |
| 637 | panic! | Failed to read {:?}: {} |
| 637 | unwrap_or_else panic | Failed to read {:?}: {} |
| 637 | unwrap_or_else panic with args | Failed to read {:?}: {} |
| 650 | panic! | Failed to read {:?}: {} |
| 650 | unwrap_or_else panic | Failed to read {:?}: {} |
| 650 | unwrap_or_else panic with args | Failed to read {:?}: {} |

### hoop-daemon/tests/hoop_dies_nothing_notices.rs

**Total messages in file:** 66

| Line | Pattern Type | Message |
|------|--------------|---------|
| 30 | .expect() | workspace root is parent of hoop-daemon/ |
| 43 | .expect() | create temp dir for test HOOP home |
| 45 | .expect() | create .hoop dir |
| 61 | .expect() | write projects.yaml |
| 71 | .expect() | write config.yml |
| 74 | .expect() | create data dir |
| 168 | anyhow::bail! | testrepo should exist at {:?} |
| 189 | .expect() | testrepo should exist |
| 197 | .expect() | init fleet.db |
| 205 | .expect() | write claim event |
| 208 | .expect() | write dispatch event |
| 211 | assert! | worker should have written at least 2 events |
| 215 | assert! | events.jsonl should contain at least 2 events |
| 219 | .expect() | read events.jsonl |
| 234 | assertion message | should be able to parse at least 2 events from events.jsonl |
| 243 | .expect() | write complete event during HOOP absence |
| 246 | .expect() | write claim event during HOOP absence |
| 263 | .expect() | read events.jsonl after restart |
| 291 | .expect() | testrepo should exist |
| 299 | .expect() | init fleet.db |
| 310 | .expect() | write claim before HOOP |
| 313 | .expect() | write dispatch before HOOP |
| 332 | .expect() | write claim during HOOP absence |
| 335 | .expect() | write complete during HOOP absence |
| 362 | .expect() | read events.jsonl after restart |
| 393 | .expect() | testrepo should exist |
| 400 | .expect() | init fleet.db |
| 412 | .expect() | write claim event |
| 417 | .expect() | write dispatch event |
| 424 | .expect() | write complete event |
| 435 | .expect() | read events.jsonl for rebuild |
| 475 | .expect() | testrepo should exist |
| 482 | .expect() | init fleet.db |
| 489 | .expect() | write claim |
| 490 | .expect() | write dispatch |
| 491 | .expect() | write complete |
| 499 | .expect() | write claim |
| 500 | .expect() | write dispatch |
| 512 | .expect() | read events after third run |
| 531 | .expect() | testrepo should exist |
| 539 | .expect() | init fleet.db |
| 573 | .expect() | insert draft before restart |
| 577 | .expect() | get draft before restart |
| 578 | .expect() | draft should exist before restart |
| 596 | .expect() | re-init fleet.db after restart |
| 600 | .expect() | get draft after restart |
| 601 | .expect() | draft should exist after restart |
| 622 | .expect() | testrepo should exist |
| 629 | .expect() | init fleet.db |
| 635 | .expect() | write valid claim |
| 636 | .expect() | write valid dispatch |
| 644 | .expect() | open events.jsonl for corruption |
| 646 | .expect() | write corrupted line |
| 650 | .expect() | write valid claim after corruption |
| 651 | .expect() | write valid complete after corruption |
| 655 | .expect() | read events with corruption |
| 671 | assert_* | should detect exactly one corrupted line |
| 671 | assertion message | should detect exactly one corrupted line |
| 672 | assert! | should still parse all valid events |
| 672 | assertion message | should still parse all valid events |
| 683 | .expect() | testrepo should exist |
| 690 | .expect() | init fleet.db |
| 696 | .expect() | empty events.jsonl |
| 698 | .expect() | create empty events.jsonl |
| 703 | .expect() | read empty events.jsonl |
| 707 | assert_* | empty events.jsonl should have 0 events |

### hoop-daemon/tests/integration_harness.rs

**Total messages in file:** 332

| Line | Pattern Type | Message |
|------|--------------|---------|
| 33 | expect | workspace root is parent of hoop-daemon/ |
| 33 | .expect() | workspace root is parent of hoop-daemon/ |
| 61 | expect | Failed to create temp dir for test HOOP home |
| 61 | .expect() | Failed to create temp dir for test HOOP home |
| 63 | expect | Failed to create .hoop dir |
| 63 | .expect() | Failed to create .hoop dir |
| 79 | expect | Failed to write projects.yaml |
| 79 | .expect() | Failed to write projects.yaml |
| 88 | expect | config.yml |
| 88 | .expect() | Failed to write config.yml |
| 91 | expect | data |
| 91 | .expect() | Failed to create data dir |
| 109 | anyhow | testrepo should exist at {:?} |
| 109 | anyhow::bail! | testrepo should exist at {:?} |
| 115 | anyhow | testrepo/.beads/events.jsonl should exist |
| 115 | anyhow::bail! | testrepo/.beads/events.jsonl should exist |
| 121 | anyhow | testrepo/.beads/heartbeats.jsonl should exist |
| 121 | anyhow::bail! | testrepo/.beads/heartbeats.jsonl should exist |
| 125 | expect | Failed to read events.jsonl |
| 125 | .expect() | Failed to read events.jsonl |
| 127 | anyhow | events.jsonl should not be empty |
| 127 | anyhow::bail! | events.jsonl should not be empty |
| 136 | anyhow | events.jsonl line {} is not valid JSON |
| 136 | anyhow::bail! | events.jsonl line {} is not valid JSON |
| 141 | expect | Failed to read heartbeats.jsonl |
| 141 | .expect() | Failed to read heartbeats.jsonl |
| 143 | anyhow | heartbeats.jsonl should not be empty |
| 143 | anyhow::bail! | heartbeats.jsonl should not be empty |
| 152 | anyhow | heartbeats.jsonl line {} is not valid JSON |
| 152 | anyhow::bail! | heartbeats.jsonl line {} is not valid JSON |
| 171 | anyhow | Failed to parse event line {}: {} |
| 171 | anyhow::anyhow! | Failed to parse event line {}: {} |
| 191 | anyhow | Failed to parse heartbeat line {}: {} |
| 191 | anyhow::anyhow! | Failed to parse heartbeat line {}: {} |
| 217 | anyhow | Events fixture should contain at least one claim event |
| 217 | anyhow::bail! | Events fixture should contain at least one claim event |
| 220 | anyhow | Events fixture should contain at least one dispatch event |
| 220 | anyhow::bail! | Events fixture should contain at least one dispatch event |
| 223 | anyhow | Events fixture should contain at least one complete event |
| 223 | anyhow::bail! | Events fixture should contain at least one complete event |
| 226 | anyhow | Events fixture should contain at least one fail event |
| 226 | anyhow::bail! | Events fixture should contain at least one fail event |
| 247 | anyhow | Heartbeats fixture should contain at least one idle state |
| 247 | anyhow::bail! | Heartbeats fixture should contain at least one idle state |
| 250 | anyhow | Heartbeats fixture should contain at least one executing state |
| 250 | anyhow::bail! | Heartbeats fixture should contain at least one executing state |
| 338 | assert | Should have 2 open beads |
| 338 | assert_* | Should have 2 open beads |
| 339 | assert | Should have 1 closed bead |
| 339 | assert_* | Should have 1 closed bead |
| 359 | anyhow | projects.yaml should be created |
| 359 | anyhow::bail! | projects.yaml should be created |
| 365 | anyhow | config.yml should be created |
| 365 | anyhow::bail! | config.yml should be created |
| 371 | anyhow | projects.yaml should reference testrepo |
| 371 | anyhow::bail! | projects.yaml should reference testrepo |
| 385 | expect | testrepo fixtures should be valid |
| 385 | .expect() | testrepo fixtures should be valid |
| 391 | expect | events should parse correctly |
| 391 | .expect() | events should parse correctly |
| 397 | expect | heartbeats should parse correctly |
| 397 | .expect() | heartbeats should parse correctly |
| 403 | expect | bead event data should extract |
| 403 | .expect() | bead event data should extract |
| 409 | expect | bead projections should be correct |
| 409 | .expect() | bead projections should be correct |
| 415 | expect | HOOP home setup should work |
| 415 | .expect() | HOOP home setup should work |
| 421 | expect | Failed to parse events |
| 421 | .expect() | Failed to parse events |
| 456 | expect | Failed to parse heartbeats |
| 456 | .expect() | Failed to parse heartbeats |
| 483 | expect | Failed to parse events |
| 483 | .expect() | Failed to parse events |
| 558 | expect | Failed to parse events |
| 558 | .expect() | Failed to parse events |
| 698 | anyhow | Daemon failed to become ready within 10 seconds |
| 698 | anyhow::bail! | Daemon failed to become ready within 10 seconds |
| 710 | expect | Failed to spawn test daemon |
| 710 | .expect() | Failed to spawn test daemon |
| 719 | expect | Failed to connect to healthz |
| 719 | .expect() | Failed to connect to healthz |
| 721 | assert | healthz should return 200 |
| 721 | assert_* | healthz should return 200 |
| 723 | expect | Failed to parse healthz response |
| 723 | .expect() | Failed to parse healthz response |
| 725 | assert | status |
| 725 | assert_* | healthz status should be ok |
| 732 | expect | Failed to connect to readyz |
| 732 | .expect() | Failed to connect to readyz |
| 734 | assert | readyz should return 200 |
| 734 | assert_* | readyz should return 200 |
| 741 | expect | Failed to spawn test daemon |
| 741 | .expect() | Failed to spawn test daemon |
| 750 | expect | Failed to GET /api/beads |
| 750 | .expect() | Failed to GET /api/beads |
| 752 | assert | GET /api/beads should return 200 |
| 752 | assert_* | GET /api/beads should return 200 |
| 754 | expect | Failed to parse beads response |
| 754 | .expect() | Failed to parse beads response |
| 764 | expect | Failed to GET /api/projects |
| 764 | .expect() | Failed to GET /api/projects |
| 766 | assert | GET /api/projects should return 200 |
| 766 | assert_* | GET /api/projects should return 200 |
| 771 | expect | Failed to parse projects response |
| 771 | .expect() | Failed to parse projects response |
| 773 | assert | projects should be a list |
| 773 | assert! | projects should be a list |
| 784 | expect | Failed to spawn test daemon |
| 784 | .expect() | Failed to spawn test daemon |
| 793 | expect | Failed to connect to WebSocket |
| 793 | .expect() | Failed to connect to WebSocket |
| 800 | expect | Timeout waiting for init message |
| 800 | .expect() | Timeout waiting for init message |
| 801 | expect | WebSocket stream ended |
| 801 | .expect() | WebSocket stream ended |
| 803 | expect | Failed to receive init message |
| 803 | .expect() | Failed to receive init message |
| 807 | expect | Failed to parse init event as JSON |
| 807 | .expect() | Failed to parse init event as JSON |
| 809 | assert | type |
| 809 | assert_* | First message should be init event |
| 815 | panic | Expected text message, got {:?} |
| 815 | panic! | Expected text message, got {:?} |
| 821 | expect | Timeout waiting for workers_snapshot message |
| 821 | .expect() | Timeout waiting for workers_snapshot message |
| 822 | expect | WebSocket stream ended |
| 822 | .expect() | WebSocket stream ended |
| 824 | expect | Failed to receive workers_snapshot |
| 824 | .expect() | Failed to receive workers_snapshot |
| 828 | expect | Failed to parse workers_snapshot event as JSON |
| 828 | .expect() | Failed to parse workers_snapshot event as JSON |
| 839 | expect | Timeout waiting for beads_snapshot message |
| 839 | .expect() | Timeout waiting for beads_snapshot message |
| 840 | expect | WebSocket stream ended |
| 840 | .expect() | WebSocket stream ended |
| 842 | expect | Failed to receive beads_snapshot |
| 842 | .expect() | Failed to receive beads_snapshot |
| 846 | expect | Failed to parse beads_snapshot event as JSON |
| 846 | .expect() | Failed to parse beads_snapshot event as JSON |
| 865 | expect | Failed to send subscribe message |
| 865 | .expect() | Failed to send subscribe message |
| 871 | expect | Failed to send close frame |
| 871 | .expect() | Failed to send close frame |
| 883 | expect | Failed to spawn test daemon |
| 883 | .expect() | Failed to spawn test daemon |
| 892 | expect | Failed to connect to healthz |
| 892 | .expect() | Failed to connect to healthz |
| 894 | assert | Daemon should be healthy after boot |
| 894 | assert_* | Daemon should be healthy after boot |
| 901 | expect | Failed to GET /api/beads |
| 901 | .expect() | Failed to GET /api/beads |
| 903 | assert | Should be able to read beads |
| 903 | assert_* | Should be able to read beads |
| 910 | expect | Failed to GET /api/projects |
| 910 | .expect() | Failed to GET /api/projects |
| 912 | assert | Should be able to get projects |
| 912 | assert_* | Should be able to get projects |
| 933 | expect | Failed to spawn test daemon |
| 933 | .expect() | Failed to spawn test daemon |
| 942 | expect | Failed to GET /api/projects |
| 942 | .expect() | Failed to GET /api/projects |
| 949 | expect | Failed to parse projects response |
| 949 | .expect() | Failed to parse projects response |
| 969 | expect | Failed to spawn test daemon |
| 969 | .expect() | Failed to spawn test daemon |
| 978 | expect | Failed to GET /api/beads |
| 978 | .expect() | Failed to GET /api/beads |
| 982 | expect | Failed to parse beads response |
| 982 | .expect() | Failed to parse beads response |
| 986 | assert | bead id should not be empty |
| 986 | assert! | bead id should not be empty |
| 987 | assert | bead title should not be empty |
| 987 | assert! | bead title should not be empty |
| 988 | assert | bead project should not be empty |
| 988 | assert! | bead project should not be empty |
| 996 | expect | Failed to spawn test daemon |
| 996 | .expect() | Failed to spawn test daemon |
| 1005 | expect | Failed to GET /api/metrics |
| 1005 | .expect() | Failed to GET /api/metrics |
| 1009 | expect | Failed to read metrics response |
| 1009 | .expect() | Failed to read metrics response |
| 1026 | expect | Failed to spawn test daemon |
| 1026 | .expect() | Failed to spawn test daemon |
| 1033 | expect | Failed to connect to WebSocket |
| 1033 | .expect() | Failed to connect to WebSocket |
| 1066 | assert | Should receive init event |
| 1066 | assert! | Should receive init event |
| 1081 | expect | Failed to spawn test daemon |
| 1081 | .expect() | Failed to spawn test daemon |
| 1088 | expect | Failed to connect to WebSocket |
| 1088 | .expect() | Failed to connect to WebSocket |
| 1095 | expect | Timeout waiting for init |
| 1095 | .expect() | Timeout waiting for init |
| 1108 | expect | Failed to send subscribe message |
| 1108 | .expect() | Failed to send subscribe message |
| 1151 | expect | Failed to spawn test daemon |
| 1151 | .expect() | Failed to spawn test daemon |
| 1160 | expect | Failed to connect to healthz |
| 1160 | .expect() | Failed to connect to healthz |
| 1183 | expect | Failed to spawn test daemon |
| 1183 | .expect() | Failed to spawn test daemon |
| 1190 | expect | Failed to connect to WebSocket |
| 1190 | .expect() | Failed to connect to WebSocket |
| 1197 | expect | Timeout waiting for init |
| 1197 | .expect() | Timeout waiting for init |
| 1205 | expect | Failed to send malformed message |
| 1205 | .expect() | Failed to send malformed message |
| 1217 | expect | Failed to send unknown event type |
| 1217 | .expect() | Failed to send unknown event type |
| 1225 | expect | Failed to send empty message |
| 1225 | .expect() | Failed to send empty message |
| 1232 | expect | Health check failed |
| 1232 | .expect() | Health check failed |
| 1234 | assert | Daemon should still be healthy after malformed messages |
| 1234 | assert_* | Daemon should still be healthy after malformed messages |
| 1248 | expect | Failed to spawn test daemon |
| 1248 | .expect() | Failed to spawn test daemon |
| 1275 | expect | Task failed |
| 1275 | .expect() | Task failed |
| 1281 | assert | All concurrent requests should succeed |
| 1281 | assert_* | All concurrent requests should succeed |
| 1291 | expect | Failed to spawn first daemon |
| 1291 | .expect() | Failed to spawn first daemon |
| 1305 | expect | Failed to create bead |
| 1305 | .expect() | Failed to create bead |
| 1307 | assert | Bead creation should succeed |
| 1307 | assert! | Bead creation should succeed |
| 1309 | expect | Failed to parse bead |
| 1309 | .expect() | Failed to parse bead |
| 1310 | expect | id |
| 1310 | .expect() | Bead should have an ID |
| 1319 | expect | Failed to spawn second daemon |
| 1319 | .expect() | Failed to spawn second daemon |
| 1326 | expect | Failed to fetch beads |
| 1326 | .expect() | Failed to fetch beads |
| 1328 | assert | Should be able to fetch beads |
| 1328 | assert! | Should be able to fetch beads |
| 1336 | expect | Failed to spawn test daemon |
| 1336 | .expect() | Failed to spawn test daemon |
| 1376 | expect | Task failed |
| 1376 | .expect() | Task failed |
| 1382 | assert | All WebSocket connections should receive init |
| 1382 | assert_* | All WebSocket connections should receive init |
| 1390 | expect | Failed to spawn test daemon |
| 1390 | .expect() | Failed to spawn test daemon |
| 1399 | expect | Request failed |
| 1399 | .expect() | Request failed |
| 1401 | assert | Non-existent endpoint should return 404 |
| 1401 | assert_* | Non-existent endpoint should return 404 |
| 1408 | expect | Request failed |
| 1408 | .expect() | Request failed |
| 1410 | assert | Non-existent bead should return error |
| 1410 | assert! | Non-existent bead should return error |
| 1419 | expect | Request failed |
| 1419 | .expect() | Request failed |
| 1421 | assert | Invalid JSON should return error |
| 1421 | assert! | Invalid JSON should return error |
| 1429 | expect | Failed to spawn test daemon |
| 1429 | .expect() | Failed to spawn test daemon |
| 1437 | expect | Failed to fetch metrics |
| 1437 | .expect() | Failed to fetch metrics |
| 1439 | assert | Metrics endpoint should return 200 |
| 1439 | assert! | Metrics endpoint should return 200 |
| 1441 | expect | Failed to read metrics |
| 1441 | .expect() | Failed to read metrics |
| 1444 | assert | Metrics should not be empty |
| 1444 | assert! | Metrics should not be empty |
| 1456 | assert | Metrics should contain at least one valid metric line |
| 1456 | assert! | Metrics should contain at least one valid metric line |
| 1464 | expect | Failed to spawn test daemon |
| 1464 | .expect() | Failed to spawn test daemon |
| 1474 | expect | Failed to list files |
| 1474 | .expect() | Failed to list files |
| 1476 | assert | File listing should succeed |
| 1476 | assert! | File listing should succeed |
| 1478 | expect | Failed to parse files |
| 1478 | .expect() | Failed to parse files |
| 1481 | assert | Files should be an array or object |
| 1481 | assert! | Files should be an array or object |
| 1489 | expect | Failed to spawn test daemon |
| 1489 | .expect() | Failed to spawn test daemon |
| 1504 | expect | Failed to create bead |
| 1504 | .expect() | Failed to create bead |
| 1506 | assert | Bead creation should succeed |
| 1506 | assert! | Bead creation should succeed |
| 1508 | expect | Failed to parse bead |
| 1508 | .expect() | Failed to parse bead |
| 1509 | expect | id |
| 1509 | .expect() | Bead should have an ID |
| 1516 | expect | Failed to get bead |
| 1516 | .expect() | Failed to get bead |
| 1518 | assert | Getting bead should succeed |
| 1518 | assert! | Getting bead should succeed |
| 1520 | expect | Failed to parse fetched bead |
| 1520 | .expect() | Failed to parse fetched bead |
| 1521 | assert | id |
| 1521 | assert_* | Fetched bead ID should match |
| 1522 | assert | title |
| 1522 | assert_* | Fetched bead title should match |
| 1529 | expect | Failed to list beads |
| 1529 | .expect() | Failed to list beads |
| 1531 | assert | Listing beads should succeed |
| 1531 | assert! | Listing beads should succeed |
| 1533 | expect | Failed to parse beads list |
| 1533 | .expect() | Failed to parse beads list |
| 1535 | assert | New bead should appear in list |
| 1535 | assert! | New bead should appear in list |
| 1543 | expect | Failed to spawn test daemon |
| 1543 | .expect() | Failed to spawn test daemon |
| 1551 | expect | Failed to fetch capacity |
| 1551 | .expect() | Failed to fetch capacity |
| 1553 | assert | Capacity endpoint should return 200 |
| 1553 | assert! | Capacity endpoint should return 200 |
| 1555 | expect | Failed to parse capacity |
| 1555 | .expect() | Failed to parse capacity |
| 1558 | assert | Capacity should be object or array |
| 1558 | assert! | Capacity should be object or array |
| 1566 | expect | Failed to spawn test daemon |
| 1566 | .expect() | Failed to spawn test daemon |
| 1574 | expect | Failed to fetch config status |
| 1574 | .expect() | Failed to fetch config status |
| 1576 | assert | Config status endpoint should return 200 |
| 1576 | assert! | Config status endpoint should return 200 |
| 1578 | expect | Failed to parse config status |
| 1578 | .expect() | Failed to parse config status |
| 1592 | expect | Failed to spawn test daemon |
| 1592 | .expect() | Failed to spawn test daemon |
| 1601 | expect | Failed to GET /api/beads |
| 1601 | .expect() | Failed to GET /api/beads |
| 1609 | expect | Failed to GET /api/projects |
| 1609 | .expect() | Failed to GET /api/projects |

### hoop-daemon/tests/load_test.rs

**Total messages in file:** 11

| Line | Pattern Type | Message |
|------|--------------|---------|
| 209 | .expect() | Failed to spawn test daemon |
| 214 | .expect() | Load test should complete |
| 262 | .expect() | Failed to spawn test daemon |
| 271 | .expect() | Load test timed out after 10 minutes |
| 272 | .expect() | Load test should complete |
| 282 | .expect() | Performance budgets must be satisfied |
| 285 | assert! | Should process events |
| 329 | .expect() | Failed to spawn test daemon |
| 337 | .expect() | Medium-scale load test timed out |
| 338 | .expect() | Load test should complete |
| 345 | .expect() | Medium-scale load test should pass performance budgets |

### hoop-daemon/tests/load_test_integration.rs

**Total messages in file:** 23

| Line | Pattern Type | Message |
|------|--------------|---------|
| 72 | .expect() | Failed to spawn daemon with load test data |
| 81 | .expect() | Health check request failed |
| 83 | assert_* | Daemon should be healthy |
| 100 | .expect() | Failed to spawn daemon |
| 167 | .expect() | Failed to spawn daemon |
| 219 | .expect() | Failed to spawn daemon |
| 286 | .expect() | Failed to spawn daemon |
| 306 | panic! | Failed to connect WS client {}: {} |
| 345 | .expect() | Failed to spawn daemon |
| 350 | .expect() | Load test failed |
| 355 | .expect() | Performance budget violations detected |
| 360 | assert! | Load test should pass all budgets |
| 380 | .expect() | Failed to populate testrepo with load test data |
| 396 | .expect() | Failed to create project directory |
| 410 | .expect() | Failed to serialize projects.yaml |
| 412 | .expect() | Failed to write projects.yaml |
| 467 | .expect() | Failed to spawn daemon with load test data |
| 472 | .expect() | Failed to write daemon URL to file |
| 480 | .expect() | Load test failed |
| 490 | .expect() | Performance budget violations detected - blocking merge per hoop-ttb.7.11 |
| 492 | assert! | Load test should pass all budgets |
| 528 | .expect() | Failed to spawn daemon |
| 557 | .expect() | Failed to spawn daemon |

### hoop-daemon/tests/multi_operator_concurrency.rs

**Total messages in file:** 74

| Line | Pattern Type | Message |
|------|--------------|---------|
| 26 | .expect() | create temp dir |
| 28 | .expect() | create .hoop dir |
| 34 | .expect() | init fleet.db |
| 114 | .expect() | insert draft_a |
| 115 | .expect() | insert draft_b |
| 119 | .expect() | get draft_a |
| 120 | .expect() | draft_a exists |
| 124 | .expect() | get draft_b |
| 125 | .expect() | draft_b exists |
| 166 | .expect() | insert draft |
| 176 | .expect() | autosave draft |
| 179 | .expect() | get draft |
| 180 | .expect() | draft exists |
| 186 | assert_* | version should NOT increment on autosave |
| 187 | assert! | last_autosave_at should be set |
| 226 | .expect() | insert draft |
| 229 | .expect() | abandon draft |
| 232 | .expect() | get draft |
| 233 | .expect() | draft exists |
| 236 | assert! | abandoned_at should be set |
| 276 | .expect() | insert existing draft |
| 284 | .expect() | detect similar drafts |
| 287 | assert! | should detect similar existing draft |
| 287 | assertion message | should detect similar existing draft |
| 307 | .expect() | propose from operator A |
| 315 | .expect() | propose from operator B |
| 318 | assert_* | duplicate proposal should return the same ID |
| 322 | .expect() | list proposals |
| 324 | assert_* | should have only one proposal |
| 324 | assertion message | should have only one proposal |
| 325 | assert_* | proposal ID should match first proposal |
| 329 | .expect() | parse source_stitches |
| 330 | assert_* | should have 3 merged source stitches |
| 330 | assertion message | should have 3 merged source stitches |
| 364 | .expect() | insert proposal |
| 370 | .expect() | approve proposal |
| 372 | assert! | proposal should be approved |
| 376 | .expect() | get proposal |
| 377 | .expect() | proposal exists |
| 383 | .expect() | list approved entries |
| 385 | assert! | should have approved entries |
| 385 | assertion message | should have approved entries |
| 416 | .expect() | insert proposal |
| 420 | .expect() | reject proposal |
| 422 | assert! | proposal should be rejected |
| 426 | .expect() | get proposal |
| 430 | assert! | rejected proposal should not appear in proposed list |
| 451 | .expect() | update presence |
| 457 | .expect() | query presence |
| 477 | .expect() | update presence hidden |
| 483 | .expect() | query presence |
| 485 | assert_* | hidden presence should not be returned |
| 498 | .expect() | _HOOP_FLEET_DB_PATH not set |
| 500 | .expect() | open db |
| 512 | .expect() | insert stale presence |
| 518 | .expect() | query presence |
| 520 | assert_* | stale presence should be filtered out |
| 535 | .expect() | update presence |
| 541 | .expect() | query presence |
| 549 | .expect() | remove presence |
| 555 | .expect() | query presence |
| 591 | .expect() | insert session A |
| 613 | .expect() | insert session B |
| 617 | .expect() | list agent sessions |
| 625 | assert_* | both operator sessions should coexist |
| 665 | .expect() | insert draft |
| 668 | .expect() | get draft |
| 669 | .expect() | draft exists |
| 702 | .expect() | create stitch A |
| 713 | .expect() | create stitch B |
| 717 | .expect() | load stitch A |
| 718 | .expect() | stitch A exists |
| 721 | .expect() | load stitch B |
| 722 | .expect() | stitch B exists |

### hoop-daemon/tests/mutation_handler_test.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 289 | assert! | Should reject empty title |

### hoop-daemon/tests/needle_events_roundtrip.rs

**Total messages in file:** 46

| Line | Pattern Type | Message |
|------|--------------|---------|
| 25 | .expect() | workspace root is parent of hoop-daemon/ |
| 41 | panic! | Failed to parse event line: {e}\n  Line: {line} |
| 41 | unwrap_or_else panic | Failed to parse event line: {e}\n  Line: {line} |
| 41 | unwrap_or_else panic with args | Failed to parse event line: {e}\n  Line: {line} |
| 64 | .expect() | testrepo/.beads/events.jsonl must be readable |
| 85 | .expect() | testrepo/.beads/heartbeats.jsonl must be readable |
| 106 | .expect() | fixture must have a claim event |
| 115 | assert! | claim: worker must be non-empty |
| 116 | assert! | claim: bead must start with 'bd-' |
| 122 | panic! | Expected Claim, got {other:?} |
| 132 | .expect() | fixture must have a dispatch event |
| 142 | assert! | dispatch: worker must be non-empty |
| 151 | assert! | dispatch in fixture should include model |
| 153 | panic! | Expected Dispatch, got {other:?} |
| 163 | .expect() | fixture must have a complete event |
| 174 | assert! | complete: worker must be non-empty |
| 192 | panic! | Expected Complete, got {other:?} |
| 202 | .expect() | fixture must have a fail event |
| 212 | assert! | fail: worker must be non-empty |
| 213 | assert! | fail: bead must start with 'bd-' |
| 214 | assert! | fail in fixture should include error |
| 220 | panic! | Expected Fail, got {other:?} |
| 230 | .expect() | fixture must have a release event |
| 234 | assert! | release: worker must be non-empty |
| 240 | panic! | Expected Release, got {other:?} |
| 250 | .expect() | fixture must have a timeout event |
| 254 | assert! | timeout: worker must be non-empty |
| 260 | panic! | Expected Timeout, got {other:?} |
| 270 | .expect() | fixture must have a crash event |
| 279 | assert! | crash: worker must be non-empty |
| 280 | assert! | crash: bead must start with 'bd-' |
| 286 | panic! | Expected Crash, got {other:?} |
| 451 | .expect() | fixture must have an executing heartbeat |
| 454 | .expect() | executing heartbeat must parse successfully |
| 456 | assert! | heartbeat: worker must be non-empty |
| 463 | assert! | executing: pid must be positive |
| 464 | assert! | executing: adapter must be non-empty |
| 466 | panic! | Expected Executing state, got {other:?} |
| 476 | .expect() | fixture must have an idle heartbeat |
| 479 | .expect() | idle heartbeat must parse successfully |
| 481 | assert! | heartbeat: worker must be non-empty |
| 491 | .expect() | fixture must have a knot heartbeat |
| 494 | .expect() | knot heartbeat must parse successfully |
| 496 | assert! | heartbeat: worker must be non-empty |
| 499 | assert! | knot: reason must be non-empty |
| 501 | panic! | Expected Knot state, got {other:?} |

### hoop-daemon/tests/orphans_integration.rs

**Total messages in file:** 7

| Line | Pattern Type | Message |
|------|--------------|---------|
| 151 | assert! | attach_orphan_to_stitch should succeed |
| 162 | assert! | stitch_beads link should exist with relationship='referenced' |
| 171 | assert! | duplicate attach should succeed (idempotent) |
| 182 | assert_* | should have exactly one stitch_beads row |
| 182 | assertion message | should have exactly one stitch_beads row |
| 230 | assert! | attach should succeed when link already exists |
| 240 | assert_* | existing relationship should be preserved |

### hoop-daemon/tests/output_capture_helpers/mod.rs

**Total messages in file:** 5

| Line | Pattern Type | Message |
|------|--------------|---------|
| 791 | assert! | Verification should pass when content matches |
| 810 | assert! | Verification should fail when content differs |
| 831 | assert! | Verification should fail when lengths differ |
| 850 | assert! | Should handle unicode and special characters |
| 893 | assert! | Large output verification should pass |

### hoop-daemon/tests/panic_isolation.rs

**Total messages in file:** 2

| Line | Pattern Type | Message |
|------|--------------|---------|
| 104 | assert | Connection refused |
| 105 | assert | Timeout |

### hoop-daemon/tests/path_traversal_hardening.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 147 | .expect() | allowlist construction must succeed |

### hoop-daemon/tests/pattern_query_evaluator_integration.rs

**Total messages in file:** 22

| Line | Pattern Type | Message |
|------|--------------|---------|
| 170 | assert_* | should have 1 pattern query result |
| 170 | assertion message | should have 1 pattern query result |
| 172 | assert! | query should match the stitch title |
| 173 | assert! | query should not be slow |
| 181 | assert! | first insert should succeed |
| 189 | assert! | second insert should return false (idempotent) |
| 199 | assert_* | should have exactly 1 pattern member |
| 199 | assertion message | should have exactly 1 pattern member |
| 332 | assert_* | should have 3 pattern query results |
| 332 | assertion message | should have 3 pattern query results |
| 336 | assert_* | should match 2 patterns |
| 336 | assertion message | should match 2 patterns |
| 367 | assert! | should parse query '{}': {:?} |
| 367 | assertion message | should parse query '{}': {:?} |
| 375 | assert! | AND query should match |
| 382 | assert! | NOT query should match |
| 389 | assert! | OR query should match |
| 396 | assert! | non-matching query should not match |
| 420 | assert! | kind:operator should match operator stitch |
| 423 | assert! | kind:operator should not match worker stitch |
| 439 | assert! | standalone word should match as label |
| 444 | assert! | non-matching standalone word should not match |

### hoop-daemon/tests/per_project_redaction_integration.rs

**Total messages in file:** 5

| Line | Pattern Type | Message |
|------|--------------|---------|
| 103 | panic! | Expected Variant0 project |
| 118 | panic! | Expected Variant0 project |
| 128 | panic! | Expected Variant0 project |
| 277 | assert! | customer-data should allow clean content |
| 307 | assert! | customer-data should block Anthropic keys |

### hoop-daemon/tests/performance_budget.rs

**Total messages in file:** 14

| Line | Pattern Type | Message |
|------|--------------|---------|
| 64 | .expect() | Failed to populate testrepo with load test data |
| 81 | .expect() | Failed to create project directory |
| 98 | .expect() | Failed to serialize projects.yaml |
| 100 | .expect() | Failed to write projects.yaml |
| 111 | .expect() | Failed to spawn daemon |
| 125 | .expect() | healthz request failed |
| 141 | .expect() | readyz request failed |
| 157 | .expect() | projects request failed |
| 171 | assert_* | Expected {} projects |
| 181 | .expect() | metrics request failed |
| 248 | .expect() | Failed to populate testrepo |
| 261 | .expect() | Failed to create project directory |
| 279 | .expect() | Failed to spawn daemon |
| 288 | .expect() | readyz request failed |

### hoop-daemon/tests/phase2_exit_gate.rs

**Total messages in file:** 2

| Line | Pattern Type | Message |
|------|--------------|---------|
| 438 | .expect() | Report must serialize to JSON |
| 448 | assert_* | Phase 2 must have exactly 13 core deliverables |

### hoop-daemon/tests/privacy_surface_audit.rs

**Total messages in file:** 13

| Line | Pattern Type | Message |
|------|--------------|---------|
| 50 | assertion message | expected anthropic_api_key or env_var_secret pattern; got: {findings:?} |
| 66 | assertion message | expected github_token_ghp pattern; got: {findings:?} |
| 92 | assertion message | expected anthropic_api_key pattern; got: {findings:?} |
| 107 | assertion message | expected jwt pattern; got: {findings:?} |
| 141 | assert! | should find secrets |
| 141 | assertion message | should find secrets |
| 152 | assert! | finding match_len must be > 0; got: {f:?} |
| 186 | assertion message | expected aws_access_key pattern; got: {findings:?} |
| 242 | assertion message | expected json_secret_field pattern; got: {findings:?} |
| 293 | assertion message | expected anthropic_api_key finding; got: {findings:?} |
| 313 | assertion message | expected github_token_ghp finding; got: {findings:?} |
| 359 | assertion message | expected aws_access_key finding; got: {findings:?} |
| 389 | assertion message | expected jwt pattern in propagation draft findings; got: {findings:?} |

### hoop-daemon/tests/projection_file_audit.rs

**Total messages in file:** 7

| Line | Pattern Type | Message |
|------|--------------|---------|
| 195 | .expect() | CARGO_MANIFEST_DIR not set |
| 198 | .expect() | workspace root is the parent of hoop-daemon/ |
| 216 | .expect() | valid regex |
| 229 | panic! | failed to read {}: {} |
| 229 | assertion message | failed to read {}: {} |
| 262 | panic! | {} |
| 436 | .expect() | valid regex |

### hoop-daemon/tests/property_invariants.rs

**Total messages in file:** 20

| Line | Pattern Type | Message |
|------|--------------|---------|
| 270 | assert | Event {} timestamp mismatch |
| 270 | assert_* | Event {} timestamp mismatch |
| 273 | assert | Event {} timestamp mismatch |
| 273 | assert_* | Event {} timestamp mismatch |
| 276 | assert | Event {} timestamp mismatch |
| 276 | assert_* | Event {} timestamp mismatch |
| 278 | panic | Event type mismatch at index {} |
| 278 | panic! | Event type mismatch at index {} |
| 379 | assert | First and second calls differ |
| 379 | assert_* | First and second calls differ |
| 380 | assert | Second and third calls differ |
| 380 | assert_* | Second and third calls differ |
| 579 | assert | Status derivation is non-deterministic |
| 579 | assert_* | Status derivation is non-deterministic |
| 853 | assert | First and second replays differ |
| 853 | assert_* | First and second replays differ |
| 854 | assert | Second and third replays differ |
| 854 | assert_* | Second and third replays differ |
| 894 | assert | bd-2 |
| 895 | assert | beta |

### hoop-daemon/tests/protocol_contract.rs

**Total messages in file:** 46

| Line | Pattern Type | Message |
|------|--------------|---------|
| 24 | expect | workspace root |
| 24 | .expect() | workspace root |
| 28 | panic | fixture file missing: {} |
| 28 | panic! | fixture file missing: {} |
| 28 | unwrap_or_else panic | fixture file missing: {} |
| 28 | unwrap_or_else panic with args | fixture file missing: {} |
| 30 | panic | invalid JSON in fixture {}: {} |
| 30 | panic! | invalid JSON in fixture {}: {} |
| 30 | unwrap_or_else panic | invalid JSON in fixture {}: {} |
| 30 | unwrap_or_else panic with args | invalid JSON in fixture {}: {} |
| 30 | assertion message | invalid JSON in fixture {}: {} |
| 47 | expect | CreateDraftRequest must deserialize from fixture (daemon side) |
| 47 | .expect() | CreateDraftRequest must deserialize from fixture (daemon side) |
| 49 | assert | project |
| 50 | assert | title |
| 51 | assert | kind |
| 52 | assert | source |
| 57 | assert | priority |
| 87 | assert | field '{}' value mismatch |
| 87 | assert_* | field '{}' value mismatch |
| 212 | assert | test-project |
| 259 | assert | test-project |
| 268 | panic | expected ControlResponse::Status |
| 268 | panic! | expected ControlResponse::Status |
| 268 | assertion message | expected ControlResponse::Status |
| 286 | assert | daemon not running |
| 288 | panic | expected ControlResponse::Error |
| 288 | panic! | expected ControlResponse::Error |
| 288 | assertion message | expected ControlResponse::Error |
| 310 | assert | type |
| 369 | assert | type |
| 405 | assert | type |
| 434 | assert | type |
| 457 | assert | type |
| 484 | assert | type |
| 509 | assert | type |
| 539 | assert | type |
| 566 | assert | type |
| 591 | assert | type |
| 624 | assert | type |
| 654 | panic | fixture {} must deserialize as WsEvent: {} |
| 654 | panic! | fixture {} must deserialize as WsEvent: {} |
| 654 | unwrap_or_else panic | fixture {} must deserialize as WsEvent: {} |
| 654 | unwrap_or_else panic with args | fixture {} must deserialize as WsEvent: {} |
| 698 | assert | fixture {} must be a JSON object |
| 698 | assert! | fixture {} must be a JSON object |

### hoop-daemon/tests/pure_functions.rs

**Total messages in file:** 58

| Line | Pattern Type | Message |
|------|--------------|---------|
| 42 | assert | \x1b[0m |
| 43 | assert | \x1b[31mRed\x1b[0m |
| 48 | assert | \x1b[38;5;123m |
| 49 | assert | \x1b[48;5;255m |
| 54 | assert | \x1b[38;2;255;0;128m |
| 55 | assert | \x1b[48;2;0;128;255m |
| 60 | assert | Just normal text |
| 61 | assert | Text with 🎉 emoji |
| 149 | assert | alpha |
| 176 | assert | auth |
| 177 | assert | authentication |
| 193 | assert | hello |
| 193 | assert_* | world |
| 199 | assert | hello |
| 199 | assert_* | world |
| 219 | assert | subdir |
| 238 | expect | sanitize should not fail |
| 238 | .expect() | sanitize should not fail |
| 241 | assert | script |
| 247 | expect | sanitize should not fail |
| 247 | .expect() | sanitize should not fail |
| 250 | assert | onclick |
| 260 | expect | sanitize should not fail |
| 260 | .expect() | sanitize should not fail |
| 263 | assert | /JavaScript |
| 269 | expect | sanitize should not fail |
| 269 | .expect() | sanitize should not fail |
| 281 | assert | Working on myproject |
| 294 | assert | custom |
| 294 | assert_* | file |
| 307 | assert | {\ |
| 307 | assert_* | {\ |
| 320 | assert | { |
| 351 | assert | ANSI strip too slow: {:?} |
| 351 | assert! | ANSI strip too slow: {:?} |
| 359 | assert | Cost functions too slow: {:?} |
| 359 | assert! | Cost functions too slow: {:?} |
| 368 | assert | Embedding too slow: {:?} |
| 368 | assert! | Embedding too slow: {:?} |
| 376 | assert | Similarity too slow: {:?} |
| 376 | assert! | Similarity too slow: {:?} |
| 398 | assert | Status derivation too slow: {:?} |
| 398 | assert! | Status derivation too slow: {:?} |
| 406 | assert | Tag join too slow: {:?} |
| 406 | assert! | Tag join too slow: {:?} |
| 415 | assert | Prompt substitute too slow: {:?} |
| 415 | assert! | Prompt substitute too slow: {:?} |
| 436 | assert | \x1b[31m\x1b[0m |
| 438 | assert | \x1b[31m你好\x1b[0m |
| 440 | assert | \x1b[0m\x1b[0m\x1b[0m |
| 473 | assert | p p p |
| 483 | assert | /any/path |
| 487 | assert | /any/path |
| 492 | assert | deep/nested/path |
| 493 | assert | /tmp/other |
| 510 | assert | onclick |
| 544 | panic | Expected Quiet with 999 days |
| 544 | panic! | Expected Quiet with 999 days |

### hoop-daemon/tests/quarantine_integration.rs

**Total messages in file:** 19

| Line | Pattern Type | Message |
|------|--------------|---------|
| 57 | assert_* | should parse 3 good lines |
| 57 | assertion message | should parse 3 good lines |
| 58 | assert_* | should quarantine 1 bad line |
| 58 | assertion message | should quarantine 1 bad line |
| 59 | assert_* | should skip 1 empty line |
| 59 | assertion message | should skip 1 empty line |
| 62 | assert! | quarantine dir should exist |
| 67 | assert_* | should have one date directory |
| 67 | assertion message | should have one date directory |
| 73 | assert_* | should have one quarantined entry |
| 73 | assertion message | should have one quarantined entry |
| 218 | assert_* | Codex should parse 4 good lines |
| 219 | assert_* | Codex should quarantine 1 bad line |
| 220 | assert_* | Gemini should parse 3 good lines |
| 221 | assert_* | Gemini should quarantine 1 bad line |
| 228 | assert_* | should have one date directory |
| 228 | assertion message | should have one date directory |
| 234 | assert_* | should have two quarantined entries (one per adapter) |
| 234 | assertion message | should have two quarantined entries (one per adapter) |

### hoop-daemon/tests/reflection_detector_integration.rs

**Total messages in file:** 14

| Line | Pattern Type | Message |
|------|--------------|---------|
| 168 | assert! | run_detection should succeed |
| 171 | assert_* | Should propose 1 pattern from 3 similar negatives |
| 186 | assert_* | Should have 1 reflection ledger entry |
| 196 | assert_* | Should have 3 source stitches |
| 235 | assert_* | Should propose 1 preference pattern |
| 273 | assert_* | Should propose 1 correction pattern |
| 326 | assert_* | Should not propose patterns: worker stitches ignored, operator below threshold |
| 446 | assert_* | Should not propose patterns: old stitches outside window |
| 554 | assert! | build_reflection_rules_with_audit should succeed |
| 572 | assert_* | Should have 2 audit rows, one per injected rule |
| 605 | assert! | last_applied should be set |
| 606 | assert_* | applied_count should be 1 after injection |
| 624 | assert_* | applied_count should be 2 after second injection |
| 633 | assert_* | Should have 4 audit rows total (2 per injection) |

### hoop-daemon/tests/risk_patterns_standalone.rs

**Total messages in file:** 9

| Line | Pattern Type | Message |
|------|--------------|---------|
| 84 | assert_* | Should find exactly one match for 'test' keyword |
| 85 | assert_* | Matched pattern should have the expected ID |
| 139 | assert_* | Library should contain exactly 2 patterns |
| 142 | assert_* | Should find exactly one match for keyword1 |
| 146 | assert_* | Should find exactly one match for keyword2 |
| 167 | assert_* | Should find match via label keyword |
| 188 | assert_* | Should find match via title keyword |
| 192 | assert_* | Should find match via label keyword |
| 196 | assert_* | Should find match with both keywords |

### hoop-daemon/tests/s1_morning_review.rs

**Total messages in file:** 29

| Line | Pattern Type | Message |
|------|--------------|---------|
| 29 | .expect() | Failed to spawn daemon |
| 38 | .expect() | Failed to fetch dashboard |
| 49 | .expect() | Failed to parse dashboard response |
| 58 | .expect() | total_workers must be a number |
| 72 | .expect() | total_spend_usd must be a number |
| 86 | .expect() | longest_running must be an array |
| 94 | .expect() | Failed to fetch worker timeline |
| 102 | .expect() | Failed to parse timeline |
| 117 | .expect() | Failed to spawn daemon |
| 127 | .expect() | Failed to fetch dashboard |
| 131 | assert_* | Dashboard should return 200 |
| 150 | .expect() | Failed to spawn daemon |
| 159 | .expect() | Failed to fetch dashboard |
| 167 | .expect() | Failed to parse response |
| 188 | .expect() | Failed to spawn daemon |
| 197 | .expect() | Failed to fetch dashboard |
| 199 | .expect() | Failed to parse response |
| 209 | .expect() | Failed to fetch dashboard |
| 211 | .expect() | Failed to parse response |
| 229 | .expect() | Failed to spawn daemon |
| 237 | .expect() | Failed to fetch dashboard |
| 239 | .expect() | Failed to parse response |
| 244 | .expect() | total_spend_usd must be present |
| 254 | .expect() | spend_by_project must be an array |
| 279 | .expect() | Failed to spawn daemon |
| 287 | .expect() | Failed to fetch dashboard |
| 289 | .expect() | Failed to parse response |
| 293 | .expect() | total_workers must be present |
| 297 | .expect() | workers_by_project must be an array |

### hoop-daemon/tests/s2_transcript_archaeology.rs

**Total messages in file:** 31

| Line | Pattern Type | Message |
|------|--------------|---------|
| 32 | .expect() | Failed to spawn daemon |
| 41 | .expect() | Failed to fetch beads |
| 49 | .expect() | Failed to parse beads |
| 56 | .expect() | Bead should have an id |
| 63 | .expect() | Failed to fetch bead events |
| 73 | .expect() | Failed to parse events |
| 74 | assert! | Events should be an array |
| 90 | .expect() | Failed to spawn daemon |
| 99 | .expect() | Failed to fetch beads |
| 101 | .expect() | Failed to parse beads |
| 107 | .expect() | Bead should have an id |
| 116 | .expect() | Failed to fetch bead events |
| 139 | .expect() | Failed to spawn daemon |
| 148 | .expect() | Failed to connect to stitch endpoint |
| 167 | .expect() | Failed to spawn daemon |
| 184 | .expect() | Failed to connect to endpoint |
| 203 | .expect() | Failed to spawn daemon |
| 212 | .expect() | Failed to fetch conversations |
| 220 | .expect() | Failed to parse conversations |
| 238 | .expect() | Failed to spawn daemon |
| 247 | .expect() | Failed to fetch beads |
| 249 | .expect() | Failed to parse beads |
| 272 | .expect() | Failed to spawn daemon |
| 281 | .expect() | Failed to fetch cost trends |
| 289 | .expect() | Failed to parse cost data |
| 308 | .expect() | Failed to spawn daemon |
| 317 | .expect() | Failed to fetch beads |
| 319 | .expect() | Failed to parse beads |
| 325 | .expect() | Bead should have an id |
| 332 | .expect() | Failed to fetch bead events |
| 335 | .expect() | Failed to parse events |

### hoop-daemon/tests/s3_bead_creation_from_chat.rs

**Total messages in file:** 78

| Line | Pattern Type | Message |
|------|--------------|---------|
| 41 | .expect() | create temp dir |
| 56 | .expect() | create br script |
| 57 | .expect() | write br script |
| 62 | .expect() | chmod br script |
| 107 | .expect() | Failed to spawn daemon |
| 133 | .expect() | Failed to create draft |
| 145 | .expect() | Failed to parse draft response |
| 149 | .expect() | draft_id should be present |
| 162 | .expect() | Failed to list drafts |
| 164 | assert_* | List drafts should return 200 |
| 169 | .expect() | Failed to parse list response |
| 173 | .expect() | drafts should be an array |
| 179 | assert! | Draft should appear in the draft queue |
| 186 | .expect() | Failed to get draft |
| 188 | assert_* | Get draft should return 200 |
| 193 | .expect() | Failed to parse draft |
| 195 | assert_* | Draft title should match chat input |
| 196 | assert_* | Draft kind should be fix |
| 197 | assert_* | Draft source should be chat |
| 198 | assert_* | Draft project should be testrepo |
| 199 | assert_* | Draft status should be pending |
| 217 | .expect() | Failed to spawn daemon |
| 235 | .expect() | Failed to create draft |
| 240 | .expect() | Failed to parse draft response |
| 244 | .expect() | draft_id should be present |
| 254 | .expect() | Failed to approve draft |
| 266 | .expect() | Failed to parse approve response |
| 270 | .expect() | stitch_id should be present |
| 292 | .expect() | Failed to get draft |
| 297 | .expect() | Failed to parse draft |
| 299 | assert_* | Draft status should be submitted |
| 300 | assert_* | Draft should have stitch_id |
| 318 | .expect() | Failed to spawn daemon |
| 336 | .expect() | Failed to create draft |
| 341 | .expect() | Failed to parse draft response |
| 345 | .expect() | draft_id should be present |
| 353 | .expect() | Failed to approve draft |
| 358 | .expect() | Failed to parse approve response |
| 362 | .expect() | stitch_id should be present |
| 369 | .expect() | Failed to query audit log |
| 371 | assert_* | Audit query should return 200 |
| 376 | .expect() | Failed to parse audit response |
| 380 | .expect() | audit_rows should be an array |
| 388 | assert! | Audit log should contain DraftCreated entry |
| 392 | .expect() | args should be an object |
| 393 | assert_* | DraftCreated source should be chat |
| 401 | assert! | Audit log should contain DraftApproved entry |
| 404 | .expect() | args should be an object |
| 412 | .expect() | actor should be present |
| 413 | assert! | Operator identity should be present in audit log |
| 434 | .expect() | Failed to spawn daemon |
| 459 | .expect() | Failed to create draft |
| 464 | .expect() | Failed to parse response |
| 465 | .expect() | draft_id present |
| 472 | .expect() | Failed to list drafts |
| 474 | .expect() | Failed to parse list |
| 475 | .expect() | drafts array |
| 478 | assert! | Draft should be in queue |
| 488 | .expect() | Failed to approve draft |
| 493 | .expect() | Failed to parse approve |
| 494 | .expect() | stitch_id present |
| 509 | .expect() | Failed to query audit |
| 511 | .expect() | Failed to parse audit |
| 512 | .expect() | audit_rows array |
| 522 | assert! | Audit should have DraftCreated |
| 523 | assert! | Audit should have DraftApproved |
| 526 | .expect() | args object |
| 527 | assert_* | source should be chat |
| 530 | .expect() | args object |
| 531 | assert_* | stitch_id should match |
| 534 | .expect() | actor present |
| 535 | assert! | operator identity should be present |
| 556 | .expect() | Failed to spawn daemon |
| 577 | .expect() | Failed to create draft |
| 581 | .expect() | Failed to parse |
| 582 | .expect() | draft_id present |
| 589 | .expect() | Failed to get draft |
| 593 | .expect() | Failed to parse draft |

### hoop-daemon/tests/s4_daemon_restart.rs

**Total messages in file:** 43

| Line | Pattern Type | Message |
|------|--------------|---------|
| 33 | .expect() | workspace root is parent of hoop-daemon/ |
| 107 | .expect() | create temp dir for test HOOP home |
| 109 | .expect() | create .hoop dir |
| 124 | .expect() | write projects.yaml |
| 133 | .expect() | write config.yml |
| 135 | .expect() | create data dir |
| 155 | .expect() | init fleet.db |
| 159 | .expect() | write claim |
| 160 | .expect() | write complete |
| 161 | .expect() | write claim |
| 170 | .expect() | Failed to spawn first daemon |
| 195 | .expect() | Failed to fetch beads from first daemon |
| 203 | .expect() | Failed to parse beads |
| 212 | .expect() | write complete |
| 213 | .expect() | write claim |
| 226 | .expect() | Failed to spawn second daemon |
| 249 | .expect() | Failed to fetch beads from second daemon |
| 257 | .expect() | Failed to parse beads |
| 290 | .expect() | init fleet.db |
| 296 | .expect() | write claim |
| 298 | .expect() | write complete |
| 305 | .expect() | Failed to spawn first daemon |
| 332 | .expect() | Failed to spawn second daemon |
| 364 | .expect() | Failed to fetch beads |
| 366 | assert_* | Should be able to fetch beads after rebuild |
| 387 | .expect() | init fleet.db |
| 392 | .expect() | Failed to spawn first daemon |
| 418 | .expect() | write claim |
| 419 | .expect() | write complete |
| 420 | .expect() | write claim |
| 432 | .expect() | Failed to spawn second daemon |
| 451 | .expect() | write complete |
| 452 | .expect() | write claim |
| 466 | .expect() | Failed to fetch beads |
| 468 | assert_* | Should see all beads including those created during restart |
| 488 | .expect() | init fleet.db |
| 495 | .expect() | write claim |
| 496 | .expect() | write complete |
| 502 | .expect() | Failed to spawn daemon |
| 525 | .expect() | Failed to fetch beads |
| 527 | assert_* | Should fetch beads in cycle {} |
| 529 | .expect() | Failed to parse beads |
| 552 | .expect() | write claim |

### hoop-daemon/tests/s5_workspace_deleted.rs

**Total messages in file:** 29

| Line | Pattern Type | Message |
|------|--------------|---------|
| 30 | .expect() | Failed to create .beads dir |
| 32 | .expect() | Failed to create issues.jsonl |
| 40 | .expect() | Failed to create temp dir |
| 42 | .expect() | Failed to create .hoop dir |
| 71 | .expect() | Failed to write projects.yaml |
| 79 | .expect() | Failed to write config.yml |
| 80 | .expect() | Failed to create data dir |
| 120 | .expect() | Failed to bind to random port |
| 121 | .expect() | Failed to get local address |
| 166 | .expect() | Failed to get readyz status |
| 168 | assert_* | Initial readyz should return 200 |
| 169 | assert_* | Initial readyz status should be ok |
| 173 | .expect() | Failed to remove .beads from project A |
| 222 | .expect() | Failed to bind to random port |
| 223 | .expect() | Failed to get local address |
| 265 | .expect() | Failed to remove .beads from project A |
| 275 | .expect() | Failed to fetch projects |
| 277 | assert_* | Projects endpoint should still work |
| 279 | .expect() | Failed to parse projects |
| 292 | .expect() | Failed to check health |
| 323 | .expect() | Failed to bind to random port |
| 324 | .expect() | Failed to get local address |
| 367 | .expect() | Failed to get readyz status |
| 372 | .expect() | Failed to remove .beads from project A |
| 379 | .expect() | Failed to get readyz status after deletion |
| 427 | .expect() | Failed to bind to random port |
| 428 | .expect() | Failed to get local address |
| 470 | .expect() | Failed to remove .beads |
| 479 | .expect() | Failed to check health |

### hoop-daemon/tests/secrets_scanner_integration.rs

**Total messages in file:** 5

| Line | Pattern Type | Message |
|------|--------------|---------|
| 252 | assert! | High-entropy string should be detected |
| 302 | assert! | high |
| 341 | assert! | API key should have high entropy: {} |
| 346 | assert! | Normal text should have low entropy: {} |
| 360 | assert! | Very short strings should not be flagged |

### hoop-daemon/tests/secrets_scanner_parity.rs

**Total messages in file:** 8

| Line | Pattern Type | Message |
|------|--------------|---------|
| 212 | assert! | Default patterns should not be empty |
| 216 | assert! | Pattern ID should not be empty |
| 217 | assert! | Pattern name should not be empty |
| 224 | assert! | Pattern '{}' should have at least one regex |
| 238 | .expect() | Pattern should serialize to JSON |
| 239 | .expect() | Serialized pattern should deserialize |
| 279 | assert! | Should detect Anthropic API key |
| 308 | assert! | Custom pattern should detect test secret |

### hoop-daemon/tests/session_redaction.rs

**Total messages in file:** 11

| Line | Pattern Type | Message |
|------|--------------|---------|
| 105 | assertion message | expected [REDACTED], got: {out} |
| 123 | assertion message | expected [REDACTED], got: {out} |
| 160 | assert_* | clean content must pass through unchanged |
| 170 | assert_* | cache must return same result |
| 171 | assert_* | cache must return same result |
| 172 | assert! | must be redacted: {r1} |
| 172 | assertion message | must be redacted: {r1} |
| 173 | assert! | raw key must not appear: {r1} |
| 216 | .expect() | valid JSON |
| 245 | assert! | JWT must be redacted: {out} |
| 246 | assert! | raw JWT must not appear: {out} |

### hoop-daemon/tests/skills_integration.rs

**Total messages in file:** 31

| Line | Pattern Type | Message |
|------|--------------|---------|
| 18 | .expect() | Failed to create temp dir |
| 21 | .expect() | Failed to create skill dir |
| 44 | .expect() | Failed to write manifest |
| 56 | .expect() | Failed to create temp dir |
| 59 | .expect() | Failed to create skill dir |
| 77 | .expect() | Failed to write manifest |
| 86 | .expect() | Failed to write run script |
| 90 | .expect() | Failed to get metadata |
| 94 | .expect() | Failed to set permissions |
| 117 | .expect() | Failed to create temp dir |
| 120 | .expect() | Failed to create skill dir |
| 140 | .expect() | Failed to write manifest |
| 162 | .expect() | Failed to create temp dir |
| 165 | .expect() | Failed to create skill dir |
| 187 | .expect() | Failed to write manifest |
| 211 | .expect() | Failed to create temp dir |
| 214 | .expect() | Failed to create skill dir |
| 233 | .expect() | Failed to write manifest |
| 252 | .expect() | Failed to create temp dir |
| 255 | .expect() | Failed to create skill dir |
| 275 | .expect() | Failed to write manifest |
| 350 | .expect() | Failed to create temp dir |
| 353 | .expect() | Failed to create skill dir |
| 369 | .expect() | Failed to write manifest |
| 374 | assert_* | project-b |
| 379 | .expect() | Failed to create temp dir |
| 382 | .expect() | Failed to create skill dir |
| 396 | .expect() | Failed to write manifest |
| 406 | .expect() | Failed to create temp dir |
| 409 | .expect() | Failed to create skill dir |
| 422 | .expect() | Failed to write manifest |

### hoop-daemon/tests/skills_quarantine_integration.rs

**Total messages in file:** 11

| Line | Pattern Type | Message |
|------|--------------|---------|
| 56 | .expect() | Failed to create temp dir |
| 83 | .expect() | Failed to create temp dir |
| 111 | .expect() | Failed to create temp dir |
| 138 | .expect() | Failed to create temp dir |
| 162 | .expect() | Failed to create temp dir |
| 206 | .expect() | Failed to create temp dir |
| 234 | .expect() | Failed to create temp dir |
| 264 | .expect() | Failed to create temp dir |
| 289 | .expect() | Failed to create temp dir |
| 307 | .expect() | Failed to create temp dir |
| 326 | .expect() | Failed to create temp dir |

### hoop-daemon/tests/state_projections.rs

**Total messages in file:** 76

| Line | Pattern Type | Message |
|------|--------------|---------|
| 153 | .expect() | Failed to spawn daemon |
| 161 | .expect() | Health check request failed |
| 163 | assert! | Health check should return 200 |
| 171 | .expect() | Failed to spawn daemon |
| 178 | .expect() | Failed to connect to WebSocket |
| 184 | .expect() | Timeout waiting for first message |
| 185 | .expect() | WebSocket stream ended |
| 187 | .expect() | Failed to receive first message |
| 191 | .expect() | Failed to parse init event |
| 193 | assert_* | First message must be init |
| 211 | panic! | Expected text message for init, got {:?} |
| 220 | .expect() | Failed to spawn daemon |
| 224 | .expect() | Failed to collect snapshots |
| 226 | assert! | Must receive workers_snapshot |
| 227 | assert! | Must receive beads_snapshot |
| 228 | assert! | Must receive conversations_snapshot |
| 229 | assert! | Must receive projects_snapshot |
| 230 | assert! | Must receive config_status |
| 238 | .expect() | Failed to spawn daemon |
| 243 | .expect() | Failed to collect WS snapshots |
| 253 | .expect() | REST workers request failed |
| 256 | .expect() | Failed to parse REST workers response |
| 263 | .expect() | REST beads request failed |
| 266 | .expect() | Failed to parse REST beads response |
| 273 | .expect() | REST projects request failed |
| 276 | .expect() | Failed to parse REST projects response |
| 298 | .expect() | Failed to spawn daemon |
| 305 | .expect() | Failed to connect |
| 312 | .expect() | Timeout waiting for init |
| 313 | .expect() | Stream ended |
| 314 | .expect() | Failed to receive init |
| 329 | .expect() | Failed to send subscribe |
| 339 | .expect() | Failed to send unsubscribe |
| 344 | assert! | Should receive messages after subscribe/unsubscribe |
| 352 | .expect() | Failed to spawn daemon |
| 360 | .expect() | Config status request failed |
| 363 | .expect() | Failed to parse config status |
| 376 | .expect() | Failed to spawn daemon |
| 384 | .expect() | Beads request failed |
| 387 | .expect() | Failed to parse beads response |
| 390 | assert! | Beads must be an array |
| 394 | assert! | Each bead must have an id |
| 395 | assert! | Each bead must have a title |
| 396 | assert! | Each bead must have a status |
| 405 | .expect() | Failed to spawn daemon |
| 413 | .expect() | Workers request failed |
| 416 | .expect() | Failed to parse workers response |
| 419 | assert! | Workers response is valid array |
| 427 | .expect() | Failed to spawn daemon |
| 435 | .expect() | Projects request failed |
| 438 | .expect() | Failed to parse projects response |
| 441 | assert! | Projects response is valid array |
| 449 | .expect() | Failed to spawn daemon |
| 469 | .expect() | Stream ended |
| 475 | .expect() | Failed to parse |
| 488 | assert! | Connection should receive init |
| 488 | .expect() | Task failed |
| 497 | .expect() | Failed to spawn daemon |
| 505 | .expect() | Failed to connect first time |
| 535 | .expect() | Failed to reconnect |
| 572 | assert! | Reconnect should receive init event |
| 573 | assert! | Reconnect should receive beads_snapshot |
| 590 | .expect() | Failed to spawn daemon |
| 599 | .expect() | Failed to connect |
| 653 | assert! | Should receive all snapshot events |
| 662 | assert! | global should be valid |
| 663 | assert! | project:testrepo should be valid |
| 664 | assert! | project with colons should be valid |
| 667 | assert! | empty project name should be invalid |
| 668 | assert! | fleet: prefix should be invalid |
| 669 | assert! | empty string should be invalid |
| 670 | assert! | GLOBAL (uppercase) should be invalid |
| 679 | .expect() | Failed to spawn first daemon |
| 683 | .expect() | Failed to spawn second daemon |
| 698 | .expect() | First daemon health check failed |
| 704 | .expect() | Second daemon health check failed |

### hoop-daemon/tests/stderr_stdout_capture.rs

**Total messages in file:** 2

| Line | Pattern Type | Message |
|------|--------------|---------|
| 166 | assert! | Large config should generate more output than default |
| 171 | assert_* | Same configuration should produce identical output size |

### hoop-daemon/tests/stdout_generation_test.rs

**Total messages in file:** 19

| Line | Pattern Type | Message |
|------|--------------|---------|
| 150 | .expect() | Failed to execute subprocess |
| 183 | .expect() | Failed to execute test binary |
| 266 | .expect() | Failed to execute multi-line subprocess |
| 283 | assert! | Subprocess should succeed |
| 284 | assert! | Should have stdout output |
| 296 | assert! | Stderr subprocess should succeed |
| 308 | assert! | Mixed subprocess should succeed |
| 309 | assert! | Should have stdout output |
| 310 | assert! | Should have stderr output |
| 322 | assert! | Multi-line subprocess should succeed |
| 323 | assert! | Should have stdout output |
| 324 | assert! | Should have stderr output |
| 330 | assert_* | Should have 5 stdout lines |
| 331 | assert_* | Should have 5 stderr lines |
| 351 | assert! | Configured subprocess should succeed |
| 365 | assert! | Should have exit code |
| 366 | assert! | Should succeed |
| 367 | assert! | Should have stdout |
| 380 | assert! | Path should be in target directory |

### hoop-daemon/tests/stdout_verification.rs

**Total messages in file:** 3

| Line | Pattern Type | Message |
|------|--------------|---------|
| 88 | assert! | In-memory verification should pass |
| 117 | assert! | Verification should fail for mismatched content |
| 149 | assert! | Unicode verification should pass |

### hoop-daemon/tests/stitch_percentile_index_integration.rs

**Total messages in file:** 54

| Line | Pattern Type | Message |
|------|--------------|---------|
| 22 | .expect() | Failed to open test DB |
| 41 | .expect() | Failed to create stitches table |
| 57 | .expect() | Failed to create stitch_messages table |
| 72 | .expect() | Failed to create actions table |
| 76 | .expect() | Failed to initialize percentile index |
| 113 | .expect() | Failed to insert stitch |
| 131 | .expect() | Failed to insert message |
| 150 | .expect() | Failed to insert action |
| 156 | .expect() | Failed to create temp dir |
| 166 | .expect() | Failed to check table existence |
| 168 | assert! | stitch_percentile_index table should exist |
| 177 | .expect() | Failed to check metadata table existence |
| 179 | assert! | stitch_percentile_index_meta table should exist |
| 188 | .expect() | Failed to get schema version |
| 198 | .expect() | Failed to create temp dir |
| 204 | .expect() | Failed to check schema version |
| 208 | .expect() | Failed to check rebuild needed |
| 220 | .expect() | Failed to corrupt schema version |
| 225 | .expect() | Failed to check schema version |
| 229 | .expect() | Failed to check rebuild needed |
| 324 | .expect() | Failed to create temp dir |
| 360 | .expect() | Failed to rebuild index |
| 369 | .expect() | Failed to count index entries |
| 371 | assert_* | Should have one bucket for 3 similar stitches |
| 380 | .expect() | Failed to query bucket |
| 383 | assert! | Cost p50 should be positive |
| 384 | assert! | Cost p90 should be >= p50 |
| 385 | assert_* | Should have 3 samples |
| 390 | .expect() | Failed to create temp dir |
| 421 | .expect() | Failed to rebuild index |
| 430 | .expect() | Failed to count buckets |
| 446 | .expect() | Query should succeed |
| 467 | .expect() | Failed to create temp dir |
| 482 | .expect() | Failed to rebuild index |
| 492 | .expect() | Query should succeed |
| 506 | .expect() | Failed to create temp dir |
| 514 | .expect() | Failed to rebuild index |
| 523 | .expect() | Query should succeed |
| 546 | assert_* | Should take first 5 tokens |
| 557 | .expect() | Failed to create temp dir |
| 572 | .expect() | Failed to rebuild index |
| 582 | .expect() | Query should succeed |
| 598 | .expect() | Query should succeed |
| 609 | .expect() | Failed to create temp dir |
| 617 | .expect() | Failed to rebuild index |
| 625 | .expect() | Failed to count |
| 627 | assert_* | Should have one bucket |
| 642 | .expect() | Failed to rebuild index |
| 650 | .expect() | Failed to count |
| 653 | assert_* | Should have two buckets after rebuild |
| 658 | .expect() | Failed to create temp dir |
| 678 | .expect() | Failed to rebuild index |
| 687 | .expect() | Failed to query bucket |
| 689 | assert_* | Should have 5 samples |

### hoop-daemon/tests/supervisor_health.rs

**Total messages in file:** 17

| Line | Pattern Type | Message |
|------|--------------|---------|
| 57 | .expect() | Failed to create CostAggregator |
| 141 | .expect() | Reconcile should succeed |
| 178 | .expect() | Reconcile should succeed |
| 186 | assert! | Should receive status update |
| 216 | .expect() | Reconcile should succeed |
| 246 | assert! | Should not be ready with no runtimes |
| 257 | .expect() | Reconcile should succeed |
| 263 | assert! | Should be ready with healthy runtime |
| 284 | .expect() | Reconcile should succeed |
| 309 | assert! | Should not be ready when all failed |
| 323 | assert! | Should not be ready when all in error state |
| 337 | assert! | Should not be ready when all abandoned |
| 378 | assert! | Should be ready with at least one healthy |
| 429 | .expect() | Reconcile should succeed |
| 467 | .expect() | Reconcile should succeed |
| 508 | .expect() | Reconcile should succeed |
| 579 | .expect() | Reconcile should succeed |

### hoop-daemon/tests/supervisor_hotreload.rs

**Total messages in file:** 13

| Line | Pattern Type | Message |
|------|--------------|---------|
| 115 | .expect() | Empty reconcile should succeed |
| 118 | assert_* | Should have no runtimes initially |
| 129 | .expect() | Reconcile with new project should succeed |
| 135 | assert_* | Should have one runtime |
| 166 | .expect() | Reconcile with multiple projects should succeed |
| 172 | assert_* | Should have three runtimes |
| 202 | .expect() | Reconcile with two projects should succeed |
| 207 | assert_* | Should have two runtimes initially |
| 215 | .expect() | Reconcile after removal should succeed |
| 220 | assert_* | Should have one runtime after removal |
| 242 | .expect() | Initial reconcile should succeed |
| 253 | .expect() | No-op reconcile should succeed |
| 317 | .expect() | Reconcile should succeed |

### hoop-daemon/tests/supervisor_isolation.rs

**Total messages in file:** 14

| Line | Pattern Type | Message |
|------|--------------|---------|
| 59 | .expect() | CostAggregator creation should succeed |
| 138 | .expect() | Reconcile should succeed |
| 144 | assert_* | Should have two runtimes |
| 181 | .expect() | Reconcile should succeed |
| 204 | assert_* | Both runtimes should still exist |
| 210 | .expect() | project-a should exist |
| 215 | .expect() | project-b should exist |
| 260 | .expect() | Reconcile should succeed |
| 288 | .expect() | project-b should exist |
| 293 | .expect() | project-c should exist |
| 383 | .expect() | Reconcile should succeed |
| 428 | .expect() | Reconcile should succeed |
| 475 | .expect() | Reconcile should succeed |
| 507 | .expect() | project-b should exist |

### hoop-daemon/tests/supervisor_restart.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 57 | .expect() | Failed to create cost aggregator |

### hoop-daemon/tests/supervisor_shutdown.rs

**Total messages in file:** 15

| Line | Pattern Type | Message |
|------|--------------|---------|
| 119 | .expect() | Reconcile should succeed |
| 125 | assert_* | Should have one runtime |
| 130 | assert! | Runtime should be running |
| 136 | assert_* | Runtime should still exist |
| 161 | .expect() | Reconcile should succeed |
| 166 | assert_* | Should have two runtimes |
| 174 | .expect() | Reconcile after removal should succeed |
| 179 | assert_* | Should have one runtime after removal |
| 210 | .expect() | Reconcile should succeed |
| 215 | assert_* | Should have three runtimes |
| 223 | .expect() | Reconcile to empty should succeed |
| 228 | assert_* | Should have no runtimes after shutdown |
| 252 | .expect() | Reconcile should succeed |
| 301 | .expect() | Reconcile should succeed |
| 318 | .expect() | Reconcile to empty should succeed |

### hoop-daemon/tests/testrepo_harness_integration.rs

**Total messages in file:** 68

| Line | Pattern Type | Message |
|------|--------------|---------|
| 57 | anyhow::bail! | Daemon did not become ready |
| 258 | .expect() | Failed to spawn daemon |
| 260 | .expect() | Failed to create test client |
| 263 | .expect() | Health check failed |
| 264 | assert_* | Health check should return ok |
| 267 | .expect() | Ready check failed |
| 268 | assert_* | Ready check should return ok |
| 276 | .expect() | Failed to spawn daemon |
| 278 | .expect() | Failed to create test client |
| 285 | .expect() | Failed to connect to WebSocket |
| 291 | .expect() | Timeout waiting for first message |
| 292 | .expect() | WebSocket stream ended |
| 294 | .expect() | Failed to receive first message |
| 298 | .expect() | Failed to parse init event |
| 300 | assert_* | First message must be init |
| 309 | .expect() | subscriptions should be array |
| 317 | .expect() | subscriptions should be array |
| 327 | panic! | First message must be text, got {:?} |
| 336 | .expect() | Failed to spawn daemon |
| 338 | .expect() | Failed to create test client |
| 340 | .expect() | Failed to collect snapshots |
| 370 | .expect() | Failed to spawn daemon |
| 372 | .expect() | Failed to create test client |
| 375 | .expect() | Failed to fetch beads |
| 377 | assert! | Beads response should be an array |
| 380 | .expect() | Failed to fetch workers |
| 382 | assert! | Workers response should be an array |
| 385 | .expect() | Failed to fetch conversations |
| 387 | assert! | Conversations response should be an array |
| 390 | .expect() | Failed to fetch projects |
| 392 | assert! | Projects response should be an array |
| 395 | .expect() | Failed to fetch config status |
| 396 | assert! | Config status must include 'valid' field |
| 399 | .expect() | Failed to fetch capacity |
| 401 | assert! | Capacity should be object or array |
| 409 | .expect() | Failed to spawn daemon |
| 411 | .expect() | Failed to create test client |
| 413 | .expect() | Failed to fetch metrics |
| 444 | .expect() | Failed to spawn daemon |
| 451 | .expect() | Failed to connect |
| 458 | .expect() | Timeout waiting for init |
| 459 | .expect() | Stream ended |
| 460 | .expect() | Failed to receive init |
| 474 | .expect() | Failed to send subscribe |
| 483 | .expect() | Failed to send unsubscribe |
| 488 | assert! | Should receive messages after subscribe/unsubscribe |
| 496 | .expect() | Failed to spawn daemon |
| 516 | .expect() | Stream ended |
| 522 | .expect() | Failed to parse |
| 535 | assert! | Connection should receive init |
| 535 | .expect() | Task failed |
| 544 | .expect() | Failed to spawn daemon |
| 553 | .expect() | Failed to connect first time |
| 560 | .expect() | Timeout on first connection |
| 561 | .expect() | Stream ended |
| 562 | .expect() | No init on first connection |
| 573 | .expect() | Failed to reconnect |
| 580 | .expect() | Timeout on reconnection |
| 581 | .expect() | Stream ended |
| 582 | .expect() | No init on reconnection |
| 592 | .expect() | Timeout waiting for snapshots after reconnect |
| 593 | .expect() | Stream ended |
| 594 | .expect() | No snapshots after reconnect |
| 613 | .expect() | Failed to spawn daemon |
| 615 | .expect() | Failed to create test client |
| 618 | .expect() | Failed to fetch beads |
| 635 | .expect() | Failed to fetch workers |
| 645 | .expect() | Failed to fetch projects |

### hoop-daemon/tests/testrepo_integration.rs

**Total messages in file:** 73

| Line | Pattern Type | Message |
|------|--------------|---------|
| 67 | anyhow::bail! | Daemon did not become ready |
| 238 | .expect() | Failed to spawn daemon |
| 240 | .expect() | Failed to create test client |
| 243 | .expect() | Health check failed |
| 244 | assert_* | Health check should return ok |
| 247 | .expect() | Ready check failed |
| 248 | assert_* | Ready check should return ok |
| 256 | .expect() | Failed to spawn daemon |
| 258 | .expect() | Failed to create test client |
| 265 | .expect() | Failed to connect to WebSocket |
| 271 | .expect() | Timeout waiting for first message |
| 272 | .expect() | WebSocket stream ended |
| 274 | .expect() | Failed to receive first message |
| 278 | .expect() | Failed to parse init event |
| 280 | assert_* | First message must be init |
| 289 | .expect() | subscriptions should be array |
| 299 | panic! | First message must be text, got {:?} |
| 308 | .expect() | Failed to spawn daemon |
| 310 | .expect() | Failed to create test client |
| 312 | .expect() | Failed to collect snapshots |
| 342 | .expect() | Failed to spawn daemon |
| 344 | .expect() | Failed to create test client |
| 347 | .expect() | Failed to collect WS snapshots |
| 350 | .expect() | Failed to fetch beads via REST |
| 351 | .expect() | Failed to fetch workers via REST |
| 352 | .expect() | Failed to fetch projects via REST |
| 353 | .expect() | Failed to fetch config via REST |
| 390 | .expect() | Failed to spawn daemon |
| 392 | .expect() | Failed to create test client |
| 395 | .expect() | Failed to fetch beads |
| 396 | assert! | Beads response should not be empty |
| 399 | .expect() | Failed to fetch workers |
| 400 | assert! | Workers response should not be empty |
| 403 | .expect() | Failed to fetch projects |
| 404 | assert! | Projects response should not be empty |
| 410 | assert! | testrepo should be in projects list |
| 413 | .expect() | Failed to fetch config status |
| 414 | assert! | Config status must include 'valid' field |
| 417 | .expect() | Failed to fetch capacity |
| 418 | assert! | Capacity should be object or array |
| 426 | .expect() | Failed to spawn daemon |
| 428 | .expect() | Failed to create test client |
| 430 | .expect() | Failed to fetch metrics |
| 461 | .expect() | Failed to spawn daemon |
| 468 | .expect() | Failed to connect |
| 475 | .expect() | Timeout waiting for init |
| 476 | .expect() | Stream ended |
| 477 | .expect() | Failed to receive init |
| 491 | .expect() | Failed to send subscribe |
| 500 | .expect() | Failed to send unsubscribe |
| 505 | assert! | Should receive messages after subscribe/unsubscribe |
| 513 | .expect() | Failed to spawn daemon |
| 533 | .expect() | Stream ended |
| 539 | .expect() | Failed to parse |
| 552 | assert! | Connection should receive init |
| 552 | .expect() | Task failed |
| 561 | .expect() | Failed to spawn daemon |
| 570 | .expect() | Failed to connect first time |
| 577 | .expect() | Timeout on first connection |
| 578 | .expect() | Stream ended |
| 579 | .expect() | No init on first connection |
| 590 | .expect() | Failed to reconnect |
| 597 | .expect() | Timeout on reconnection |
| 598 | .expect() | Stream ended |
| 599 | .expect() | No init on reconnection |
| 609 | .expect() | Timeout waiting for snapshots after reconnect |
| 610 | .expect() | Stream ended |
| 611 | .expect() | No snapshots after reconnect |
| 630 | .expect() | Failed to spawn daemon |
| 632 | .expect() | Failed to create test client |
| 635 | .expect() | Failed to fetch beads |
| 652 | .expect() | Failed to fetch workers |
| 665 | .expect() | Failed to fetch projects |

### hoop-daemon/tests/upload_secrets_scan.rs

**Total messages in file:** 8

| Line | Pattern Type | Message |
|------|--------------|---------|
| 42 | assert! | Should detect secret in attachment |
| 67 | assert! | Should detect at least 3 secrets |
| 94 | assert! | Clean attachment should have no findings |
| 138 | assert! | Binary files should not be scanned |
| 165 | assert! | Should detect secrets in JSON |
| 198 | assert! | Should detect at least 2 env var secrets |
| 218 | assert! | Large files should be skipped |
| 264 | assert_* | Should write one audit entry |

### hoop-daemon/tests/zero_write_invariant.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 221 | panic! | invariant violated: this test should not run under create-only-write |

### hoop-daemon/tests_phase5/adapter_failover_test.rs

**Total messages in file:** 110

| Line | Pattern Type | Message |
|------|--------------|---------|
| 46 | anyhow::bail! | Daemon did not become ready |
| 152 | .expect() | Failed to spawn daemon |
| 154 | .expect() | Failed to create client |
| 157 | .expect() | Health check failed |
| 158 | assert_* | Daemon should be healthy |
| 161 | .expect() | Failed to spawn agent |
| 162 | assert_* | Agent spawn should succeed |
| 169 | .expect() | Failed to get agent status |
| 170 | assert_* | Agent should be active |
| 173 | .expect() | Health check failed |
| 174 | assert_* | Daemon should remain healthy after 5xx |
| 184 | .expect() | Failed to spawn daemon |
| 186 | .expect() | Failed to create client |
| 189 | .expect() | Failed to spawn agent |
| 190 | assert_* | Agent spawn should succeed |
| 194 | .expect() | Should have session_db_id |
| 200 | .expect() | Failed to get agent status |
| 201 | assert_* | Agent should be active |
| 212 | .expect() | Failed to switch adapter |
| 213 | assert_* | Adapter switch should succeed |
| 217 | .expect() | Should have new session_db_id |
| 229 | .expect() | Failed to list sessions |
| 242 | assert_* | Should have exactly 1 active session |
| 243 | assert_* | Should have 1 switched (archived) session |
| 249 | .expect() | Failed to get agent status |
| 250 | assert_* | Agent should still be active |
| 251 | assert_* | Adapter should be zai |
| 252 | assert_* | Model should be glm-5 |
| 262 | .expect() | Failed to spawn daemon |
| 264 | .expect() | Failed to create client |
| 267 | .expect() | Failed to spawn agent |
| 272 | .expect() | Should have session_db_id |
| 278 | .expect() | Failed to switch adapter |
| 284 | .expect() | Failed to list sessions |
| 290 | .expect() | Should find archived session |
| 308 | .expect() | Failed to query stitch from fleet.db |
| 341 | .expect() | Failed to spawn daemon |
| 343 | .expect() | Failed to create client |
| 364 | .expect() | Failed to insert reflection entry |
| 367 | .expect() | Failed to spawn agent |
| 371 | .expect() | Failed to switch adapter |
| 375 | .expect() | Failed to list reflection entries |
| 386 | .expect() | Entry should exist |
| 399 | .expect() | Failed to spawn daemon |
| 401 | .expect() | Failed to create client |
| 404 | .expect() | Failed to spawn agent |
| 408 | .expect() | Should have session_db_id |
| 414 | .expect() | Failed to switch adapter |
| 420 | .expect() | Failed to switch adapter back |
| 424 | .expect() | Should have second session_db_id |
| 430 | .expect() | Failed to list sessions |
| 441 | assert_* | Should have 2 switched sessions |
| 447 | .expect() | Should find first archived session |
| 451 | .expect() | Should find second archived session |
| 477 | .expect() | Failed to spawn daemon |
| 479 | .expect() | Failed to create client |
| 482 | .expect() | Failed to spawn agent |
| 503 | .expect() | Failed to insert reflection entry |
| 509 | .expect() | Failed to switch adapter |
| 515 | .expect() | Failed to get agent status |
| 521 | .expect() | Failed to list reflection entries |
| 536 | .expect() | Failed to spawn daemon |
| 538 | .expect() | Failed to create client |
| 541 | .expect() | Failed to spawn agent |
| 568 | .expect() | Switch 1 should complete |
| 571 | .expect() | Switch 2 should complete |
| 580 | .expect() | Health check failed |
| 581 | assert_* | Daemon should remain healthy |
| 597 | .expect() | Failed to spawn daemon |
| 599 | .expect() | Failed to create client |
| 602 | .expect() | Failed to spawn agent |
| 603 | assert_* | Agent spawn should succeed |
| 607 | .expect() | Should have session_db_id |
| 613 | .expect() | Failed to get agent status |
| 614 | assert_* | Agent should be active |
| 639 | .expect() | Failed to write updated config.yml |
| 650 | .expect() | Failed to get agent status after config reload |
| 651 | assert_* | Agent should still be active |
| 657 | assert_* | Model should be glm-5 |
| 663 | .expect() | Failed to list sessions |
| 676 | assert_* | Should have exactly 1 active session |
| 677 | assert_* | Should have 1 switched (archived) session |
| 683 | .expect() | Should find original archived session |
| 700 | .expect() | Failed to query stitch from fleet.db |
| 722 | .expect() | Health check failed |
| 723 | assert_* | Daemon should remain healthy after hot-reload |
| 808 | .expect() | Failed to start mock Anthropic server |
| 815 | .expect() | Failed to spawn daemon |
| 817 | .expect() | Failed to create client |
| 820 | .expect() | Health check failed |
| 821 | assert_* | Daemon should be healthy initially |
| 840 | .expect() | Failed to write config with mock server URL |
| 856 | .expect() | Health check failed |
| 868 | .expect() | Ready endpoint request failed |
| 885 | .expect() | Health check failed |
| 895 | .expect() | Health check failed |
| 902 | assert! | Should have performed at least 6 health checks over 30s |
| 913 | .expect() | Failed to start mock Anthropic server |
| 920 | .expect() | Failed to spawn daemon |
| 922 | .expect() | Failed to create client |
| 925 | .expect() | Health check failed |
| 940 | .expect() | Failed to write config |
| 946 | .expect() | Health check failed |
| 953 | .expect() | Adapter switch should succeed |
| 955 | assert_* | Switch to ZAI should succeed |
| 961 | .expect() | Failed to get agent status |
| 962 | assert_* | Agent should be active after switch |
| 963 | assert_* | Should be using ZAI adapter |
| 966 | .expect() | Health check failed |
| 967 | assert_* | Daemon should be healthy after recovery |

### hoop-mcp/tests/create_only_stub.rs

**Total messages in file:** 13

| Line | Pattern Type | Message |
|------|--------------|---------|
| 20 | .expect() | create temp dir |
| 37 | .expect() | create br script |
| 38 | .expect() | write br script |
| 43 | .expect() | chmod br script |
| 96 | .expect() | run fake br |
| 97 | assert! | fake br should succeed |
| 103 | assertion message | expected exactly one invocation, got {:?} |
| 137 | assert_* | expected 3 invocations, got {:?} |
| 137 | assertion message | expected 3 invocations, got {:?} |
| 266 | .expect() | run fake br |
| 275 | assert_* | expected 3 invocations, got {:?} |
| 275 | assertion message | expected 3 invocations, got {:?} |
| 291 | assertion message | should contain stitch label |

### hoop-mcp/tests/forbidden_worker_steering.rs

**Total messages in file:** 7

| Line | Pattern Type | Message |
|------|--------------|---------|
| 87 | assert | HOOP cannot perform worker-steering actions |
| 89 | assert | br close |
| 90 | assert | NEEDLE's tooling |
| 116 | expect | Failed to create McpServerState for test |
| 116 | .expect() | Failed to create McpServerState for test |
| 149 | expect | Failed to create McpServerState for test |
| 149 | .expect() | Failed to create McpServerState for test |

### hoop-mcp/tests/protocol_contract.rs

**Total messages in file:** 50

| Line | Pattern Type | Message |
|------|--------------|---------|
| 22 | expect | workspace root |
| 22 | .expect() | workspace root |
| 26 | panic | fixture file missing: {} |
| 26 | panic! | fixture file missing: {} |
| 26 | unwrap_or_else panic | fixture file missing: {} |
| 26 | unwrap_or_else panic with args | fixture file missing: {} |
| 28 | panic | invalid JSON in fixture {}: {} |
| 28 | panic! | invalid JSON in fixture {}: {} |
| 28 | unwrap_or_else panic | invalid JSON in fixture {}: {} |
| 28 | unwrap_or_else panic with args | invalid JSON in fixture {}: {} |
| 28 | assertion message | invalid JSON in fixture {}: {} |
| 43 | expect | JsonRpcRequest must deserialize from initialize fixture |
| 43 | .expect() | JsonRpcRequest must deserialize from initialize fixture |
| 58 | panic | expected Method::Initialize |
| 58 | panic! | expected Method::Initialize |
| 58 | assertion message | expected Method::Initialize |
| 74 | expect | JsonRpcRequest must deserialize from tools/list fixture |
| 74 | .expect() | JsonRpcRequest must deserialize from tools/list fixture |
| 80 | panic | expected Method::ToolsList |
| 80 | panic! | expected Method::ToolsList |
| 80 | assertion message | expected Method::ToolsList |
| 122 | assert | jsonrpc |
| 145 | expect | JsonRpcRequest must deserialize from prompts/list fixture |
| 145 | .expect() | JsonRpcRequest must deserialize from prompts/list fixture |
| 151 | panic | expected Method::PromptsList |
| 151 | panic! | expected Method::PromptsList |
| 151 | assertion message | expected Method::PromptsList |
| 174 | assert | jsonrpc |
| 197 | expect | JsonRpcRequest must deserialize from resources/list fixture |
| 197 | .expect() | JsonRpcRequest must deserialize from resources/list fixture |
| 203 | panic | expected Method::ResourcesList |
| 203 | panic! | expected Method::ResourcesList |
| 203 | assertion message | expected Method::ResourcesList |
| 226 | assert | jsonrpc |
| 249 | expect | JsonRpcRequest must deserialize from shutdown fixture |
| 249 | .expect() | JsonRpcRequest must deserialize from shutdown fixture |
| 255 | panic | expected Method::Shutdown |
| 255 | panic! | expected Method::Shutdown |
| 255 | assertion message | expected Method::Shutdown |
| 278 | assert | jsonrpc |
| 333 | assert | jsonrpc |
| 376 | expect | JsonRpcRequest must deserialize from tools_call fixture |
| 376 | .expect() | JsonRpcRequest must deserialize from tools_call fixture |
| 396 | panic | expected Method::ToolsCall |
| 396 | panic! | expected Method::ToolsCall |
| 396 | assertion message | expected Method::ToolsCall |
| 431 | assert | jsonrpc |
| 432 | assert | result |
| 586 | assert | fixture {} must be a JSON object |
| 586 | assert! | fixture {} must be a JSON object |

### hoop-mcp/tests/socket_permissions.rs

**Total messages in file:** 8

| Line | Pattern Type | Message |
|------|--------------|---------|
| 14 | .expect() | temp dir |
| 18 | .expect() | bind socket |
| 21 | .expect() | set permissions |
| 24 | .expect() | metadata |
| 127 | .expect() | temp dir |
| 130 | .expect() | bind socket |
| 132 | .expect() | set permissions |
| 135 | .expect() | metadata |

### hoop-schema/tests/schema_drift.rs

**Total messages in file:** 14

| Line | Pattern Type | Message |
|------|--------------|---------|
| 756 | .expect() | Failed to create fixture directory |
| 778 | .expect() | Failed to write index |
| 860 | panic! | Failed to read fixture {}: {} |
| 860 | unwrap_or_else panic | Failed to read fixture {}: {} |
| 860 | unwrap_or_else panic with args | Failed to read fixture {}: {} |
| 864 | panic! | Failed to parse fixture {} as JSON: {} |
| 864 | unwrap_or_else panic | Failed to parse fixture {} as JSON: {} |
| 864 | unwrap_or_else panic with args | Failed to parse fixture {} as JSON: {} |
| 868 | panic! | Failed to serialize fixture {}: {} |
| 868 | unwrap_or_else panic | Failed to serialize fixture {}: {} |
| 868 | unwrap_or_else panic with args | Failed to serialize fixture {}: {} |
| 872 | panic! | Failed to parse normalized JSON for {}: {} |
| 885 | .expect() | Failed to read index.json |
| 887 | .expect() | Failed to parse index.json |

### testrepo/tests/integration/test_01.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 01 passed |

### testrepo/tests/integration/test_02.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 02 passed |

### testrepo/tests/integration/test_03.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 03 passed |

### testrepo/tests/integration/test_04.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 04 passed |

### testrepo/tests/integration/test_05.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 05 passed |

### testrepo/tests/integration/test_06.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 06 passed |

### testrepo/tests/integration/test_07.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 07 passed |

### testrepo/tests/integration/test_08.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 08 passed |

### testrepo/tests/integration/test_09.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 09 passed |

### testrepo/tests/integration/test_10.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 10 passed |

### testrepo/tests/integration/test_11.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 11 passed |

### testrepo/tests/integration/test_12.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 12 passed |

### testrepo/tests/integration/test_13.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 13 passed |

### testrepo/tests/integration/test_14.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 14 passed |

### testrepo/tests/integration/test_15.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 15 passed |

### testrepo/tests/integration/test_16.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 16 passed |

### testrepo/tests/integration/test_17.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 17 passed |

### testrepo/tests/integration/test_18.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 18 passed |

### testrepo/tests/integration/test_19.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 19 passed |

### testrepo/tests/integration/test_20.rs

**Total messages in file:** 1

| Line | Pattern Type | Message |
|------|--------------|---------|
| 5 | assert! | Integration test 20 passed |

### tests/acceptance/s1_morning_review.rs

**Total messages in file:** 78

| Line | Pattern Type | Message |
|------|--------------|---------|
| 34 | expect | workspace root |
| 34 | .expect() | workspace root |
| 96 | anyhow | Daemon failed to start within timeout |
| 96 | anyhow::anyhow! | Daemon failed to start within timeout |
| 103 | expect | Failed to spawn daemon |
| 103 | .expect() | Failed to spawn daemon |
| 111 | expect | Failed to fetch dashboard |
| 111 | .expect() | Failed to fetch dashboard |
| 113 | assert | Dashboard endpoint should return 200 |
| 113 | assert_* | Dashboard endpoint should return 200 |
| 115 | expect | Failed to parse dashboard |
| 115 | .expect() | Failed to parse dashboard |
| 123 | expect | total_workers must be a number |
| 123 | .expect() | total_workers must be a number |
| 131 | expect | total_spend_usd must be a number |
| 131 | .expect() | total_spend_usd must be a number |
| 132 | assert | total_spend_usd must be non-negative |
| 132 | assert! | total_spend_usd must be non-negative |
| 140 | expect | longest_running must be an array |
| 140 | .expect() | longest_running must be an array |
| 146 | expect | Failed to fetch worker timeline |
| 146 | .expect() | Failed to fetch worker timeline |
| 148 | assert | Worker timeline endpoint should return 200 |
| 148 | assert_* | Worker timeline endpoint should return 200 |
| 150 | expect | Failed to parse timeline |
| 150 | .expect() | Failed to parse timeline |
| 161 | expect | Failed to spawn daemon |
| 161 | .expect() | Failed to spawn daemon |
| 171 | expect | Failed to fetch dashboard |
| 171 | .expect() | Failed to fetch dashboard |
| 175 | assert | Dashboard should return 200 |
| 175 | assert_* | Dashboard should return 200 |
| 190 | expect | Failed to spawn daemon |
| 190 | .expect() | Failed to spawn daemon |
| 198 | expect | Failed to fetch dashboard |
| 198 | .expect() | Failed to fetch dashboard |
| 206 | expect | Failed to parse response |
| 206 | .expect() | Failed to parse response |
| 208 | assert | range |
| 209 | assert | total_workers |
| 210 | assert | total_spend_usd |
| 211 | assert | spend_by_project |
| 212 | assert | spend_by_adapter |
| 213 | assert | workers_by_project |
| 214 | assert | longest_running |
| 223 | expect | Failed to spawn daemon |
| 223 | .expect() | Failed to spawn daemon |
| 231 | expect | Failed to fetch dashboard |
| 231 | .expect() | Failed to fetch dashboard |
| 233 | expect | Failed to parse response |
| 233 | .expect() | Failed to parse response |
| 241 | expect | Failed to fetch dashboard |
| 241 | .expect() | Failed to fetch dashboard |
| 243 | expect | Failed to parse response |
| 243 | .expect() | Failed to parse response |
| 245 | assert | range |
| 254 | expect | Failed to spawn daemon |
| 254 | .expect() | Failed to spawn daemon |
| 262 | expect | Failed to fetch dashboard |
| 262 | .expect() | Failed to fetch dashboard |
| 264 | expect | Failed to parse response |
| 264 | .expect() | Failed to parse response |
| 268 | expect | total_spend_usd must be present |
| 268 | .expect() | total_spend_usd must be present |
| 270 | assert | Total cost must be non-negative |
| 270 | assert! | Total cost must be non-negative |
| 274 | expect | spend_by_project must be an array |
| 274 | .expect() | spend_by_project must be an array |
| 295 | expect | Failed to spawn daemon |
| 295 | .expect() | Failed to spawn daemon |
| 303 | expect | Failed to fetch dashboard |
| 303 | .expect() | Failed to fetch dashboard |
| 305 | expect | Failed to parse response |
| 305 | .expect() | Failed to parse response |
| 309 | expect | total_workers must be present |
| 309 | .expect() | total_workers must be present |
| 313 | expect | workers_by_project must be an array |
| 313 | .expect() | workers_by_project must be an array |

### tests/acceptance/s2_transcript_archaeology.rs

**Total messages in file:** 38

| Line | Pattern Type | Message |
|------|--------------|---------|
| 35 | .expect() | workspace root |
| 97 | anyhow::anyhow! | Daemon failed to start within timeout |
| 104 | .expect() | Failed to spawn daemon |
| 112 | .expect() | Failed to fetch beads |
| 114 | assert_* | Beads endpoint should return 200 |
| 116 | .expect() | Failed to parse beads |
| 122 | .expect() | Bead should have an id |
| 128 | .expect() | Failed to fetch bead events |
| 136 | .expect() | Failed to parse events |
| 137 | assert! | Events should be an array |
| 152 | .expect() | Failed to spawn daemon |
| 160 | .expect() | Failed to fetch beads |
| 162 | .expect() | Failed to parse beads |
| 168 | .expect() | Bead should have an id |
| 176 | .expect() | Failed to fetch bead events |
| 197 | .expect() | Failed to spawn daemon |
| 205 | .expect() | Failed to connect to stitch endpoint |
| 219 | .expect() | Failed to spawn daemon |
| 235 | .expect() | Failed to connect to endpoint |
| 251 | .expect() | Failed to spawn daemon |
| 259 | .expect() | Failed to fetch conversations |
| 261 | assert_* | Conversations endpoint should return 200 |
| 263 | .expect() | Failed to parse conversations |
| 265 | assert! | Conversations should be an array |
| 274 | .expect() | Failed to spawn daemon |
| 282 | .expect() | Failed to fetch beads |
| 284 | .expect() | Failed to parse beads |
| 300 | .expect() | Failed to spawn daemon |
| 308 | .expect() | Failed to fetch cost trends |
| 310 | assert_* | Cost trends endpoint should return 200 |
| 312 | .expect() | Failed to parse cost data |
| 314 | assert! | Cost data should be an object |
| 323 | .expect() | Failed to spawn daemon |
| 331 | .expect() | Failed to fetch beads |
| 333 | .expect() | Failed to parse beads |
| 339 | .expect() | Bead should have an id |
| 345 | .expect() | Failed to fetch bead events |
| 348 | .expect() | Failed to parse events |

### tests/acceptance/s3_bead_creation_from_chat.rs

**Total messages in file:** 24

| Line | Pattern Type | Message |
|------|--------------|---------|
| 37 | .expect() | workspace root |
| 99 | anyhow::anyhow! | Daemon failed to start within timeout |
| 106 | .expect() | Failed to spawn daemon |
| 124 | .expect() | Failed to create draft |
| 140 | .expect() | Failed to spawn daemon |
| 148 | .expect() | Failed to fetch drafts |
| 162 | .expect() | Failed to spawn daemon |
| 170 | .expect() | Failed to fetch audit log |
| 184 | .expect() | Failed to spawn daemon |
| 192 | .expect() | Failed to fetch beads |
| 194 | assert_* | Bead list endpoint should return 200 |
| 205 | .expect() | Failed to spawn daemon |
| 225 | .expect() | Failed to create draft |
| 228 | .expect() | Failed to parse draft |
| 248 | .expect() | Failed to spawn daemon |
| 256 | .expect() | Failed to fetch audit log |
| 259 | .expect() | Failed to parse audit |
| 279 | .expect() | Failed to spawn daemon |
| 298 | .expect() | Failed to create draft |
| 302 | .expect() | Failed to parse draft |
| 306 | .expect() | draft_id should be present |
| 320 | .expect() | Failed to list drafts |
| 323 | .expect() | Failed to parse list |
| 328 | assert! | Draft should appear in queue |

### tests/acceptance/s4_daemon_restart.rs

**Total messages in file:** 41

| Line | Pattern Type | Message |
|------|--------------|---------|
| 31 | .expect() | workspace root |
| 105 | .expect() | create temp dir |
| 107 | .expect() | create .hoop dir |
| 122 | .expect() | write projects.yaml |
| 131 | .expect() | write config.yml |
| 132 | .expect() | create data dir |
| 177 | anyhow::anyhow! | Daemon failed to start |
| 191 | .expect() | write claim |
| 192 | .expect() | write complete |
| 193 | .expect() | write claim |
| 197 | .expect() | Failed to spawn first daemon |
| 205 | .expect() | Failed to fetch beads from first daemon |
| 207 | assert_* | First daemon should return beads |
| 209 | .expect() | Failed to parse beads |
| 215 | .expect() | write complete |
| 216 | .expect() | write claim |
| 219 | assert! | Worker should have written events |
| 223 | .expect() | Failed to spawn second daemon |
| 229 | .expect() | Failed to fetch beads from second daemon |
| 231 | assert_* | Second daemon should return beads |
| 233 | .expect() | Failed to parse beads |
| 263 | .expect() | write claim |
| 265 | .expect() | write complete |
| 271 | .expect() | Failed to spawn first daemon |
| 277 | .expect() | Failed to spawn second daemon |
| 302 | .expect() | Failed to spawn first daemon |
| 307 | .expect() | write claim |
| 308 | .expect() | write complete |
| 309 | .expect() | write claim |
| 320 | .expect() | Failed to spawn second daemon |
| 322 | .expect() | write complete |
| 323 | .expect() | write claim |
| 337 | .expect() | Failed to fetch beads |
| 339 | assert_* | Should see all beads |
| 358 | .expect() | write claim |
| 359 | .expect() | write complete |
| 364 | .expect() | Failed to spawn daemon |
| 385 | .expect() | Failed to fetch beads |
| 387 | assert_* | Should fetch beads in cycle {} |
| 389 | .expect() | Failed to parse beads |
| 408 | .expect() | write claim |

### tests/acceptance/s5_workspace_deleted.rs

**Total messages in file:** 26

| Line | Pattern Type | Message |
|------|--------------|---------|
| 27 | .expect() | Failed to create .beads dir |
| 29 | .expect() | Failed to create issues.jsonl |
| 37 | .expect() | Failed to create temp dir |
| 39 | .expect() | Failed to create .hoop dir |
| 68 | .expect() | Failed to write projects.yaml |
| 77 | .expect() | Failed to write config.yml |
| 79 | .expect() | Failed to create data dir |
| 130 | anyhow::anyhow! | Daemon failed to start |
| 166 | .expect() | Failed to spawn daemon |
| 173 | .expect() | Failed to get readyz status |
| 175 | assert_* | Initial readyz should return 200 |
| 179 | .expect() | Failed to remove .beads from project A |
| 228 | .expect() | Failed to spawn daemon |
| 237 | .expect() | Failed to remove .beads from project A |
| 247 | .expect() | Failed to fetch projects |
| 249 | assert_* | Projects endpoint should still work |
| 251 | .expect() | Failed to parse projects |
| 263 | .expect() | Failed to check health |
| 265 | assert! | Daemon should still be healthy |
| 291 | .expect() | Failed to spawn daemon |
| 299 | .expect() | Failed to get readyz status |
| 304 | .expect() | Failed to remove .beads from project A |
| 311 | .expect() | Failed to get readyz status after deletion |
| 364 | .expect() | Failed to spawn daemon |
| 373 | .expect() | Failed to remove .beads |
| 382 | .expect() | Failed to check health |

### tests/acceptance/s6_machine_mode.rs

**Total messages in file:** 62

| Line | Pattern Type | Message |
|------|--------------|---------|
| 36 | expect | workspace root |
| 36 | .expect() | workspace root |
| 98 | anyhow | Daemon failed to start within timeout |
| 98 | anyhow::anyhow! | Daemon failed to start within timeout |
| 107 | expect | Failed to spawn daemon |
| 107 | .expect() | Failed to spawn daemon |
| 116 | expect | Failed to fetch status |
| 116 | .expect() | Failed to fetch status |
| 118 | assert | Status endpoint should return 200 |
| 118 | assert_* | Status endpoint should return 200 |
| 120 | expect | Failed to parse status |
| 120 | .expect() | Failed to parse status |
| 122 | assert | Status should be a JSON object |
| 122 | assert! | Status should be a JSON object |
| 131 | expect | Failed to spawn daemon |
| 131 | .expect() | Failed to spawn daemon |
| 139 | expect | Failed to fetch projects |
| 139 | .expect() | Failed to fetch projects |
| 141 | assert | Projects endpoint should return 200 |
| 141 | assert_* | Projects endpoint should return 200 |
| 152 | expect | Failed to spawn daemon |
| 152 | .expect() | Failed to spawn daemon |
| 160 | expect | Failed to fetch projects |
| 160 | .expect() | Failed to fetch projects |
| 164 | expect | Failed to parse projects |
| 164 | .expect() | Failed to parse projects |
| 166 | assert | Projects should be an array |
| 166 | assert! | Projects should be an array |
| 175 | expect | Failed to spawn daemon |
| 175 | .expect() | Failed to spawn daemon |
| 192 | expect | Failed to fetch endpoint |
| 192 | .expect() | Failed to fetch endpoint |
| 210 | expect | Failed to spawn daemon |
| 210 | .expect() | Failed to spawn daemon |
| 218 | expect | Failed to fetch projects |
| 218 | .expect() | Failed to fetch projects |
| 220 | expect | Failed to parse projects |
| 220 | .expect() | Failed to parse projects |
| 223 | assert | Should be parseable by jq |
| 223 | assert! | Should be parseable by jq |
| 227 | assert | Each project should be an object |
| 227 | assert! | Each project should be an object |
| 242 | expect | Failed to spawn daemon |
| 242 | .expect() | Failed to spawn daemon |
| 250 | expect | Failed to fetch healthz |
| 250 | .expect() | Failed to fetch healthz |
| 252 | assert | Healthz endpoint should return 200 |
| 252 | assert_* | Healthz endpoint should return 200 |
| 261 | expect | Failed to spawn daemon |
| 261 | .expect() | Failed to spawn daemon |
| 269 | expect | Failed to fetch readyz |
| 269 | .expect() | Failed to fetch readyz |
| 284 | expect | Failed to spawn daemon |
| 284 | .expect() | Failed to spawn daemon |
| 293 | expect | Failed to fetch bead |
| 293 | .expect() | Failed to fetch bead |
| 310 | expect | Failed to spawn daemon |
| 310 | .expect() | Failed to spawn daemon |
| 327 | expect | Task panicked |
| 327 | .expect() | Task panicked |
| 337 | assert | All concurrent requests should succeed |
| 337 | assert_* | All concurrent requests should succeed |

### tests/cli_test_helpers.rs

**Total messages in file:** 152

| Line | Pattern Type | Message |
|------|--------------|---------|
| 79 | assert | /tmp |
| 81 | panic | Expected Scan command |
| 81 | panic! | Expected Scan command |
| 130 | assert | /tmp |
| 132 | panic | Expected Scan command |
| 132 | panic! | Expected Scan command |
| 180 | assert | Both positions must yield the same value |
| 180 | assert_* | Both positions must yield the same value |
| 181 | assert | no_interactive should be true |
| 181 | assert_* | no_interactive should be true |
| 204 | assert | no_interactive value must be consistent |
| 204 | assert_* | no_interactive value must be consistent |
| 205 | assert | no_interactive should be true |
| 205 | assert_* | no_interactive should be true |
| 338 | assert | Flag must be true after extraction |
| 338 | assert! | Flag must be true after extraction |
| 356 | assert | pub fn remove_project(name: &str, no_interactive: bool) |
| 364 | assert | let no_interactive = cli.no_interactive; |
| 367 | assert | projects::remove_project(&name, no_interactive) |
| 390 | assert | CLI must parse flag as true |
| 390 | assert! | CLI must parse flag as true |
| 394 | assert | Extracted value must match CLI value |
| 394 | assert_* | Extracted value must match CLI value |
| 398 | expect | projects.rs must exist |
| 398 | .expect() | projects.rs must exist |
| 412 | expect | main.rs must exist |
| 412 | .expect() | main.rs must exist |
| 455 | assert | --no-interactive |
| 464 | assert | --no-interactive |
| 476 | assert | no_interactive |
| 498 | assert | Parent must have flag set |
| 498 | assert! | Parent must have flag set |
| 514 | assert | --no-interactive |
| 571 | assert | Top level must have flag |
| 571 | assert! | Top level must have flag |
| 577 | assert | Flag accessible at Projects level |
| 577 | assert! | Flag accessible at Projects level |
| 582 | assert | my-project |
| 583 | assert | Confirm flag must be true |
| 583 | assert! | Confirm flag must be true |
| 585 | assert | Flag accessible at Remove level |
| 585 | assert! | Flag accessible at Remove level |
| 587 | panic | Expected Remove command |
| 587 | panic! | Expected Remove command |
| 590 | panic | Expected Projects command |
| 590 | panic! | Expected Projects command |
| 615 | assert | non-interactive mode |
| 646 | assert | Level 0: Global flag must be true |
| 646 | assert! | Level 0: Global flag must be true |
| 652 | assert | Level 1: Flag accessible in Projects |
| 652 | assert! | Level 1: Flag accessible in Projects |
| 657 | assert | my-project |
| 658 | assert | Remove's --confirm flag must be true |
| 658 | assert! | Remove's --confirm flag must be true |
| 661 | assert | Level 2: Flag accessible in Remove |
| 661 | assert! | Level 2: Flag accessible in Remove |
| 663 | panic | Expected Remove command at Level 2 |
| 663 | panic! | Expected Remove command at Level 2 |
| 666 | panic | Expected Projects command at Level 1 |
| 666 | panic! | Expected Projects command at Level 1 |
| 711 | assert | HOOP_NO_INTERACTIVE |
| 720 | assert | HOOP_NO_INTERACTIVE |
| 727 | assert | HOOP_NO_INTERACTIVE |
| 738 | assert | 1 |
| 760 | assert | Flag must be parsed as true |
| 760 | assert! | Flag must be parsed as true |
| 784 | assert | HOOP_NO_INTERACTIVE |
| 790 | assert | Flag must be false when not specified |
| 790 | assert! | Flag must be false when not specified |
| 822 | assert | Flag must be true at top level |
| 822 | assert! | Flag must be true at top level |
| 827 | assert | Flag accessible at Projects level |
| 827 | assert! | Flag accessible at Projects level |
| 830 | panic | Expected Projects command |
| 830 | panic! | Expected Projects command |
| 836 | assert | Child must receive no_interactive flag |
| 836 | assert! | Child must receive no_interactive flag |
| 840 | assert | 1 |
| 840 | assert_* | Environment variable must be '1' |
| 844 | assert | no_interactive: bool |
| 917 | assert | --no-interactive |
| 926 | assert | --no-interactive |
| 938 | assert | no_interactive |
| 960 | assert | Parent must have flag set |
| 960 | assert! | Parent must have flag set |
| 975 | assert | --no-interactive |
| 995 | assert | Top level must have flag |
| 995 | assert! | Top level must have flag |
| 1001 | assert | Flag accessible at Projects level |
| 1001 | assert! | Flag accessible at Projects level |
| 1006 | assert | my-project |
| 1008 | assert | Flag accessible at Remove level |
| 1008 | assert! | Flag accessible at Remove level |
| 1010 | panic | Expected Remove command |
| 1010 | panic! | Expected Remove command |
| 1013 | panic | Expected Projects command |
| 1013 | panic! | Expected Projects command |
| 1038 | assert | non-interactive mode |
| 1069 | assert | Level 0: Global flag must be true |
| 1069 | assert! | Level 0: Global flag must be true |
| 1075 | assert | Level 1: Flag accessible in Projects |
| 1075 | assert! | Level 1: Flag accessible in Projects |
| 1080 | assert | my-project |
| 1081 | assert | Remove's --confirm flag must be true |
| 1081 | assert! | Remove's --confirm flag must be true |
| 1084 | assert | Level 2: Flag accessible in Remove |
| 1084 | assert! | Level 2: Flag accessible in Remove |
| 1086 | panic | Expected Remove command at Level 2 |
| 1086 | panic! | Expected Remove command at Level 2 |
| 1089 | panic | Expected Projects command at Level 1 |
| 1089 | panic! | Expected Projects command at Level 1 |
| 1119 | assert | HOOP_NO_INTERACTIVE |
| 1128 | assert | HOOP_NO_INTERACTIVE |
| 1135 | assert | HOOP_NO_INTERACTIVE |
| 1146 | assert | 1 |
| 1168 | assert | Flag must be parsed as true |
| 1168 | assert! | Flag must be parsed as true |
| 1192 | assert | HOOP_NO_INTERACTIVE |
| 1198 | assert | Flag must be false when not specified |
| 1198 | assert! | Flag must be false when not specified |
| 1230 | assert | Flag must be true at top level |
| 1230 | assert! | Flag must be true at top level |
| 1235 | assert | Flag accessible at Projects level |
| 1235 | assert! | Flag accessible at Projects level |
| 1238 | panic | Expected Projects command |
| 1238 | panic! | Expected Projects command |
| 1244 | assert | Child must receive no_interactive flag |
| 1244 | assert! | Child must receive no_interactive flag |
| 1248 | assert | 1 |
| 1248 | assert_* | Environment variable must be '1' |
| 1338 | assert | flag value must be position-independent |
| 1338 | assert_* | flag value must be position-independent |
| 1409 | assertion message | expected command: {} |
| 1779 | assert | no_interactive value must be consistent |
| 1779 | assert_* | no_interactive value must be consistent |
| 1780 | assert | no_interactive should be true |
| 1780 | assert_* | no_interactive should be true |
| 1837 | assert | no_interactive value must be position-independent |
| 1837 | assert_* | no_interactive value must be position-independent |
| 1838 | assert | no_interactive should be true |
| 1838 | assert_* | no_interactive should be true |
| 2178 | panic | Expected Scan command |
| 2178 | panic! | Expected Scan command |
| 2192 | panic | Expected Remove command |
| 2192 | panic! | Expected Remove command |
| 2206 | panic | Expected Projects subcommand |
| 2206 | panic! | Expected Projects subcommand |
| 2216 | assert | Values must match |
| 2216 | assert_* | Values must match |
| 2344 | assert | --no-interactive |
| 2368 | panic | Expected Projects command |
| 2368 | panic! | Expected Projects command |
