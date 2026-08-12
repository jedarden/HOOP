# HOOP Test Suite Error Message Catalog

Generated: Wed Aug 12 10:07:37 AM EDT 2026

Total files with errors: 129
Total error messages: 4821

## Summary by Error Type

- anyhow: 9 occurrences
- assert: 528 occurrences
- assert_eq: 474 occurrences
- assert_ne: 1 occurrences
- bail: 37 occurrences
- expect: 2064 occurrences
- panic: 94 occurrences
- should_panic: 17 occurrences
- unwrap: 1597 occurrences

## Detailed Error Messages by File

### hoop-cli/tests/clap_test_utils.rs

Total errors: 84

#### assert (10 occurrences)

- Line 477: `/tmp`
  ```rust
  /// assert!(verify_flag_propagation(&["scan", "/tmp"], true).is_ok());
  ```

- Line 480: `scan`
  ```rust
  /// assert!(verify_flag_propagation(&["projects", "scan", "/tmp"], true).is_ok());
  ```

- Line 984: `/tmp`
  ```rust
  assert!(verify_flag_at_position(&["scan", "/tmp"], "before", true).is_ok());
  ```

- Line 989: `/tmp`
  ```rust
  assert!(verify_flag_at_position(&["scan", "/tmp"], "after", true).is_ok());
  ```

- Line 994: `/tmp`
  ```rust
  assert!(verify_position_independence(&["scan", "/tmp"]).is_ok());
  ```

- Line 999: `/tmp`
  ```rust
  assert!(verify_flag_default_false(&["scan", "/tmp"]).is_ok());
  ```

- Line 1319: `/tmp`
  ```rust
  assert!(verify_flag_propagation(&["scan", "/tmp"], true).is_ok());
  ```

- Line 1325: `scan`
  ```rust
  assert!(verify_flag_propagation(&["projects", "scan", "/tmp"], true).is_ok());
  ```

- Line 1331: `test-project`
  ```rust
  assert!(verify_flag_propagation(&["remove", "test-project", "--confirm"], true).is_ok());
  ```

- Line 1384: `scan`
  ```rust
  assert!(verify_flag_propagation(&["projects", "scan", "/tmp"], no_interactive).is_ok());
  ```

#### assert_eq (1 occurrences)

- Line 1450: `Some tests failed: {:?}`
  ```rust
  //     assert_eq!(failures.len(), 0, "Some tests failed: {:?}", failures);
  ```

#### expect (8 occurrences)

- Line 681: `Should parse with flag before command`
  ```rust
  .expect("Should parse with flag before command");
  ```

- Line 703: `Should parse with flag after command`
  ```rust
  .expect("Should parse with flag after command");
  ```

- Line 725: `Should parse with -y flag`
  ```rust
  .expect("Should parse with -y flag");
  ```

- Line 771: `Should parse without flag`
  ```rust
  .expect("Should parse without flag");
  ```

- Line 803: `Should parse with flag before command`
  ```rust
  .expect("Should parse with flag before command");
  ```

- Line 811: `Should parse with flag after command`
  ```rust
  .expect("Should parse with flag after command");
  ```

- Line 819: `Should parse with -y flag`
  ```rust
  .expect("Should parse with -y flag");
  ```

- Line 840: `Should parse without flag`
  ```rust
  .expect("Should parse without flag");
  ```

#### panic (13 occurrences)

- Line 1039: `Expected Scan command`
  ```rust
  _ => panic!("Expected Scan command"),
  ```

- Line 1124: `Expected Scan command`
  ```rust
  _ => panic!("Expected Scan command"),
  ```

- Line 1159: `Expected Scan command`
  ```rust
  _ => panic!("Expected Scan command"),
  ```

- Line 1177: `Expected Scan command`
  ```rust
  _ => panic!("Expected Scan command"),
  ```

- Line 1191: `Expected Scan command`
  ```rust
  _ => panic!("Expected Scan command"),
  ```

- Line 1205: `Expected Remove command`
  ```rust
  _ => panic!("Expected Remove command"),
  ```

- Line 1218: `Expected Projects::Scan command`
  ```rust
  _ => panic!("Expected Projects::Scan command"),
  ```

- Line 1252: `Expected Scan command`
  ```rust
  _ => panic!("Expected Scan command"),
  ```

- Line 1266: `Expected Projects::Scan command`
  ```rust
  _ => panic!("Expected Projects::Scan command"),
  ```

- Line 1268: `Expected Projects command`
  ```rust
  _ => panic!("Expected Projects command"),
  ```

- Line 1472: `Expected Scan command`
  ```rust
  //         _ => panic!("Expected Scan command"),
  ```

- Line 1503: `Expected Scan command`
  ```rust
  //         _ => panic!("Expected Scan command"),
  ```

- Line 1512: `Expected Remove command`
  ```rust
  //         _ => panic!("Expected Remove command"),
  ```

#### unwrap (52 occurrences)

- Line 23: `\.unwrap\(\)`
  ```rust
  //!     let cli = parse_cli(&["hoop", "--no-interactive", "scan", "/tmp"]).unwrap();
  ```

- Line 27: `\.unwrap\(\)`
  ```rust
  //!     let cli = parse_cli(&["hoop", "scan", "/tmp", "--no-interactive"]).unwrap();
  ```

- Line 252: `\.unwrap\(\)`
  ```rust
  let cli_before = parse_cli(&full_args_before).unwrap();
  ```

- Line 262: `\.unwrap\(\)`
  ```rust
  let cli_after = parse_cli(&full_args_after).unwrap();
  ```

- Line 920: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "--no-interactive", "scan", "/tmp"]).unwrap();
  ```

- Line 926: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "scan", "/tmp", "--no-interactive"]).unwrap();
  ```

- Line 932: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "-y", "scan", "/tmp"]).unwrap();
  ```

- Line 938: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
  ```

- Line 972: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "--no-interactive", "scan", "/tmp"]).unwrap();
  ```

- Line 978: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
  ```

- Line 1006: `\.unwrap\(\)`
  ```rust
  let cli = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
  ```

- Line 1012: `\.unwrap\(\)`
  ```rust
  let cli = parse_flag_after_subcommand(&["scan", "/tmp"]).unwrap();
  ```

- Line 1018: `\.unwrap\(\)`
  ```rust
  let cli = parse_with_short_flag(&["scan", "/tmp"]).unwrap();
  ```

- Line 1033: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
  ```

- Line 1045: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
  ```

- Line 1048: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "remove", "test", "--confirm"]).unwrap();
  ```

- Line 1054: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "remove", "test", "--confirm"]).unwrap();
  ```

- Line 1057: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
  ```

- Line 1063: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "init"]).unwrap();
  ```

- Line 1066: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
  ```

- Line 1117: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 1135: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 1140: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 1152: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 1169: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 1183: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
  ```

- Line 1197: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "remove", "test-project", "--confirm"]).unwrap();
  ```

- Line 1211: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "projects", "scan", "/tmp"]).unwrap();
  ```

- Line 1228: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 1233: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 1244: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "--no-interactive", "scan", "/tmp", "--yes"]).unwrap();
  ```

- Line 1258: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "--no-interactive", "projects", "scan", "/tmp"]).unwrap();
  ```

- Line 1275: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "--no-interactive", "projects", "scan", "/tmp"]).unwrap();
  ```

- Line 1282: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
  ```

- Line 1290: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "--no-interactive", "scan", "/tmp"]).unwrap();
  ```

- Line 1297: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
  ```

- Line 1304: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "scan", "/tmp", "--no-interactive"]).unwrap();
  ```

- Line 1311: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "--no-interactive", "projects", "scan", "/tmp"]).unwrap();
  ```

- Line 1355: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 1360: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 1375: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli(&["hoop", "--no-interactive", "projects", "scan", "/tmp"]).unwrap();
  ```

- Line 1401: `\.unwrap\(\)`
  ```rust
  //     let cli = parse_flag_before_subcommand(&["my-command", "arg1"]).unwrap();
  ```

- Line 1405: `\.unwrap\(\)`
  ```rust
  //     let cli = parse_flag_after_subcommand(&["my-command", "arg1"]).unwrap();
  ```

- Line 1415: `\.unwrap\(\)`
  ```rust
  //     let cli = parse_cli(&["hoop", "my-command", "arg1"]).unwrap();
  ```

- Line 1461: `\.unwrap\(\)`
  ```rust
  //     let cli = parse_cli(&["hoop", "--no-interactive", "scan", "/tmp"]).unwrap();
  ```

- Line 1490: `\.unwrap\(\)`
  ```rust
  //     ).unwrap();
  ```

- Line 1495: `\.unwrap\(\)`
  ```rust
  //     ).unwrap();
  ```

- Line 1528: `\.unwrap\(\)`
  ```rust
  //     ).unwrap();
  ```

- Line 1533: `\.unwrap\(\)`
  ```rust
  //     ).unwrap();
  ```

- Line 1551: `\.unwrap\(\)`
  ```rust
  //     ).unwrap();
  ```

- Line 1558: `\.unwrap\(\)`
  ```rust
  //     ).unwrap();
  ```

- Line 1565: `\.unwrap\(\)`
  ```rust
  //     ).unwrap();
  ```

### hoop-cli/tests/cli_test_helpers.rs

Total errors: 104

#### assert (12 occurrences)

- Line 75: `/tmp`
  ```rust
  //! assert!(assert_flag_propagation(&["scan", "/tmp"]).is_ok());
  ```

- Line 127: `/tmp`
  ```rust
  //! assert!(verify_flag_position_consistency(&["scan", "/tmp"]).is_ok());
  ```

- Line 161: `/tmp`
  ```rust
  //! assert!(assert_flag_propagation(&["scan", "/tmp"]).is_ok());
  ```

- Line 281: `Parsing should succeed`
  ```rust
  //! assert!(result.is_ok(), "Parsing should succeed");
  ```

- Line 339: `Parsing should succeed even with flag at end`
  ```rust
  //! assert!(result.is_ok(), "Parsing should succeed even with flag at end");
  ```

- Line 410: `/tmp`
  ```rust
  //! assert!(verify_flag_position_consistency(&["scan", "/tmp"]).is_ok());
  ```

- Line 445: `/tmp`
  ```rust
  //!     assert!(verify_flag_position_consistency(&["scan", "/tmp"]).is_ok());
  ```

- Line 927: `/tmp`
  ```rust
  //!     assert!(assert_flag_propagation(&["scan", "/tmp"]).is_ok());
  ```

- Line 930: `/tmp`
  ```rust
  //!     assert!(verify_default_flag_value(&["scan", "/tmp"]).is_ok());
  ```

- Line 933: `/tmp`
  ```rust
  //!     assert!(verify_flag_position_consistency(&["scan", "/tmp"]).is_ok());
  ```

- Line 2108: `Failed to parse command without flag`
  ```rust
  assert!(parsed.is_ok(), "Failed to parse command without flag");
  ```

- Line 2919: `Empty args should error`
  ```rust
  assert!(empty_result.is_err(), "Empty args should error");
  ```

#### assert_eq (11 occurrences)

- Line 287: `Flag should be true`
  ```rust
  //! assert_eq!(parsed.no_interactive, true, "Flag should be true");
  ```

- Line 345: `Flag should be true at any position`
  ```rust
  //! assert_eq!(parsed.no_interactive, true, "Flag should be true at any position");
  ```

- Line 2552: `--no-interactive`
  ```rust
  assert_eq!(extract_flag_value(&["scan", "/tmp", "--no-interactive"]), true);
  ```

- Line 2553: `/tmp`
  ```rust
  assert_eq!(extract_flag_value(&["-y", "scan", "/tmp"]), true);
  ```

- Line 2554: `/tmp`
  ```rust
  assert_eq!(extract_flag_value(&["scan", "--no-interactive", "/tmp"]), true);
  ```

- Line 2559: `false);
`
  ```rust
  assert_eq!(extract_flag_value(&["scan", "/tmp"]), false);
  ```

- Line 2560: `false);
`
  ```rust
  assert_eq!(extract_flag_value(&["status", "--json"]), false);
  ```

- Line 2574: `Some(`
  ```rust
  assert_eq!(extract_subcommand(&["--no-interactive", "status"]), Some("status".to_string()));
  ```

- Line 2580: `None);
`
  ```rust
  assert_eq!(extract_subcommand(&["--json", "--verbose"]), None);
  ```

- Line 2903: `Direct extraction should work`
  ```rust
  assert_eq!(extracted, true, "Direct extraction should work");
  ```

- Line 2929: `
`
  ```rust
  assert_eq!(extract_flag_value(multi_flag), true,
  ```

#### expect (19 occurrences)

- Line 229: `Failed to read main.rs`
  ```rust
  //!             .expect("Failed to read main.rs");
  ```

- Line 753: `Failed to read mycommand.rs`
  ```rust
  //!             .expect("Failed to read mycommand.rs");
  ```

- Line 770: `Failed to read main.rs`
  ```rust
  //!             .expect("Failed to read main.rs");
  ```

- Line 804: `Failed to read projects.rs`
  ```rust
  //!         .expect("Failed to read projects.rs");
  ```

- Line 835: `Failed to read init.rs`
  ```rust
  //!         .expect("Failed to read init.rs");
  ```

- Line 876: `Failed to read main.rs`
  ```rust
  //!         .expect("Failed to read main.rs");
  ```

- Line 891: `Failed to read projects.rs`
  ```rust
  //!         .expect("Failed to read projects.rs");
  ```

- Line 2174: `Flag before subcommand assertion failed`
  ```rust
  .expect("Flag before subcommand assertion failed");
  ```

- Line 2195: `Flag after subcommand assertion failed`
  ```rust
  .expect("Flag after subcommand assertion failed");
  ```

- Line 2237: `Default flag assertion failed`
  ```rust
  .expect("Default flag assertion failed");
  ```

- Line 2830: `Should parse flag before subcommand`
  ```rust
  .expect("Should parse flag before subcommand");
  ```

- Line 2838: `Should parse flag after subcommand`
  ```rust
  .expect("Should parse flag after subcommand");
  ```

- Line 2846: `Should parse short flag`
  ```rust
  .expect("Should parse short flag");
  ```

- Line 2855: `Should parse nested command`
  ```rust
  .expect("Should parse nested command");
  ```

- Line 2866: `Should parse nested command with flag`
  ```rust
  .expect("Should parse nested command with flag");
  ```

- Line 2882: `Should parse command with multiple flags`
  ```rust
  .expect("Should parse command with multiple flags");
  ```

- Line 2890: `Should parse command without flag`
  ```rust
  .expect("Should parse command without flag");
  ```

- Line 2911: `Should parse successfully`
  ```rust
  .expect("Should parse successfully");
  ```

- Line 2923: `Should parse flag-only args`
  ```rust
  .expect("Should parse flag-only args");
  ```

#### unwrap (62 occurrences)

- Line 73: `\.unwrap\(\)`
  ```rust
  //! let parsed = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
  ```

- Line 156: `\.unwrap\(\)`
  ```rust
  //! let parsed = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
  ```

- Line 284: `\.unwrap\(\)`
  ```rust
  //! let parsed = result.unwrap();
  ```

- Line 315: `\.unwrap\(\)`
  ```rust
  //!     let parsed = result.unwrap();
  ```

- Line 342: `\.unwrap\(\)`
  ```rust
  //! let parsed = result.unwrap();
  ```

- Line 374: `\.unwrap\(\)`
  ```rust
  //!     let parsed = result.unwrap();
  ```

- Line 399: `\.unwrap\(\)`
  ```rust
  //! let before = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]).unwrap();
  ```

- Line 400: `\.unwrap\(\)`
  ```rust
  //! let after = parse_flag_after_subcommand(&["scan", "/tmp", "--no-interactive"]).unwrap();
  ```

- Line 435: `\.unwrap\(\)`
  ```rust
  //!     let before = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]).unwrap();
  ```

- Line 440: `\.unwrap\(\)`
  ```rust
  //!     let after = parse_flag_after_subcommand(&["scan", "/tmp", "--no-interactive"]).unwrap();
  ```

- Line 595: `\.unwrap\(\)`
  ```rust
  //!     assert_eq!(parsed_before.unwrap().no_interactive, true);
  ```

- Line 600: `\.unwrap\(\)`
  ```rust
  //!     assert_eq!(parsed_after.unwrap().no_interactive, true);
  ```

- Line 604: `\.unwrap\(\)`
  ```rust
  //!         parsed_before.unwrap().no_interactive,
  ```

- Line 605: `\.unwrap\(\)`
  ```rust
  //!         parsed_after.unwrap().no_interactive
  ```

- Line 915: `\.unwrap\(\)`
  ```rust
  //!     let parsed = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
  ```

- Line 1064: `\.unwrap\(\)`
  ```rust
  /// let parsed = result.unwrap();
  ```

- Line 1071: `\.unwrap\(\)`
  ```rust
  /// let parsed = result.unwrap();
  ```

- Line 1079: `\.unwrap\(\)`
  ```rust
  /// let parsed = result.unwrap();
  ```

- Line 1159: `\.unwrap\(\)`
  ```rust
  /// let parsed = result.unwrap();
  ```

- Line 1166: `\.unwrap\(\)`
  ```rust
  /// let parsed = result.unwrap();
  ```

- Line 1174: `\.unwrap\(\)`
  ```rust
  /// let parsed = result.unwrap();
  ```

- Line 1253: `\.unwrap\(\)`
  ```rust
  /// let parsed = result.unwrap();
  ```

- Line 1261: `\.unwrap\(\)`
  ```rust
  /// let parsed = result.unwrap();
  ```

- Line 1269: `\.unwrap\(\)`
  ```rust
  /// let parsed = result.unwrap();
  ```

- Line 1485: `\.unwrap\(\)`
  ```rust
  /// let result = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]).unwrap();
  ```

- Line 1516: `\.unwrap\(\)`
  ```rust
  /// let result = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
  ```

- Line 1548: `\.unwrap\(\)`
  ```rust
  /// let result = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]).unwrap();
  ```

- Line 1551: `\.unwrap\(\)`
  ```rust
  /// let result = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
  ```

- Line 1976: `\.unwrap\(\)`
  ```rust
  let before_parsed = before.unwrap();
  ```

- Line 1995: `\.unwrap\(\)`
  ```rust
  let after_parsed = after.unwrap();
  ```

- Line 2057: `\.unwrap\(\)`
  ```rust
  let result = parsed.unwrap();
  ```

- Line 2079: `\.unwrap\(\)`
  ```rust
  assert!(parsed_before.unwrap().no_interactive);
  ```

- Line 2088: `\.unwrap\(\)`
  ```rust
  assert!(parsed_after.unwrap().no_interactive);
  ```

- Line 2110: `\.unwrap\(\)`
  ```rust
  let result = parsed.unwrap();
  ```

- Line 2167: `\.unwrap\(\)`
  ```rust
  let before_result = parsed_before.unwrap();
  ```

- Line 2188: `\.unwrap\(\)`
  ```rust
  let after_result = parsed_after.unwrap();
  ```

- Line 2230: `\.unwrap\(\)`
  ```rust
  let default_result = parsed_default.unwrap();
  ```

- Line 2292: `\.unwrap\(\)`
  ```rust
  let valid_result = parsed_valid.unwrap();
  ```

- Line 2312: `\.unwrap\(\)`
  ```rust
  let invalid_result = parsed_invalid.unwrap();
  ```

- Line 2408: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 2426: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 2437: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 2447: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 2457: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 2474: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 2485: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 2495: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 2511: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 2522: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 2533: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 2544: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 2625: `\.unwrap\(\)`
  ```rust
  let result = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]).unwrap();
  ```

- Line 2642: `\.unwrap\(\)`
  ```rust
  let result = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]).unwrap();
  ```

- Line 2648: `\.unwrap\(\)`
  ```rust
  let result = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
  ```

- Line 2656: `\.unwrap\(\)`
  ```rust
  let result = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
  ```

- Line 2662: `\.unwrap\(\)`
  ```rust
  let result = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]).unwrap();
  ```

- Line 2670: `\.unwrap\(\)`
  ```rust
  let result = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]).unwrap();
  ```

- Line 2676: `\.unwrap\(\)`
  ```rust
  let result = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
  ```

- Line 2682: `\.unwrap\(\)`
  ```rust
  let result = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]).unwrap();
  ```

- Line 2765: `\.unwrap\(\)`
  ```rust
  let result = parse_flag_before_subcommand(&["-y", "scan", "/tmp"]).unwrap();
  ```

- Line 2789: `\.unwrap\(\)`
  ```rust
  let parsed = parse_flag_after_subcommand(args).unwrap();
  ```

- Line 2805: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

### hoop-cli/tests/cli_test_utils.rs

Total errors: 69

#### assert (14 occurrences)

- Line 504: `Failed to parse args: {:?}`
  ```rust
  assert!(result.is_ok(), "Failed to parse args: {:?}", full_args);
  ```

- Line 532: `Failed to parse args: {:?}`
  ```rust
  assert!(result.is_ok(), "Failed to parse args: {:?}", full_args);
  ```

- Line 557: `Failed to parse args: {:?}`
  ```rust
  assert!(result.is_ok(), "Failed to parse args: {:?}", full_args);
  ```

- Line 629: `Failed to parse args: {:?}`
  ```rust
  assert!(result.is_ok(), "Failed to parse args: {:?}", full_args);
  ```

- Line 671: `Failed to parse with flag before command`
  ```rust
  assert!(result_before.is_ok(), "Failed to parse with flag before command");
  ```

- Line 682: `Failed to parse with flag after command`
  ```rust
  assert!(result_after.is_ok(), "Failed to parse with flag after command");
  ```

- Line 693: `Failed to parse with -y flag`
  ```rust
  assert!(result_short.is_ok(), "Failed to parse with -y flag");
  ```

- Line 711: `Failed to parse without flag`
  ```rust
  assert!(result_default.is_ok(), "Failed to parse without flag");
  ```

- Line 776: `before`
  ```rust
  assert!(verify_flag_extraction(&parsed_before, "before").is_ok());
  ```

- Line 777: `after`
  ```rust
  assert!(verify_flag_extraction(&parsed_after, "after").is_ok());
  ```

- Line 929: `before`
  ```rust
  assert!(verify_flag_extraction(&parsed_before, "before").is_ok());
  ```

- Line 936: `after`
  ```rust
  assert!(verify_flag_extraction(&parsed_after, "after").is_ok());
  ```

- Line 1084: `before`
  ```rust
  assert!(verify_flag_extraction(&parsed, "before").is_ok());
  ```

- Line 1090: `after`
  ```rust
  assert!(verify_flag_extraction(&parsed, "after").is_ok());
  ```

#### assert_eq (12 occurrences)

- Line 506: `no_interactive should be true`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true");
  ```

- Line 534: `no_interactive should be true`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true");
  ```

- Line 559: `no_interactive should be true with -y`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true with -y");
  ```

- Line 673: `no_interactive should be true before command`
  ```rust
  assert_eq!(parsed_before.no_interactive, true, "no_interactive should be true before command");
  ```

- Line 684: `no_interactive should be true after command`
  ```rust
  assert_eq!(parsed_after.no_interactive, true, "no_interactive should be true after command");
  ```

- Line 695: `no_interactive should be true with -y`
  ```rust
  assert_eq!(parsed_short.no_interactive, true, "no_interactive should be true with -y");
  ```

- Line 880: `All 4 test cases should succeed`
  ```rust
  assert_eq!(successes.len(), 4, "All 4 test cases should succeed");
  ```

- Line 881: `No test cases should fail`
  ```rust
  assert_eq!(failures.len(), 0, "No test cases should fail");
  ```

- Line 1025: `/tmp`
  ```rust
  assert_eq!(parsed.args, vec!["scan", "/tmp"]);
  ```

- Line 1036: `/tmp`
  ```rust
  assert_eq!(parsed.args, vec!["scan", "/tmp"]);
  ```

- Line 1047: `/tmp`
  ```rust
  assert_eq!(parsed.args, vec!["scan", "/tmp"]);
  ```

- Line 1058: `/tmp`
  ```rust
  assert_eq!(parsed.args, vec!["scan", "/tmp"]);
  ```

#### expect (17 occurrences)

- Line 407: `Failed to create .beads/ directory`
  ```rust
  .expect("Failed to create .beads/ directory");
  ```

- Line 414: `Failed to create .hoop/ directory`
  ```rust
  std::fs::create_dir_all(&hoop_dir).expect("Failed to create .hoop/ directory");
  ```

- Line 423: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 586: `Failed to parse with flag before command`
  ```rust
  .expect("Failed to parse with flag before command");
  ```

- Line 595: `Failed to parse with flag after command`
  ```rust
  .expect("Failed to parse with flag after command");
  ```

- Line 757: `Failed to parse with flag before subcommand`
  ```rust
  .expect("Failed to parse with flag before subcommand");
  ```

- Line 764: `Failed to parse with flag after subcommand`
  ```rust
  .expect("Failed to parse with flag after subcommand");
  ```

- Line 782: `Failed to parse with -y flag`
  ```rust
  .expect("Failed to parse with -y flag");
  ```

- Line 788: `Failed to parse without flag`
  ```rust
  .expect("Failed to parse without flag");
  ```

- Line 800: `Failed to parse with flag before subcommand`
  ```rust
  .expect("Failed to parse with flag before subcommand");
  ```

- Line 805: `Failed to parse with flag after subcommand`
  ```rust
  .expect("Failed to parse with flag after subcommand");
  ```

- Line 890: `Failed to create temp dir`
  ```rust
  let tmp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 917: `Failed to create temp dir`
  ```rust
  let tmp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 926: `Failed to parse remove with flag before`
  ```rust
  .expect("Failed to parse remove with flag before");
  ```

- Line 933: `Failed to parse remove with flag after`
  ```rust
  .expect("Failed to parse remove with flag after");
  ```

- Line 1139: `Failed to create temp dir`
  ```rust
  let tmp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 1148: `Failed to create temp dir`
  ```rust
  let tmp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

#### unwrap (26 occurrences)

- Line 86: `\.unwrap\(\)`
  ```rust
  //!     let cli = result.unwrap();
  ```

- Line 144: `\.unwrap\(\)`
  ```rust
  /// assert_eq!(result.unwrap().no_interactive, true);
  ```

- Line 198: `\.unwrap\(\)`
  ```rust
  /// assert_eq!(result.unwrap().no_interactive, true);
  ```

- Line 213: `\.unwrap\(\)`
  ```rust
  /// assert_eq!(result.unwrap().no_interactive, true);
  ```

- Line 505: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 533: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 558: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 630: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 672: `\.unwrap\(\)`
  ```rust
  let parsed_before = result_before.unwrap();
  ```

- Line 683: `\.unwrap\(\)`
  ```rust
  let parsed_after = result_after.unwrap();
  ```

- Line 694: `\.unwrap\(\)`
  ```rust
  let parsed_short = result_short.unwrap();
  ```

- Line 712: `\.unwrap\(\)`
  ```rust
  let parsed_default = result_default.unwrap();
  ```

- Line 905: `\.unwrap\(\)`
  ```rust
  assert!(registry_path.file_name().unwrap() == "projects.yaml");
  ```

- Line 981: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 992: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 1000: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 1022: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 1033: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 1044: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 1055: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 1066: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 1076: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 1083: `\.unwrap\(\)`
  ```rust
  let parsed = parse_flag_before_subcommand(&["remove", "test", "--confirm"]).unwrap();
  ```

- Line 1089: `\.unwrap\(\)`
  ```rust
  let parsed = parse_flag_after_subcommand(&["remove", "test", "--confirm"]).unwrap();
  ```

- Line 1095: `\.unwrap\(\)`
  ```rust
  let parsed = parse_cli_with_flag(&["hoop", "scan", "/tmp"]).unwrap();
  ```

- Line 1152: `\.unwrap\(\)`
  ```rust
  assert!(registry_path.parent().unwrap().ends_with(".hoop"));
  ```

### hoop-cli/tests/cli_test_utils_examples.rs

Total errors: 42

#### assert (17 occurrences)

- Line 89: `Verification should succeed: {:?}`
  ```rust
  assert!(verification.is_ok(), "Verification should succeed: {:?}", verification);
  ```

- Line 99: `Verification should succeed: {:?}`
  ```rust
  assert!(verification.is_ok(), "Verification should succeed: {:?}", verification);
  ```

- Line 108: `Should verify no flag is present: {:?}`
  ```rust
  assert!(verification.is_ok(), "Should verify no flag is present: {:?}", verification);
  ```

- Line 134: `Prompt should be suppressed: {:?}`
  ```rust
  assert!(verification.is_ok(), "Prompt should be suppressed: {:?}", verification);
  ```

- Line 169: `Should pass with --confirm flag`
  ```rust
  assert!(verification.is_ok(), "Should pass with --confirm flag");
  ```

- Line 249: `Workspace directory should exist`
  ```rust
  assert!(workspace.exists(), "Workspace directory should exist");
  ```

- Line 262: `Registry file should exist`
  ```rust
  assert!(registry_path.exists(), "Registry file should exist");
  ```

- Line 271: `Registry should have empty projects list`
  ```rust
  assert!(content.contains("projects: []"), "Registry should have empty projects list");
  ```

- Line 285: `Should parse scan command successfully`
  ```rust
  assert!(result.is_ok(), "Should parse scan command successfully");
  ```

- Line 323: `Should succeed with --confirm flag`
  ```rust
  assert!(verification.is_ok(), "Should succeed with --confirm flag");
  ```

- Line 386: `All complex multi-command tests should pass`
  ```rust
  assert!(all_passed, "All complex multi-command tests should pass");
  ```

- Line 395: `Should fail with empty args`
  ```rust
  assert!(result.is_err(), "Should fail with empty args");
  ```

- Line 398: `Should have descriptive error message`
  ```rust
  assert!(err.contains("No arguments provided"), "Should have descriptive error message");
  ```

- Line 408: `Should fail with invalid expected_position`
  ```rust
  assert!(result.is_err(), "Should fail with invalid expected_position");
  ```

- Line 421: `Should require --confirm flag`
  ```rust
  assert!(result.is_err(), "Should require --confirm flag");
  ```

- Line 450: `before`
  ```rust
  assert!(verify_flag_extraction(&parsed_before, "before").is_ok());
  ```

- Line 451: `after`
  ```rust
  assert!(verify_flag_extraction(&parsed_after, "after").is_ok());
  ```

#### assert_eq (2 occurrences)

- Line 235: `All test cases should succeed`
  ```rust
  assert_eq!(successes.len(), 5, "All test cases should succeed");
  ```

- Line 236: `No test cases should fail`
  ```rust
  assert_eq!(failures.len(), 0, "No test cases should fail");
  ```

#### expect (9 occurrences)

- Line 246: `Failed to create temp dir`
  ```rust
  let tmp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 259: `Failed to create temp dir`
  ```rust
  let tmp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 270: `Failed to read registry file`
  ```rust
  .expect("Failed to read registry file");
  ```

- Line 279: `Failed to create temp dir`
  ```rust
  let tmp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 313: `Should parse remove command successfully`
  ```rust
  .expect("Should parse remove command successfully");
  ```

- Line 405: `Should parse successfully`
  ```rust
  .expect("Should parse successfully");
  ```

- Line 433: `Failed to create temp dir`
  ```rust
  let tmp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 440: `Parse with flag before should succeed`
  ```rust
  .expect("Parse with flag before should succeed");
  ```

- Line 446: `Parse with flag after should succeed`
  ```rust
  .expect("Parse with flag after should succeed");
  ```

#### unwrap (14 occurrences)

- Line 19: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 31: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 50: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 65: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 76: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 86: `\.unwrap\(\)`
  ```rust
  let parsed = parse_flag_before_subcommand(&["remove", "test", "--confirm"]).unwrap();
  ```

- Line 96: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 105: `\.unwrap\(\)`
  ```rust
  let parsed = parse_cli_with_flag(&["hoop", "scan", "/tmp"]).unwrap();
  ```

- Line 114: `\.unwrap\(\)`
  ```rust
  let parsed = parse_cli_with_flag(&["hoop", "-y", "scan", "/tmp"]).unwrap();
  ```

- Line 264: `\.unwrap\(\)`
  ```rust
  registry_path.parent().unwrap().ends_with(".hoop"),
  ```

- Line 284: `\.unwrap\(\)`
  ```rust
  let result = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", tmp_dir.path().to_str().unwrap()]);
  ```

- Line 287: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 439: `\.unwrap\(\)`
  ```rust
  parse_flag_before_subcommand(&["scan", tmp_dir.path().to_str().unwrap()])
  ```

- Line 445: `\.unwrap\(\)`
  ```rust
  parse_flag_after_subcommand(&["scan", tmp_dir.path().to_str().unwrap()])
  ```

### hoop-cli/tests/init_no_interactive_flag.rs

Total errors: 44

#### assert (11 occurrences)

- Line 24: `Should successfully parse flag before subcommand`
  ```rust
  assert!(result.is_ok(), "Should successfully parse flag before subcommand");
  ```

- Line 40: `Should successfully parse flag after subcommand`
  ```rust
  assert!(result.is_ok(), "Should successfully parse flag after subcommand");
  ```

- Line 56: `Should successfully parse short flag before subcommand`
  ```rust
  assert!(result.is_ok(), "Should successfully parse short flag before subcommand");
  ```

- Line 68: `Should successfully parse short flag after subcommand`
  ```rust
  assert!(result.is_ok(), "Should successfully parse short flag after subcommand");
  ```

- Line 80: `Should successfully parse command without flag`
  ```rust
  assert!(result.is_ok(), "Should successfully parse command without flag");
  ```

- Line 95: `Flag extraction should verify for `
  ```rust
  assert!(verification_result.is_ok(), "Flag extraction should verify for 'before' position");
  ```

- Line 108: `Flag extraction should verify for `
  ```rust
  assert!(verification_result.is_ok(), "Flag extraction should verify for 'after' position");
  ```

- Line 121: `Should verify no flag is present`
  ```rust
  assert!(verification_result.is_ok(), "Should verify no flag is present");
  ```

- Line 381: `Should parse flag before command`
  ```rust
  assert!(before.is_ok(), "Should parse flag before command");
  ```

- Line 386: `Should parse flag after command`
  ```rust
  assert!(after.is_ok(), "Should parse flag after command");
  ```

- Line 461: `All Init command no_interactive tests verified`
  ```rust
  assert!(true, "All Init command no_interactive tests verified");
  ```

#### assert_eq (10 occurrences)

- Line 27: `no_interactive should be true`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true");
  ```

- Line 28: `Command should be `
  ```rust
  assert_eq!(parsed.command, "init", "Command should be 'init'");
  ```

- Line 43: `no_interactive should be true`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true");
  ```

- Line 44: `Command should be `
  ```rust
  assert_eq!(parsed.command, "init", "Command should be 'init'");
  ```

- Line 59: `no_interactive should be true with -y`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true with -y");
  ```

- Line 60: `Command should be `
  ```rust
  assert_eq!(parsed.command, "init", "Command should be 'init'");
  ```

- Line 71: `no_interactive should be true with -y`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true with -y");
  ```

- Line 72: `Command should be `
  ```rust
  assert_eq!(parsed.command, "init", "Command should be 'init'");
  ```

- Line 83: `no_interactive should default to false`
  ```rust
  assert_eq!(parsed.no_interactive, false, "no_interactive should default to false");
  ```

- Line 84: `Command should be `
  ```rust
  assert_eq!(parsed.command, "init", "Command should be 'init'");
  ```

#### expect (12 occurrences)

- Line 92: `Parse should succeed`
  ```rust
  let parsed = parse_flag_before_subcommand(&["init"]).expect("Parse should succeed");
  ```

- Line 105: `Parse should succeed`
  ```rust
  let parsed = parse_flag_after_subcommand(&["init"]).expect("Parse should succeed");
  ```

- Line 118: `Parse should succeed`
  ```rust
  let parsed = parse_cli_with_flag(&["hoop", "init"]).expect("Parse should succeed");
  ```

- Line 134: `Failed to read main.rs`
  ```rust
  .expect("Failed to read main.rs");
  ```

- Line 159: `Failed to read init.rs`
  ```rust
  .expect("Failed to read init.rs");
  ```

- Line 182: `Failed to read init.rs`
  ```rust
  .expect("Failed to read init.rs");
  ```

- Line 219: `Failed to read init.rs`
  ```rust
  .expect("Failed to read init.rs");
  ```

- Line 316: `Failed to read init.rs`
  ```rust
  .expect("Failed to read init.rs");
  ```

- Line 353: `Failed to read init.rs`
  ```rust
  .expect("Failed to read init.rs");
  ```

- Line 358: `Should find no_interactive check`
  ```rust
  .expect("Should find no_interactive check");
  ```

- Line 364: `Should find exit(2) in no_interactive section`
  ```rust
  .expect("Should find exit(2) in no_interactive section");
  ```

- Line 417: `Failed to read init.rs`
  ```rust
  .expect("Failed to read init.rs");
  ```

#### unwrap (11 occurrences)

- Line 25: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 41: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 57: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 69: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 81: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 241: `\.unwrap\(\)`
  ```rust
  no_interactive_check.unwrap() < banner_print.unwrap(),
  ```

- Line 241: `\.unwrap\(\)`
  ```rust
  no_interactive_check.unwrap() < banner_print.unwrap(),
  ```

- Line 245: `\.unwrap\(\)`
  ```rust
  banner_print.unwrap() < stage_1.unwrap(),
  ```

- Line 245: `\.unwrap\(\)`
  ```rust
  banner_print.unwrap() < stage_1.unwrap(),
  ```

- Line 382: `\.unwrap\(\)`
  ```rust
  let before_parsed = before.unwrap();
  ```

- Line 387: `\.unwrap\(\)`
  ```rust
  let after_parsed = after.unwrap();
  ```

### hoop-cli/tests/no_interactive_flag_behavior.rs

Total errors: 99

#### assert (30 occurrences)

- Line 50: `Test workspace should have .beads/`
  ```rust
  assert!(workspace.join(".beads").exists(), "Test workspace should have .beads/");
  ```

- Line 67: `Interactive scan requires prompts (verified by code review)`
  ```rust
  assert!(true, "Interactive scan requires prompts (verified by code review)");
  ```

- Line 78: `Scan combines no_interactive || yes correctly`
  ```rust
  assert!(true, "Scan combines no_interactive || yes correctly");
  ```

- Line 115: `Should successfully parse flag before subcommand`
  ```rust
  assert!(result.is_ok(), "Should successfully parse flag before subcommand");
  ```

- Line 120: `Should include `
  ```rust
  assert!(parsed.args.contains(&"remove".to_string()), "Should include 'remove' in args");
  ```

- Line 121: `Should include project name`
  ```rust
  assert!(parsed.args.contains(&"my-project".to_string()), "Should include project name");
  ```

- Line 131: `Should successfully parse flag after subcommand`
  ```rust
  assert!(result.is_ok(), "Should successfully parse flag after subcommand");
  ```

- Line 136: `Should include `
  ```rust
  assert!(parsed.args.contains(&"remove".to_string()), "Should include 'remove' in args");
  ```

- Line 137: `Should include project name`
  ```rust
  assert!(parsed.args.contains(&"my-project".to_string()), "Should include project name");
  ```

- Line 185: `Should have confirm requirement check`
  ```rust
  assert!(confirm_check.is_some(), "Should have confirm requirement check");
  ```

- Line 189: `Should have prompt suppression check`
  ```rust
  assert!(prompt_check.is_some(), "Should have prompt suppression check");
  ```

- Line 261: `Should successfully parse short flag variant`
  ```rust
  assert!(result.is_ok(), "Should successfully parse short flag variant");
  ```

- Line 350: `Should parse flag before command`
  ```rust
  assert!(before.is_ok(), "Should parse flag before command");
  ```

- Line 355: `Should parse flag after command`
  ```rust
  assert!(after.is_ok(), "Should parse flag after command");
  ```

- Line 378: `Should successfully parse command without flag`
  ```rust
  assert!(result.is_ok(), "Should successfully parse command without flag");
  ```

- Line 516: `Should successfully parse flag before subcommand`
  ```rust
  assert!(result.is_ok(), "Should successfully parse flag before subcommand");
  ```

- Line 521: `Should include `
  ```rust
  assert!(parsed.args.contains(&"restore".to_string()), "Should include 'restore' in args");
  ```

- Line 522: `Should include --from flag`
  ```rust
  assert!(parsed.args.contains(&"--from".to_string()), "Should include --from flag");
  ```

- Line 523: `Should include URI`
  ```rust
  assert!(parsed.args.contains(&"s3://my-bucket/backups/snap-001".to_string()), "Should include URI");
  ```

- Line 533: `Should successfully parse flag after subcommand`
  ```rust
  assert!(result.is_ok(), "Should successfully parse flag after subcommand");
  ```

- Line 538: `Should include `
  ```rust
  assert!(parsed.args.contains(&"restore".to_string()), "Should include 'restore' in args");
  ```

- Line 539: `Should include --from flag`
  ```rust
  assert!(parsed.args.contains(&"--from".to_string()), "Should include --from flag");
  ```

- Line 540: `Should include URI`
  ```rust
  assert!(parsed.args.contains(&"s3://my-bucket/backups/snap-001".to_string()), "Should include URI");
  ```

- Line 588: `Should have confirm requirement check`
  ```rust
  assert!(confirm_check.is_some(), "Should have confirm requirement check");
  ```

- Line 592: `Should have prompt suppression check`
  ```rust
  assert!(prompt_check.is_some(), "Should have prompt suppression check");
  ```

- Line 671: `Should successfully parse short flag variant`
  ```rust
  assert!(result.is_ok(), "Should successfully parse short flag variant");
  ```

- Line 675: `Should include --confirm flag`
  ```rust
  assert!(parsed.args.contains(&"--confirm".to_string()), "Should include --confirm flag");
  ```

- Line 767: `Should parse flag before command`
  ```rust
  assert!(before.is_ok(), "Should parse flag before command");
  ```

- Line 778: `Should parse flag after command`
  ```rust
  assert!(after.is_ok(), "Should parse flag after command");
  ```

- Line 806: `Should successfully parse command without flag`
  ```rust
  assert!(result.is_ok(), "Should successfully parse command without flag");
  ```

#### assert_eq (10 occurrences)

- Line 118: `Flag should be extracted as true`
  ```rust
  assert_eq!(parsed.no_interactive, true, "Flag should be extracted as true");
  ```

- Line 119: `Should identify `
  ```rust
  assert_eq!(parsed.command, "projects", "Should identify 'projects' as command");
  ```

- Line 134: `Flag should be extracted as true`
  ```rust
  assert_eq!(parsed.no_interactive, true, "Flag should be extracted as true");
  ```

- Line 135: `Should identify `
  ```rust
  assert_eq!(parsed.command, "projects", "Should identify 'projects' as command");
  ```

- Line 264: `Short flag -y should set no_interactive to true`
  ```rust
  assert_eq!(parsed.no_interactive, true, "Short flag -y should set no_interactive to true");
  ```

- Line 519: `Flag should be extracted as true`
  ```rust
  assert_eq!(parsed.no_interactive, true, "Flag should be extracted as true");
  ```

- Line 520: `Should identify `
  ```rust
  assert_eq!(parsed.command, "restore", "Should identify 'restore' as command");
  ```

- Line 536: `Flag should be extracted as true`
  ```rust
  assert_eq!(parsed.no_interactive, true, "Flag should be extracted as true");
  ```

- Line 537: `Should identify `
  ```rust
  assert_eq!(parsed.command, "restore", "Should identify 'restore' as command");
  ```

- Line 674: `Short flag -y should set no_interactive to true`
  ```rust
  assert_eq!(parsed.no_interactive, true, "Short flag -y should set no_interactive to true");
  ```

#### expect (37 occurrences)

- Line 24: `Failed to create .beads/`
  ```rust
  fs::create_dir_all(workspace.join(".beads")).expect("Failed to create .beads/");
  ```

- Line 31: `Failed to create .hoop/`
  ```rust
  fs::create_dir_all(&hoop_dir).expect("Failed to create .hoop/");
  ```

- Line 33: `Failed to write registry`
  ```rust
  fs::write(&registry_path, "projects: []").expect("Failed to write registry");
  ```

- Line 42: `Failed to create temp dir`
  ```rust
  let tmp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 60: `Failed to create temp dir`
  ```rust
  let tmp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 94: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 146: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 167: `Failed to read main.rs`
  ```rust
  .expect("Failed to read main.rs");
  ```

- Line 181: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 212: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 271: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 392: `Failed to read main.rs`
  ```rust
  .expect("Failed to read main.rs");
  ```

- Line 413: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 432: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 458: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 476: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 494: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 549: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 570: `Failed to read main.rs`
  ```rust
  .expect("Failed to read main.rs");
  ```

- Line 584: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 615: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 682: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 820: `Failed to read main.rs`
  ```rust
  .expect("Failed to read main.rs");
  ```

- Line 840: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 866: `Failed to read init.rs`
  ```rust
  .expect("Failed to read init.rs");
  ```

- Line 889: `Failed to read init.rs`
  ```rust
  .expect("Failed to read init.rs");
  ```

- Line 913: `Failed to read main.rs`
  ```rust
  .expect("Failed to read main.rs");
  ```

- Line 950: `Failed to read main.rs`
  ```rust
  .expect("Failed to read main.rs");
  ```

- Line 963: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 982: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 1001: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 1020: `Failed to read init.rs`
  ```rust
  .expect("Failed to read init.rs");
  ```

- Line 1043: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 1046: `Should find scan_projects function`
  ```rust
  let scan_start = code.find("pub fn scan_projects").expect("Should find scan_projects function");
  ```

- Line 1084: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 1086: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 1107: `Failed to read init.rs`
  ```rust
  .expect("Failed to read init.rs");
  ```

#### unwrap (22 occurrences)

- Line 116: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 132: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 194: `\.unwrap\(\)`
  ```rust
  confirm_check.unwrap() < prompt_check.unwrap(),
  ```

- Line 194: `\.unwrap\(\)`
  ```rust
  confirm_check.unwrap() < prompt_check.unwrap(),
  ```

- Line 262: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 351: `\.unwrap\(\)`
  ```rust
  let before_parsed = before.unwrap();
  ```

- Line 356: `\.unwrap\(\)`
  ```rust
  let after_parsed = after.unwrap();
  ```

- Line 379: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 445: `\.unwrap\(\)`
  ```rust
  confirm_check.unwrap() < prompt_check.unwrap(),
  ```

- Line 445: `\.unwrap\(\)`
  ```rust
  confirm_check.unwrap() < prompt_check.unwrap(),
  ```

- Line 517: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 534: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 597: `\.unwrap\(\)`
  ```rust
  confirm_check.unwrap() < prompt_check.unwrap(),
  ```

- Line 597: `\.unwrap\(\)`
  ```rust
  confirm_check.unwrap() < prompt_check.unwrap(),
  ```

- Line 672: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 768: `\.unwrap\(\)`
  ```rust
  let before_parsed = before.unwrap();
  ```

- Line 779: `\.unwrap\(\)`
  ```rust
  let after_parsed = after.unwrap();
  ```

- Line 807: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 853: `\.unwrap\(\)`
  ```rust
  confirm_check.unwrap() < prompt_check.unwrap(),
  ```

- Line 853: `\.unwrap\(\)`
  ```rust
  confirm_check.unwrap() < prompt_check.unwrap(),
  ```

- Line 902: `\.unwrap\(\)`
  ```rust
  no_interactive_check.unwrap() < first_stage.unwrap(),
  ```

- Line 902: `\.unwrap\(\)`
  ```rust
  no_interactive_check.unwrap() < first_stage.unwrap(),
  ```

### hoop-cli/tests/remove_no_interactive_flag.rs

Total errors: 88

#### assert (11 occurrences)

- Line 25: `Should successfully parse flag before subcommand`
  ```rust
  assert!(result.is_ok(), "Should successfully parse flag before subcommand");
  ```

- Line 45: `Should successfully parse flag after subcommand`
  ```rust
  assert!(result.is_ok(), "Should successfully parse flag after subcommand");
  ```

- Line 65: `Should successfully parse short flag before subcommand`
  ```rust
  assert!(result.is_ok(), "Should successfully parse short flag before subcommand");
  ```

- Line 77: `Should successfully parse short flag after subcommand`
  ```rust
  assert!(result.is_ok(), "Should successfully parse short flag after subcommand");
  ```

- Line 89: `Should successfully parse command without flag`
  ```rust
  assert!(result.is_ok(), "Should successfully parse command without flag");
  ```

- Line 105: `Flag extraction should verify for `
  ```rust
  assert!(verification_result.is_ok(), "Flag extraction should verify for 'before' position");
  ```

- Line 119: `Flag extraction should verify for `
  ```rust
  assert!(verification_result.is_ok(), "Flag extraction should verify for 'after' position");
  ```

- Line 133: `Should verify no flag is present`
  ```rust
  assert!(verification_result.is_ok(), "Should verify no flag is present");
  ```

- Line 573: `Should parse flag before command`
  ```rust
  assert!(before.is_ok(), "Should parse flag before command");
  ```

- Line 578: `Should parse flag after command`
  ```rust
  assert!(after.is_ok(), "Should parse flag after command");
  ```

- Line 1076: `All Remove command no_interactive tests verified`
  ```rust
  assert!(true, "All Remove command no_interactive tests verified");
  ```

#### assert_eq (13 occurrences)

- Line 28: `no_interactive should be true`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true");
  ```

- Line 29: `Command should be `
  ```rust
  assert_eq!(parsed.command, "remove", "Command should be 'remove'");
  ```

- Line 48: `no_interactive should be true`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true");
  ```

- Line 49: `Command should be `
  ```rust
  assert_eq!(parsed.command, "remove", "Command should be 'remove'");
  ```

- Line 68: `no_interactive should be true with -y`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true with -y");
  ```

- Line 69: `Command should be `
  ```rust
  assert_eq!(parsed.command, "remove", "Command should be 'remove'");
  ```

- Line 80: `no_interactive should be true with -y`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true with -y");
  ```

- Line 81: `Command should be `
  ```rust
  assert_eq!(parsed.command, "remove", "Command should be 'remove'");
  ```

- Line 92: `no_interactive should default to false`
  ```rust
  assert_eq!(parsed.no_interactive, false, "no_interactive should default to false");
  ```

- Line 93: `Command should be `
  ```rust
  assert_eq!(parsed.command, "remove", "Command should be 'remove'");
  ```

- Line 642: `Global flag should produce true`
  ```rust
  assert_eq!(value_global, true, "Global flag should produce true");
  ```

- Line 648: `No flag should produce false`
  ```rust
  assert_eq!(value_none, false, "No flag should produce false");
  ```

- Line 691: `Both should produce true`
  ```rust
  assert_eq!(value_before, true, "Both should produce true");
  ```

#### expect (57 occurrences)

- Line 102: `Parse should succeed`
  ```rust
  .expect("Parse should succeed");
  ```

- Line 116: `Parse should succeed`
  ```rust
  .expect("Parse should succeed");
  ```

- Line 130: `Parse should succeed`
  ```rust
  .expect("Parse should succeed");
  ```

- Line 146: `Failed to read main.rs`
  ```rust
  .expect("Failed to read main.rs");
  ```

- Line 171: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 195: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 199: `Should find remove_project function`
  ```rust
  .expect("Should find remove_project function");
  ```

- Line 203: `Should find confirm requirement check`
  ```rust
  .expect("Should find confirm requirement check");
  ```

- Line 227: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 231: `Should find remove_project function`
  ```rust
  .expect("Should find remove_project function");
  ```

- Line 235: `Should find confirm requirement check`
  ```rust
  .expect("Should find confirm requirement check");
  ```

- Line 242: `Should have prompt check after confirm requirement`
  ```rust
  .expect("Should have prompt check after confirm requirement");
  ```

- Line 270: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 274: `Should find remove_project function`
  ```rust
  .expect("Should find remove_project function");
  ```

- Line 278: `Should find prompt check`
  ```rust
  .expect("Should find prompt check");
  ```

- Line 324: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 328: `Should find remove_project function`
  ```rust
  .expect("Should find remove_project function");
  ```

- Line 332: `Should find prompt check`
  ```rust
  .expect("Should find prompt check");
  ```

- Line 359: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 363: `Should find remove_project function`
  ```rust
  .expect("Should find remove_project function");
  ```

- Line 367: `Should find confirm requirement check`
  ```rust
  .expect("Should find confirm requirement check");
  ```

- Line 371: `Should find end of confirm requirement block`
  ```rust
  .expect("Should find end of confirm requirement block");
  ```

- Line 385: `Should find prompt check after confirm requirement`
  ```rust
  .expect("Should find prompt check after confirm requirement");
  ```

- Line 607: `Should parse global --no-interactive flag`
  ```rust
  .expect("Should parse global --no-interactive flag");
  ```

- Line 622: `Should parse remove command without flags`
  ```rust
  .expect("Should parse remove command without flags");
  ```

- Line 640: `Parse with global flag`
  ```rust
  .expect("Parse with global flag");
  ```

- Line 646: `Parse without flags`
  ```rust
  .expect("Parse without flags");
  ```

- Line 655: `Should parse short -y flag`
  ```rust
  .expect("Should parse short -y flag");
  ```

- Line 678: `Parse flag before subcommand`
  ```rust
  .expect("Parse flag before subcommand");
  ```

- Line 683: `Parse flag after subcommand`
  ```rust
  .expect("Parse flag after subcommand");
  ```

- Line 729: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 733: `Should find remove_project function`
  ```rust
  .expect("Should find remove_project function");
  ```

- Line 737: `Should find confirm requirement check`
  ```rust
  .expect("Should find confirm requirement check");
  ```

- Line 770: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 774: `Should find remove_project function`
  ```rust
  .expect("Should find remove_project function");
  ```

- Line 778: `Should find prompt check for interactive mode`
  ```rust
  .expect("Should find prompt check for interactive mode");
  ```

- Line 813: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 817: `Should find remove_project function`
  ```rust
  .expect("Should find remove_project function");
  ```

- Line 821: `Should find prompt check`
  ```rust
  .expect("Should find prompt check");
  ```

- Line 851: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 855: `Should find remove_project function`
  ```rust
  .expect("Should find remove_project function");
  ```

- Line 859: `Should find confirm requirement check`
  ```rust
  .expect("Should find confirm requirement check");
  ```

- Line 863: `Should find prompt check after confirm requirement`
  ```rust
  .expect("Should find prompt check after confirm requirement");
  ```

- Line 890: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 894: `Should find remove_project function`
  ```rust
  .expect("Should find remove_project function");
  ```

- Line 911: `Should find confirm requirement check`
  ```rust
  .expect("Should find confirm requirement check");
  ```

- Line 915: `Should find prompt check after confirm requirement`
  ```rust
  .expect("Should find prompt check after confirm requirement");
  ```

- Line 949: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 953: `Should find remove_project function`
  ```rust
  .expect("Should find remove_project function");
  ```

- Line 957: `Should find confirm requirement check`
  ```rust
  .expect("Should find confirm requirement check");
  ```

- Line 961: `Should find end of confirm requirement block`
  ```rust
  .expect("Should find end of confirm requirement block");
  ```

- Line 965: `Should find prompt check after confirm requirement`
  ```rust
  .expect("Should find prompt check after confirm requirement");
  ```

- Line 970: `Should find removal call after checks`
  ```rust
  .expect("Should find removal call after checks");
  ```

- Line 988: `Failed to read main.rs`
  ```rust
  .expect("Failed to read main.rs");
  ```

- Line 992: `Should find Remove command handler in main.rs`
  ```rust
  .expect("Should find Remove command handler in main.rs");
  ```

- Line 1010: `Failed to read main.rs`
  ```rust
  .expect("Failed to read main.rs");
  ```

- Line 1012: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

#### unwrap (7 occurrences)

- Line 26: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 46: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 66: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 78: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 90: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 574: `\.unwrap\(\)`
  ```rust
  let before_parsed = before.unwrap();
  ```

- Line 579: `\.unwrap\(\)`
  ```rust
  let after_parsed = after.unwrap();
  ```

### hoop-cli/tests/restore_no_interactive_flag.rs

Total errors: 66

#### assert (5 occurrences)

- Line 32: `Should successfully parse flag before subcommand`
  ```rust
  assert!(result.is_ok(), "Should successfully parse flag before subcommand");
  ```

- Line 63: `Should successfully parse flag after subcommand`
  ```rust
  assert!(result.is_ok(), "Should successfully parse flag after subcommand");
  ```

- Line 133: `Should successfully parse command without flag`
  ```rust
  assert!(result.is_ok(), "Should successfully parse command without flag");
  ```

- Line 156: `Should successfully parse with --dry-run flag`
  ```rust
  assert!(result.is_ok(), "Should successfully parse with --dry-run flag");
  ```

- Line 218: `Should verify no flag is present`
  ```rust
  assert!(verification_result.is_ok(), "Should verify no flag is present");
  ```

#### assert_eq (12 occurrences)

- Line 35: `no_interactive should be true`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true");
  ```

- Line 36: `Command should be `
  ```rust
  assert_eq!(parsed.command, "restore", "Command should be 'restore'");
  ```

- Line 66: `no_interactive should be true`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true");
  ```

- Line 67: `Command should be `
  ```rust
  assert_eq!(parsed.command, "restore", "Command should be 'restore'");
  ```

- Line 96: `no_interactive should be true with -y`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true with -y");
  ```

- Line 97: `Command should be `
  ```rust
  assert_eq!(parsed.command, "restore", "Command should be 'restore'");
  ```

- Line 118: `no_interactive should be true with -y`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true with -y");
  ```

- Line 119: `Command should be `
  ```rust
  assert_eq!(parsed.command, "restore", "Command should be 'restore'");
  ```

- Line 141: `Command should be `
  ```rust
  assert_eq!(parsed.command, "restore", "Command should be 'restore'");
  ```

- Line 159: `no_interactive should be true`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true");
  ```

- Line 160: `Command should be `
  ```rust
  assert_eq!(parsed.command, "restore", "Command should be 'restore'");
  ```

- Line 702: `Command should be `
  ```rust
  assert_eq!(parsed.command, "restore", "Command should be 'restore'");
  ```

#### expect (43 occurrences)

- Line 174: `Parse should succeed`
  ```rust
  .expect("Parse should succeed");
  ```

- Line 192: `Parse should succeed`
  ```rust
  .expect("Parse should succeed");
  ```

- Line 215: `Parse should succeed`
  ```rust
  .expect("Parse should succeed");
  ```

- Line 230: `Failed to read main.rs`
  ```rust
  let main_code = fs::read_to_string("src/main.rs").expect("Failed to read main.rs");
  ```

- Line 255: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 281: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 286: `Should find run_restore function`
  ```rust
  .expect("Should find run_restore function");
  ```

- Line 291: `Should find confirm requirement check`
  ```rust
  .expect("Should find confirm requirement check");
  ```

- Line 322: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 327: `Should find run_restore function`
  ```rust
  .expect("Should find run_restore function");
  ```

- Line 332: `Should find confirm requirement check`
  ```rust
  .expect("Should find confirm requirement check");
  ```

- Line 340: `Should have prompt check after confirm requirement`
  ```rust
  .expect("Should have prompt check after confirm requirement");
  ```

- Line 373: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 378: `Should find run_restore function`
  ```rust
  .expect("Should find run_restore function");
  ```

- Line 383: `Should find prompt check`
  ```rust
  .expect("Should find prompt check");
  ```

- Line 440: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 445: `Should find run_restore function`
  ```rust
  .expect("Should find run_restore function");
  ```

- Line 450: `Should find prompt check`
  ```rust
  .expect("Should find prompt check");
  ```

- Line 478: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 483: `Should find run_restore function`
  ```rust
  .expect("Should find run_restore function");
  ```

- Line 488: `Should find confirm requirement check`
  ```rust
  .expect("Should find confirm requirement check");
  ```

- Line 493: `Should find end of confirm requirement block`
  ```rust
  .expect("Should find end of confirm requirement block");
  ```

- Line 510: `Should find prompt check after confirm requirement`
  ```rust
  .expect("Should find prompt check after confirm requirement");
  ```

- Line 531: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 536: `run_restore must have dry_run mode`
  ```rust
  .expect("run_restore must have dry_run mode");
  ```

- Line 565: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 570: `restore.rs must define run_restore()`
  ```rust
  .expect("restore.rs must define run_restore()");
  ```

- Line 575: `run_restore must call manifest.validate(current)`
  ```rust
  .expect("run_restore must call manifest.validate(current)");
  ```

- Line 578: `run_restore must call move_aside_for_rollback()`
  ```rust
  .expect("run_restore must call move_aside_for_rollback()");
  ```

- Line 593: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 598: `restore.rs must define run_restore()`
  ```rust
  .expect("restore.rs must define run_restore()");
  ```

- Line 603: `run_restore must check no_interactive && !confirm`
  ```rust
  .expect("run_restore must check no_interactive && !confirm");
  ```

- Line 606: `run_restore must check !no_interactive for prompting`
  ```rust
  .expect("run_restore must check !no_interactive for prompting");
  ```

- Line 619: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 624: `Must have --confirm requirement check`
  ```rust
  .expect("Must have --confirm requirement check");
  ```

- Line 663: `Should parse flag before command`
  ```rust
  .expect("Should parse flag before command");
  ```

- Line 668: `Should parse flag after command`
  ```rust
  .expect("Should parse flag after command");
  ```

- Line 694: `Should parse -y flag`
  ```rust
  .expect("Should parse -y flag");
  ```

- Line 710: `Failed to read main.rs`
  ```rust
  let main_code = fs::read_to_string("src/main.rs").expect("Failed to read main.rs");
  ```

- Line 712: `Failed to read restore.rs`
  ```rust
  .expect("Failed to read restore.rs");
  ```

- Line 779: `run_restore function must exist`
  ```rust
  .expect("run_restore function must exist");
  ```

- Line 788: `manifest.validate() must be called in function body`
  ```rust
  .expect("manifest.validate() must be called in function body");
  ```

- Line 791: `move_aside_for_rollback() must be called in function body`
  ```rust
  .expect("move_aside_for_rollback() must be called in function body");
  ```

#### unwrap (6 occurrences)

- Line 33: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 64: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 94: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 116: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 134: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 157: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

### hoop-cli/tests/scan_no_interactive_flag.rs

Total errors: 99

#### assert (13 occurrences)

- Line 25: `Should successfully parse flag before subcommand`
  ```rust
  assert!(result.is_ok(), "Should successfully parse flag before subcommand");
  ```

- Line 45: `Should successfully parse flag after subcommand`
  ```rust
  assert!(result.is_ok(), "Should successfully parse flag after subcommand");
  ```

- Line 65: `Should successfully parse short flag before subcommand`
  ```rust
  assert!(result.is_ok(), "Should successfully parse short flag before subcommand");
  ```

- Line 77: `Should successfully parse short flag after subcommand`
  ```rust
  assert!(result.is_ok(), "Should successfully parse short flag after subcommand");
  ```

- Line 89: `Should successfully parse command without flag`
  ```rust
  assert!(result.is_ok(), "Should successfully parse command without flag");
  ```

- Line 102: `Should successfully parse local --yes flag`
  ```rust
  assert!(result.is_ok(), "Should successfully parse local --yes flag");
  ```

- Line 119: `Should successfully parse both flags`
  ```rust
  assert!(result.is_ok(), "Should successfully parse both flags");
  ```

- Line 138: `Flag extraction should verify for `
  ```rust
  assert!(verification_result.is_ok(), "Flag extraction should verify for 'before' position");
  ```

- Line 151: `Flag extraction should verify for `
  ```rust
  assert!(verification_result.is_ok(), "Flag extraction should verify for 'after' position");
  ```

- Line 164: `Should verify no flag is present`
  ```rust
  assert!(verification_result.is_ok(), "Should verify no flag is present");
  ```

- Line 652: `Should parse flag before command`
  ```rust
  assert!(before.is_ok(), "Should parse flag before command");
  ```

- Line 657: `Should parse flag after command`
  ```rust
  assert!(after.is_ok(), "Should parse flag after command");
  ```

- Line 818: `All Scan command no_interactive tests verified`
  ```rust
  assert!(true, "All Scan command no_interactive tests verified");
  ```

#### assert_eq (19 occurrences)

- Line 28: `no_interactive should be true`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true");
  ```

- Line 29: `Command should be `
  ```rust
  assert_eq!(parsed.command, "scan", "Command should be 'scan'");
  ```

- Line 48: `no_interactive should be true`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true");
  ```

- Line 49: `Command should be `
  ```rust
  assert_eq!(parsed.command, "scan", "Command should be 'scan'");
  ```

- Line 68: `no_interactive should be true with -y`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true with -y");
  ```

- Line 69: `Command should be `
  ```rust
  assert_eq!(parsed.command, "scan", "Command should be 'scan'");
  ```

- Line 80: `no_interactive should be true with -y`
  ```rust
  assert_eq!(parsed.no_interactive, true, "no_interactive should be true with -y");
  ```

- Line 81: `Command should be `
  ```rust
  assert_eq!(parsed.command, "scan", "Command should be 'scan'");
  ```

- Line 92: `no_interactive should default to false`
  ```rust
  assert_eq!(parsed.no_interactive, false, "no_interactive should default to false");
  ```

- Line 93: `Command should be `
  ```rust
  assert_eq!(parsed.command, "scan", "Command should be 'scan'");
  ```

- Line 105: `Global no_interactive should remain false with local --yes`
  ```rust
  assert_eq!(parsed.no_interactive, false, "Global no_interactive should remain false with local --yes");
  ```

- Line 106: `Command should be `
  ```rust
  assert_eq!(parsed.command, "scan", "Command should be 'scan'");
  ```

- Line 122: `Global no_interactive should be true`
  ```rust
  assert_eq!(parsed.no_interactive, true, "Global no_interactive should be true");
  ```

- Line 123: `Command should be `
  ```rust
  assert_eq!(parsed.command, "scan", "Command should be 'scan'");
  ```

- Line 915: `Global flag should produce true`
  ```rust
  assert_eq!(value_global, true, "Global flag should produce true");
  ```

- Line 921: `Local flag should produce true`
  ```rust
  assert_eq!(value_local, true, "Local flag should produce true");
  ```

- Line 927: `Both flags should produce true`
  ```rust
  assert_eq!(value_both, true, "Both flags should produce true");
  ```

- Line 933: `No flags should produce false`
  ```rust
  assert_eq!(value_neither, false, "No flags should produce false");
  ```

- Line 1005: `Both should produce true`
  ```rust
  assert_eq!(value_before, true, "Both should produce true");
  ```

#### expect (58 occurrences)

- Line 135: `Parse should succeed`
  ```rust
  let parsed = parse_flag_before_subcommand(&["scan", "/tmp"]).expect("Parse should succeed");
  ```

- Line 148: `Parse should succeed`
  ```rust
  let parsed = parse_flag_after_subcommand(&["scan", "/tmp"]).expect("Parse should succeed");
  ```

- Line 161: `Parse should succeed`
  ```rust
  let parsed = parse_cli_with_flag(&["hoop", "scan", "/tmp"]).expect("Parse should succeed");
  ```

- Line 177: `Failed to read main.rs`
  ```rust
  .expect("Failed to read main.rs");
  ```

- Line 202: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 222: `Failed to read main.rs`
  ```rust
  .expect("Failed to read main.rs");
  ```

- Line 232: `Should find Scan command handler`
  ```rust
  .expect("Should find Scan command handler");
  ```

- Line 236: `Should find scan_projects call with || logic`
  ```rust
  .expect("Should find scan_projects call with || logic");
  ```

- Line 252: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 256: `Should find scan_projects function`
  ```rust
  .expect("Should find scan_projects function");
  ```

- Line 260: `Should find no_interactive check in scan_projects`
  ```rust
  .expect("Should find no_interactive check in scan_projects");
  ```

- Line 286: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 290: `Should find scan_projects function`
  ```rust
  .expect("Should find scan_projects function");
  ```

- Line 322: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 326: `Should find scan_projects function`
  ```rust
  .expect("Should find scan_projects function");
  ```

- Line 354: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 358: `Should find scan_projects function`
  ```rust
  .expect("Should find scan_projects function");
  ```

- Line 362: `Should find no_interactive check in scan_projects`
  ```rust
  .expect("Should find no_interactive check in scan_projects");
  ```

- Line 687: `Failed to read main.rs`
  ```rust
  .expect("Failed to read main.rs");
  ```

- Line 700: `Failed to read main.rs`
  ```rust
  .expect("Failed to read main.rs");
  ```

- Line 704: `Should find Scan command documentation`
  ```rust
  .expect("Should find Scan command documentation");
  ```

- Line 726: `Failed to read main.rs`
  ```rust
  .expect("Failed to read main.rs");
  ```

- Line 730: `Should find Scan command handler`
  ```rust
  .expect("Should find Scan command handler");
  ```

- Line 733: `Should find scan_projects call`
  ```rust
  .expect("Should find scan_projects call");
  ```

- Line 752: `Failed to read main.rs`
  ```rust
  .expect("Failed to read main.rs");
  ```

- Line 754: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 827: `Should parse global --no-interactive flag`
  ```rust
  .expect("Should parse global --no-interactive flag");
  ```

- Line 842: `Should parse local --yes flag`
  ```rust
  .expect("Should parse local --yes flag");
  ```

- Line 857: `Should parse both global --no-interactive and local --yes flags`
  ```rust
  .expect("Should parse both global --no-interactive and local --yes flags");
  ```

- Line 872: `Should parse scan command without flags`
  ```rust
  .expect("Should parse scan command without flags");
  ```

- Line 913: `Parse with global flag`
  ```rust
  .expect("Parse with global flag");
  ```

- Line 919: `Parse with local flag`
  ```rust
  .expect("Parse with local flag");
  ```

- Line 925: `Parse with both flags`
  ```rust
  .expect("Parse with both flags");
  ```

- Line 931: `Parse without flags`
  ```rust
  .expect("Parse without flags");
  ```

- Line 940: `Should parse short -y flag`
  ```rust
  .expect("Should parse short -y flag");
  ```

- Line 961: `Should parse with global flag only`
  ```rust
  .expect("Should parse with global flag only");
  ```

- Line 975: `Should parse with local flag only`
  ```rust
  .expect("Should parse with local flag only");
  ```

- Line 992: `Parse flag before subcommand`
  ```rust
  .expect("Parse flag before subcommand");
  ```

- Line 997: `Parse flag after subcommand`
  ```rust
  .expect("Parse flag after subcommand");
  ```

- Line 1048: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 1052: `Should find scan_projects function`
  ```rust
  .expect("Should find scan_projects function");
  ```

- Line 1056: `Should find no_interactive check`
  ```rust
  .expect("Should find no_interactive check");
  ```

- Line 1098: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 1102: `Should find scan_projects function`
  ```rust
  .expect("Should find scan_projects function");
  ```

- Line 1106: `Should find else branch with interactive prompts`
  ```rust
  .expect("Should find else branch with interactive prompts");
  ```

- Line 1153: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 1157: `Should find scan_projects function`
  ```rust
  .expect("Should find scan_projects function");
  ```

- Line 1193: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 1197: `Should find scan_projects function`
  ```rust
  .expect("Should find scan_projects function");
  ```

- Line 1201: `Should find no_interactive check`
  ```rust
  .expect("Should find no_interactive check");
  ```

- Line 1205: `Should find else branch for interactive mode`
  ```rust
  .expect("Should find else branch for interactive mode");
  ```

- Line 1232: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 1236: `Should find scan_projects function`
  ```rust
  .expect("Should find scan_projects function");
  ```

- Line 1248: `Should find no_interactive check`
  ```rust
  .expect("Should find no_interactive check");
  ```

- Line 1253: `Should find else branch after no_interactive check`
  ```rust
  .expect("Should find else branch after no_interactive check");
  ```

- Line 1286: `Failed to read projects.rs`
  ```rust
  .expect("Failed to read projects.rs");
  ```

- Line 1290: `Should find scan_projects function`
  ```rust
  .expect("Should find scan_projects function");
  ```

- Line 1294: `Should find no_interactive check`
  ```rust
  .expect("Should find no_interactive check");
  ```

#### unwrap (9 occurrences)

- Line 26: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 46: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 66: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 78: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 90: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 103: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 120: `\.unwrap\(\)`
  ```rust
  let parsed = result.unwrap();
  ```

- Line 653: `\.unwrap\(\)`
  ```rust
  let before_parsed = before.unwrap();
  ```

- Line 658: `\.unwrap\(\)`
  ```rust
  let after_parsed = after.unwrap();
  ```

### hoop-daemon/examples/populate-testrepo.rs

Total errors: 1

#### expect (1 occurrences)

- Line 37: `workspace root is parent of hoop-daemon/`
  ```rust
  .expect("workspace root is parent of hoop-daemon/")
  ```

### hoop-daemon/examples/test_yaml_parsing.rs

Total errors: 3

#### unwrap (3 occurrences)

- Line 11: `\.unwrap\(\)`
  ```rust
  let yml: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
  ```

- Line 17: `\.unwrap\(\)`
  ```rust
  let audit = yml.get("audit").unwrap();
  ```

- Line 20: `\.unwrap\(\)`
  ```rust
  let retention_days = audit.get("retention_days").unwrap();
  ```

### hoop-daemon/src/integration_test_client.rs

Total errors: 17

#### anyhow (1 occurrences)

- Line 322: `WebSocket error: {}`
  ```rust
  Some(Err(e)) => Err(anyhow::anyhow!("WebSocket error: {}", e)),
  ```

#### bail (14 occurrences)

- Line 69: `Daemon did not become ready within {:?}`
  ```rust
  anyhow::bail!("Daemon did not become ready within {:?}", timeout);
  ```

- Line 99: `GET /api/beads failed: {}`
  ```rust
  anyhow::bail!("GET /api/beads failed: {}", resp.status());
  ```

- Line 115: `GET /api/beads/{} failed: {}`
  ```rust
  anyhow::bail!("GET /api/beads/{} failed: {}", bead_id, resp.status());
  ```

- Line 178: `GET /api/capacity failed: {}`
  ```rust
  anyhow::bail!("GET /api/capacity failed: {}", resp.status());
  ```

- Line 193: `GET /metrics failed: {}`
  ```rust
  anyhow::bail!("GET /metrics failed: {}", resp.status());
  ```

- Line 208: `GET /api/workers/timeline failed: {}`
  ```rust
  anyhow::bail!("GET /api/workers/timeline failed: {}", resp.status());
  ```

- Line 234: `Health check failed: {}`
  ```rust
  anyhow::bail!("Health check failed: {}", resp.status());
  ```

- Line 243: `Readiness check failed: {}`
  ```rust
  anyhow::bail!("Readiness check failed: {}", resp.status());
  ```

- Line 252: `Bead ID mismatch: expected {}, got {}`
  ```rust
  anyhow::bail!("Bead ID mismatch: expected {}, got {}", bead_id, bead["id"]);
  ```

- Line 262: `No bead with title `
  ```rust
  anyhow::bail!("No bead with title '{}' found", title);
  ```

- Line 286: `Capacity response is not an object`
  ```rust
  anyhow::bail!("Capacity response is not an object");
  ```

- Line 352: `WebSocket connection closed`
  ```rust
  anyhow::bail!("WebSocket connection closed");
  ```

- Line 355: `WebSocket connection terminated`
  ```rust
  anyhow::bail!("WebSocket connection terminated");
  ```

- Line 361: `Timeout waiting for bead event`
  ```rust
  anyhow::bail!("Timeout waiting for bead event");
  ```

#### unwrap (2 occurrences)

- Line 19: `\.unwrap\(\)`
  ```rust
  //!     let bead = client.create_bead("test-project", "Test bead").await.unwrap();
  ```

- Line 23: `\.unwrap\(\)`
  ```rust
  //!     let beads = client.list_beads().await.unwrap();
  ```

### hoop-daemon/src/load_test.rs

Total errors: 6

#### bail (1 occurrences)

- Line 369: `Performance budget violations:\n{}`
  ```rust
  anyhow::bail!("Performance budget violations:\n{}", failures.join("\n"));
  ```

#### unwrap (5 occurrences)

- Line 386: `\.unwrap\(\)`
  ```rust
  let max = self.api_latencies.iter().max().unwrap();
  ```

- Line 392: `\.unwrap\(\)`
  ```rust
  let max = self.ws_fanout_lags.iter().max().unwrap();
  ```

- Line 398: `\.unwrap\(\)`
  ```rust
  let max = self.memory_samples.iter().max().unwrap();
  ```

- Line 768: `\.unwrap\(\)`
  ```rust
  let temp_dir = tempfile::TempDir::new().unwrap();
  ```

- Line 770: `\.unwrap\(\)`
  ```rust
  populate_testrepo(config, temp_dir.path()).unwrap();
  ```

### hoop-daemon/tests/acceptance/s1_morning_review.rs

Total errors: 29

#### assert_eq (1 occurrences)

- Line 132: `Dashboard should return 200`
  ```rust
  assert_eq!(resp.status(), 200, "Dashboard should return 200");
  ```

#### expect (28 occurrences)

- Line 29: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 38: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 49: `Failed to parse dashboard response`
  ```rust
  .expect("Failed to parse dashboard response");
  ```

- Line 58: `total_workers must be a number`
  ```rust
  .expect("total_workers must be a number");
  ```

- Line 72: `total_spend_usd must be a number`
  ```rust
  .expect("total_spend_usd must be a number");
  ```

- Line 86: `longest_running must be an array`
  ```rust
  .expect("longest_running must be an array");
  ```

- Line 94: `Failed to fetch worker timeline`
  ```rust
  .expect("Failed to fetch worker timeline");
  ```

- Line 102: `Failed to parse timeline`
  ```rust
  let _timeline: JsonValue = resp.json().await.expect("Failed to parse timeline");
  ```

- Line 118: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 128: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 153: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 162: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 170: `Failed to parse response`
  ```rust
  let dashboard: JsonValue = resp.json().await.expect("Failed to parse response");
  ```

- Line 194: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 203: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 205: `Failed to parse response`
  ```rust
  let dashboard1: JsonValue = resp1.json().await.expect("Failed to parse response");
  ```

- Line 215: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 217: `Failed to parse response`
  ```rust
  let dashboard2: JsonValue = resp2.json().await.expect("Failed to parse response");
  ```

- Line 238: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 246: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 248: `Failed to parse response`
  ```rust
  let dashboard: JsonValue = resp.json().await.expect("Failed to parse response");
  ```

- Line 253: `total_spend_usd must be present`
  ```rust
  .expect("total_spend_usd must be present");
  ```

- Line 263: `spend_by_project must be an array`
  ```rust
  .expect("spend_by_project must be an array");
  ```

- Line 291: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 299: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 301: `Failed to parse response`
  ```rust
  let dashboard: JsonValue = resp.json().await.expect("Failed to parse response");
  ```

- Line 305: `total_workers must be present`
  ```rust
  .expect("total_workers must be present");
  ```

- Line 309: `workers_by_project must be an array`
  ```rust
  .expect("workers_by_project must be an array");
  ```

### hoop-daemon/tests/acceptance/s2_transcript_archaeology.rs

Total errors: 31

#### assert (1 occurrences)

- Line 73: `Events should be an array`
  ```rust
  assert!(events.is_array(), "Events should be an array");
  ```

#### expect (30 occurrences)

- Line 31: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 40: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 48: `Failed to parse beads`
  ```rust
  let beads: JsonValue = resp.json().await.expect("Failed to parse beads");
  ```

- Line 55: `Bead should have an id`
  ```rust
  .expect("Bead should have an id");
  ```

- Line 62: `Failed to fetch bead events`
  ```rust
  .expect("Failed to fetch bead events");
  ```

- Line 72: `Failed to parse events`
  ```rust
  let events: JsonValue = resp.json().await.expect("Failed to parse events");
  ```

- Line 92: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 101: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 103: `Failed to parse beads`
  ```rust
  let beads: JsonValue = resp.json().await.expect("Failed to parse beads");
  ```

- Line 109: `Bead should have an id`
  ```rust
  .expect("Bead should have an id");
  ```

- Line 118: `Failed to fetch bead events`
  ```rust
  .expect("Failed to fetch bead events");
  ```

- Line 143: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 152: `Failed to connect to stitch endpoint`
  ```rust
  .expect("Failed to connect to stitch endpoint");
  ```

- Line 173: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 190: `Failed to connect to endpoint`
  ```rust
  .expect("Failed to connect to endpoint");
  ```

- Line 212: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 221: `Failed to fetch conversations`
  ```rust
  .expect("Failed to fetch conversations");
  ```

- Line 229: `Failed to parse conversations`
  ```rust
  let conversations: JsonValue = resp.json().await.expect("Failed to parse conversations");
  ```

- Line 250: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 259: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 261: `Failed to parse beads`
  ```rust
  let beads: JsonValue = resp.json().await.expect("Failed to parse beads");
  ```

- Line 287: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 296: `Failed to fetch cost trends`
  ```rust
  .expect("Failed to fetch cost trends");
  ```

- Line 304: `Failed to parse cost data`
  ```rust
  let cost_data: JsonValue = resp.json().await.expect("Failed to parse cost data");
  ```

- Line 326: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 335: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 337: `Failed to parse beads`
  ```rust
  let beads: JsonValue = resp.json().await.expect("Failed to parse beads");
  ```

- Line 343: `Bead should have an id`
  ```rust
  .expect("Bead should have an id");
  ```

- Line 350: `Failed to fetch bead events`
  ```rust
  .expect("Failed to fetch bead events");
  ```

- Line 353: `Failed to parse events`
  ```rust
  let events: JsonValue = resp.json().await.expect("Failed to parse events");
  ```

### hoop-daemon/tests/acceptance/s3_bead_creation_from_chat.rs

Total errors: 85

#### assert (8 occurrences)

- Line 179: `Draft should appear in the draft queue`
  ```rust
  assert!(found, "Draft should appear in the draft queue");
  ```

- Line 388: `Audit log should contain DraftCreated entry`
  ```rust
  assert!(draft_created.is_some(), "Audit log should contain DraftCreated entry");
  ```

- Line 401: `Audit log should contain DraftApproved entry`
  ```rust
  assert!(draft_approved.is_some(), "Audit log should contain DraftApproved entry");
  ```

- Line 413: `Operator identity should be present in audit log`
  ```rust
  assert!(!actor.is_empty(), "Operator identity should be present in audit log");
  ```

- Line 478: `Draft should be in queue`
  ```rust
  assert!(draft_in_queue, "Draft should be in queue");
  ```

- Line 522: `Audit should have DraftCreated`
  ```rust
  assert!(draft_created.is_some(), "Audit should have DraftCreated");
  ```

- Line 523: `Audit should have DraftApproved`
  ```rust
  assert!(draft_approved.is_some(), "Audit should have DraftApproved");
  ```

- Line 535: `operator identity should be present`
  ```rust
  assert!(!actor.is_empty(), "operator identity should be present");
  ```

#### assert_eq (13 occurrences)

- Line 164: `List drafts should return 200`
  ```rust
  assert_eq!(list_resp.status(), 200, "List drafts should return 200");
  ```

- Line 188: `Get draft should return 200`
  ```rust
  assert_eq!(get_resp.status(), 200, "Get draft should return 200");
  ```

- Line 195: `Draft title should match chat input`
  ```rust
  assert_eq!(draft["title"], chat_input, "Draft title should match chat input");
  ```

- Line 196: `Draft kind should be fix`
  ```rust
  assert_eq!(draft["kind"], "fix", "Draft kind should be fix");
  ```

- Line 197: `Draft source should be chat`
  ```rust
  assert_eq!(draft["source"], "chat", "Draft source should be chat");
  ```

- Line 198: `Draft project should be testrepo`
  ```rust
  assert_eq!(draft["project"], "testrepo", "Draft project should be testrepo");
  ```

- Line 199: `Draft status should be pending`
  ```rust
  assert_eq!(draft["status"], "pending", "Draft status should be pending");
  ```

- Line 299: `Draft status should be submitted`
  ```rust
  assert_eq!(draft["status"], "submitted", "Draft status should be submitted");
  ```

- Line 300: `Draft should have stitch_id`
  ```rust
  assert_eq!(draft["stitch_id"], stitch_id, "Draft should have stitch_id");
  ```

- Line 371: `Audit query should return 200`
  ```rust
  assert_eq!(audit_resp.status(), 200, "Audit query should return 200");
  ```

- Line 393: `DraftCreated source should be chat`
  ```rust
  assert_eq!(args["source"], "chat", "DraftCreated source should be chat");
  ```

- Line 527: `source should be chat`
  ```rust
  assert_eq!(dc_args["source"], "chat", "source should be chat");
  ```

- Line 531: `stitch_id should match`
  ```rust
  assert_eq!(da_args["stitch_id"], stitch_id, "stitch_id should match");
  ```

#### expect (57 occurrences)

- Line 41: `create temp dir`
  ```rust
  let bin_dir = TempDir::new().expect("create temp dir");
  ```

- Line 56: `create br script`
  ```rust
  let mut f = fs::File::create(&br_path).expect("create br script");
  ```

- Line 57: `write br script`
  ```rust
  f.write_all(script.as_bytes()).expect("write br script");
  ```

- Line 62: `chmod br script`
  ```rust
  .expect("chmod br script");
  ```

- Line 107: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 133: `Failed to create draft`
  ```rust
  .expect("Failed to create draft");
  ```

- Line 145: `Failed to parse draft response`
  ```rust
  .expect("Failed to parse draft response");
  ```

- Line 149: `draft_id should be present`
  ```rust
  .expect("draft_id should be present");
  ```

- Line 162: `Failed to list drafts`
  ```rust
  .expect("Failed to list drafts");
  ```

- Line 169: `Failed to parse list response`
  ```rust
  .expect("Failed to parse list response");
  ```

- Line 173: `drafts should be an array`
  ```rust
  .expect("drafts should be an array");
  ```

- Line 186: `Failed to get draft`
  ```rust
  .expect("Failed to get draft");
  ```

- Line 193: `Failed to parse draft`
  ```rust
  .expect("Failed to parse draft");
  ```

- Line 217: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 235: `Failed to create draft`
  ```rust
  .expect("Failed to create draft");
  ```

- Line 240: `Failed to parse draft response`
  ```rust
  .expect("Failed to parse draft response");
  ```

- Line 244: `draft_id should be present`
  ```rust
  .expect("draft_id should be present");
  ```

- Line 254: `Failed to approve draft`
  ```rust
  .expect("Failed to approve draft");
  ```

- Line 266: `Failed to parse approve response`
  ```rust
  .expect("Failed to parse approve response");
  ```

- Line 270: `stitch_id should be present`
  ```rust
  .expect("stitch_id should be present");
  ```

- Line 292: `Failed to get draft`
  ```rust
  .expect("Failed to get draft");
  ```

- Line 297: `Failed to parse draft`
  ```rust
  .expect("Failed to parse draft");
  ```

- Line 318: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 336: `Failed to create draft`
  ```rust
  .expect("Failed to create draft");
  ```

- Line 341: `Failed to parse draft response`
  ```rust
  .expect("Failed to parse draft response");
  ```

- Line 345: `draft_id should be present`
  ```rust
  .expect("draft_id should be present");
  ```

- Line 353: `Failed to approve draft`
  ```rust
  .expect("Failed to approve draft");
  ```

- Line 358: `Failed to parse approve response`
  ```rust
  .expect("Failed to parse approve response");
  ```

- Line 362: `stitch_id should be present`
  ```rust
  .expect("stitch_id should be present");
  ```

- Line 369: `Failed to query audit log`
  ```rust
  .expect("Failed to query audit log");
  ```

- Line 376: `Failed to parse audit response`
  ```rust
  .expect("Failed to parse audit response");
  ```

- Line 380: `audit_rows should be an array`
  ```rust
  .expect("audit_rows should be an array");
  ```

- Line 392: `args should be an object`
  ```rust
  let args = draft_created["args"].as_object().expect("args should be an object");
  ```

- Line 404: `args should be an object`
  ```rust
  let approved_args = draft_approved["args"].as_object().expect("args should be an object");
  ```

- Line 412: `actor should be present`
  ```rust
  let actor = draft_approved["actor"].as_str().expect("actor should be present");
  ```

- Line 434: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 459: `Failed to create draft`
  ```rust
  .expect("Failed to create draft");
  ```

- Line 464: `Failed to parse response`
  ```rust
  let create_response: serde_json::Value = create_resp.json().await.expect("Failed to parse response");
  ```

- Line 465: `draft_id present`
  ```rust
  let draft_id = create_response["draft_id"].as_str().expect("draft_id present");
  ```

- Line 472: `Failed to list drafts`
  ```rust
  .expect("Failed to list drafts");
  ```

- Line 474: `Failed to parse list`
  ```rust
  let list_response: serde_json::Value = list_resp.json().await.expect("Failed to parse list");
  ```

- Line 475: `drafts array`
  ```rust
  let drafts = list_response["drafts"].as_array().expect("drafts array");
  ```

- Line 488: `Failed to approve draft`
  ```rust
  .expect("Failed to approve draft");
  ```

- Line 493: `Failed to parse approve`
  ```rust
  let approve_response: serde_json::Value = approve_resp.json().await.expect("Failed to parse approve");
  ```

- Line 494: `stitch_id present`
  ```rust
  let stitch_id = approve_response["stitch_id"].as_str().expect("stitch_id present");
  ```

- Line 509: `Failed to query audit`
  ```rust
  .expect("Failed to query audit");
  ```

- Line 511: `Failed to parse audit`
  ```rust
  let audit_response: serde_json::Value = audit_resp.json().await.expect("Failed to parse audit");
  ```

- Line 512: `audit_rows array`
  ```rust
  let audit_rows = audit_response["audit_rows"].as_array().expect("audit_rows array");
  ```

- Line 526: `args object`
  ```rust
  let dc_args = draft_created.unwrap()["args"].as_object().expect("args object");
  ```

- Line 530: `args object`
  ```rust
  let da_args = draft_approved.unwrap()["args"].as_object().expect("args object");
  ```

- Line 534: `actor present`
  ```rust
  let actor = draft_approved.unwrap()["actor"].as_str().expect("actor present");
  ```

- Line 556: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 577: `Failed to create draft`
  ```rust
  .expect("Failed to create draft");
  ```

- Line 581: `Failed to parse`
  ```rust
  let create_response: serde_json::Value = create_resp.json().await.expect("Failed to parse");
  ```

- Line 582: `draft_id present`
  ```rust
  let draft_id = create_response["draft_id"].as_str().expect("draft_id present");
  ```

- Line 589: `Failed to get draft`
  ```rust
  .expect("Failed to get draft");
  ```

- Line 593: `Failed to parse draft`
  ```rust
  let draft: serde_json::Value = get_resp.json().await.expect("Failed to parse draft");
  ```

#### unwrap (7 occurrences)

- Line 46: `\.unwrap\(\)`
  ```rust
  let log_path_str = log_path.to_str().unwrap();
  ```

- Line 70: `\.unwrap\(\)`
  ```rust
  self.bin_dir.path().to_str().unwrap().to_string()
  ```

- Line 391: `\.unwrap\(\)`
  ```rust
  let draft_created = draft_created.unwrap();
  ```

- Line 403: `\.unwrap\(\)`
  ```rust
  let draft_approved = draft_approved.unwrap();
  ```

- Line 526: `\.unwrap\(\)`
  ```rust
  let dc_args = draft_created.unwrap()["args"].as_object().expect("args object");
  ```

- Line 530: `\.unwrap\(\)`
  ```rust
  let da_args = draft_approved.unwrap()["args"].as_object().expect("args object");
  ```

- Line 534: `\.unwrap\(\)`
  ```rust
  let actor = draft_approved.unwrap()["actor"].as_str().expect("actor present");
  ```

### hoop-daemon/tests/acceptance/s4_daemon_restart.rs

Total errors: 48

#### assert_eq (3 occurrences)

- Line 370: `Should be able to fetch beads after rebuild`
  ```rust
  assert_eq!(resp.status(), 200, "Should be able to fetch beads after rebuild");
  ```

- Line 474: `Should see all beads including those created during restart`
  ```rust
  assert_eq!(resp.status(), 200, "Should see all beads including those created during restart");
  ```

- Line 535: `Should fetch beads in cycle {}`
  ```rust
  assert_eq!(resp.status(), 200, "Should fetch beads in cycle {}", cycle);
  ```

#### expect (40 occurrences)

- Line 32: `workspace root is parent of hoop-daemon/`
  ```rust
  .expect("workspace root is parent of hoop-daemon/")
  ```

- Line 106: `create temp dir for test HOOP home`
  ```rust
  let temp_dir = TempDir::new().expect("create temp dir for test HOOP home");
  ```

- Line 108: `create .hoop dir`
  ```rust
  fs::create_dir_all(&hoop_dir).expect("create .hoop dir");
  ```

- Line 123: `write projects.yaml`
  ```rust
  .expect("write projects.yaml");
  ```

- Line 132: `write config.yml`
  ```rust
  .expect("write config.yml");
  ```

- Line 134: `create data dir`
  ```rust
  fs::create_dir_all(hoop_dir.join("data")).expect("create data dir");
  ```

- Line 157: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 161: `write claim`
  ```rust
  worker.write_claim("bd-001").expect("write claim");
  ```

- Line 162: `write complete`
  ```rust
  worker.write_complete("bd-001").expect("write complete");
  ```

- Line 163: `write claim`
  ```rust
  worker.write_claim("bd-002").expect("write claim");
  ```

- Line 172: `Failed to spawn first daemon`
  ```rust
  .expect("Failed to spawn first daemon");
  ```

- Line 197: `Failed to fetch beads from first daemon`
  ```rust
  .expect("Failed to fetch beads from first daemon");
  ```

- Line 205: `Failed to parse beads`
  ```rust
  let beads1: serde_json::Value = resp1.json().await.expect("Failed to parse beads");
  ```

- Line 214: `write complete`
  ```rust
  worker.write_complete("bd-002").expect("write complete");
  ```

- Line 215: `write claim`
  ```rust
  worker.write_claim("bd-003").expect("write claim");
  ```

- Line 228: `Failed to spawn second daemon`
  ```rust
  .expect("Failed to spawn second daemon");
  ```

- Line 251: `Failed to fetch beads from second daemon`
  ```rust
  .expect("Failed to fetch beads from second daemon");
  ```

- Line 259: `Failed to parse beads`
  ```rust
  let beads2: serde_json::Value = resp2.json().await.expect("Failed to parse beads");
  ```

- Line 294: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 300: `write claim`
  ```rust
  worker.write_claim(&bead_id).expect("write claim");
  ```

- Line 302: `write complete`
  ```rust
  worker.write_complete(&bead_id).expect("write complete");
  ```

- Line 309: `Failed to spawn first daemon`
  ```rust
  .expect("Failed to spawn first daemon");
  ```

- Line 336: `Failed to spawn second daemon`
  ```rust
  .expect("Failed to spawn second daemon");
  ```

- Line 368: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 393: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 398: `Failed to spawn first daemon`
  ```rust
  .expect("Failed to spawn first daemon");
  ```

- Line 424: `write claim`
  ```rust
  worker.write_claim("bd-restart-1").expect("write claim");
  ```

- Line 425: `write complete`
  ```rust
  worker.write_complete("bd-restart-1").expect("write complete");
  ```

- Line 426: `write claim`
  ```rust
  worker.write_claim("bd-restart-2").expect("write claim");
  ```

- Line 438: `Failed to spawn second daemon`
  ```rust
  .expect("Failed to spawn second daemon");
  ```

- Line 457: `write complete`
  ```rust
  worker.write_complete("bd-restart-2").expect("write complete");
  ```

- Line 458: `write claim`
  ```rust
  worker.write_claim("bd-restart-3").expect("write claim");
  ```

- Line 472: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 496: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 503: `write claim`
  ```rust
  worker.write_claim("bd-s4-1").expect("write claim");
  ```

- Line 504: `write complete`
  ```rust
  worker.write_complete("bd-s4-1").expect("write complete");
  ```

- Line 510: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 533: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 537: `Failed to parse beads`
  ```rust
  let beads: serde_json::Value = resp.json().await.expect("Failed to parse beads");
  ```

- Line 560: `write claim`
  ```rust
  worker.write_claim(&format!("bd-s4-{}", cycle * 10 + 2)).expect("write claim");
  ```

#### unwrap (5 occurrences)

- Line 104: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 146: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 283: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 382: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 485: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

### hoop-daemon/tests/acceptance/s5_workspace_deleted.rs

Total errors: 39

#### assert_eq (3 occurrences)

- Line 169: `Initial readyz should return 200`
  ```rust
  assert_eq!(status, 200, "Initial readyz should return 200");
  ```

- Line 170: `Initial readyz status should be ok`
  ```rust
  assert_eq!(readyz.status, "ok", "Initial readyz status should be ok");
  ```

- Line 280: `Projects endpoint should still work`
  ```rust
  assert_eq!(resp.status(), 200, "Projects endpoint should still work");
  ```

#### expect (26 occurrences)

- Line 29: `Failed to create .beads dir`
  ```rust
  fs::create_dir_all(&beads_dir).expect("Failed to create .beads dir");
  ```

- Line 31: `Failed to create issues.jsonl`
  ```rust
  fs::write(&issues_path, b"").expect("Failed to create issues.jsonl");
  ```

- Line 39: `Failed to create temp dir`
  ```rust
  let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
  ```

- Line 41: `Failed to create .hoop dir`
  ```rust
  fs::create_dir_all(&hoop_dir).expect("Failed to create .hoop dir");
  ```

- Line 70: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 78: `Failed to write config.yml`
  ```rust
  fs::write(hoop_dir.join("config.yml"), config_yaml).expect("Failed to write config.yml");
  ```

- Line 79: `Failed to create data dir`
  ```rust
  fs::create_dir_all(hoop_dir.join("data")).expect("Failed to create data dir");
  ```

- Line 121: `Failed to bind to random port`
  ```rust
  .expect("Failed to bind to random port");
  ```

- Line 122: `Failed to get local address`
  ```rust
  let addr = listener.local_addr().expect("Failed to get local address");
  ```

- Line 167: `Failed to get readyz status`
  ```rust
  .expect("Failed to get readyz status");
  ```

- Line 174: `Failed to remove .beads from project A`
  ```rust
  fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads from project A");
  ```

- Line 225: `Failed to bind to random port`
  ```rust
  .expect("Failed to bind to random port");
  ```

- Line 226: `Failed to get local address`
  ```rust
  let addr = listener.local_addr().expect("Failed to get local address");
  ```

- Line 268: `Failed to remove .beads from project A`
  ```rust
  fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads from project A");
  ```

- Line 278: `Failed to fetch projects`
  ```rust
  .expect("Failed to fetch projects");
  ```

- Line 282: `Failed to parse projects`
  ```rust
  let projects: JsonValue = resp.json().await.expect("Failed to parse projects");
  ```

- Line 295: `Failed to check health`
  ```rust
  .expect("Failed to check health");
  ```

- Line 328: `Failed to bind to random port`
  ```rust
  .expect("Failed to bind to random port");
  ```

- Line 329: `Failed to get local address`
  ```rust
  let addr = listener.local_addr().expect("Failed to get local address");
  ```

- Line 372: `Failed to get readyz status`
  ```rust
  .expect("Failed to get readyz status");
  ```

- Line 377: `Failed to remove .beads from project A`
  ```rust
  fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads from project A");
  ```

- Line 384: `Failed to get readyz status after deletion`
  ```rust
  .expect("Failed to get readyz status after deletion");
  ```

- Line 435: `Failed to bind to random port`
  ```rust
  .expect("Failed to bind to random port");
  ```

- Line 436: `Failed to get local address`
  ```rust
  let addr = listener.local_addr().expect("Failed to get local address");
  ```

- Line 478: `Failed to remove .beads`
  ```rust
  fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads");
  ```

- Line 487: `Failed to check health`
  ```rust
  .expect("Failed to check health");
  ```

#### unwrap (10 occurrences)

- Line 104: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 108: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 112: `\.unwrap\(\)`
  ```rust
  let project_c_dir = tempfile::tempdir().unwrap();
  ```

- Line 209: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 213: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 217: `\.unwrap\(\)`
  ```rust
  let project_c_dir = tempfile::tempdir().unwrap();
  ```

- Line 312: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 316: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 320: `\.unwrap\(\)`
  ```rust
  let project_c_dir = tempfile::tempdir().unwrap();
  ```

- Line 427: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

### hoop-daemon/tests/acceptance/s6_machine_mode.rs

Total errors: 55

#### assert (4 occurrences)

- Line 127: `JSON output should be an object`
  ```rust
  assert!(json.is_object(), "JSON output should be an object");
  ```

- Line 138: `Each project should be an object`
  ```rust
  assert!(project.is_object(), "Each project should be an object");
  ```

- Line 512: `Each project should be an object`
  ```rust
  assert!(project.is_object(), "Each project should be an object");
  ```

- Line 608: `JSON should be an object`
  ```rust
  assert!(json.is_object(), "JSON should be an object");
  ```

#### assert_eq (1 occurrences)

- Line 134: `Should have 3 projects`
  ```rust
  assert_eq!(projects.len(), 3, "Should have 3 projects");
  ```

#### expect (47 occurrences)

- Line 32: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 34: `Failed to create .hoop dir`
  ```rust
  fs::create_dir_all(&hoop_dir).expect("Failed to create .hoop dir");
  ```

- Line 42: `Failed to write config.yml`
  ```rust
  .expect("Failed to write config.yml");
  ```

- Line 47: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 61: `Failed to create project dir`
  ```rust
  fs::create_dir_all(&project_dir).expect("Failed to create project dir");
  ```

- Line 64: `Failed to create .beads dir`
  ```rust
  fs::create_dir_all(&beads_dir).expect("Failed to create .beads dir");
  ```

- Line 68: `Failed to create issues.jsonl`
  ```rust
  fs::write(&issues_path, b"").expect("Failed to create issues.jsonl");
  ```

- Line 102: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 111: `Failed to run hoop status --json`
  ```rust
  .expect("Failed to run hoop status --json");
  ```

- Line 120: `Invalid UTF-8 in stdout`
  ```rust
  let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
  ```

- Line 124: `hoop status --json should produce valid JSON`
  ```rust
  .expect("hoop status --json should produce valid JSON");
  ```

- Line 133: `projects should be an array`
  ```rust
  let projects = json["projects"].as_array().expect("projects should be an array");
  ```

- Line 176: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 192: `Failed to run hoop status --json`
  ```rust
  .expect("Failed to run hoop status --json");
  ```

- Line 206: `Failed to spawn jq`
  ```rust
  .expect("Failed to spawn jq");
  ```

- Line 209: `Failed to open jq stdin`
  ```rust
  let mut jq_stdin = jq_output.stdin.expect("Failed to open jq stdin");
  ```

- Line 212: `Failed to write to jq stdin`
  ```rust
  .expect("Failed to write to jq stdin");
  ```

- Line 217: `Failed to read jq output`
  ```rust
  .expect("Failed to read jq output");
  ```

- Line 236: `Failed to create root dir`
  ```rust
  fs::create_dir_all(&root_dir).expect("Failed to create root dir");
  ```

- Line 241: `Failed to move project`
  ```rust
  fs::rename(project_path, &new_path).expect("Failed to move project");
  ```

- Line 260: `Failed to run hoop projects scan --yes`
  ```rust
  .expect("Failed to run hoop projects scan --yes");
  ```

- Line 262: `Invalid UTF-8 in stdout`
  ```rust
  let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
  ```

- Line 263: `Invalid UTF-8 in stderr`
  ```rust
  let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8 in stderr");
  ```

- Line 313: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 322: `Failed to run hoop status`
  ```rust
  .expect("Failed to run hoop status");
  ```

- Line 353: `Failed to run hoop status`
  ```rust
  .expect("Failed to run hoop status");
  ```

- Line 361: `Invalid UTF-8 in stdout`
  ```rust
  let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
  ```

- Line 365: `Error output should still be valid JSON`
  ```rust
  .expect("Error output should still be valid JSON");
  ```

- Line 384: `Failed to create root dir`
  ```rust
  fs::create_dir_all(&root_dir).expect("Failed to create root dir");
  ```

- Line 387: `Failed to move project`
  ```rust
  fs::rename(&project_paths[0], &new_path).expect("Failed to move project");
  ```

- Line 406: `Invalid UTF-8 in stdout`
  ```rust
  let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
  ```

- Line 446: `Failed to run hoop restore`
  ```rust
  .expect("Failed to run hoop restore");
  ```

- Line 451: `Invalid UTF-8 in stderr`
  ```rust
  let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8 in stderr");
  ```

- Line 486: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 495: `Failed to run hoop status --json`
  ```rust
  .expect("Failed to run hoop status --json");
  ```

- Line 497: `Invalid UTF-8 in stdout`
  ```rust
  let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
  ```

- Line 501: `Output should be valid JSON`
  ```rust
  .expect("Output should be valid JSON");
  ```

- Line 540: `Failed to run hoop status`
  ```rust
  .expect("Failed to run hoop status");
  ```

- Line 542: `Invalid UTF-8 in stdout`
  ```rust
  let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
  ```

- Line 543: `Invalid UTF-8 in stderr`
  ```rust
  let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8 in stderr");
  ```

- Line 554: `Error output should be valid JSON`
  ```rust
  .expect("Error output should be valid JSON");
  ```

- Line 584: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 594: `Failed to run hoop status without TTY`
  ```rust
  .expect("Failed to run hoop status without TTY");
  ```

- Line 602: `Invalid UTF-8 in stdout`
  ```rust
  let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
  ```

- Line 606: `Machine mode should produce valid JSON`
  ```rust
  .expect("Machine mode should produce valid JSON");
  ```

- Line 636: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 655: `Thread panicked`
  ```rust
  let output = handle.join().expect("Thread panicked");
  ```

#### panic (1 occurrences)

- Line 404: `Failed to run hoop with args: {:?}`
  ```rust
  .unwrap_or_else(|_| panic!("Failed to run hoop with args: {:?}", args));
  ```

#### unwrap (2 occurrences)

- Line 253: `\.unwrap\(\)`
  ```rust
  root_dir.to_str().unwrap(),
  ```

- Line 391: `\.unwrap\(\)`
  ```rust
  vec!["projects", "scan", root_dir.to_str().unwrap(), "--yes"],
  ```

### hoop-daemon/tests/adapter_failover.rs

Total errors: 73

#### assert (6 occurrences)

- Line 99: `Adapter build should succeed`
  ```rust
  assert!(adapter_result.is_ok(), "Adapter build should succeed");
  ```

- Line 120: `ZAI adapter build should succeed after Anthropic`
  ```rust
  assert!(adapter_result2.is_ok(), "ZAI adapter build should succeed after Anthropic");
  ```

- Line 448: `Global rule should be preserved`
  ```rust
  assert!(scopes.contains(&"global"), "Global rule should be preserved");
  ```

- Line 716: `Multi-line content should be preserved`
  ```rust
  assert!(messages[1].1.contains('\n'), "Multi-line content should be preserved");
  ```

- Line 717: `Quotes should be preserved`
  ```rust
  assert!(messages[1].1.contains('"'), "Quotes should be preserved");
  ```

- Line 718: `Code blocks should be preserved`
  ```rust
  assert!(messages[3].1.contains("```rust"), "Code blocks should be preserved");
  ```

#### assert_eq (19 occurrences)

- Line 187: `Stitch should be in hoop-agent project`
  ```rust
  assert_eq!(stitch_project, "hoop-agent", "Stitch should be in hoop-agent project");
  ```

- Line 188: `Stitch should be kind=operator`
  ```rust
  assert_eq!(stitch_kind, "operator", "Stitch should be kind=operator");
  ```

- Line 203: `All history messages should be stored`
  ```rust
  assert_eq!(msg_count, 4, "All history messages should be stored");
  ```

- Line 266: `Session should be marked as switched`
  ```rust
  assert_eq!(status, "switched", "Session should be marked as switched");
  ```

- Line 333: `Only one session should be active`
  ```rust
  assert_eq!(active_count, 1, "Only one session should be active");
  ```

- Line 343: `Active adapter should be zai`
  ```rust
  assert_eq!(active_adapter, "zai", "Active adapter should be zai");
  ```

- Line 445: `Both Reflection Ledger entries should be preserved`
  ```rust
  assert_eq!(entries.len(), 2, "Both Reflection Ledger entries should be preserved");
  ```

- Line 521: `Should have exactly one active session`
  ```rust
  assert_eq!(active.len(), 1, "Should have exactly one active session");
  ```

- Line 524: `Active adapter should be zai`
  ```rust
  assert_eq!(active_session.adapter, "zai", "Active adapter should be zai");
  ```

- Line 525: `Active model should be glm-5`
  ```rust
  assert_eq!(active_session.model, "glm-5", "Active model should be glm-5");
  ```

- Line 526: `New session should have 0 turns`
  ```rust
  assert_eq!(active_session.turn_count, 0, "New session should have 0 turns");
  ```

- Line 535: `Should have one archived session`
  ```rust
  assert_eq!(archived_sessions.len(), 1, "Should have one archived session");
  ```

- Line 621: `Created by should be hoop:agent`
  ```rust
  assert_eq!(created_by, "hoop:agent", "Created by should be hoop:agent");
  ```

- Line 632: `All 4 messages should be stored`
  ```rust
  assert_eq!(messages.len(), 4, "All 4 messages should be stored");
  ```

- Line 640: `Tool message should be preserved`
  ```rust
  assert_eq!(tool_messages.len(), 1, "Tool message should be preserved");
  ```

- Line 708: `Message count should match`
  ```rust
  assert_eq!(messages.len(), history.len(), "Message count should match");
  ```

- Line 711: `Role mismatch at message {}`
  ```rust
  assert_eq!(orig.0, retrieved.0, "Role mismatch at message {}", i);
  ```

- Line 712: `Content mismatch at message {}`
  ```rust
  assert_eq!(orig.1, retrieved.1, "Content mismatch at message {}", i);
  ```

- Line 792: `Only approved entries should appear`
  ```rust
  assert_eq!(approved.len(), 2, "Only approved entries should appear");
  ```

#### expect (44 occurrences)

- Line 26: `create temp dir`
  ```rust
  let tmp = TempDir::new().expect("create temp dir");
  ```

- Line 28: `create .hoop dir`
  ```rust
  std::fs::create_dir_all(&hoop_dir).expect("create .hoop dir");
  ```

- Line 34: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 73: `write config.yml`
  ```rust
  std::fs::write(path, yaml).expect("write config.yml");
  ```

- Line 156: `insert session`
  ```rust
  hoop_daemon::fleet::insert_agent_session(&session_row).expect("insert session");
  ```

- Line 174: `archive session as stitch`
  ```rust
  .expect("archive session as stitch");
  ```

- Line 177: `open db`
  ```rust
  let conn = rusqlite::Connection::open(hoop_daemon::fleet::db_path()).expect("open db");
  ```

- Line 185: `query stitch`
  ```rust
  .expect("query stitch");
  ```

- Line 201: `count messages`
  ```rust
  .expect("count messages");
  ```

- Line 212: `query linked stitch`
  ```rust
  .expect("query linked stitch");
  ```

- Line 249: `insert session`
  ```rust
  hoop_daemon::fleet::insert_agent_session(&session_row).expect("insert session");
  ```

- Line 253: `archive session`
  ```rust
  .expect("archive session");
  ```

- Line 256: `open db`
  ```rust
  let conn = rusqlite::Connection::open(hoop_daemon::fleet::db_path()).expect("open db");
  ```

- Line 264: `query archived session`
  ```rust
  .expect("query archived session");
  ```

- Line 311: `insert session`
  ```rust
  hoop_daemon::fleet::insert_agent_session(&session_row).expect("insert session");
  ```

- Line 323: `open db`
  ```rust
  let conn = rusqlite::Connection::open(hoop_daemon::fleet::db_path()).expect("open db");
  ```

- Line 331: `count active`
  ```rust
  .expect("count active");
  ```

- Line 341: `get active adapter`
  ```rust
  .expect("get active adapter");
  ```

- Line 392: `insert entry 1`
  ```rust
  hoop_daemon::fleet::insert_reflection_entry(&entry1).expect("insert entry 1");
  ```

- Line 393: `insert entry 2`
  ```rust
  hoop_daemon::fleet::insert_reflection_entry(&entry2).expect("insert entry 2");
  ```

- Line 415: `insert session`
  ```rust
  hoop_daemon::fleet::insert_agent_session(&session_row).expect("insert session");
  ```

- Line 417: `archive session`
  ```rust
  .expect("archive session");
  ```

- Line 439: `insert new session`
  ```rust
  hoop_daemon::fleet::insert_agent_session(&new_session_row).expect("insert new session");
  ```

- Line 443: `list approved entries`
  ```rust
  .expect("list approved entries");
  ```

- Line 484: `insert old session`
  ```rust
  hoop_daemon::fleet::insert_agent_session(&old_session).expect("insert old session");
  ```

- Line 488: `archive old session`
  ```rust
  .expect("archive old session");
  ```

- Line 510: `insert new session`
  ```rust
  hoop_daemon::fleet::insert_agent_session(&new_session).expect("insert new session");
  ```

- Line 514: `list sessions`
  ```rust
  .expect("list sessions");
  ```

- Line 530: `list sessions`
  ```rust
  .expect("list sessions")
  ```

- Line 579: `insert session`
  ```rust
  hoop_daemon::fleet::insert_agent_session(&session_row).expect("insert session");
  ```

- Line 600: `archive as stitch`
  ```rust
  .expect("archive as stitch");
  ```

- Line 603: `open db`
  ```rust
  let conn = rusqlite::Connection::open(hoop_daemon::fleet::db_path()).expect("open db");
  ```

- Line 611: `query stitch metadata`
  ```rust
  .expect("query stitch metadata");
  ```

- Line 626: `prepare query`
  ```rust
  .expect("prepare query")
  ```

- Line 628: `query messages`
  ```rust
  .expect("query messages")
  ```

- Line 676: `insert session`
  ```rust
  hoop_daemon::fleet::insert_agent_session(&session_row).expect("insert session");
  ```

- Line 694: `archive as stitch`
  ```rust
  .expect("archive as stitch");
  ```

- Line 697: `open db`
  ```rust
  let conn = rusqlite::Connection::open(hoop_daemon::fleet::db_path()).expect("open db");
  ```

- Line 701: `prepare query`
  ```rust
  .expect("prepare query")
  ```

- Line 703: `query messages`
  ```rust
  .expect("query messages")
  ```

- Line 765: `insert entry 1`
  ```rust
  hoop_daemon::fleet::insert_reflection_entry(&entry1).expect("insert entry 1");
  ```

- Line 766: `insert entry 2`
  ```rust
  hoop_daemon::fleet::insert_reflection_entry(&entry2).expect("insert entry 2");
  ```

- Line 786: `insert rejected`
  ```rust
  hoop_daemon::fleet::insert_reflection_entry(&rejected).expect("insert rejected");
  ```

- Line 790: `list approved`
  ```rust
  .expect("list approved");
  ```

#### unwrap (4 occurrences)

- Line 24: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 41: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 101: `\.unwrap\(\)`
  ```rust
  let adapter = adapter_result.unwrap();
  ```

- Line 122: `\.unwrap\(\)`
  ```rust
  let adapter2 = adapter_result2.unwrap();
  ```

### hoop-daemon/tests/adapter_failover_integration.rs

Total errors: 77

#### assert (2 occurrences)

- Line 73: `Adapter build should succeed`
  ```rust
  assert!(adapter_result.is_ok(), "Adapter build should succeed");
  ```

- Line 93: `ZAI adapter build should succeed after Anthropic`
  ```rust
  assert!(adapter_result2.is_ok(), "ZAI adapter build should succeed after Anthropic");
  ```

#### assert_eq (10 occurrences)

- Line 173: `Stitch should be created`
  ```rust
  assert_eq!(stitch_count, 1, "Stitch should be created");
  ```

- Line 184: `Stitch should be in hoop-agent project`
  ```rust
  assert_eq!(stitch_project, "hoop-agent", "Stitch should be in hoop-agent project");
  ```

- Line 185: `Stitch should be kind=operator`
  ```rust
  assert_eq!(stitch_kind, "operator", "Stitch should be kind=operator");
  ```

- Line 213: `Session should be marked as switched`
  ```rust
  assert_eq!(status, "switched", "Session should be marked as switched");
  ```

- Line 367: `Cost should be preserved`
  ```rust
  assert_eq!(cost_usd, 0.125, "Cost should be preserved");
  ```

- Line 368: `Input tokens should be preserved`
  ```rust
  assert_eq!(input_tokens, 5000, "Input tokens should be preserved");
  ```

- Line 369: `Output tokens should be preserved`
  ```rust
  assert_eq!(output_tokens, 2000, "Output tokens should be preserved");
  ```

- Line 370: `Turn count should be preserved`
  ```rust
  assert_eq!(turn_count, 7, "Turn count should be preserved");
  ```

- Line 543: `All approved rules should be preserved`
  ```rust
  assert_eq!(entries.len(), 2, "All approved rules should be preserved");
  ```

- Line 671: `Only approved rules should be returned`
  ```rust
  assert_eq!(entries.len(), 2, "Only approved rules should be returned");
  ```

#### expect (15 occurrences)

- Line 27: `create temp dir`
  ```rust
  let tmp = TempDir::new().expect("create temp dir");
  ```

- Line 29: `create .hoop dir`
  ```rust
  fs::create_dir_all(&hoop_dir).expect("create .hoop dir");
  ```

- Line 35: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 140: `load active session`
  ```rust
  .expect("load active session")
  ```

- Line 141: `should have active session`
  ```rust
  .expect("should have active session");
  ```

- Line 161: `archive session as stitch`
  ```rust
  fleet::archive_session_as_stitch(&session_row, &history).expect("archive session as stitch");
  ```

- Line 165: `archive agent session`
  ```rust
  .expect("archive agent session");
  ```

- Line 356: `archive session`
  ```rust
  .expect("archive session");
  ```

- Line 540: `list approved entries`
  ```rust
  let entries = fleet::list_approved_reflection_entries(None).expect("list approved entries");
  ```

- Line 607: `load active session should succeed`
  ```rust
  .expect("load active session should succeed")
  ```

- Line 608: `should have an active session`
  ```rust
  .expect("should have an active session");
  ```

- Line 668: `list approved entries`
  ```rust
  let entries = fleet::list_approved_reflection_entries(None).expect("list approved entries");
  ```

- Line 706: `load active session`
  ```rust
  .expect("load active session")
  ```

- Line 707: `should have active session`
  ```rust
  .expect("should have active session");
  ```

- Line 711: `archive as stitch`
  ```rust
  fleet::archive_session_as_stitch(&session_row, &history).expect("archive as stitch");
  ```

#### unwrap (50 occurrences)

- Line 25: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 42: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 75: `\.unwrap\(\)`
  ```rust
  let adapter = adapter_result.unwrap();
  ```

- Line 95: `\.unwrap\(\)`
  ```rust
  let adapter2 = adapter_result2.unwrap();
  ```

- Line 119: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 127: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 136: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 172: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 182: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 198: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 211: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 226: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 241: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 263: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 271: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 289: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 298: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 312: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 324: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 344: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 352: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 365: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 385: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 397: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 406: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 411: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 423: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 432: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 437: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 449: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 458: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 464: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 474: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 484: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 500: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 510: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 518: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 528: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 537: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 572: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 584: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 602: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 624: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 641: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 650: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 657: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 665: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 694: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 702: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 720: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

### hoop-daemon/tests/adapter_failover_test.rs

Total errors: 114

#### assert (1 occurrences)

- Line 899: `Should have performed at least 6 health checks over 30s`
  ```rust
  assert!(checks >= 6, "Should have performed at least 6 health checks over 30s");
  ```

#### assert_eq (26 occurrences)

- Line 161: `Daemon should be healthy`
  ```rust
  assert_eq!(health["status"], "ok", "Daemon should be healthy");
  ```

- Line 165: `Agent spawn should succeed`
  ```rust
  assert_eq!(spawn_resp["status"], "ok", "Agent spawn should succeed");
  ```

- Line 173: `Agent should be active`
  ```rust
  assert_eq!(status["active"], true, "Agent should be active");
  ```

- Line 177: `Daemon should remain healthy after 5xx`
  ```rust
  assert_eq!(health["status"], "ok", "Daemon should remain healthy after 5xx");
  ```

- Line 193: `Agent spawn should succeed`
  ```rust
  assert_eq!(spawn_resp["status"], "ok", "Agent spawn should succeed");
  ```

- Line 204: `Agent should be active`
  ```rust
  assert_eq!(status["active"], true, "Agent should be active");
  ```

- Line 216: `Adapter switch should succeed`
  ```rust
  assert_eq!(switch_resp["status"], "ok", "Adapter switch should succeed");
  ```

- Line 245: `Should have exactly 1 active session`
  ```rust
  assert_eq!(active_count, 1, "Should have exactly 1 active session");
  ```

- Line 246: `Should have 1 switched (archived) session`
  ```rust
  assert_eq!(archived_count, 1, "Should have 1 switched (archived) session");
  ```

- Line 253: `Agent should still be active`
  ```rust
  assert_eq!(status["active"], true, "Agent should still be active");
  ```

- Line 254: `Adapter should be zai`
  ```rust
  assert_eq!(status["adapter"], "zai", "Adapter should be zai");
  ```

- Line 255: `Model should be glm-5`
  ```rust
  assert_eq!(status["model"], "glm-5", "Model should be glm-5");
  ```

- Line 444: `Should have 2 switched sessions`
  ```rust
  assert_eq!(archived_count, 2, "Should have 2 switched sessions");
  ```

- Line 580: `Daemon should remain healthy`
  ```rust
  assert_eq!(health["status"], "ok", "Daemon should remain healthy");
  ```

- Line 603: `Agent spawn should succeed`
  ```rust
  assert_eq!(spawn_resp["status"], "ok", "Agent spawn should succeed");
  ```

- Line 614: `Agent should be active`
  ```rust
  assert_eq!(status["active"], true, "Agent should be active");
  ```

- Line 651: `Agent should still be active`
  ```rust
  assert_eq!(status["active"], true, "Agent should still be active");
  ```

- Line 657: `Model should be glm-5`
  ```rust
  assert_eq!(status["model"], "glm-5", "Model should be glm-5");
  ```

- Line 676: `Should have exactly 1 active session`
  ```rust
  assert_eq!(active_count, 1, "Should have exactly 1 active session");
  ```

- Line 677: `Should have 1 switched (archived) session`
  ```rust
  assert_eq!(archived_count, 1, "Should have 1 switched (archived) session");
  ```

- Line 723: `Daemon should remain healthy after hot-reload`
  ```rust
  assert_eq!(health["status"], "ok", "Daemon should remain healthy after hot-reload");
  ```

- Line 818: `Daemon should be healthy initially`
  ```rust
  assert_eq!(health["status"], "ok", "Daemon should be healthy initially");
  ```

- Line 952: `Switch to ZAI should succeed`
  ```rust
  assert_eq!(switch_resp["status"], "ok", "Switch to ZAI should succeed");
  ```

- Line 959: `Agent should be active after switch`
  ```rust
  assert_eq!(status["active"], true, "Agent should be active after switch");
  ```

- Line 960: `Should be using ZAI adapter`
  ```rust
  assert_eq!(status["adapter"], "zai", "Should be using ZAI adapter");
  ```

- Line 964: `Daemon should be healthy after recovery`
  ```rust
  assert_eq!(final_health["status"], "ok", "Daemon should be healthy after recovery");
  ```

#### bail (1 occurrences)

- Line 49: `Daemon did not become ready`
  ```rust
  anyhow::bail!("Daemon did not become ready");
  ```

#### expect (82 occurrences)

- Line 155: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 157: `Failed to create client`
  ```rust
  let client = FailoverClient::new(_base_url.clone()).await.expect("Failed to create client");
  ```

- Line 160: `Health check failed`
  ```rust
  let health = client.healthz().await.expect("Health check failed");
  ```

- Line 164: `Failed to spawn agent`
  ```rust
  let spawn_resp = client.spawn_agent().await.expect("Failed to spawn agent");
  ```

- Line 172: `Failed to get agent status`
  ```rust
  .expect("Failed to get agent status");
  ```

- Line 176: `Health check failed`
  ```rust
  let health = client.healthz().await.expect("Health check failed");
  ```

- Line 187: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 189: `Failed to create client`
  ```rust
  let client = FailoverClient::new(_base_url.clone()).await.expect("Failed to create client");
  ```

- Line 192: `Failed to spawn agent`
  ```rust
  let spawn_resp = client.spawn_agent().await.expect("Failed to spawn agent");
  ```

- Line 197: `Should have session_db_id`
  ```rust
  .expect("Should have session_db_id");
  ```

- Line 203: `Failed to get agent status`
  ```rust
  .expect("Failed to get agent status");
  ```

- Line 215: `Failed to switch adapter`
  ```rust
  .expect("Failed to switch adapter");
  ```

- Line 220: `Should have new session_db_id`
  ```rust
  .expect("Should have new session_db_id");
  ```

- Line 232: `Failed to list sessions`
  ```rust
  .expect("Failed to list sessions");
  ```

- Line 252: `Failed to get agent status`
  ```rust
  .expect("Failed to get agent status");
  ```

- Line 265: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 267: `Failed to create client`
  ```rust
  let client = FailoverClient::new(_base_url.clone()).await.expect("Failed to create client");
  ```

- Line 270: `Failed to spawn agent`
  ```rust
  let spawn_resp = client.spawn_agent().await.expect("Failed to spawn agent");
  ```

- Line 275: `Should have session_db_id`
  ```rust
  .expect("Should have session_db_id");
  ```

- Line 281: `Failed to switch adapter`
  ```rust
  .expect("Failed to switch adapter");
  ```

- Line 287: `Failed to list sessions`
  ```rust
  .expect("Failed to list sessions");
  ```

- Line 293: `Should find archived session`
  ```rust
  .expect("Should find archived session");
  ```

- Line 311: `Failed to query stitch from fleet.db`
  ```rust
  .expect("Failed to query stitch from fleet.db");
  ```

- Line 344: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 346: `Failed to create client`
  ```rust
  let client = FailoverClient::new(_base_url.clone()).await.expect("Failed to create client");
  ```

- Line 367: `Failed to insert reflection entry`
  ```rust
  .expect("Failed to insert reflection entry");
  ```

- Line 370: `Failed to spawn agent`
  ```rust
  let _spawn_resp = client.spawn_agent().await.expect("Failed to spawn agent");
  ```

- Line 374: `Failed to switch adapter`
  ```rust
  .expect("Failed to switch adapter");
  ```

- Line 378: `Failed to list reflection entries`
  ```rust
  .expect("Failed to list reflection entries");
  ```

- Line 389: `Entry should exist`
  ```rust
  .expect("Entry should exist");
  ```

- Line 402: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 404: `Failed to create client`
  ```rust
  let client = FailoverClient::new(_base_url.clone()).await.expect("Failed to create client");
  ```

- Line 407: `Failed to spawn agent`
  ```rust
  let spawn_resp = client.spawn_agent().await.expect("Failed to spawn agent");
  ```

- Line 411: `Should have session_db_id`
  ```rust
  .expect("Should have session_db_id");
  ```

- Line 417: `Failed to switch adapter`
  ```rust
  .expect("Failed to switch adapter");
  ```

- Line 423: `Failed to switch adapter back`
  ```rust
  .expect("Failed to switch adapter back");
  ```

- Line 427: `Should have second session_db_id`
  ```rust
  .expect("Should have second session_db_id");
  ```

- Line 433: `Failed to list sessions`
  ```rust
  .expect("Failed to list sessions");
  ```

- Line 450: `Should find first archived session`
  ```rust
  .expect("Should find first archived session");
  ```

- Line 454: `Should find second archived session`
  ```rust
  .expect("Should find second archived session");
  ```

- Line 480: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 482: `Failed to create client`
  ```rust
  let client = FailoverClient::new(_base_url.clone()).await.expect("Failed to create client");
  ```

- Line 485: `Failed to spawn agent`
  ```rust
  let _spawn_resp = client.spawn_agent().await.expect("Failed to spawn agent");
  ```

- Line 506: `Failed to insert reflection entry`
  ```rust
  .expect("Failed to insert reflection entry");
  ```

- Line 512: `Failed to switch adapter`
  ```rust
  .expect("Failed to switch adapter");
  ```

- Line 518: `Failed to get agent status`
  ```rust
  .expect("Failed to get agent status");
  ```

- Line 524: `Failed to list reflection entries`
  ```rust
  .expect("Failed to list reflection entries");
  ```

- Line 539: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 541: `Failed to create client`
  ```rust
  let client = FailoverClient::new(_base_url.clone()).await.expect("Failed to create client");
  ```

- Line 544: `Failed to spawn agent`
  ```rust
  let _spawn_resp = client.spawn_agent().await.expect("Failed to spawn agent");
  ```

- Line 567: `Switch 1 should complete`
  ```rust
  .expect("Switch 1 should complete");
  ```

- Line 570: `Switch 2 should complete`
  ```rust
  .expect("Switch 2 should complete");
  ```

- Line 579: `Health check failed`
  ```rust
  let health = client.healthz().await.expect("Health check failed");
  ```

- Line 597: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 599: `Failed to create client`
  ```rust
  let client = FailoverClient::new(base_url.clone()).await.expect("Failed to create client");
  ```

- Line 602: `Failed to spawn agent`
  ```rust
  let spawn_resp = client.spawn_agent().await.expect("Failed to spawn agent");
  ```

- Line 607: `Should have session_db_id`
  ```rust
  .expect("Should have session_db_id");
  ```

- Line 613: `Failed to get agent status`
  ```rust
  .expect("Failed to get agent status");
  ```

- Line 639: `Failed to write updated config.yml`
  ```rust
  .expect("Failed to write updated config.yml");
  ```

- Line 650: `Failed to get agent status after config reload`
  ```rust
  .expect("Failed to get agent status after config reload");
  ```

- Line 663: `Failed to list sessions`
  ```rust
  .expect("Failed to list sessions");
  ```

- Line 683: `Should find original archived session`
  ```rust
  .expect("Should find original archived session");
  ```

- Line 700: `Failed to query stitch from fleet.db`
  ```rust
  .expect("Failed to query stitch from fleet.db");
  ```

- Line 722: `Health check failed`
  ```rust
  let health = client.healthz().await.expect("Health check failed");
  ```

- Line 805: `Failed to start mock Anthropic server`
  ```rust
  .expect("Failed to start mock Anthropic server");
  ```

- Line 812: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 814: `Failed to create client`
  ```rust
  let client = FailoverClient::new(_base_url.clone()).await.expect("Failed to create client");
  ```

- Line 817: `Health check failed`
  ```rust
  let health = client.healthz().await.expect("Health check failed");
  ```

- Line 837: `Failed to write config with mock server URL`
  ```rust
  .expect("Failed to write config with mock server URL");
  ```

- Line 853: `Health check failed`
  ```rust
  let health_after = client.healthz().await.expect("Health check failed");
  ```

- Line 865: `Ready endpoint request failed`
  ```rust
  .expect("Ready endpoint request failed");
  ```

- Line 882: `Health check failed`
  ```rust
  let health = client.healthz().await.expect("Health check failed");
  ```

- Line 892: `Health check failed`
  ```rust
  let final_health = client.healthz().await.expect("Health check failed");
  ```

- Line 910: `Failed to start mock Anthropic server`
  ```rust
  .expect("Failed to start mock Anthropic server");
  ```

- Line 917: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 919: `Failed to create client`
  ```rust
  let client = FailoverClient::new(_base_url.clone()).await.expect("Failed to create client");
  ```

- Line 922: `Health check failed`
  ```rust
  let health = client.healthz().await.expect("Health check failed");
  ```

- Line 937: `Failed to write config`
  ```rust
  std::fs::write(&config_path, mock_config).expect("Failed to write config");
  ```

- Line 943: `Health check failed`
  ```rust
  let health_after_503 = client.healthz().await.expect("Health check failed");
  ```

- Line 950: `Adapter switch should succeed`
  ```rust
  .expect("Adapter switch should succeed");
  ```

- Line 958: `Failed to get agent status`
  ```rust
  .expect("Failed to get agent status");
  ```

- Line 963: `Health check failed`
  ```rust
  let final_health = client.healthz().await.expect("Health check failed");
  ```

#### unwrap (4 occurrences)

- Line 310: `\.unwrap\(\)`
  ```rust
  let stitch_row_opt = fleet::load_stitch_by_id(stitch_id.as_ref().unwrap())
  ```

- Line 318: `\.unwrap\(\)`
  ```rust
  let stitch_row = stitch_row_opt.unwrap();
  ```

- Line 699: `\.unwrap\(\)`
  ```rust
  let stitch_row_opt = fleet::load_stitch_by_id(stitch_id.as_ref().unwrap())
  ```

- Line 707: `\.unwrap\(\)`
  ```rust
  let stitch_row = stitch_row_opt.unwrap();
  ```

### hoop-daemon/tests/agent_turn_audit_trail.rs

Total errors: 30

#### assert (1 occurrences)

- Line 164: `System message should reference the turn_id`
  ```rust
  assert!(message_content.contains(turn_id), "System message should reference the turn_id");
  ```

#### assert_eq (7 occurrences)

- Line 139: `created_by_actor should be set`
  ```rust
  assert_eq!(stitch_row.1, Some(actor.to_string()), "created_by_actor should be set");
  ```

- Line 140: `created_by_session_id should be set`
  ```rust
  assert_eq!(stitch_row.2, Some(session_id.to_string()), "created_by_session_id should be set");
  ```

- Line 141: `created_by_adapter should be set`
  ```rust
  assert_eq!(stitch_row.3, Some(adapter.to_string()), "created_by_adapter should be set");
  ```

- Line 142: `created_by_model should be set`
  ```rust
  assert_eq!(stitch_row.4, Some(model.to_string()), "created_by_model should be set");
  ```

- Line 143: `turn_id should be set`
  ```rust
  assert_eq!(stitch_row.5, Some(turn_id.to_string()), "turn_id should be set");
  ```

- Line 154: `Should have one system note with turn reference`
  ```rust
  assert_eq!(message_count, 1, "Should have one system note with turn reference");
  ```

- Line 326: `session_id, turn_id));
`
  ```rust
  assert_eq!(turn_url, format!("/agent?session={}&turn={}", session_id, turn_id));
  ```

#### expect (18 occurrences)

- Line 25: `create temp dir`
  ```rust
  let tmp = TempDir::new().expect("create temp dir");
  ```

- Line 27: `create .hoop dir`
  ```rust
  std::fs::create_dir_all(&hoop_dir).expect("create .hoop dir");
  ```

- Line 33: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 83: `insert draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");
  ```

- Line 87: `get draft`
  ```rust
  .expect("get draft")
  ```

- Line 88: `draft exists`
  ```rust
  .expect("draft exists");
  ```

- Line 114: `create stitch with audit`
  ```rust
  .expect("create stitch with audit");
  ```

- Line 118: `open fleet.db`
  ```rust
  .expect("open fleet.db");
  ```

- Line 136: `query stitch`
  ```rust
  .expect("query stitch");
  ```

- Line 152: `count system messages`
  ```rust
  .expect("count system messages");
  ```

- Line 162: `get system message content`
  ```rust
  .expect("get system message content");
  ```

- Line 208: `write audit row`
  ```rust
  .expect("write audit row");
  ```

- Line 212: `query audit rows`
  ```rust
  .expect("query audit rows");
  ```

- Line 217: `should find audit row for our stitch`
  ```rust
  .expect("should find audit row for our stitch");
  ```

- Line 227: `args_json should be valid JSON`
  ```rust
  .expect("args_json should be valid JSON");
  ```

- Line 292: `create stitch for reconstruction`
  ```rust
  .expect("create stitch for reconstruction");
  ```

- Line 296: `open fleet.db`
  ```rust
  .expect("open fleet.db");
  ```

- Line 311: `query stitch for reconstruction`
  ```rust
  .expect("query stitch for reconstruction");
  ```

#### unwrap (4 occurrences)

- Line 23: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 40: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 226: `\.unwrap\(\)`
  ```rust
  .map(|s| serde_json::from_str(s).unwrap())
  ```

- Line 251: `\.unwrap\(\)`
  ```rust
  let extracted_session_id = actor.strip_prefix("hoop:agent:").unwrap();
  ```

### hoop-daemon/tests/backup_config_deserialization.rs

Total errors: 7

#### expect (7 occurrences)

- Line 46: `YAML should parse`
  ```rust
  serde_yaml::from_str(yaml_input).expect("YAML should parse");
  ```

- Line 49: `YAML→JSON conversion should succeed`
  ```rust
  .expect("YAML→JSON conversion should succeed");
  ```

- Line 52: `BackupFileConfig should deserialize`
  ```rust
  .expect("BackupFileConfig should deserialize");
  ```

- Line 72: `YAML should parse`
  ```rust
  serde_yaml::from_str(yaml_input).expect("YAML should parse");
  ```

- Line 75: `YAML→JSON conversion should succeed`
  ```rust
  .expect("YAML→JSON conversion should succeed");
  ```

- Line 78: `BackupFileConfig should deserialize`
  ```rust
  .expect("BackupFileConfig should deserialize");
  ```

- Line 97: `Should deserialize from JSON directly`
  ```rust
  .expect("Should deserialize from JSON directly");
  ```

### hoop-daemon/tests/backup_restore_cycle.rs

Total errors: 63

#### assert (13 occurrences)

- Line 67: `State should be deleted`
  ```rust
  assert!(!hoop_dir.exists(), "State should be deleted");
  ```

- Line 144: `Should return None when credentials missing`
  ```rust
  assert!(creds.is_none(), "Should return None when credentials missing");
  ```

- Line 153: `Should succeed when encryption disabled`
  ```rust
  assert!(creds.is_some(), "Should succeed when encryption disabled");
  ```

- Line 158: `age_key should be None when encryption disabled`
  ```rust
  assert!(creds.age_key.is_none(), "age_key should be None when encryption disabled");
  ```

- Line 166: `Should succeed when age key provided`
  ```rust
  assert!(creds.is_some(), "Should succeed when age key provided");
  ```

- Line 169: `age_key should be Some when encryption enabled`
  ```rust
  assert!(creds.age_key.is_some(), "age_key should be Some when encryption enabled");
  ```

- Line 178: `Should return None when age key missing but encryption enabled`
  ```rust
  assert!(creds.is_none(), "Should return None when age key missing but encryption enabled");
  ```

- Line 229: `Encrypted file should exist`
  ```rust
  assert!(encrypted_file.exists(), "Encrypted file should exist");
  ```

- Line 311: `Backup should fail when encryption enabled but age key missing`
  ```rust
  assert!(result.is_err(), "Backup should fail when encryption enabled but age key missing");
  ```

- Line 350: `Config should have encryption enabled`
  ```rust
  assert!(config.encryption, "Config should have encryption enabled");
  ```

- Line 351: `Credentials should have age key`
  ```rust
  assert!(credentials.age_key.is_some(), "Credentials should have age key");
  ```

- Line 391: `Config should have encryption disabled`
  ```rust
  assert!(!config.encryption, "Config should have encryption disabled");
  ```

- Line 392: `Credentials should not have age key`
  ```rust
  assert!(credentials.age_key.is_none(), "Credentials should not have age key");
  ```

#### assert_eq (1 occurrences)

- Line 445: `Cron schedule should have 5 fields`
  ```rust
  assert_eq!(parts.len(), 5, "Cron schedule should have 5 fields");
  ```

#### expect (2 occurrences)

- Line 638: `age-keygen should be installed for this test`
  ```rust
  .expect("age-keygen should be installed for this test");
  ```

- Line 651: `age-keygen output should contain public key`
  ```rust
  .expect("age-keygen output should contain public key")
  ```

#### panic (1 occurrences)

- Line 641: `age-keygen failed: {}`
  ```rust
  panic!("age-keygen failed: {}", String::from_utf8_lossy(&output.stderr));
  ```

#### unwrap (46 occurrences)

- Line 21: `\.unwrap\(\)`
  ```rust
  let test_dir = TempDir::new().unwrap();
  ```

- Line 23: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&hoop_dir).unwrap();
  ```

- Line 35: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&snapshot_dir).unwrap();
  ```

- Line 42: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 48: `\.unwrap\(\)`
  ```rust
  copy_dir_recursive(&attachments_src, &attachments_dst).unwrap();
  ```

- Line 56: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 61: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 64: `\.unwrap\(\)`
  ```rust
  fs::remove_dir_all(&hoop_dir).unwrap();
  ```

- Line 70: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&hoop_dir).unwrap();
  ```

- Line 71: `\.unwrap\(\)`
  ```rust
  fs::copy(snapshot_dir.join("fleet.db"), hoop_dir.join("fleet.db")).unwrap();
  ```

- Line 76: `\.unwrap\(\)`
  ```rust
  copy_dir_recursive(&attachments_src, &attachments_dst).unwrap();
  ```

- Line 83: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 88: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 155: `\.unwrap\(\)`
  ```rust
  let creds = creds.unwrap();
  ```

- Line 168: `\.unwrap\(\)`
  ```rust
  let creds = creds.unwrap();
  ```

- Line 170: `\.unwrap\(\)`
  ```rust
  assert_eq!(creds.age_key.unwrap(), "age1test-key-for-encryption");
  ```

- Line 200: `\.unwrap\(\)`
  ```rust
  let key_dir = TempDir::new().unwrap();
  ```

- Line 205: `\.unwrap\(\)`
  ```rust
  let test_dir = TempDir::new().unwrap();
  ```

- Line 207: `\.unwrap\(\)`
  ```rust
  fs::write(&original_file, b"test fleet.db data for encryption").unwrap();
  ```

- Line 231: `\.unwrap\(\)`
  ```rust
  let original_data = fs::read(&original_file).unwrap();
  ```

- Line 232: `\.unwrap\(\)`
  ```rust
  let encrypted_data = fs::read(&encrypted_file).unwrap();
  ```

- Line 241: `\.unwrap\(\)`
  ```rust
  key_file.to_str().unwrap(),
  ```

- Line 264: `\.unwrap\(\)`
  ```rust
  let decrypted_data = fs::read(&decrypted_file).unwrap();
  ```

- Line 468: `\.unwrap\(\)`
  ```rust
  let data = fs::read(&fleet_db_path).unwrap();
  ```

- Line 478: `\.unwrap\(\)`
  ```rust
  let data = fs::read(&config_yml_path).unwrap();
  ```

- Line 488: `\.unwrap\(\)`
  ```rust
  let data = fs::read(&projects_yaml_path).unwrap();
  ```

- Line 505: `\.unwrap\(\)`
  ```rust
  let size = fs::metadata(entry.path()).unwrap().len();
  ```

- Line 528: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(db_path.parent().unwrap()).unwrap();
  ```

- Line 528: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(db_path.parent().unwrap()).unwrap();
  ```

- Line 531: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 532: `\.unwrap\(\)`
  ```rust
  conn.pragma_update(None, "journal_mode", "WAL").unwrap();
  ```

- Line 548: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 560: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 566: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 571: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 579: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&attachments_dir).unwrap();
  ```

- Line 583: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&stitch_dir).unwrap();
  ```

- Line 584: `\.unwrap\(\)`
  ```rust
  fs::write(stitch_dir.join("audio.m4a"), b"fake audio data").unwrap();
  ```

- Line 585: `\.unwrap\(\)`
  ```rust
  fs::write(stitch_dir.join("image.png"), b"\x89PNG\r\n\x1a\nfake png").unwrap();
  ```

- Line 588: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&bead_dir).unwrap();
  ```

- Line 589: `\.unwrap\(\)`
  ```rust
  fs::write(bead_dir.join("screenshot.jpg"), b"fake jpg data").unwrap();
  ```

- Line 601: `\.unwrap\(\)`
  ```rust
  fs::write(hoop_dir.join("config.yml"), config_yml).unwrap();
  ```

- Line 608: `\.unwrap\(\)`
  ```rust
  fs::write(hoop_dir.join("projects.yaml"), projects_yaml).unwrap();
  ```

- Line 615: `\.unwrap\(\)`
  ```rust
  let rel = entry.path().strip_prefix(src).unwrap();
  ```

- Line 617: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(dst_path.parent().unwrap())?;
  ```

- Line 645: `\.unwrap\(\)`
  ```rust
  let key_content = fs::read_to_string(key_file).unwrap();
  ```

### hoop-daemon/tests/bead_created_by_hoop_broadcast.rs

Total errors: 4

#### expect (4 occurrences)

- Line 70: `Fleet notification should be received within 200ms`
  ```rust
  .expect("Fleet notification should be received within 200ms")
  ```

- Line 71: `Fleet notification channel should not be closed`
  ```rust
  .expect("Fleet notification channel should not be closed");
  ```

- Line 110: `Should serialize`
  ```rust
  let json = serde_json::to_string(&event).expect("Should serialize");
  ```

- Line 111: `Should deserialize`
  ```rust
  let parsed: BeadCreatedByHoopData = serde_json::from_str(&json).expect("Should deserialize");
  ```

### hoop-daemon/tests/bead_real_line_deserialization.rs

Total errors: 8

#### expect (4 occurrences)

- Line 40: `Real br line must deserialize successfully`
  ```rust
  .expect("Real br line must deserialize successfully");
  ```

- Line 62: `Minimal bead line (without created_by/dependencies) must deserialize`
  ```rust
  .expect("Minimal bead line (without created_by/dependencies) must deserialize");
  ```

- Line 194: `Bead line with extra unknown keys must deserialize`
  ```rust
  .expect("Bead line with extra unknown keys must deserialize");
  ```

- Line 216: `Bead line with null description must deserialize`
  ```rust
  .expect("Bead line with null description must deserialize");
  ```

#### panic (4 occurrences)

- Line 89: `Status `
  ```rust
  .unwrap_or_else(|_| panic!("Status '{}' must deserialize", wire_value));
  ```

- Line 123: `Issue type `
  ```rust
  .unwrap_or_else(|_| panic!("Issue type '{}' must deserialize", wire_value));
  ```

- Line 145: `Unrecognized status `
  ```rust
  .unwrap_or_else(|_| panic!("Unrecognized status '{}' must deserialize as Unknown", wire_value));
  ```

- Line 167: `Unrecognized issue type `
  ```rust
  .unwrap_or_else(|_| panic!("Unrecognized issue type '{}' must deserialize as Unknown", wire_value));
  ```

### hoop-daemon/tests/bead_status_deserialization.rs

Total errors: 7

#### unwrap (7 occurrences)

- Line 15: `\.unwrap\(\)`
  ```rust
  serde_json::from_str::<BeadStatus>("\"open\"").unwrap(),
  ```

- Line 19: `\.unwrap\(\)`
  ```rust
  serde_json::from_str::<BeadStatus>("\"closed\"").unwrap(),
  ```

- Line 23: `\.unwrap\(\)`
  ```rust
  serde_json::from_str::<BeadStatus>("\"blocked\"").unwrap(),
  ```

- Line 27: `\.unwrap\(\)`
  ```rust
  serde_json::from_str::<BeadStatus>("\"completed\"").unwrap(),
  ```

- Line 31: `\.unwrap\(\)`
  ```rust
  serde_json::from_str::<BeadStatus>("\"done\"").unwrap(),
  ```

- Line 41: `\.unwrap\(\)`
  ```rust
  serde_json::from_str::<BeadStatus>("\"cancelled\"").unwrap(),
  ```

- Line 45: `\.unwrap\(\)`
  ```rust
  serde_json::from_str::<BeadStatus>("\"in-progress\"").unwrap(),
  ```

### hoop-daemon/tests/beads_deletion_http.rs

Total errors: 44

#### assert (2 occurrences)

- Line 356: `project-a should be degraded`
  ```rust
  assert!(degraded, "project-a should be degraded");
  ```

- Line 416: `Should be healthy initially`
  ```rust
  assert!(resp.status().is_success(), "Should be healthy initially");
  ```

#### assert_eq (1 occurrences)

- Line 361: `API should still be accessible`
  ```rust
  assert_eq!(resp.status(), 200, "API should still be accessible");
  ```

#### expect (6 occurrences)

- Line 111: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 114: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 184: `project-a should be in degraded list`
  ```rust
  .expect("project-a should be in degraded list");
  ```

- Line 313: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 316: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 410: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

#### unwrap (35 occurrences)

- Line 83: `\.unwrap\(\)`
  ```rust
  let (project_a_dir, project_a_path) = setup_project_dir("project-a").unwrap();
  ```

- Line 84: `\.unwrap\(\)`
  ```rust
  let (project_b_dir, project_b_path) = setup_project_dir("project-b").unwrap();
  ```

- Line 85: `\.unwrap\(\)`
  ```rust
  let (project_c_dir, project_c_path) = setup_project_dir("project-c").unwrap();
  ```

- Line 109: `\.unwrap\(\)`
  ```rust
  let hoop_dir = config.control_socket_path.parent().unwrap();
  ```

- Line 128: `\.unwrap\(\)`
  ```rust
  let body: ReadinessResponse = resp.json().await.unwrap();
  ```

- Line 146: `\.unwrap\(\)`
  ```rust
  fs::remove_dir_all(&beads_a_path).unwrap();
  ```

- Line 157: `\.unwrap\(\)`
  ```rust
  let body: ReadinessResponse = resp.json().await.unwrap();
  ```

- Line 179: `\.unwrap\(\)`
  ```rust
  let degraded = degraded_response.unwrap();
  ```

- Line 204: `\.unwrap\(\)`
  ```rust
  let resp = client.get(&projects_url).send().await.unwrap();
  ```

- Line 207: `\.unwrap\(\)`
  ```rust
  let projects: Vec<ProjectStatus> = resp.json().await.unwrap();
  ```

- Line 209: `\.unwrap\(\)`
  ```rust
  let project_a_api = projects.iter().find(|p| p.name == "project-a").unwrap();
  ```

- Line 215: `\.unwrap\(\)`
  ```rust
  let project_b_api = projects.iter().find(|p| p.name == "project-b").unwrap();
  ```

- Line 222: `\.unwrap\(\)`
  ```rust
  let project_c_api = projects.iter().find(|p| p.name == "project-c").unwrap();
  ```

- Line 230: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&beads_a_path).unwrap();
  ```

- Line 232: `\.unwrap\(\)`
  ```rust
  fs::write(&issues_path, "").unwrap();
  ```

- Line 244: `\.unwrap\(\)`
  ```rust
  let body: ReadinessResponse = resp.json().await.unwrap();
  ```

- Line 261: `\.unwrap\(\)`
  ```rust
  let resp = client.get(&projects_url).send().await.unwrap();
  ```

- Line 262: `\.unwrap\(\)`
  ```rust
  let projects: Vec<ProjectStatus> = resp.json().await.unwrap();
  ```

- Line 285: `\.unwrap\(\)`
  ```rust
  let (project_a_dir, project_a_path) = setup_project_dir("project-a").unwrap();
  ```

- Line 286: `\.unwrap\(\)`
  ```rust
  let (project_b_dir, project_b_path) = setup_project_dir("project-b").unwrap();
  ```

- Line 287: `\.unwrap\(\)`
  ```rust
  let (project_c_dir, project_c_path) = setup_project_dir("project-c").unwrap();
  ```

- Line 311: `\.unwrap\(\)`
  ```rust
  let hoop_dir = config.control_socket_path.parent().unwrap();
  ```

- Line 334: `\.unwrap\(\)`
  ```rust
  let resp_before = client.get(&metrics_url).send().await.unwrap();
  ```

- Line 335: `\.unwrap\(\)`
  ```rust
  let _metrics_before = resp_before.text().await.unwrap();
  ```

- Line 339: `\.unwrap\(\)`
  ```rust
  fs::remove_dir_all(&beads_a_path).unwrap();
  ```

- Line 347: `\.unwrap\(\)`
  ```rust
  let body: ReadinessResponse = resp.json().await.unwrap();
  ```

- Line 360: `\.unwrap\(\)`
  ```rust
  let resp = client.get(&projects_url).send().await.unwrap();
  ```

- Line 363: `\.unwrap\(\)`
  ```rust
  let projects: Vec<ProjectStatus> = resp.json().await.unwrap();
  ```

- Line 366: `\.unwrap\(\)`
  ```rust
  let resp_during = client.get(&metrics_url).send().await.unwrap();
  ```

- Line 367: `\.unwrap\(\)`
  ```rust
  let metrics_during = resp_during.text().await.unwrap();
  ```

- Line 376: `\.unwrap\(\)`
  ```rust
  let project_b = projects.iter().find(|p| p.name == "project-b").unwrap();
  ```

- Line 377: `\.unwrap\(\)`
  ```rust
  let project_c = projects.iter().find(|p| p.name == "project-c").unwrap();
  ```

- Line 392: `\.unwrap\(\)`
  ```rust
  let resp = client.get(&beads_url).send().await.unwrap();
  ```

- Line 415: `\.unwrap\(\)`
  ```rust
  let resp = client.get(&readyz_url).send().await.unwrap();
  ```

- Line 418: `\.unwrap\(\)`
  ```rust
  let body: ReadinessResponse = resp.json().await.unwrap();
  ```

### hoop-daemon/tests/beads_removal_recovery.rs

Total errors: 38

#### expect (29 occurrences)

- Line 26: `Failed to create temp dir`
  ```rust
  let project_dir = tempfile::tempdir().expect("Failed to create temp dir");
  ```

- Line 31: `Failed to create .beads dir`
  ```rust
  fs::create_dir_all(&beads_dir).expect("Failed to create .beads dir");
  ```

- Line 35: `Failed to create issues.jsonl`
  ```rust
  fs::write(&issues_path, b"").expect("Failed to create issues.jsonl");
  ```

- Line 39: `Failed to create events.jsonl`
  ```rust
  fs::write(&events_path, b"").expect("Failed to create events.jsonl");
  ```

- Line 47: `Failed to remove .beads dir`
  ```rust
  fs::remove_dir_all(&beads_dir).expect("Failed to remove .beads dir");
  ```

- Line 53: `Failed to recreate .beads dir`
  ```rust
  fs::create_dir_all(&beads_dir).expect("Failed to recreate .beads dir");
  ```

- Line 56: `Failed to recreate issues.jsonl`
  ```rust
  fs::write(&issues_path, b"").expect("Failed to recreate issues.jsonl");
  ```

- Line 59: `Failed to recreate events.jsonl`
  ```rust
  fs::write(&events_path, b"").expect("Failed to recreate events.jsonl");
  ```

- Line 104: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 107: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 118: `Failed to GET /api/projects`
  ```rust
  .expect("Failed to GET /api/projects");
  ```

- Line 124: `Failed to parse projects response`
  ```rust
  .expect("Failed to parse projects response");
  ```

- Line 147: `Failed to GET /readyz`
  ```rust
  .expect("Failed to GET /readyz");
  ```

- Line 167: `Failed to GET /api/projects`
  ```rust
  .expect("Failed to GET /api/projects");
  ```

- Line 173: `Failed to parse projects response`
  ```rust
  .expect("Failed to parse projects response");
  ```

- Line 202: `Failed to GET /api/projects`
  ```rust
  .expect("Failed to GET /api/projects");
  ```

- Line 207: `Failed to parse projects response`
  ```rust
  .expect("Failed to parse projects response");
  ```

- Line 238: `Failed to GET /readyz`
  ```rust
  .expect("Failed to GET /readyz");
  ```

- Line 249: `Failed to parse readiness response`
  ```rust
  .expect("Failed to parse readiness response");
  ```

- Line 280: `Failed to POST /api/config/reload`
  ```rust
  .expect("Failed to POST /api/config/reload");
  ```

- Line 296: `Failed to GET /readyz`
  ```rust
  .expect("Failed to GET /readyz");
  ```

- Line 315: `Failed to GET /readyz`
  ```rust
  .expect("Failed to GET /readyz");
  ```

- Line 364: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 367: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 374: `Failed to GET /readyz`
  ```rust
  let resp = client.get(&format!("{}/readyz", base_url)).send().await.expect("Failed to GET /readyz");
  ```

- Line 394: `Failed to GET /readyz`
  ```rust
  .expect("Failed to GET /readyz");
  ```

- Line 400: `Failed to parse readiness response`
  ```rust
  .expect("Failed to parse readiness response");
  ```

- Line 420: `Failed to GET /api/projects`
  ```rust
  .expect("Failed to GET /api/projects");
  ```

- Line 425: `Failed to parse projects response`
  ```rust
  .expect("Failed to parse projects response");
  ```

#### unwrap (9 occurrences)

- Line 102: `\.unwrap\(\)`
  ```rust
  let hoop_dir = config.control_socket_path.parent().unwrap();
  ```

- Line 128: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 175: `\.unwrap\(\)`
  ```rust
  for project in projects.as_array().unwrap() {
  ```

- Line 176: `\.unwrap\(\)`
  ```rust
  let name = project.get("name").and_then(|n| n.as_str()).unwrap();
  ```

- Line 177: `\.unwrap\(\)`
  ```rust
  let state = project.get("state").and_then(|s| s.as_str()).unwrap();
  ```

- Line 211: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 218: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 362: `\.unwrap\(\)`
  ```rust
  let hoop_dir = config.control_socket_path.parent().unwrap();
  ```

- Line 429: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

### hoop-daemon/tests/config_field_validation.rs

Total errors: 117

#### assert (53 occurrences)

- Line 43: `missing schema_version should fail`
  ```rust
  assert!(err.is_some(), "missing schema_version should fail");
  ```

- Line 45: `error should include field path`
  ```rust
  assert!(err.field.is_some(), "error should include field path");
  ```

- Line 59: `integer schema_version should fail`
  ```rust
  assert!(err.is_some(), "integer schema_version should fail");
  ```

- Line 66: `error should include field path`
  ```rust
  assert!(err.field.is_some(), "error should include field path");
  ```

- Line 75: `invalid schema_version format should fail`
  ```rust
  assert!(err.is_some(), "invalid schema_version format should fail");
  ```

- Line 90: `invalid schema_version text should fail`
  ```rust
  assert!(err.is_some(), "invalid schema_version text should fail");
  ```

- Line 109: `missing agent.adapter should fail`
  ```rust
  assert!(err.is_some(), "missing agent.adapter should fail");
  ```

- Line 126: `integer adapter should fail`
  ```rust
  assert!(err.is_some(), "integer adapter should fail");
  ```

- Line 148: `invalid adapter value should fail`
  ```rust
  assert!(err.is_some(), "invalid adapter value should fail");
  ```

- Line 165: `null adapter should fail`
  ```rust
  assert!(err.is_some(), "null adapter should fail");
  ```

- Line 185: `integer model should fail`
  ```rust
  assert!(err.is_some(), "integer model should fail");
  ```

- Line 209: `object model should fail`
  ```rust
  assert!(err.is_some(), "object model should fail");
  ```

- Line 228: `integer bind_addr should fail`
  ```rust
  assert!(err.is_some(), "integer bind_addr should fail");
  ```

- Line 252: `object bind_addr should fail`
  ```rust
  assert!(err.is_some(), "object bind_addr should fail");
  ```

- Line 271: `string metrics.enabled should fail`
  ```rust
  assert!(err.is_some(), "string metrics.enabled should fail");
  ```

- Line 293: `integer metrics.enabled should fail`
  ```rust
  assert!(err.is_some(), "integer metrics.enabled should fail");
  ```

- Line 312: `string metrics.port should fail`
  ```rust
  assert!(err.is_some(), "string metrics.port should fail");
  ```

- Line 355: `string retention_days should fail`
  ```rust
  assert!(err.is_some(), "string retention_days should fail");
  ```

- Line 377: `boolean retention_days should fail`
  ```rust
  assert!(err.is_some(), "boolean retention_days should fail");
  ```

- Line 396: `string hash_chain should fail`
  ```rust
  assert!(err.is_some(), "string hash_chain should fail");
  ```

- Line 413: `integer hash_chain should fail`
  ```rust
  assert!(err.is_some(), "integer hash_chain should fail");
  ```

- Line 432: `integer ui.theme should fail`
  ```rust
  assert!(err.is_some(), "integer ui.theme should fail");
  ```

- Line 449: `invalid ui.theme value should fail`
  ```rust
  assert!(err.is_some(), "invalid ui.theme value should fail");
  ```

- Line 466: `boolean ui.theme should fail`
  ```rust
  assert!(err.is_some(), "boolean ui.theme should fail");
  ```

- Line 485: `string archive_after_days should fail`
  ```rust
  assert!(err.is_some(), "string archive_after_days should fail");
  ```

- Line 502: `boolean archive_after_days should fail`
  ```rust
  assert!(err.is_some(), "boolean archive_after_days should fail");
  ```

- Line 521: `string reflection.enabled should fail`
  ```rust
  assert!(err.is_some(), "string reflection.enabled should fail");
  ```

- Line 540: `string detection_threshold should fail`
  ```rust
  assert!(err.is_some(), "string detection_threshold should fail");
  ```

- Line 557: `boolean detection_threshold should fail`
  ```rust
  assert!(err.is_some(), "boolean detection_threshold should fail");
  ```

- Line 576: `string auto_archive_after_days should fail`
  ```rust
  assert!(err.is_some(), "string auto_archive_after_days should fail");
  ```

- Line 595: `string roles.viewers should fail (must be array)`
  ```rust
  assert!(err.is_some(), "string roles.viewers should fail (must be array)");
  ```

- Line 614: `integer in viewers array should fail`
  ```rust
  assert!(err.is_some(), "integer in viewers array should fail");
  ```

- Line 633: `string roles.drafters should fail (must be array)`
  ```rust
  assert!(err.is_some(), "string roles.drafters should fail (must be array)");
  ```

- Line 652: `integer agent_extensions.skills should fail`
  ```rust
  assert!(err.is_some(), "integer agent_extensions.skills should fail");
  ```

- Line 670: `array agent_extensions.scripts should fail`
  ```rust
  assert!(err.is_some(), "array agent_extensions.scripts should fail");
  ```

- Line 688: `missing project name should fail`
  ```rust
  assert!(err.is_some(), "missing project name should fail");
  ```

- Line 706: `integer project name should fail`
  ```rust
  assert!(err.is_some(), "integer project name should fail");
  ```

- Line 722: `missing project path should fail`
  ```rust
  assert!(err.is_some(), "missing project path should fail");
  ```

- Line 740: `integer project path should fail`
  ```rust
  assert!(err.is_some(), "integer project path should fail");
  ```

- Line 757: `boolean project path should fail`
  ```rust
  assert!(err.is_some(), "boolean project path should fail");
  ```

- Line 775: `integer project label should fail`
  ```rust
  assert!(err.is_some(), "integer project label should fail");
  ```

- Line 793: `integer project color should fail`
  ```rust
  assert!(err.is_some(), "integer project color should fail");
  ```

- Line 811: `string project disabled should fail`
  ```rust
  assert!(err.is_some(), "string project disabled should fail");
  ```

- Line 826: `non-array projects should fail`
  ```rust
  assert!(err.is_some(), "non-array projects should fail");
  ```

- Line 843: `string in projects array should fail`
  ```rust
  assert!(err.is_some(), "string in projects array should fail");
  ```

- Line 862: `unknown field should be rejected`
  ```rust
  assert!(err.is_some(), "unknown field should be rejected");
  ```

- Line 880: `unknown nested field should be rejected`
  ```rust
  assert!(err.is_some(), "unknown nested field should be rejected");
  ```

- Line 898: `unknown nested field in ui should be rejected`
  ```rust
  assert!(err.is_some(), "unknown nested field in ui should be rejected");
  ```

- Line 916: `unknown field in project entry should be rejected`
  ```rust
  assert!(err.is_some(), "unknown field in project entry should be rejected");
  ```

- Line 933: `unclosed quote should fail`
  ```rust
  assert!(err.is_some(), "unclosed quote should fail");
  ```

- Line 950: `unmatched bracket should fail`
  ```rust
  assert!(err.is_some(), "unmatched bracket should fail");
  ```

- Line 967: `invalid escape sequence should fail`
  ```rust
  assert!(err.is_some(), "invalid escape sequence should fail");
  ```

- Line 986: `trailing comma should fail`
  ```rust
  assert!(err.is_some(), "trailing comma should fail");
  ```

#### unwrap (64 occurrences)

- Line 44: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 47: `\.unwrap\(\)`
  ```rust
  err.field.as_ref().unwrap().contains("schema_version"),
  ```

- Line 60: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 76: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 91: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 110: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 112: `\.unwrap\(\)`
  ```rust
  err.field.as_ref().unwrap().contains("adapter") || err.message.to_lowercase().contains("adapter"),
  ```

- Line 127: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 134: `\.unwrap\(\)`
  ```rust
  err.field.as_ref().unwrap().contains("adapter"),
  ```

- Line 149: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 166: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 186: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 193: `\.unwrap\(\)`
  ```rust
  err.field.as_ref().unwrap().contains("model"),
  ```

- Line 210: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 229: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 236: `\.unwrap\(\)`
  ```rust
  err.field.as_ref().unwrap().contains("bind_addr"),
  ```

- Line 253: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 272: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 279: `\.unwrap\(\)`
  ```rust
  err.field.as_ref().unwrap().contains("enabled"),
  ```

- Line 294: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 313: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 320: `\.unwrap\(\)`
  ```rust
  err.field.as_ref().unwrap().contains("port"),
  ```

- Line 356: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 363: `\.unwrap\(\)`
  ```rust
  err.field.as_ref().unwrap().contains("retention_days"),
  ```

- Line 378: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 397: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 414: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 433: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 450: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 467: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 486: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 503: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 522: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 541: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 558: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 577: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 596: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 615: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 634: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 653: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 671: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 689: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 707: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 723: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 741: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 758: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 776: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 794: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 812: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 827: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 844: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 863: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 881: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 899: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 917: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 934: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 951: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 968: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 987: `\.unwrap\(\)`
  ```rust
  let err = err.unwrap();
  ```

- Line 1004: `\.unwrap\(\)`
  ```rust
  let err = parse_and_get_error(yaml).unwrap();
  ```

- Line 1024: `\.unwrap\(\)`
  ```rust
  let err = parse_and_get_error(yaml).unwrap();
  ```

- Line 1030: `\.unwrap\(\)`
  ```rust
  let field = err.field.unwrap();
  ```

- Line 1045: `\.unwrap\(\)`
  ```rust
  let err = parse_and_get_error(yaml).unwrap();
  ```

- Line 1065: `\.unwrap\(\)`
  ```rust
  let err = parse_and_get_error(yaml).unwrap();
  ```

### hoop-daemon/tests/config_reload_audit.rs

Total errors: 63

#### assert (1 occurrences)

- Line 120: `hash chain must advance`
  ```rust
  assert!(row.hash_prev != row.hash_self, "hash chain must advance");
  ```

#### assert_eq (1 occurrences)

- Line 125: `should find exactly one config_reloaded row`
  ```rust
  assert_eq!(rows.len(), 1, "should find exactly one config_reloaded row");
  ```

#### expect (16 occurrences)

- Line 48: `tempdir`
  ```rust
  let dir = tempfile::tempdir().expect("tempdir");
  ```

- Line 51: `init fleet db`
  ```rust
  fleet::init_fleet_db_at(db_path).expect("init fleet db");
  ```

- Line 66: `tempdir for projects`
  ```rust
  let tmp = tempfile::tempdir().expect("tempdir for projects");
  ```

- Line 115: `write audit row`
  ```rust
  .expect("write audit row");
  ```

- Line 124: `query`
  ```rust
  .expect("query");
  ```

- Line 133: `delta_keys should be array`
  ```rust
  .expect("delta_keys should be array");
  ```

- Line 142: `hash chain should be valid`
  ```rust
  fleet::verify_hash_chain().expect("hash chain should be valid");
  ```

- Line 154: `tempdir for projects`
  ```rust
  let tmp = tempfile::tempdir().expect("tempdir for projects");
  ```

- Line 189: `write audit row`
  ```rust
  .expect("write audit row");
  ```

- Line 203: `query`
  ```rust
  .expect("query");
  ```

- Line 216: `hash chain should be valid`
  ```rust
  fleet::verify_hash_chain().expect("hash chain should be valid");
  ```

- Line 224: `tempdir`
  ```rust
  let tmp = tempfile::tempdir().expect("tempdir");
  ```

- Line 277: `tempdir for projects`
  ```rust
  let tmp = tempfile::tempdir().expect("tempdir for projects");
  ```

- Line 318: `write audit row`
  ```rust
  .expect("write audit row");
  ```

- Line 322: `query`
  ```rust
  .expect("query");
  ```

- Line 351: `hash chain should be valid after round-trip`
  ```rust
  fleet::verify_hash_chain().expect("hash chain should be valid after round-trip");
  ```

#### unwrap (45 occurrences)

- Line 69: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&repo1).unwrap();
  ```

- Line 70: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&repo2).unwrap();
  ```

- Line 75: `\.unwrap\(\)`
  ```rust
  let v1 = yaml_one_project(repo1.to_str().unwrap());
  ```

- Line 76: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, &v1).unwrap();
  ```

- Line 77: `\.unwrap\(\)`
  ```rust
  let cfg1 = projects::ProjectsConfig::load_from(&yaml_path).unwrap();
  ```

- Line 81: `\.unwrap\(\)`
  ```rust
  let v2 = yaml_two_projects(repo1.to_str().unwrap(), repo2.to_str().unwrap());
  ```

- Line 81: `\.unwrap\(\)`
  ```rust
  let v2 = yaml_two_projects(repo1.to_str().unwrap(), repo2.to_str().unwrap());
  ```

- Line 82: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, &v2).unwrap();
  ```

- Line 83: `\.unwrap\(\)`
  ```rust
  let cfg2 = projects::ProjectsConfig::load_from(&yaml_path).unwrap();
  ```

- Line 102: `\.unwrap\(\)`
  ```rust
  let args_json = serde_json::to_string(&audit_args).unwrap();
  ```

- Line 128: `\.unwrap\(\)`
  ```rust
  serde_json::from_str(fetched.args_json.as_ref().unwrap()).unwrap();
  ```

- Line 128: `\.unwrap\(\)`
  ```rust
  serde_json::from_str(fetched.args_json.as_ref().unwrap()).unwrap();
  ```

- Line 137: `\.unwrap\(\)`
  ```rust
  .any(|v| v.as_str().unwrap().contains("+project:proj-two")),
  ```

- Line 159: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&repo).unwrap();
  ```

- Line 160: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, yaml_one_project(repo.to_str().unwrap())).unwrap();
  ```

- Line 160: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, yaml_one_project(repo.to_str().unwrap())).unwrap();
  ```

- Line 161: `\.unwrap\(\)`
  ```rust
  let cfg = projects::ProjectsConfig::load_from(&yaml_path).unwrap();
  ```

- Line 165: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, yaml_invalid()).unwrap();
  ```

- Line 176: `\.unwrap\(\)`
  ```rust
  let args_json = serde_json::to_string(&audit_args).unwrap();
  ```

- Line 211: `\.unwrap\(\)`
  ```rust
  serde_json::from_str(fetched.args_json.as_ref().unwrap()).unwrap();
  ```

- Line 211: `\.unwrap\(\)`
  ```rust
  serde_json::from_str(fetched.args_json.as_ref().unwrap()).unwrap();
  ```

- Line 227: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&repo1).unwrap();
  ```

- Line 228: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&repo2).unwrap();
  ```

- Line 233: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, yaml_one_project(repo1.to_str().unwrap())).unwrap();
  ```

- Line 233: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, yaml_one_project(repo1.to_str().unwrap())).unwrap();
  ```

- Line 234: `\.unwrap\(\)`
  ```rust
  let cfg1 = projects::ProjectsConfig::load_from(&yaml_path).unwrap();
  ```

- Line 239: `\.unwrap\(\)`
  ```rust
  yaml_two_projects(repo1.to_str().unwrap(), repo2.to_str().unwrap()),
  ```

- Line 239: `\.unwrap\(\)`
  ```rust
  yaml_two_projects(repo1.to_str().unwrap(), repo2.to_str().unwrap()),
  ```

- Line 241: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 242: `\.unwrap\(\)`
  ```rust
  let cfg2 = projects::ProjectsConfig::load_from(&yaml_path).unwrap();
  ```

- Line 253: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, yaml_one_project(repo2.to_str().unwrap())).unwrap();
  ```

- Line 253: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, yaml_one_project(repo2.to_str().unwrap())).unwrap();
  ```

- Line 254: `\.unwrap\(\)`
  ```rust
  let cfg3 = projects::ProjectsConfig::load_from(&yaml_path).unwrap();
  ```

- Line 280: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&repo1).unwrap();
  ```

- Line 281: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&repo2).unwrap();
  ```

- Line 286: `\.unwrap\(\)`
  ```rust
  let v1 = yaml_one_project(repo1.to_str().unwrap());
  ```

- Line 287: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, &v1).unwrap();
  ```

- Line 288: `\.unwrap\(\)`
  ```rust
  let cfg1 = projects::ProjectsConfig::load_from(&yaml_path).unwrap();
  ```

- Line 291: `\.unwrap\(\)`
  ```rust
  let v2 = yaml_two_projects(repo1.to_str().unwrap(), repo2.to_str().unwrap());
  ```

- Line 291: `\.unwrap\(\)`
  ```rust
  let v2 = yaml_two_projects(repo1.to_str().unwrap(), repo2.to_str().unwrap());
  ```

- Line 292: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, &v2).unwrap();
  ```

- Line 293: `\.unwrap\(\)`
  ```rust
  let cfg2 = projects::ProjectsConfig::load_from(&yaml_path).unwrap();
  ```

- Line 305: `\.unwrap\(\)`
  ```rust
  let args_json = serde_json::to_string(&audit_args).unwrap();
  ```

- Line 325: `\.unwrap\(\)`
  ```rust
  serde_json::from_str(rows[0].args_json.as_ref().unwrap()).unwrap();
  ```

- Line 325: `\.unwrap\(\)`
  ```rust
  serde_json::from_str(rows[0].args_json.as_ref().unwrap()).unwrap();
  ```

### hoop-daemon/tests/config_reload_cycle.rs

Total errors: 63

#### assert (9 occurrences)

- Line 106: `content hash must be set`
  ```rust
  assert!(!hash_v1.is_empty(), "content hash must be set");
  ```

- Line 111: `truncated YAML must be rejected`
  ```rust
  assert!(result_bad.is_err(), "truncated YAML must be rejected");
  ```

- Line 153: `missing field must be rejected`
  ```rust
  assert!(result_bad2.is_err(), "missing field must be rejected");
  ```

- Line 261: `missing name should fail`
  ```rust
  assert!(result.is_err(), "missing name should fail");
  ```

- Line 270: `error message should not be empty`
  ```rust
  assert!(!err.message.is_empty(), "error message should not be empty",);
  ```

- Line 274: `integer name should fail`
  ```rust
  assert!(result2.is_err(), "integer name should fail");
  ```

- Line 292: `truncated YAML should fail`
  ```rust
  assert!(result3.is_err(), "truncated YAML should fail");
  ```

- Line 383: `semantic error should have field path`
  ```rust
  assert!(err.field.is_some(), "semantic error should have field path");
  ```

- Line 398: `missing path error should have field`
  ```rust
  assert!(err.field.is_some(), "missing path error should have field");
  ```

#### assert_eq (7 occurrences)

- Line 103: `v1: one project`
  ```rust
  assert_eq!(cfg_v1.registry.projects.len(), 1, "v1: one project");
  ```

- Line 136: `v2: two projects`
  ```rust
  assert_eq!(cfg_v2.registry.projects.len(), 2, "v2: two projects");
  ```

- Line 161: `v2 hash unchanged`
  ```rust
  assert_eq!(cfg_v2.content_hash, hash_v2, "v2 hash unchanged");
  ```

- Line 168: `v3: back to one project`
  ```rust
  assert_eq!(cfg_v3.registry.projects.len(), 1, "v3: back to one project");
  ```

- Line 231: `one rejected audit row`
  ```rust
  assert_eq!(rejected_rows.len(), 1, "one rejected audit row");
  ```

- Line 240: `one success audit row`
  ```rust
  assert_eq!(success_rows.len(), 1, "one success audit row");
  ```

- Line 444: `hash unchanged`
  ```rust
  assert_eq!(cfg_v1.content_hash, hash_v1, "hash unchanged");
  ```

#### assert_ne (1 occurrences)

- Line 140: `content hash must change on valid edit`
  ```rust
  assert_ne!(hash_v1, hash_v2, "content hash must change on valid edit");
  ```

#### expect (20 occurrences)

- Line 68: `tempdir`
  ```rust
  let dir = tempfile::tempdir().expect("tempdir");
  ```

- Line 71: `init fleet db`
  ```rust
  fleet::init_fleet_db_at(db_path).expect("init fleet db");
  ```

- Line 90: `tempdir for projects`
  ```rust
  let tmp = tempfile::tempdir().expect("tempdir for projects");
  ```

- Line 102: `v1 should parse successfully`
  ```rust
  projects::ProjectsConfig::load_from(&yaml_path).expect("v1 should parse successfully");
  ```

- Line 135: `v2 should parse successfully`
  ```rust
  projects::ProjectsConfig::load_from(&yaml_path).expect("v2 should parse successfully");
  ```

- Line 167: `v3 should parse successfully`
  ```rust
  projects::ProjectsConfig::load_from(&yaml_path).expect("v3 should parse successfully");
  ```

- Line 199: `write rejected audit row`
  ```rust
  .expect("write rejected audit row");
  ```

- Line 221: `write success audit row`
  ```rust
  .expect("write success audit row");
  ```

- Line 230: `query rejected`
  ```rust
  .expect("query rejected");
  ```

- Line 239: `query success`
  ```rust
  .expect("query success");
  ```

- Line 247: `hash chain intact after full cycle`
  ```rust
  fleet::verify_hash_chain().expect("hash chain intact after full cycle");
  ```

- Line 336: `tempdir`
  ```rust
  let tmp = tempfile::tempdir().expect("tempdir");
  ```

- Line 364: `YAML should parse fine`
  ```rust
  let cfg = projects::ProjectsConfig::load_from(&yaml_path).expect("YAML should parse fine");
  ```

- Line 411: `tempdir`
  ```rust
  let tmp = tempfile::tempdir().expect("tempdir");
  ```

- Line 420: `valid config should load`
  ```rust
  let cfg_v1 = projects::ProjectsConfig::load_from(&yaml_path).expect("valid config should load");
  ```

- Line 430: `YAML should still parse`
  ```rust
  let cfg_bad = projects::ProjectsConfig::load_from(&yaml_path).expect("YAML should still parse");
  ```

- Line 449: `fixed config should load`
  ```rust
  let cfg_v2 = projects::ProjectsConfig::load_from(&yaml_path).expect("fixed config should load");
  ```

- Line 490: `write rejected audit`
  ```rust
  .expect("write rejected audit");
  ```

- Line 498: `query`
  ```rust
  .expect("query");
  ```

- Line 509: `hash chain intact`
  ```rust
  fleet::verify_hash_chain().expect("hash chain intact");
  ```

#### unwrap (26 occurrences)

- Line 93: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&repo1).unwrap();
  ```

- Line 94: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&repo2).unwrap();
  ```

- Line 99: `\.unwrap\(\)`
  ```rust
  let v1 = yaml_one_project("proj-alpha", repo1.to_str().unwrap());
  ```

- Line 100: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, &v1).unwrap();
  ```

- Line 109: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, yaml_truncated()).unwrap();
  ```

- Line 129: `\.unwrap\(\)`
  ```rust
  repo1.to_str().unwrap(),
  ```

- Line 131: `\.unwrap\(\)`
  ```rust
  repo2.to_str().unwrap(),
  ```

- Line 133: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, &v2).unwrap();
  ```

- Line 151: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, yaml_missing_required_field()).unwrap();
  ```

- Line 164: `\.unwrap\(\)`
  ```rust
  let v3 = yaml_one_project("proj-alpha", repo2.to_str().unwrap());
  ```

- Line 165: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, &v3).unwrap();
  ```

- Line 186: `\.unwrap\(\)`
  ```rust
  let rejected_json = serde_json::to_string(&rejected_audit).unwrap();
  ```

- Line 208: `\.unwrap\(\)`
  ```rust
  let success_json = serde_json::to_string(&success_audit).unwrap();
  ```

- Line 338: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(repo_exists.join(".beads")).unwrap();
  ```

- Line 341: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&repo_no_beads).unwrap();
  ```

- Line 363: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, &yaml).unwrap();
  ```

- Line 382: `\.unwrap\(\)`
  ```rust
  let err = no_beads_err.unwrap();
  ```

- Line 397: `\.unwrap\(\)`
  ```rust
  let err = missing_err.unwrap();
  ```

- Line 413: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(repo.join(".beads")).unwrap();
  ```

- Line 418: `\.unwrap\(\)`
  ```rust
  let v1 = yaml_one_project("good-proj", repo.to_str().unwrap());
  ```

- Line 419: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, &v1).unwrap();
  ```

- Line 429: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, &bad_yaml).unwrap();
  ```

- Line 447: `\.unwrap\(\)`
  ```rust
  let v2 = yaml_one_project("another-proj", repo.to_str().unwrap());
  ```

- Line 448: `\.unwrap\(\)`
  ```rust
  std::fs::write(&yaml_path, &v2).unwrap();
  ```

- Line 470: `\.unwrap\(\)`
  ```rust
  let first_err = validation_errors.into_iter().next().unwrap();
  ```

- Line 477: `\.unwrap\(\)`
  ```rust
  let rejected_json = serde_json::to_string(&rejected_audit).unwrap();
  ```

### hoop-daemon/tests/create_only_stub.rs

Total errors: 12

#### assert (1 occurrences)

- Line 106: `fake br should succeed`
  ```rust
  assert!(output.status.success(), "fake br should succeed");
  ```

#### assert_eq (2 occurrences)

- Line 159: `expected 3 invocations, got {:?}`
  ```rust
  assert_eq!(verbs.len(), 3, "expected 3 invocations, got {:?}", verbs);
  ```

- Line 380: `expected 3 invocations, got {:?}`
  ```rust
  assert_eq!(verbs.len(), 3, "expected 3 invocations, got {:?}", verbs);
  ```

#### expect (7 occurrences)

- Line 25: `create temp dir`
  ```rust
  let bin_dir = tempfile::TempDir::new().expect("create temp dir");
  ```

- Line 40: `create br script`
  ```rust
  let mut f = fs::File::create(&br_path).expect("create br script");
  ```

- Line 41: `write br script`
  ```rust
  f.write_all(script.as_bytes()).expect("write br script");
  ```

- Line 46: `chmod br script`
  ```rust
  .expect("chmod br script");
  ```

- Line 105: `run fake br`
  ```rust
  let output = cmd.output().expect("run fake br");
  ```

- Line 307: `run fake br`
  ```rust
  let output = cmd.output().expect("run fake br");
  ```

- Line 370: `run fake br`
  ```rust
  let output = cmd.output().expect("run fake br");
  ```

#### unwrap (2 occurrences)

- Line 30: `\.unwrap\(\)`
  ```rust
  let log_path_str = log_path.to_str().unwrap();
  ```

- Line 54: `\.unwrap\(\)`
  ```rust
  self.bin_dir.path().to_str().unwrap().to_string()
  ```

### hoop-daemon/tests/create_stitch_no_auto_submit.rs

Total errors: 48

#### assert (1 occurrences)

- Line 379: `stitch_id must be None before approval`
  ```rust
  assert!(fetched.stitch_id.is_none(), "stitch_id must be None before approval");
  ```

#### assert_eq (5 occurrences)

- Line 276: `draft ID should match`
  ```rust
  assert_eq!(fetched.id, draft_id, "draft ID should match");
  ```

- Line 277: `draft status should be pending`
  ```rust
  assert_eq!(fetched.status, "pending", "draft status should be pending");
  ```

- Line 278: `draft title should match`
  ```rust
  assert_eq!(fetched.title, draft.title, "draft title should match");
  ```

- Line 279: `source should match combo`
  ```rust
  assert_eq!(fetched.source, combo.source, "source should match combo");
  ```

- Line 400: `status should be submitted after approval`
  ```rust
  assert_eq!(approved.status, "submitted", "status should be submitted after approval");
  ```

#### expect (36 occurrences)

- Line 143: `create temp dir for test project`
  ```rust
  let tmp = tempfile::TempDir::new().expect("create temp dir for test project");
  ```

- Line 145: `create project dir`
  ```rust
  fs::create_dir_all(&project_dir).expect("create project dir");
  ```

- Line 148: `create .beads dir`
  ```rust
  fs::create_dir_all(&beads_dir).expect("create .beads dir");
  ```

- Line 152: `create beads.db`
  ```rust
  fs::write(&beads_db, b"").expect("create beads.db");
  ```

- Line 192: `create temp HOOP home`
  ```rust
  let tmp = tempfile::TempDir::new().expect("create temp HOOP home");
  ```

- Line 194: `create .hoop dir`
  ```rust
  fs::create_dir_all(&hoop_dir).expect("create .hoop dir");
  ```

- Line 207: `write projects.yaml`
  ```rust
  .expect("write projects.yaml");
  ```

- Line 217: `write config.yml`
  ```rust
  .expect("write config.yml");
  ```

- Line 227: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 332: `create temp HOOP home`
  ```rust
  let tmp = tempfile::TempDir::new().expect("create temp HOOP home");
  ```

- Line 334: `create .hoop dir`
  ```rust
  fs::create_dir_all(&hoop_dir).expect("create .hoop dir");
  ```

- Line 337: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 371: `insert draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");
  ```

- Line 375: `get draft`
  ```rust
  .expect("get draft")
  ```

- Line 376: `draft exists`
  ```rust
  .expect("draft exists");
  ```

- Line 393: `update draft status`
  ```rust
  .expect("update draft status");
  ```

- Line 397: `get approved draft`
  ```rust
  .expect("get approved draft")
  ```

- Line 398: `approved draft exists`
  ```rust
  .expect("approved draft exists");
  ```

- Line 419: `create temp HOOP home`
  ```rust
  let tmp = tempfile::TempDir::new().expect("create temp HOOP home");
  ```

- Line 421: `create .hoop dir`
  ```rust
  fs::create_dir_all(&hoop_dir).expect("create .hoop dir");
  ```

- Line 424: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 458: `insert first draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft_1).expect("insert first draft");
  ```

- Line 492: `insert second draft with force_create bypass`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft_2).expect("insert second draft with force_create bypass");
  ```

- Line 496: `get first draft`
  ```rust
  .expect("get first draft")
  ```

- Line 497: `first draft exists`
  ```rust
  .expect("first draft exists");
  ```

- Line 500: `get second draft`
  ```rust
  .expect("get second draft")
  ```

- Line 501: `second draft exists`
  ```rust
  .expect("second draft exists");
  ```

- Line 525: `create temp HOOP home`
  ```rust
  let tmp = tempfile::TempDir::new().expect("create temp HOOP home");
  ```

- Line 527: `create .hoop dir`
  ```rust
  fs::create_dir_all(&hoop_dir).expect("create .hoop dir");
  ```

- Line 530: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 564: `insert draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");
  ```

- Line 568: `get draft`
  ```rust
  .expect("get draft")
  ```

- Line 569: `draft exists`
  ```rust
  .expect("draft exists");
  ```

- Line 599: `create temp HOOP home`
  ```rust
  let tmp = tempfile::TempDir::new().expect("create temp HOOP home");
  ```

- Line 601: `create .hoop dir`
  ```rust
  fs::create_dir_all(&hoop_dir).expect("create .hoop dir");
  ```

- Line 604: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

#### unwrap (6 occurrences)

- Line 141: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 191: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 328: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 416: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 522: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 596: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

### hoop-daemon/tests/cross_workspace_blockers.rs

Total errors: 38

#### assert (2 occurrences)

- Line 191: `workspace_from column should exist`
  ```rust
  assert!(workspace_from_exists, "workspace_from column should exist");
  ```

- Line 202: `workspace_to column should exist`
  ```rust
  assert!(workspace_to_exists, "workspace_to column should exist");
  ```

#### assert_eq (6 occurrences)

- Line 119: `Should find 2 child stitches`
  ```rust
  assert_eq!(child_stitches.len(), 2, "Should find 2 child stitches");
  ```

- Line 125: `Workspace B should match`
  ```rust
  assert_eq!(found_workspace_b, workspace_b, "Workspace B should match");
  ```

- Line 131: `Workspace C should match`
  ```rust
  assert_eq!(found_workspace_c, workspace_c, "Workspace C should match");
  ```

- Line 152: `Should find 2 child beads`
  ```rust
  assert_eq!(all_child_beads.len(), 2, "Should find 2 child beads");
  ```

- Line 158: `Bead B workspace should match`
  ```rust
  assert_eq!(found_ws_b, workspace_b, "Bead B workspace should match");
  ```

- Line 164: `Bead C workspace should match`
  ```rust
  assert_eq!(found_ws_c, workspace_c, "Bead C workspace should match");
  ```

#### expect (30 occurrences)

- Line 26: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 30: `Failed to open fleet.db`
  ```rust
  .expect("Failed to open fleet.db");
  ```

- Line 42: `Failed to insert parent stitch`
  ```rust
  ).expect("Failed to insert parent stitch");
  ```

- Line 50: `Failed to insert parent bead`
  ```rust
  ).expect("Failed to insert parent bead");
  ```

- Line 59: `Failed to insert child stitch B`
  ```rust
  ).expect("Failed to insert child stitch B");
  ```

- Line 67: `Failed to insert child bead B`
  ```rust
  ).expect("Failed to insert child bead B");
  ```

- Line 76: `Failed to insert child stitch C`
  ```rust
  ).expect("Failed to insert child stitch C");
  ```

- Line 84: `Failed to insert child bead C`
  ```rust
  ).expect("Failed to insert child bead C");
  ```

- Line 91: `Failed to insert link to child B`
  ```rust
  ).expect("Failed to insert link to child B");
  ```

- Line 97: `Failed to insert link to child C`
  ```rust
  ).expect("Failed to insert link to child C");
  ```

- Line 105: `Failed to prepare stitch_links query`
  ```rust
  .expect("Failed to prepare stitch_links query");
  ```

- Line 114: `Failed to query child stitches`
  ```rust
  .expect("Failed to query child stitches")
  ```

- Line 124: `Should find child stitch B`
  ```rust
  .expect("Should find child stitch B");
  ```

- Line 130: `Should find child stitch C`
  ```rust
  .expect("Should find child stitch C");
  ```

- Line 138: `Failed to prepare stitch_beads query`
  ```rust
  .expect("Failed to prepare stitch_beads query");
  ```

- Line 142: `Failed to query child beads`
  ```rust
  .expect("Failed to query child beads")
  ```

- Line 157: `Should find child bead B`
  ```rust
  .expect("Should find child bead B");
  ```

- Line 163: `Should find child bead C`
  ```rust
  .expect("Should find child bead C");
  ```

- Line 174: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 178: `Failed to open fleet.db`
  ```rust
  .expect("Failed to open fleet.db");
  ```

- Line 189: `Failed to query workspace_from column`
  ```rust
  .expect("Failed to query workspace_from column");
  ```

- Line 200: `Failed to query workspace_to column`
  ```rust
  .expect("Failed to query workspace_to column");
  ```

- Line 209: `Failed to insert stitch link with workspaces`
  ```rust
  ).expect("Failed to insert stitch link with workspaces");
  ```

- Line 217: `Failed to query workspace columns`
  ```rust
  .expect("Failed to query workspace columns");
  ```

- Line 243: `Failed to create stitches table`
  ```rust
  ).expect("Failed to create stitches table");
  ```

- Line 256: `Failed to create stitch_beads table`
  ```rust
  ).expect("Failed to create stitch_beads table");
  ```

- Line 271: `Failed to create stitch_links table`
  ```rust
  ).expect("Failed to create stitch_links table");
  ```

- Line 277: `Failed to create idx_stitch_links_from`
  ```rust
  ).expect("Failed to create idx_stitch_links_from");
  ```

- Line 282: `Failed to create idx_stitch_links_to`
  ```rust
  ).expect("Failed to create idx_stitch_links_to");
  ```

- Line 287: `Failed to create idx_stitch_beads_project`
  ```rust
  ).expect("Failed to create idx_stitch_beads_project");
  ```

### hoop-daemon/tests/disaster_recovery_runbook.rs

Total errors: 86

#### assert (18 occurrences)

- Line 164: `fresh host has no ~/.hoop/`
  ```rust
  assert!(!hoop_dir.exists(), "fresh host has no ~/.hoop/");
  ```

- Line 180: `projects restored`
  ```rust
  assert!(content.contains("test-project"), "projects restored");
  ```

- Line 196: `newer version is rejected`
  ```rust
  assert!(result.is_err(), "newer version is rejected");
  ```

- Line 203: `error suggests upgrading`
  ```rust
  assert!(err.contains("Upgrade HOOP"), "error suggests upgrading");
  ```

- Line 224: `corrupted database fails to open`
  ```rust
  assert!(result.is_err(), "corrupted database fails to open");
  ```

- Line 251: `corrupted database is preserved`
  ```rust
  assert!(preserved.exists(), "corrupted database is preserved");
  ```

- Line 252: `filename indicates corruption`
  ```rust
  assert!(preserved.to_string_lossy().contains("corrupted"), "filename indicates corruption");
  ```

- Line 277: `~/.hoop/ is gone after deletion`
  ```rust
  assert!(!hoop_dir.exists(), "~/.hoop/ is gone after deletion");
  ```

- Line 285: `fleet.db restored`
  ```rust
  assert!(fleet_db.exists(), "fleet.db restored");
  ```

- Line 286: `projects.yaml restored`
  ```rust
  assert!(projects_yaml.exists(), "projects.yaml restored");
  ```

- Line 368: `paths updated for new host`
  ```rust
  assert!(content.contains("/new/host/"), "paths updated for new host");
  ```

- Line 479: `local restore completes in seconds`
  ```rust
  assert!(elapsed.as_secs() < 10, "local restore completes in seconds");
  ```

- Line 492: `corruption recovery is fast locally`
  ```rust
  assert!(elapsed.as_secs() < 5, "corruption recovery is fast locally");
  ```

- Line 557: `mentions snapshot version`
  ```rust
  assert!(err.contains("99.0.0"), "mentions snapshot version");
  ```

- Line 558: `mentions current version`
  ```rust
  assert!(err.contains(current), "mentions current version");
  ```

- Line 559: `explains the problem`
  ```rust
  assert!(err.contains("newer than"), "explains the problem");
  ```

- Line 569: `older schema version accepted`
  ```rust
  assert!(result.is_ok(), "older schema version accepted");
  ```

- Line 591: `{} has test coverage`
  ```rust
  assert!(!name.is_empty(), "{} has test coverage", description);
  ```

#### assert_eq (3 occurrences)

- Line 176: `restored stitch data present`
  ```rust
  assert_eq!(stitch_count, 1, "restored stitch data present");
  ```

- Line 186: `database integrity verified`
  ```rust
  assert_eq!(integrity, "ok", "database integrity verified");
  ```

- Line 432: `original database intact after rollback`
  ```rust
  assert_eq!(integrity, "ok", "original database intact after rollback");
  ```

#### unwrap (65 occurrences)

- Line 158: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 167: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&hoop_dir).unwrap();
  ```

- Line 168: `\.unwrap\(\)`
  ```rust
  create_test_fleet_db(&fleet_db).unwrap();
  ```

- Line 169: `\.unwrap\(\)`
  ```rust
  create_test_projects_yaml(&projects_yaml).unwrap();
  ```

- Line 172: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&fleet_db).unwrap();
  ```

- Line 175: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 179: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(&projects_yaml).unwrap();
  ```

- Line 185: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 213: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 217: `\.unwrap\(\)`
  ```rust
  fs::write(&fleet_db, b"corrupted garbage data not sqlite").unwrap();
  ```

- Line 228: `\.unwrap\(\)`
  ```rust
  create_test_fleet_db(&backup_db).unwrap();
  ```

- Line 231: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&backup_db).unwrap();
  ```

- Line 234: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 241: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 244: `\.unwrap\(\)`
  ```rust
  fs::write(&fleet_db, b"corrupted").unwrap();
  ```

- Line 249: `\.unwrap\(\)`
  ```rust
  fs::copy(&fleet_db, &preserved).unwrap();
  ```

- Line 262: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 266: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&hoop_dir).unwrap();
  ```

- Line 269: `\.unwrap\(\)`
  ```rust
  create_test_fleet_db(&fleet_db).unwrap();
  ```

- Line 270: `\.unwrap\(\)`
  ```rust
  create_test_projects_yaml(&projects_yaml).unwrap();
  ```

- Line 276: `\.unwrap\(\)`
  ```rust
  fs::remove_dir_all(&hoop_dir).unwrap();
  ```

- Line 280: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&hoop_dir).unwrap();
  ```

- Line 281: `\.unwrap\(\)`
  ```rust
  create_test_fleet_db(&fleet_db).unwrap();
  ```

- Line 282: `\.unwrap\(\)`
  ```rust
  create_test_projects_yaml(&projects_yaml).unwrap();
  ```

- Line 288: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(&projects_yaml).unwrap();
  ```

- Line 295: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 300: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&rollback_dir).unwrap();
  ```

- Line 302: `\.unwrap\(\)`
  ```rust
  create_test_projects_yaml(&rollback_projects).unwrap();
  ```

- Line 305: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&hoop_dir).unwrap();
  ```

- Line 309: `\.unwrap\(\)`
  ```rust
  fs::copy(&rollback_projects, &projects_yaml).unwrap();
  ```

- Line 313: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(&projects_yaml).unwrap();
  ```

- Line 324: `\.unwrap\(\)`
  ```rust
  let old_host = TempDir::new().unwrap();
  ```

- Line 325: `\.unwrap\(\)`
  ```rust
  let new_host = TempDir::new().unwrap();
  ```

- Line 329: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&old_hoop).unwrap();
  ```

- Line 343: `\.unwrap\(\)`
  ```rust
  fs::write(&old_projects, projects_content).unwrap();
  ```

- Line 344: `\.unwrap\(\)`
  ```rust
  create_test_config(&old_config).unwrap();
  ```

- Line 348: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&new_hoop).unwrap();
  ```

- Line 352: `\.unwrap\(\)`
  ```rust
  fs::copy(&old_projects, &new_projects).unwrap();
  ```

- Line 364: `\.unwrap\(\)`
  ```rust
  fs::write(&new_projects, updated_content).unwrap();
  ```

- Line 367: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(&new_projects).unwrap();
  ```

- Line 374: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 376: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&hoop_dir).unwrap();
  ```

- Line 386: `\.unwrap\(\)`
  ```rust
  fs::write(&projects_yaml, content).unwrap();
  ```

- Line 401: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 406: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&hoop_dir).unwrap();
  ```

- Line 408: `\.unwrap\(\)`
  ```rust
  create_test_fleet_db(&original_fleet).unwrap();
  ```

- Line 411: `\.unwrap\(\)`
  ```rust
  fs::rename(&hoop_dir, &rollback_dir).unwrap();
  ```

- Line 416: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&hoop_dir).unwrap();
  ```

- Line 418: `\.unwrap\(\)`
  ```rust
  fs::write(&partial_fleet, b"incomplete partial data").unwrap();
  ```

- Line 421: `\.unwrap\(\)`
  ```rust
  fs::remove_dir_all(&hoop_dir).unwrap();
  ```

- Line 422: `\.unwrap\(\)`
  ```rust
  fs::rename(&rollback_dir, &hoop_dir).unwrap();
  ```

- Line 428: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&hoop_dir.join("fleet.db")).unwrap();
  ```

- Line 431: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 438: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 443: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&rollback1).unwrap();
  ```

- Line 444: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&rollback2).unwrap();
  ```

- Line 447: `\.unwrap\(\)`
  ```rust
  for entry in fs::read_dir(temp_dir.path()).unwrap() {
  ```

- Line 448: `\.unwrap\(\)`
  ```rust
  let entry = entry.unwrap();
  ```

- Line 451: `\.unwrap\(\)`
  ```rust
  fs::remove_dir_all(entry.path()).unwrap();
  ```

- Line 470: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 472: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&hoop_dir).unwrap();
  ```

- Line 473: `\.unwrap\(\)`
  ```rust
  create_test_fleet_db(&hoop_dir.join("fleet.db")).unwrap();
  ```

- Line 474: `\.unwrap\(\)`
  ```rust
  create_test_projects_yaml(&hoop_dir.join("projects.yaml")).unwrap();
  ```

- Line 487: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 489: `\.unwrap\(\)`
  ```rust
  create_test_fleet_db(&fleet_db).unwrap();
  ```

### hoop-daemon/tests/draft_queue_invariants.rs

Total errors: 106

#### assert (8 occurrences)

- Line 209: `fleet.db must persist on disk`
  ```rust
  assert!(db_path.exists(), "fleet.db must persist on disk");
  ```

- Line 340: `audit row should be written successfully`
  ```rust
  assert!(result.is_ok(), "audit row should be written successfully");
  ```

- Line 783: `opened_at should be set`
  ```rust
  assert!(draft.opened_at.is_some(), "opened_at should be set");
  ```

- Line 814: `abandoned_at should be set`
  ```rust
  assert!(abandoned.abandoned_at.is_some(), "abandoned_at should be set");
  ```

- Line 826: `abandoned_at should be cleared on reopen`
  ```rust
  assert!(reopened.abandoned_at.is_none(), "abandoned_at should be cleared on reopen");
  ```

- Line 862: `last_autosave_at should be set`
  ```rust
  assert!(draft.last_autosave_at.is_some(), "last_autosave_at should be set");
  ```

- Line 913: `abandoned_at should be set`
  ```rust
  assert!(abandoned.abandoned_at.is_some(), "abandoned_at should be set");
  ```

- Line 1047: `old abandoned draft should be deleted`
  ```rust
  assert!(old_draft_after.is_none(), "old abandoned draft should be deleted");
  ```

#### assert_eq (5 occurrences)

- Line 650: `edit must increment version`
  ```rust
  assert_eq!(edited.version, 2, "edit must increment version");
  ```

- Line 651: `edit must set status to `
  ```rust
  assert_eq!(edited.status, "edited", "edit must set status to 'edited'");
  ```

- Line 861: `security`
  ```rust
  assert_eq!(draft.labels, vec!["urgent".to_string(), "security".to_string()]);
  ```

- Line 881: `autosave should not increment version`
  ```rust
  assert_eq!(updated.version, original_version, "autosave should not increment version");
  ```

- Line 1041: `should delete exactly one old draft`
  ```rust
  assert_eq!(deleted, 1, "should delete exactly one old draft");
  ```

#### expect (89 occurrences)

- Line 27: `create temp dir`
  ```rust
  let tmp = TempDir::new().expect("create temp dir");
  ```

- Line 29: `create .hoop dir`
  ```rust
  std::fs::create_dir_all(&hoop_dir).expect("create .hoop dir");
  ```

- Line 35: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 82: `insert draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");
  ```

- Line 85: `get draft`
  ```rust
  .expect("get draft")
  ```

- Line 86: `draft exists`
  ```rust
  .expect("draft exists");
  ```

- Line 129: `insert draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");
  ```

- Line 132: `get draft`
  ```rust
  .expect("get draft")
  ```

- Line 133: `draft exists`
  ```rust
  .expect("draft exists");
  ```

- Line 206: `insert draft1`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft1).expect("insert draft1");
  ```

- Line 207: `insert draft2`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft2).expect("insert draft2");
  ```

- Line 212: `get draft1`
  ```rust
  .expect("get draft1")
  ```

- Line 213: `draft1 exists`
  ```rust
  .expect("draft1 exists");
  ```

- Line 218: `get draft2`
  ```rust
  .expect("get draft2")
  ```

- Line 219: `draft2 exists`
  ```rust
  .expect("draft2 exists");
  ```

- Line 266: `insert draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");
  ```

- Line 270: `list pending`
  ```rust
  hoop_daemon::fleet::list_drafts(None, Some("pending"), 100).expect("list pending");
  ```

- Line 275: `list rejected`
  ```rust
  hoop_daemon::fleet::list_drafts(None, Some("rejected"), 100).expect("list rejected");
  ```

- Line 281: `list pending`
  ```rust
  hoop_daemon::fleet::list_drafts(None, Some("pending"), 100).expect("list pending");
  ```

- Line 282: `list edited`
  ```rust
  p.extend(hoop_daemon::fleet::list_drafts(None, Some("edited"), 100).expect("list edited"));
  ```

- Line 326: `insert draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");
  ```

- Line 386: `insert draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");
  ```

- Line 399: `update draft status`
  ```rust
  .expect("update draft status");
  ```

- Line 419: `write audit row`
  ```rust
  .expect("write audit row");
  ```

- Line 429: `get draft`
  ```rust
  .expect("get draft")
  ```

- Line 430: `draft exists`
  ```rust
  .expect("draft exists");
  ```

- Line 474: `insert draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");
  ```

- Line 488: `reject draft`
  ```rust
  .expect("reject draft");
  ```

- Line 491: `get draft`
  ```rust
  .expect("get draft")
  ```

- Line 492: `draft exists`
  ```rust
  .expect("draft exists");
  ```

- Line 529: `insert draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");
  ```

- Line 542: `reject draft`
  ```rust
  .expect("reject draft");
  ```

- Line 545: `get draft`
  ```rust
  .expect("get draft")
  ```

- Line 546: `draft exists`
  ```rust
  .expect("draft exists");
  ```

- Line 581: `write audit row`
  ```rust
  .expect("write audit row");
  ```

- Line 631: `insert draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");
  ```

- Line 641: `edit draft`
  ```rust
  .expect("edit draft");
  ```

- Line 644: `get draft`
  ```rust
  .expect("get draft")
  ```

- Line 645: `draft exists`
  ```rust
  .expect("draft exists");
  ```

- Line 692: `insert draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");
  ```

- Line 706: `approve and submit draft`
  ```rust
  .expect("approve and submit draft");
  ```

- Line 709: `get draft`
  ```rust
  .expect("get draft")
  ```

- Line 710: `draft exists`
  ```rust
  .expect("draft exists");
  ```

- Line 745: `write audit row`
  ```rust
  .expect("write audit row");
  ```

- Line 756: `hash chain must be valid after draft actions`
  ```rust
  hoop_daemon::fleet::verify_hash_chain().expect("hash chain must be valid after draft actions");
  ```

- Line 774: `open_draft should succeed`
  ```rust
  .expect("open_draft should succeed");
  ```

- Line 777: `get draft should succeed`
  ```rust
  .expect("get draft should succeed")
  ```

- Line 778: `draft should exist`
  ```rust
  .expect("draft should exist");
  ```

- Line 799: `first open should succeed`
  ```rust
  .expect("first open should succeed");
  ```

- Line 802: `get draft should succeed`
  ```rust
  .expect("get draft should succeed")
  ```

- Line 803: `draft should exist`
  ```rust
  .expect("draft should exist");
  ```

- Line 808: `abandon should succeed`
  ```rust
  hoop_daemon::fleet::abandon_draft(draft_id).expect("abandon should succeed");
  ```

- Line 811: `get draft should succeed`
  ```rust
  .expect("get draft should succeed")
  ```

- Line 812: `draft should exist`
  ```rust
  .expect("draft should exist");
  ```

- Line 818: `second open should succeed`
  ```rust
  .expect("second open should succeed");
  ```

- Line 821: `get draft should succeed`
  ```rust
  .expect("get draft should succeed")
  ```

- Line 822: `draft should exist`
  ```rust
  .expect("draft should exist");
  ```

- Line 840: `open should succeed`
  ```rust
  .expect("open should succeed");
  ```

- Line 851: `autosave should succeed`
  ```rust
  .expect("autosave should succeed");
  ```

- Line 854: `get draft should succeed`
  ```rust
  .expect("get draft should succeed")
  ```

- Line 855: `draft should exist`
  ```rust
  .expect("draft should exist");
  ```

- Line 875: `second autosave should succeed`
  ```rust
  .expect("second autosave should succeed");
  ```

- Line 878: `get draft should succeed`
  ```rust
  .expect("get draft should succeed")
  ```

- Line 879: `draft should exist`
  ```rust
  .expect("draft should exist");
  ```

- Line 895: `open should succeed`
  ```rust
  .expect("open should succeed");
  ```

- Line 898: `get draft should succeed`
  ```rust
  .expect("get draft should succeed")
  ```

- Line 899: `draft should exist`
  ```rust
  .expect("draft should exist");
  ```

- Line 906: `abandon should succeed`
  ```rust
  .expect("abandon should succeed");
  ```

- Line 909: `get draft should succeed`
  ```rust
  .expect("get draft should succeed")
  ```

- Line 910: `draft should exist`
  ```rust
  .expect("draft should exist");
  ```

- Line 953: `insert draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");
  ```

- Line 1002: `insert old draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&old_draft).expect("insert old draft");
  ```

- Line 1035: `insert recent draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&recent_draft).expect("insert recent draft");
  ```

- Line 1039: `cleanup should succeed`
  ```rust
  .expect("cleanup should succeed");
  ```

- Line 1045: `get draft should succeed`
  ```rust
  .expect("get draft should succeed");
  ```

- Line 1051: `get draft should succeed`
  ```rust
  .expect("get draft should succeed")
  ```

- Line 1052: `recent abandoned draft should still exist`
  ```rust
  .expect("recent abandoned draft should still exist");
  ```

- Line 1069: `open should succeed`
  ```rust
  .expect("open should succeed");
  ```

- Line 1072: `get draft should succeed`
  ```rust
  .expect("get draft should succeed")
  ```

- Line 1073: `draft should exist`
  ```rust
  .expect("draft should exist");
  ```

- Line 1087: `autosave should succeed`
  ```rust
  .expect("autosave should succeed");
  ```

- Line 1090: `get draft should succeed`
  ```rust
  .expect("get draft should succeed")
  ```

- Line 1091: `draft should exist`
  ```rust
  .expect("draft should exist");
  ```

- Line 1105: `second autosave should succeed`
  ```rust
  .expect("second autosave should succeed");
  ```

- Line 1109: `abandon should succeed`
  ```rust
  .expect("abandon should succeed");
  ```

- Line 1112: `get draft should succeed`
  ```rust
  .expect("get draft should succeed")
  ```

- Line 1113: `draft should exist`
  ```rust
  .expect("draft should exist");
  ```

- Line 1120: `get draft should succeed`
  ```rust
  .expect("get draft should succeed")
  ```

- Line 1121: `abandoned draft should still exist`
  ```rust
  .expect("abandoned draft should still exist");
  ```

#### unwrap (4 occurrences)

- Line 25: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 42: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 342: `\.unwrap\(\)`
  ```rust
  let audit_row = result.unwrap();
  ```

- Line 589: `\.unwrap\(\)`
  ```rust
  serde_json::from_str(audit_row.args_json.as_deref().unwrap_or("{}")).unwrap();
  ```

### hoop-daemon/tests/epoch_sync_invariant.rs

Total errors: 29

#### assert (3 occurrences)

- Line 101: `Should receive at least one message`
  ```rust
  assert!(!messages.is_empty(), "Should receive at least one message");
  ```

- Line 226: `Reconnect should receive init event`
  ```rust
  assert!(received_init, "Reconnect should receive init event");
  ```

- Line 332: `Connection should receive init`
  ```rust
  assert!(handle.await.expect("Task failed"), "Connection should receive init");
  ```

#### assert_eq (2 occurrences)

- Line 49: `First message should be init event`
  ```rust
  assert_eq!(event["type"], "init", "First message should be init event");
  ```

- Line 106: `First message must be init`
  ```rust
  assert_eq!(first_event["type"], "init", "First message must be init");
  ```

#### expect (21 occurrences)

- Line 26: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 33: `Failed to connect to WebSocket`
  ```rust
  .expect("Failed to connect to WebSocket");
  ```

- Line 40: `Timeout waiting for init message`
  ```rust
  .expect("Timeout waiting for init message")
  ```

- Line 41: `WebSocket stream ended`
  ```rust
  .expect("WebSocket stream ended");
  ```

- Line 43: `Failed to receive init message`
  ```rust
  let init_msg = init_msg.expect("Failed to receive init message");
  ```

- Line 47: `Failed to parse init event as JSON`
  ```rust
  serde_json::from_str(&text).expect("Failed to parse init event as JSON");
  ```

- Line 77: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 84: `Failed to connect to WebSocket`
  ```rust
  .expect("Failed to connect to WebSocket");
  ```

- Line 105: `Failed to parse first message`
  ```rust
  serde_json::from_str(&messages[0]).expect("Failed to parse first message");
  ```

- Line 147: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 155: `Failed to connect to WebSocket`
  ```rust
  .expect("Failed to connect to WebSocket");
  ```

- Line 188: `Failed to reconnect to WebSocket`
  ```rust
  .expect("Failed to reconnect to WebSocket");
  ```

- Line 244: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 253: `Failed to connect to WebSocket (iteration {})`
  ```rust
  .expect("Failed to connect to WebSocket (iteration {})");
  ```

- Line 263: `WebSocket stream ended`
  ```rust
  .expect("WebSocket stream ended");
  ```

- Line 272: `Failed to parse message as JSON`
  ```rust
  serde_json::from_str(&text).expect("Failed to parse message as JSON");
  ```

- Line 292: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 305: `Failed to connect`
  ```rust
  .expect("Failed to connect");
  ```

- Line 313: `Stream ended`
  ```rust
  .expect("Stream ended");
  ```

- Line 319: `Failed to parse`
  ```rust
  serde_json::from_str(&text).expect("Failed to parse");
  ```

- Line 332: `Task failed`
  ```rust
  assert!(handle.await.expect("Task failed"), "Connection should receive init");
  ```

#### panic (2 occurrences)

- Line 68: `Expected text message for init, got {:?}`
  ```rust
  panic!("Expected text message for init, got {:?}", init_msg);
  ```

- Line 280: `Expected text message (iteration {})`
  ```rust
  panic!("Expected text message (iteration {})", i);
  ```

#### unwrap (1 occurrences)

- Line 58: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

### hoop-daemon/tests/filesystem_failure_isolation.rs

Total errors: 40

#### assert (1 occurrences)

- Line 516: `project-a should be degraded`
  ```rust
  assert!(project_a_degraded, "project-a should be degraded");
  ```

#### assert_eq (4 occurrences)

- Line 175: `Initial readyz should return 200`
  ```rust
  assert_eq!(status, 200, "Initial readyz should return 200");
  ```

- Line 176: `Initial readyz status should be ok`
  ```rust
  assert_eq!(readyz.status, "ok", "Initial readyz status should be ok");
  ```

- Line 325: `Initial readyz should return 200`
  ```rust
  assert_eq!(status, 200, "Initial readyz should return 200");
  ```

- Line 326: `Initial readyz status should be ok`
  ```rust
  assert_eq!(readyz.status, "ok", "Initial readyz status should be ok");
  ```

#### expect (25 occurrences)

- Line 28: `Failed to create .beads dir`
  ```rust
  fs::create_dir_all(&beads_dir).expect("Failed to create .beads dir");
  ```

- Line 30: `Failed to create issues.jsonl`
  ```rust
  fs::write(&issues_path, b"").expect("Failed to create issues.jsonl");
  ```

- Line 39: `Failed to create temp dir`
  ```rust
  let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
  ```

- Line 41: `Failed to create .hoop dir`
  ```rust
  fs::create_dir_all(&hoop_dir).expect("Failed to create .hoop dir");
  ```

- Line 71: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 80: `Failed to write config.yml`
  ```rust
  fs::write(hoop_dir.join("config.yml"), config_yaml).expect("Failed to write config.yml");
  ```

- Line 83: `Failed to create data dir`
  ```rust
  fs::create_dir_all(hoop_dir.join("data")).expect("Failed to create data dir");
  ```

- Line 125: `Failed to bind to random port`
  ```rust
  .expect("Failed to bind to random port");
  ```

- Line 126: `Failed to get local address`
  ```rust
  let addr = listener.local_addr().expect("Failed to get local address");
  ```

- Line 173: `Failed to get readyz status`
  ```rust
  .expect("Failed to get readyz status");
  ```

- Line 184: `Failed to remove .beads from project A`
  ```rust
  fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads from project A");
  ```

- Line 188: `Failed to get projects.yaml metadata`
  ```rust
  let metadata = fs::metadata(&projects_path).expect("Failed to get projects.yaml metadata");
  ```

- Line 189: `Failed to get modified time`
  ```rust
  let modified = metadata.modified().expect("Failed to get modified time");
  ```

- Line 220: `project-a should be in degraded list`
  ```rust
  .expect("project-a should be in degraded list");
  ```

- Line 275: `Failed to bind to random port`
  ```rust
  .expect("Failed to bind to random port");
  ```

- Line 276: `Failed to get local address`
  ```rust
  let addr = listener.local_addr().expect("Failed to get local address");
  ```

- Line 323: `Failed to get readyz status`
  ```rust
  .expect("Failed to get readyz status");
  ```

- Line 334: `Failed to remove .beads from project A`
  ```rust
  fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads from project A");
  ```

- Line 371: `Failed to read projects.yaml`
  ```rust
  let projects_content = fs::read_to_string(&projects_path).expect("Failed to read projects.yaml");
  ```

- Line 372: `Failed to write projects.yaml`
  ```rust
  fs::write(&projects_path, projects_content).expect("Failed to write projects.yaml");
  ```

- Line 430: `Failed to bind to random port`
  ```rust
  .expect("Failed to bind to random port");
  ```

- Line 431: `Failed to get local address`
  ```rust
  let addr = listener.local_addr().expect("Failed to get local address");
  ```

- Line 479: `Failed to get readyz status`
  ```rust
  .expect("Failed to get readyz status");
  ```

- Line 488: `Failed to connect to WebSocket`
  ```rust
  .expect("Failed to connect to WebSocket");
  ```

- Line 493: `Failed to remove .beads from project A`
  ```rust
  fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads from project A");
  ```

#### unwrap (10 occurrences)

- Line 107: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 111: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 115: `\.unwrap\(\)`
  ```rust
  let project_c_dir = tempfile::tempdir().unwrap();
  ```

- Line 228: `\.unwrap\(\)`
  ```rust
  let error_msg = project_a_degraded.error.as_ref().unwrap();
  ```

- Line 257: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 261: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 265: `\.unwrap\(\)`
  ```rust
  let project_c_dir = tempfile::tempdir().unwrap();
  ```

- Line 412: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 416: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 420: `\.unwrap\(\)`
  ```rust
  let project_c_dir = tempfile::tempdir().unwrap();
  ```

### hoop-daemon/tests/fix_patterns_integration.rs

Total errors: 54

#### assert (2 occurrences)

- Line 56: `create should return non-empty ID`
  ```rust
  assert!(!id.is_empty(), "create should return non-empty ID");
  ```

- Line 101: `pattern should be deleted`
  ```rust
  assert!(deleted.is_none(), "pattern should be deleted");
  ```

#### assert_eq (8 occurrences)

- Line 63: `option,panic,null`
  ```rust
  assert_eq!(pattern.keywords, "unwrap,option,panic,null");
  ```

- Line 68: `should have 1 pattern`
  ```rust
  assert_eq!(patterns.len(), 1, "should have 1 pattern");
  ```

- Line 87: `option,pattern-matching`
  ```rust
  assert_eq!(updated.keywords, "unwrap,option,pattern-matching");
  ```

- Line 89: `0.2, 0.0, 0.5]);
`
  ```rust
  assert_eq!(updated.signature_vector, vec![0.8, 0.2, 0.0, 0.5]);
  ```

- Line 230: `should limit results`
  ```rust
  assert_eq!(matches_limited.len(), 2, "should limit results");
  ```

- Line 294: `should find 2 patterns with `
  ```rust
  assert_eq!(results.len(), 2, "should find 2 patterns with 'panic'");
  ```

- Line 303: `should find 1 pattern with `
  ```rust
  assert_eq!(results.len(), 1, "should find 1 pattern with 'bounds'");
  ```

- Line 309: `case-insensitive search should work`
  ```rust
  assert_eq!(results.len(), 1, "case-insensitive search should work");
  ```

#### expect (3 occurrences)

- Line 61: `pattern should exist`
  ```rust
  .expect("pattern should exist");
  ```

- Line 85: `pattern should exist after update`
  ```rust
  .expect("pattern should exist after update");
  ```

- Line 95: `pattern should exist`
  ```rust
  .expect("pattern should exist");
  ```

#### unwrap (41 occurrences)

- Line 12: `\.unwrap\(\)`
  ```rust
  let tmp = TempDir::new().unwrap();
  ```

- Line 16: `\.unwrap\(\)`
  ```rust
  let mut conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 17: `\.unwrap\(\)`
  ```rust
  conn.pragma_update(None, "journal_mode", "WAL").unwrap();
  ```

- Line 35: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 41: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 44: `\.unwrap\(\)`
  ```rust
  std::env::set_var("_HOOP_FLEET_DB_PATH", db_path.to_str().unwrap());
  ```

- Line 55: `\.unwrap\(\)`
  ```rust
  let id = hoop_daemon::fix_patterns::FixPatternService::create(&create_req).unwrap();
  ```

- Line 60: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 67: `\.unwrap\(\)`
  ```rust
  let patterns = hoop_daemon::fix_patterns::FixPatternService::list().unwrap();
  ```

- Line 81: `\.unwrap\(\)`
  ```rust
  hoop_daemon::fix_patterns::FixPatternService::update(&update_req).unwrap();
  ```

- Line 84: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 92: `\.unwrap\(\)`
  ```rust
  hoop_daemon::fix_patterns::FixPatternService::record_application(&id).unwrap();
  ```

- Line 94: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 99: `\.unwrap\(\)`
  ```rust
  hoop_daemon::fix_patterns::FixPatternService::delete(&id).unwrap();
  ```

- Line 100: `\.unwrap\(\)`
  ```rust
  let deleted = hoop_daemon::fix_patterns::FixPatternService::get(&id).unwrap();
  ```

- Line 108: `\.unwrap\(\)`
  ```rust
  let tmp = TempDir::new().unwrap();
  ```

- Line 111: `\.unwrap\(\)`
  ```rust
  let mut conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 112: `\.unwrap\(\)`
  ```rust
  conn.pragma_update(None, "journal_mode", "WAL").unwrap();
  ```

- Line 129: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 131: `\.unwrap\(\)`
  ```rust
  std::env::set_var("_HOOP_FLEET_DB_PATH", db_path.to_str().unwrap());
  ```

- Line 159: `\.unwrap\(\)`
  ```rust
  hoop_daemon::fix_patterns::FixPatternService::create(req).unwrap();
  ```

- Line 168: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 203: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 216: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 228: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 237: `\.unwrap\(\)`
  ```rust
  let tmp = TempDir::new().unwrap();
  ```

- Line 240: `\.unwrap\(\)`
  ```rust
  let mut conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 241: `\.unwrap\(\)`
  ```rust
  conn.pragma_update(None, "journal_mode", "WAL").unwrap();
  ```

- Line 258: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 260: `\.unwrap\(\)`
  ```rust
  std::env::set_var("_HOOP_FLEET_DB_PATH", db_path.to_str().unwrap());
  ```

- Line 287: `\.unwrap\(\)`
  ```rust
  hoop_daemon::fix_patterns::FixPatternService::create(req).unwrap();
  ```

- Line 292: `\.unwrap\(\)`
  ```rust
  hoop_daemon::fix_patterns::FixPatternService::search_by_keywords("panic").unwrap();
  ```

- Line 301: `\.unwrap\(\)`
  ```rust
  hoop_daemon::fix_patterns::FixPatternService::search_by_keywords("bounds").unwrap();
  ```

- Line 307: `\.unwrap\(\)`
  ```rust
  let results = hoop_daemon::fix_patterns::FixPatternService::search_by_keywords("TYPE").unwrap();
  ```

- Line 317: `\.unwrap\(\)`
  ```rust
  let tmp = TempDir::new().unwrap();
  ```

- Line 320: `\.unwrap\(\)`
  ```rust
  let mut conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 321: `\.unwrap\(\)`
  ```rust
  conn.pragma_update(None, "journal_mode", "WAL").unwrap();
  ```

- Line 338: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 340: `\.unwrap\(\)`
  ```rust
  std::env::set_var("_HOOP_FLEET_DB_PATH", db_path.to_str().unwrap());
  ```

- Line 352: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 360: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

### hoop-daemon/tests/fleet_notifications_integration.rs

Total errors: 19

#### assert_eq (2 occurrences)

- Line 101: `Oldest retained notification should be index 5`
  ```rust
  assert_eq!(snapshot[0].summary, "Notification 5", "Oldest retained notification should be index 5");
  ```

- Line 102: `Newest notification should be index 24`
  ```rust
  assert_eq!(snapshot[19].summary, "Notification 24", "Newest notification should be index 24");
  ```

#### expect (2 occurrences)

- Line 59: `Should serialize to JSON`
  ```rust
  let json = serde_json::to_string(&notification).expect("Should serialize to JSON");
  ```

- Line 71: `Should deserialize from JSON`
  ```rust
  serde_json::from_str(&json).expect("Should deserialize from JSON");
  ```

#### unwrap (15 occurrences)

- Line 39: `\.unwrap\(\)`
  ```rust
  let received = result.unwrap().unwrap();
  ```

- Line 39: `\.unwrap\(\)`
  ```rust
  let received = result.unwrap().unwrap();
  ```

- Line 150: `\.unwrap\(\)`
  ```rust
  let recv1 = timeout(Duration::from_secs(1), rx.recv()).await.unwrap().unwrap();
  ```

- Line 150: `\.unwrap\(\)`
  ```rust
  let recv1 = timeout(Duration::from_secs(1), rx.recv()).await.unwrap().unwrap();
  ```

- Line 153: `\.unwrap\(\)`
  ```rust
  let recv2 = timeout(Duration::from_secs(1), rx.recv()).await.unwrap().unwrap();
  ```

- Line 153: `\.unwrap\(\)`
  ```rust
  let recv2 = timeout(Duration::from_secs(1), rx.recv()).await.unwrap().unwrap();
  ```

- Line 156: `\.unwrap\(\)`
  ```rust
  let recv3 = timeout(Duration::from_secs(1), rx.recv()).await.unwrap().unwrap();
  ```

- Line 156: `\.unwrap\(\)`
  ```rust
  let recv3 = timeout(Duration::from_secs(1), rx.recv()).await.unwrap().unwrap();
  ```

- Line 175: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 237: `\.unwrap\(\)`
  ```rust
  let recv1 = timeout(Duration::from_secs(1), rx1.recv()).await.unwrap().unwrap();
  ```

- Line 237: `\.unwrap\(\)`
  ```rust
  let recv1 = timeout(Duration::from_secs(1), rx1.recv()).await.unwrap().unwrap();
  ```

- Line 238: `\.unwrap\(\)`
  ```rust
  let recv2 = timeout(Duration::from_secs(1), rx2.recv()).await.unwrap().unwrap();
  ```

- Line 238: `\.unwrap\(\)`
  ```rust
  let recv2 = timeout(Duration::from_secs(1), rx2.recv()).await.unwrap().unwrap();
  ```

- Line 239: `\.unwrap\(\)`
  ```rust
  let recv3 = timeout(Duration::from_secs(1), rx3.recv()).await.unwrap().unwrap();
  ```

- Line 239: `\.unwrap\(\)`
  ```rust
  let recv3 = timeout(Duration::from_secs(1), rx3.recv()).await.unwrap().unwrap();
  ```

### hoop-daemon/tests/golden_transcripts_regression.rs

Total errors: 18

#### expect (1 occurrences)

- Line 39: `workspace root is parent of hoop-daemon/`
  ```rust
  .expect("workspace root is parent of hoop-daemon/")
  ```

#### panic (17 occurrences)

- Line 107: `Failed to read scenario directory {scenario_path:?}: {e}`
  ```rust
  panic!("Failed to read scenario directory {scenario_path:?}: {e}")
  ```

- Line 170: `Failed to read {:?}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
  ```

- Line 199: `Failed to read {:?}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", scenario_dir, e))
  ```

- Line 212: `Failed to read {:?}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
  ```

- Line 235: `Failed to read {:?}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", simple_dir, e))
  ```

- Line 248: `Failed to read {:?}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
  ```

- Line 297: `Failed to read {:?}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", tool_dir, e))
  ```

- Line 310: `Failed to read {:?}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
  ```

- Line 361: `Failed to read {:?}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", failure_dir, e))
  ```

- Line 374: `Failed to read {:?}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
  ```

- Line 502: `Failed to read {:?}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
  ```

- Line 530: `Failed to read {:?}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", simple_dir, e))
  ```

- Line 543: `Failed to read {:?}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
  ```

- Line 579: `Failed to read {:?}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", tool_dir, e))
  ```

- Line 592: `Failed to read {:?}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
  ```

- Line 637: `Failed to read {:?}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", failure_dir, e))
  ```

- Line 650: `Failed to read {:?}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
  ```

### hoop-daemon/tests/hoop_dies_nothing_notices.rs

Total errors: 71

#### assert (3 occurrences)

- Line 211: `worker should have written at least 2 events`
  ```rust
  assert!(initial_count >= 2, "worker should have written at least 2 events");
  ```

- Line 215: `events.jsonl should contain at least 2 events`
  ```rust
  assert!(file_count >= 2, "events.jsonl should contain at least 2 events");
  ```

- Line 672: `should still parse all valid events`
  ```rust
  assert!(valid_count >= 4, "should still parse all valid events");
  ```

#### assert_eq (2 occurrences)

- Line 671: `should detect exactly one corrupted line`
  ```rust
  assert_eq!(invalid_count, 1, "should detect exactly one corrupted line");
  ```

- Line 707: `empty events.jsonl should have 0 events`
  ```rust
  assert_eq!(count, 0, "empty events.jsonl should have 0 events");
  ```

#### bail (1 occurrences)

- Line 168: `testrepo should exist at {:?}`
  ```rust
  anyhow::bail!("testrepo should exist at {:?}", testrepo);
  ```

#### expect (57 occurrences)

- Line 30: `workspace root is parent of hoop-daemon/`
  ```rust
  .expect("workspace root is parent of hoop-daemon/")
  ```

- Line 43: `create temp dir for test HOOP home`
  ```rust
  let temp_dir = TempDir::new().expect("create temp dir for test HOOP home");
  ```

- Line 45: `create .hoop dir`
  ```rust
  fs::create_dir_all(&hoop_dir).expect("create .hoop dir");
  ```

- Line 61: `write projects.yaml`
  ```rust
  .expect("write projects.yaml");
  ```

- Line 71: `write config.yml`
  ```rust
  .expect("write config.yml");
  ```

- Line 74: `create data dir`
  ```rust
  fs::create_dir_all(hoop_dir.join("data")).expect("create data dir");
  ```

- Line 189: `testrepo should exist`
  ```rust
  verify_testrepo_exists().expect("testrepo should exist");
  ```

- Line 197: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 205: `write claim event`
  ```rust
  .expect("write claim event");
  ```

- Line 208: `write dispatch event`
  ```rust
  .expect("write dispatch event");
  ```

- Line 219: `read events.jsonl`
  ```rust
  .expect("read events.jsonl");
  ```

- Line 243: `write complete event during HOOP absence`
  ```rust
  .expect("write complete event during HOOP absence");
  ```

- Line 246: `write claim event during HOOP absence`
  ```rust
  .expect("write claim event during HOOP absence");
  ```

- Line 263: `read events.jsonl after restart`
  ```rust
  .expect("read events.jsonl after restart");
  ```

- Line 291: `testrepo should exist`
  ```rust
  verify_testrepo_exists().expect("testrepo should exist");
  ```

- Line 299: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 310: `write claim before HOOP`
  ```rust
  .expect("write claim before HOOP");
  ```

- Line 313: `write dispatch before HOOP`
  ```rust
  .expect("write dispatch before HOOP");
  ```

- Line 332: `write claim during HOOP absence`
  ```rust
  .expect("write claim during HOOP absence");
  ```

- Line 335: `write complete during HOOP absence`
  ```rust
  .expect("write complete during HOOP absence");
  ```

- Line 362: `read events.jsonl after restart`
  ```rust
  .expect("read events.jsonl after restart");
  ```

- Line 393: `testrepo should exist`
  ```rust
  verify_testrepo_exists().expect("testrepo should exist");
  ```

- Line 400: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 412: `write claim event`
  ```rust
  .expect("write claim event");
  ```

- Line 417: `write dispatch event`
  ```rust
  .expect("write dispatch event");
  ```

- Line 424: `write complete event`
  ```rust
  .expect("write complete event");
  ```

- Line 435: `read events.jsonl for rebuild`
  ```rust
  .expect("read events.jsonl for rebuild");
  ```

- Line 475: `testrepo should exist`
  ```rust
  verify_testrepo_exists().expect("testrepo should exist");
  ```

- Line 482: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 489: `write claim`
  ```rust
  worker.write_claim("bd-restart-001").expect("write claim");
  ```

- Line 490: `write dispatch`
  ```rust
  worker.write_dispatch("bd-restart-001").expect("write dispatch");
  ```

- Line 491: `write complete`
  ```rust
  worker.write_complete("bd-restart-001").expect("write complete");
  ```

- Line 499: `write claim`
  ```rust
  worker.write_claim("bd-restart-002").expect("write claim");
  ```

- Line 500: `write dispatch`
  ```rust
  worker.write_dispatch("bd-restart-002").expect("write dispatch");
  ```

- Line 512: `read events after third run`
  ```rust
  .expect("read events after third run");
  ```

- Line 531: `testrepo should exist`
  ```rust
  verify_testrepo_exists().expect("testrepo should exist");
  ```

- Line 539: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 573: `insert draft before restart`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft).expect("insert draft before restart");
  ```

- Line 577: `get draft before restart`
  ```rust
  .expect("get draft before restart")
  ```

- Line 578: `draft should exist before restart`
  ```rust
  .expect("draft should exist before restart");
  ```

- Line 596: `re-init fleet.db after restart`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("re-init fleet.db after restart");
  ```

- Line 600: `get draft after restart`
  ```rust
  .expect("get draft after restart")
  ```

- Line 601: `draft should exist after restart`
  ```rust
  .expect("draft should exist after restart");
  ```

- Line 622: `testrepo should exist`
  ```rust
  verify_testrepo_exists().expect("testrepo should exist");
  ```

- Line 629: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 635: `write valid claim`
  ```rust
  worker.write_claim("bd-valid-001").expect("write valid claim");
  ```

- Line 636: `write valid dispatch`
  ```rust
  worker.write_dispatch("bd-valid-001").expect("write valid dispatch");
  ```

- Line 644: `open events.jsonl for corruption`
  ```rust
  .expect("open events.jsonl for corruption");
  ```

- Line 646: `write corrupted line`
  ```rust
  writeln!(file, "{{invalid json this is not valid at all").expect("write corrupted line");
  ```

- Line 650: `write valid claim after corruption`
  ```rust
  worker.write_claim("bd-valid-002").expect("write valid claim after corruption");
  ```

- Line 651: `write valid complete after corruption`
  ```rust
  worker.write_complete("bd-valid-001").expect("write valid complete after corruption");
  ```

- Line 655: `read events with corruption`
  ```rust
  .expect("read events with corruption");
  ```

- Line 683: `testrepo should exist`
  ```rust
  verify_testrepo_exists().expect("testrepo should exist");
  ```

- Line 690: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 696: `empty events.jsonl`
  ```rust
  fs::write(&events_path, "").expect("empty events.jsonl");
  ```

- Line 698: `create empty events.jsonl`
  ```rust
  fs::write(&events_path, "").expect("create empty events.jsonl");
  ```

- Line 703: `read empty events.jsonl`
  ```rust
  .expect("read empty events.jsonl");
  ```

#### unwrap (8 occurrences)

- Line 41: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 188: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 290: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 392: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 474: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 530: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 621: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 682: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

### hoop-daemon/tests/integration_harness.rs

Total errors: 169

#### anyhow (2 occurrences)

- Line 171: `Failed to parse event line {}: {}`
  ```rust
  .map_err(|e| anyhow::anyhow!("Failed to parse event line {}: {}", i + 1, e))?;
  ```

- Line 191: `Failed to parse heartbeat line {}: {}`
  ```rust
  .map_err(|e| anyhow::anyhow!("Failed to parse heartbeat line {}: {}", i + 1, e))?;
  ```

#### assert (21 occurrences)

- Line 773: `projects should be a list`
  ```rust
  assert!(projects.is_array(), "projects should be a list");
  ```

- Line 986: `bead id should not be empty`
  ```rust
  assert!(!bead.id.is_empty(), "bead id should not be empty");
  ```

- Line 987: `bead title should not be empty`
  ```rust
  assert!(!bead.title.is_empty(), "bead title should not be empty");
  ```

- Line 988: `bead project should not be empty`
  ```rust
  assert!(!bead.project.is_empty(), "bead project should not be empty");
  ```

- Line 1066: `Should receive init event`
  ```rust
  assert!(received_init, "Should receive init event");
  ```

- Line 1307: `Bead creation should succeed`
  ```rust
  assert!(create_resp.status().is_success(), "Bead creation should succeed");
  ```

- Line 1328: `Should be able to fetch beads`
  ```rust
  assert!(beads_resp.status().is_success(), "Should be able to fetch beads");
  ```

- Line 1410: `Non-existent bead should return error`
  ```rust
  assert!(resp.status() == 404 || resp.status() == 400, "Non-existent bead should return error");
  ```

- Line 1421: `Invalid JSON should return error`
  ```rust
  assert!(resp.status() == 400 || resp.status() == 422, "Invalid JSON should return error");
  ```

- Line 1439: `Metrics endpoint should return 200`
  ```rust
  assert!(metrics.status().is_success(), "Metrics endpoint should return 200");
  ```

- Line 1444: `Metrics should not be empty`
  ```rust
  assert!(!metrics_text.is_empty(), "Metrics should not be empty");
  ```

- Line 1456: `Metrics should contain at least one valid metric line`
  ```rust
  assert!(has_valid_metric, "Metrics should contain at least one valid metric line");
  ```

- Line 1476: `File listing should succeed`
  ```rust
  assert!(resp.status().is_success(), "File listing should succeed");
  ```

- Line 1481: `Files should be an array or object`
  ```rust
  assert!(files.is_array() || files.is_object(), "Files should be an array or object");
  ```

- Line 1506: `Bead creation should succeed`
  ```rust
  assert!(create_resp.status().is_success(), "Bead creation should succeed");
  ```

- Line 1518: `Getting bead should succeed`
  ```rust
  assert!(get_resp.status().is_success(), "Getting bead should succeed");
  ```

- Line 1531: `Listing beads should succeed`
  ```rust
  assert!(list_resp.status().is_success(), "Listing beads should succeed");
  ```

- Line 1535: `New bead should appear in list`
  ```rust
  assert!(found, "New bead should appear in list");
  ```

- Line 1553: `Capacity endpoint should return 200`
  ```rust
  assert!(resp.status().is_success(), "Capacity endpoint should return 200");
  ```

- Line 1558: `Capacity should be object or array`
  ```rust
  assert!(capacity.is_object() || capacity.is_array(), "Capacity should be object or array");
  ```

- Line 1576: `Config status endpoint should return 200`
  ```rust
  assert!(resp.status().is_success(), "Config status endpoint should return 200");
  ```

#### assert_eq (17 occurrences)

- Line 338: `Should have 2 open beads`
  ```rust
  assert_eq!(open_count, 2, "Should have 2 open beads");
  ```

- Line 339: `Should have 1 closed bead`
  ```rust
  assert_eq!(closed_count, 1, "Should have 1 closed bead");
  ```

- Line 721: `healthz should return 200`
  ```rust
  assert_eq!(resp.status(), 200, "healthz should return 200");
  ```

- Line 725: `healthz status should be ok`
  ```rust
  assert_eq!(body["status"], "ok", "healthz status should be ok");
  ```

- Line 734: `readyz should return 200`
  ```rust
  assert_eq!(resp.status(), 200, "readyz should return 200");
  ```

- Line 752: `GET /api/beads should return 200`
  ```rust
  assert_eq!(resp.status(), 200, "GET /api/beads should return 200");
  ```

- Line 766: `GET /api/projects should return 200`
  ```rust
  assert_eq!(resp.status(), 200, "GET /api/projects should return 200");
  ```

- Line 809: `First message should be init event`
  ```rust
  assert_eq!(event["type"], "init", "First message should be init event");
  ```

- Line 894: `Daemon should be healthy after boot`
  ```rust
  assert_eq!(resp.status(), 200, "Daemon should be healthy after boot");
  ```

- Line 903: `Should be able to read beads`
  ```rust
  assert_eq!(resp.status(), 200, "Should be able to read beads");
  ```

- Line 912: `Should be able to get projects`
  ```rust
  assert_eq!(resp.status(), 200, "Should be able to get projects");
  ```

- Line 1234: `Daemon should still be healthy after malformed messages`
  ```rust
  assert_eq!(resp.status(), 200, "Daemon should still be healthy after malformed messages");
  ```

- Line 1281: `All concurrent requests should succeed`
  ```rust
  assert_eq!(success_count, 10, "All concurrent requests should succeed");
  ```

- Line 1382: `All WebSocket connections should receive init`
  ```rust
  assert_eq!(success_count, 5, "All WebSocket connections should receive init");
  ```

- Line 1401: `Non-existent endpoint should return 404`
  ```rust
  assert_eq!(resp.status(), 404, "Non-existent endpoint should return 404");
  ```

- Line 1521: `Fetched bead ID should match`
  ```rust
  assert_eq!(fetched_bead["id"], bead["id"], "Fetched bead ID should match");
  ```

- Line 1522: `Fetched bead title should match`
  ```rust
  assert_eq!(fetched_bead["title"], "Integration test bead", "Fetched bead title should match");
  ```

#### bail (17 occurrences)

- Line 109: `testrepo should exist at {:?}`
  ```rust
  anyhow::bail!("testrepo should exist at {:?}", testrepo);
  ```

- Line 115: `testrepo/.beads/events.jsonl should exist`
  ```rust
  anyhow::bail!("testrepo/.beads/events.jsonl should exist");
  ```

- Line 121: `testrepo/.beads/heartbeats.jsonl should exist`
  ```rust
  anyhow::bail!("testrepo/.beads/heartbeats.jsonl should exist");
  ```

- Line 127: `events.jsonl should not be empty`
  ```rust
  anyhow::bail!("events.jsonl should not be empty");
  ```

- Line 136: `events.jsonl line {} is not valid JSON`
  ```rust
  anyhow::bail!("events.jsonl line {} is not valid JSON", i + 1);
  ```

- Line 143: `heartbeats.jsonl should not be empty`
  ```rust
  anyhow::bail!("heartbeats.jsonl should not be empty");
  ```

- Line 152: `heartbeats.jsonl line {} is not valid JSON`
  ```rust
  anyhow::bail!("heartbeats.jsonl line {} is not valid JSON", i + 1);
  ```

- Line 217: `Events fixture should contain at least one claim event`
  ```rust
  anyhow::bail!("Events fixture should contain at least one claim event");
  ```

- Line 220: `Events fixture should contain at least one dispatch event`
  ```rust
  anyhow::bail!("Events fixture should contain at least one dispatch event");
  ```

- Line 223: `Events fixture should contain at least one complete event`
  ```rust
  anyhow::bail!("Events fixture should contain at least one complete event");
  ```

- Line 226: `Events fixture should contain at least one fail event`
  ```rust
  anyhow::bail!("Events fixture should contain at least one fail event");
  ```

- Line 247: `Heartbeats fixture should contain at least one idle state`
  ```rust
  anyhow::bail!("Heartbeats fixture should contain at least one idle state");
  ```

- Line 250: `Heartbeats fixture should contain at least one executing state`
  ```rust
  anyhow::bail!("Heartbeats fixture should contain at least one executing state");
  ```

- Line 359: `projects.yaml should be created`
  ```rust
  anyhow::bail!("projects.yaml should be created");
  ```

- Line 365: `config.yml should be created`
  ```rust
  anyhow::bail!("config.yml should be created");
  ```

- Line 371: `projects.yaml should reference testrepo`
  ```rust
  anyhow::bail!("projects.yaml should reference testrepo");
  ```

- Line 698: `Daemon failed to become ready within 10 seconds`
  ```rust
  anyhow::bail!("Daemon failed to become ready within 10 seconds");
  ```

#### expect (108 occurrences)

- Line 33: `workspace root is parent of hoop-daemon/`
  ```rust
  .expect("workspace root is parent of hoop-daemon/")
  ```

- Line 61: `Failed to create temp dir for test HOOP home`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir for test HOOP home");
  ```

- Line 63: `Failed to create .hoop dir`
  ```rust
  fs::create_dir_all(&hoop_dir).expect("Failed to create .hoop dir");
  ```

- Line 79: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 88: `Failed to write config.yml`
  ```rust
  fs::write(hoop_dir.join("config.yml"), config_yaml).expect("Failed to write config.yml");
  ```

- Line 91: `Failed to create data dir`
  ```rust
  fs::create_dir_all(hoop_dir.join("data")).expect("Failed to create data dir");
  ```

- Line 125: `Failed to read events.jsonl`
  ```rust
  let events_content = fs::read_to_string(&events_path).expect("Failed to read events.jsonl");
  ```

- Line 141: `Failed to read heartbeats.jsonl`
  ```rust
  fs::read_to_string(&heartbeats_path).expect("Failed to read heartbeats.jsonl");
  ```

- Line 385: `testrepo fixtures should be valid`
  ```rust
  Assertions::testrepo_fixtures_valid().expect("testrepo fixtures should be valid");
  ```

- Line 391: `events should parse correctly`
  ```rust
  Assertions::events_parse_correctly().expect("events should parse correctly");
  ```

- Line 397: `heartbeats should parse correctly`
  ```rust
  Assertions::heartbeats_parse_correctly().expect("heartbeats should parse correctly");
  ```

- Line 403: `bead event data should extract`
  ```rust
  Assertions::bead_event_data_extracts().expect("bead event data should extract");
  ```

- Line 409: `bead projections should be correct`
  ```rust
  Assertions::bead_projections_correct().expect("bead projections should be correct");
  ```

- Line 415: `HOOP home setup should work`
  ```rust
  Assertions::hoop_home_setup_works().expect("HOOP home setup should work");
  ```

- Line 421: `Failed to parse events`
  ```rust
  let events = parse_testrepo_events().expect("Failed to parse events");
  ```

- Line 456: `Failed to parse heartbeats`
  ```rust
  let heartbeats = parse_testrepo_heartbeats().expect("Failed to parse heartbeats");
  ```

- Line 483: `Failed to parse events`
  ```rust
  let events = parse_testrepo_events().expect("Failed to parse events");
  ```

- Line 558: `Failed to parse events`
  ```rust
  let _events = parse_testrepo_events().expect("Failed to parse events");
  ```

- Line 710: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 719: `Failed to connect to healthz`
  ```rust
  .expect("Failed to connect to healthz");
  ```

- Line 723: `Failed to parse healthz response`
  ```rust
  let body: serde_json::Value = resp.json().await.expect("Failed to parse healthz response");
  ```

- Line 732: `Failed to connect to readyz`
  ```rust
  .expect("Failed to connect to readyz");
  ```

- Line 741: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 750: `Failed to GET /api/beads`
  ```rust
  .expect("Failed to GET /api/beads");
  ```

- Line 754: `Failed to parse beads response`
  ```rust
  let beads: Vec<hoop_daemon::Bead> = resp.json().await.expect("Failed to parse beads response");
  ```

- Line 764: `Failed to GET /api/projects`
  ```rust
  .expect("Failed to GET /api/projects");
  ```

- Line 771: `Failed to parse projects response`
  ```rust
  .expect("Failed to parse projects response");
  ```

- Line 784: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 793: `Failed to connect to WebSocket`
  ```rust
  .expect("Failed to connect to WebSocket");
  ```

- Line 800: `Timeout waiting for init message`
  ```rust
  .expect("Timeout waiting for init message")
  ```

- Line 801: `WebSocket stream ended`
  ```rust
  .expect("WebSocket stream ended");
  ```

- Line 803: `Failed to receive init message`
  ```rust
  let init_msg = init_msg.expect("Failed to receive init message");
  ```

- Line 807: `Failed to parse init event as JSON`
  ```rust
  serde_json::from_str(&text).expect("Failed to parse init event as JSON");
  ```

- Line 821: `Timeout waiting for workers_snapshot message`
  ```rust
  .expect("Timeout waiting for workers_snapshot message")
  ```

- Line 822: `WebSocket stream ended`
  ```rust
  .expect("WebSocket stream ended");
  ```

- Line 824: `Failed to receive workers_snapshot`
  ```rust
  let workers_msg = workers_msg.expect("Failed to receive workers_snapshot");
  ```

- Line 828: `Failed to parse workers_snapshot event as JSON`
  ```rust
  serde_json::from_str(&text).expect("Failed to parse workers_snapshot event as JSON");
  ```

- Line 839: `Timeout waiting for beads_snapshot message`
  ```rust
  .expect("Timeout waiting for beads_snapshot message")
  ```

- Line 840: `WebSocket stream ended`
  ```rust
  .expect("WebSocket stream ended");
  ```

- Line 842: `Failed to receive beads_snapshot`
  ```rust
  let beads_msg = beads_msg.expect("Failed to receive beads_snapshot");
  ```

- Line 846: `Failed to parse beads_snapshot event as JSON`
  ```rust
  serde_json::from_str(&text).expect("Failed to parse beads_snapshot event as JSON");
  ```

- Line 865: `Failed to send subscribe message`
  ```rust
  .expect("Failed to send subscribe message");
  ```

- Line 871: `Failed to send close frame`
  ```rust
  .expect("Failed to send close frame");
  ```

- Line 883: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 892: `Failed to connect to healthz`
  ```rust
  .expect("Failed to connect to healthz");
  ```

- Line 901: `Failed to GET /api/beads`
  ```rust
  .expect("Failed to GET /api/beads");
  ```

- Line 910: `Failed to GET /api/projects`
  ```rust
  .expect("Failed to GET /api/projects");
  ```

- Line 933: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 942: `Failed to GET /api/projects`
  ```rust
  .expect("Failed to GET /api/projects");
  ```

- Line 949: `Failed to parse projects response`
  ```rust
  .expect("Failed to parse projects response");
  ```

- Line 969: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 978: `Failed to GET /api/beads`
  ```rust
  .expect("Failed to GET /api/beads");
  ```

- Line 982: `Failed to parse beads response`
  ```rust
  let beads: Vec<hoop_daemon::Bead> = resp.json().await.expect("Failed to parse beads response");
  ```

- Line 996: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 1005: `Failed to GET /api/metrics`
  ```rust
  .expect("Failed to GET /api/metrics");
  ```

- Line 1009: `Failed to read metrics response`
  ```rust
  let body = resp.text().await.expect("Failed to read metrics response");
  ```

- Line 1026: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 1033: `Failed to connect to WebSocket`
  ```rust
  .expect("Failed to connect to WebSocket");
  ```

- Line 1081: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 1088: `Failed to connect to WebSocket`
  ```rust
  .expect("Failed to connect to WebSocket");
  ```

- Line 1095: `Timeout waiting for init`
  ```rust
  .expect("Timeout waiting for init");
  ```

- Line 1108: `Failed to send subscribe message`
  ```rust
  .expect("Failed to send subscribe message");
  ```

- Line 1151: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 1160: `Failed to connect to healthz`
  ```rust
  .expect("Failed to connect to healthz");
  ```

- Line 1183: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 1190: `Failed to connect to WebSocket`
  ```rust
  .expect("Failed to connect to WebSocket");
  ```

- Line 1197: `Timeout waiting for init`
  ```rust
  .expect("Timeout waiting for init");
  ```

- Line 1205: `Failed to send malformed message`
  ```rust
  .expect("Failed to send malformed message");
  ```

- Line 1217: `Failed to send unknown event type`
  ```rust
  .expect("Failed to send unknown event type");
  ```

- Line 1225: `Failed to send empty message`
  ```rust
  .expect("Failed to send empty message");
  ```

- Line 1232: `Health check failed`
  ```rust
  .expect("Health check failed");
  ```

- Line 1248: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 1275: `Task failed`
  ```rust
  let result = handle.await.expect("Task failed");
  ```

- Line 1291: `Failed to spawn first daemon`
  ```rust
  .expect("Failed to spawn first daemon");
  ```

- Line 1305: `Failed to create bead`
  ```rust
  .expect("Failed to create bead");
  ```

- Line 1309: `Failed to parse bead`
  ```rust
  let bead: serde_json::Value = create_resp.json().await.expect("Failed to parse bead");
  ```

- Line 1310: `Bead should have an ID`
  ```rust
  let bead_id = bead["id"].as_str().expect("Bead should have an ID");
  ```

- Line 1319: `Failed to spawn second daemon`
  ```rust
  .expect("Failed to spawn second daemon");
  ```

- Line 1326: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 1336: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 1376: `Task failed`
  ```rust
  let result = handle.await.expect("Task failed");
  ```

- Line 1390: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 1399: `Request failed`
  ```rust
  .expect("Request failed");
  ```

- Line 1408: `Request failed`
  ```rust
  .expect("Request failed");
  ```

- Line 1419: `Request failed`
  ```rust
  .expect("Request failed");
  ```

- Line 1429: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 1437: `Failed to fetch metrics`
  ```rust
  .expect("Failed to fetch metrics");
  ```

- Line 1441: `Failed to read metrics`
  ```rust
  let metrics_text = metrics.text().await.expect("Failed to read metrics");
  ```

- Line 1464: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 1474: `Failed to list files`
  ```rust
  .expect("Failed to list files");
  ```

- Line 1478: `Failed to parse files`
  ```rust
  let files: serde_json::Value = resp.json().await.expect("Failed to parse files");
  ```

- Line 1489: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 1504: `Failed to create bead`
  ```rust
  .expect("Failed to create bead");
  ```

- Line 1508: `Failed to parse bead`
  ```rust
  let bead: serde_json::Value = create_resp.json().await.expect("Failed to parse bead");
  ```

- Line 1509: `Bead should have an ID`
  ```rust
  let bead_id = bead["id"].as_str().expect("Bead should have an ID");
  ```

- Line 1516: `Failed to get bead`
  ```rust
  .expect("Failed to get bead");
  ```

- Line 1520: `Failed to parse fetched bead`
  ```rust
  let fetched_bead: serde_json::Value = get_resp.json().await.expect("Failed to parse fetched bead");
  ```

- Line 1529: `Failed to list beads`
  ```rust
  .expect("Failed to list beads");
  ```

- Line 1533: `Failed to parse beads list`
  ```rust
  let beads: Vec<serde_json::Value> = list_resp.json().await.expect("Failed to parse beads list");
  ```

- Line 1543: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 1551: `Failed to fetch capacity`
  ```rust
  .expect("Failed to fetch capacity");
  ```

- Line 1555: `Failed to parse capacity`
  ```rust
  let capacity: serde_json::Value = resp.json().await.expect("Failed to parse capacity");
  ```

- Line 1566: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 1574: `Failed to fetch config status`
  ```rust
  .expect("Failed to fetch config status");
  ```

- Line 1578: `Failed to parse config status`
  ```rust
  let config_status: serde_json::Value = resp.json().await.expect("Failed to parse config status");
  ```

- Line 1592: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 1601: `Failed to GET /api/beads`
  ```rust
  .expect("Failed to GET /api/beads");
  ```

- Line 1609: `Failed to GET /api/projects`
  ```rust
  .expect("Failed to GET /api/projects");
  ```

#### panic (1 occurrences)

- Line 815: `Expected text message, got {:?}`
  ```rust
  panic!("Expected text message, got {:?}", init_msg);
  ```

#### unwrap (3 occurrences)

- Line 59: `\.unwrap\(\)`
  ```rust
  let _guard = SETUP_LOCK.lock().unwrap();
  ```

- Line 634: `\.unwrap\(\)`
  ```rust
  let _guard = SETUP_LOCK.lock().unwrap();
  ```

- Line 954: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

### hoop-daemon/tests/lint_regex_global_state.rs

Total errors: 9

#### unwrap (9 occurrences)

- Line 117: `\.unwrap\(\)`
  ```rust
  let mut file = fs::File::create(temp_file).unwrap();
  ```

- Line 126: `\.unwrap\(\)`
  ```rust
  let re = BAD_RE.get_or_init(|| Regex::new(r"\d+").unwrap());
  ```

- Line 134: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 140: `\.unwrap\(\)`
  ```rust
  fs::remove_file(temp_file).unwrap();
  ```

- Line 156: `\.unwrap\(\)`
  ```rust
  let mut file = fs::File::create(temp_file).unwrap();
  ```

- Line 164: `\.unwrap\(\)`
  ```rust
  let re = Regex::new(r"\d+").unwrap();
  ```

- Line 174: `\.unwrap\(\)`
  ```rust
  let re = SAFE_RE.get_or_init(|| Regex::new(r"\d+").unwrap());
  ```

- Line 181: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 187: `\.unwrap\(\)`
  ```rust
  fs::remove_file(temp_file).unwrap();
  ```

### hoop-daemon/tests/load_test.rs

Total errors: 15

#### assert (1 occurrences)

- Line 285: `Should process events`
  ```rust
  assert!(report.total_events > 0, "Should process events");
  ```

#### expect (10 occurrences)

- Line 209: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 214: `Load test should complete`
  ```rust
  .expect("Load test should complete");
  ```

- Line 262: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 271: `Load test timed out after 10 minutes`
  ```rust
  .expect("Load test timed out after 10 minutes")
  ```

- Line 272: `Load test should complete`
  ```rust
  .expect("Load test should complete");
  ```

- Line 282: `Performance budgets must be satisfied`
  ```rust
  .expect("Performance budgets must be satisfied");
  ```

- Line 329: `Failed to spawn test daemon`
  ```rust
  .expect("Failed to spawn test daemon");
  ```

- Line 337: `Medium-scale load test timed out`
  ```rust
  .expect("Medium-scale load test timed out")
  ```

- Line 338: `Load test should complete`
  ```rust
  .expect("Load test should complete");
  ```

- Line 345: `Medium-scale load test should pass performance budgets`
  ```rust
  .expect("Medium-scale load test should pass performance budgets");
  ```

#### unwrap (4 occurrences)

- Line 85: `\.unwrap\(\)`
  ```rust
  let temp_dir = tempfile::TempDir::new().unwrap();
  ```

- Line 87: `\.unwrap\(\)`
  ```rust
  generator.write_to_disk(temp_dir.path()).unwrap();
  ```

- Line 114: `\.unwrap\(\)`
  ```rust
  let events_content = std::fs::read_to_string(&events_path).unwrap();
  ```

- Line 116: `\.unwrap\(\)`
  ```rust
  let _: NeedleEvent = serde_json::from_str(line).unwrap();
  ```

### hoop-daemon/tests/load_test_integration.rs

Total errors: 25

#### assert (2 occurrences)

- Line 360: `Load test should pass all budgets`
  ```rust
  assert!(report.passed, "Load test should pass all budgets");
  ```

- Line 492: `Load test should pass all budgets`
  ```rust
  assert!(report.passed, "Load test should pass all budgets");
  ```

#### assert_eq (1 occurrences)

- Line 83: `Daemon should be healthy`
  ```rust
  assert_eq!(health.status(), 200, "Daemon should be healthy");
  ```

#### expect (19 occurrences)

- Line 72: `Failed to spawn daemon with load test data`
  ```rust
  .expect("Failed to spawn daemon with load test data");
  ```

- Line 81: `Health check request failed`
  ```rust
  .expect("Health check request failed");
  ```

- Line 100: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 167: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 219: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 286: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 345: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 350: `Load test failed`
  ```rust
  .expect("Load test failed");
  ```

- Line 355: `Performance budget violations detected`
  ```rust
  .expect("Performance budget violations detected");
  ```

- Line 380: `Failed to populate testrepo with load test data`
  ```rust
  .expect("Failed to populate testrepo with load test data");
  ```

- Line 396: `Failed to create project directory`
  ```rust
  fs::create_dir_all(&project_path).expect("Failed to create project directory");
  ```

- Line 410: `Failed to serialize projects.yaml`
  ```rust
  .expect("Failed to serialize projects.yaml");
  ```

- Line 412: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 467: `Failed to spawn daemon with load test data`
  ```rust
  .expect("Failed to spawn daemon with load test data");
  ```

- Line 472: `Failed to write daemon URL to file`
  ```rust
  .expect("Failed to write daemon URL to file");
  ```

- Line 480: `Load test failed`
  ```rust
  .expect("Load test failed");
  ```

- Line 490: `Performance budget violations detected - blocking merge per hoop-ttb.7.11`
  ```rust
  .expect("Performance budget violations detected - blocking merge per hoop-ttb.7.11");
  ```

- Line 528: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 557: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

#### panic (1 occurrences)

- Line 306: `Failed to connect WS client {}: {}`
  ```rust
  panic!("Failed to connect WS client {}: {}", i, e);
  ```

#### unwrap (2 occurrences)

- Line 372: `\.unwrap\(\)`
  ```rust
  let hoop_dir = config.control_socket_path.parent().unwrap(); // .hoop
  ```

- Line 373: `\.unwrap\(\)`
  ```rust
  let temp_dir = hoop_dir.parent().unwrap(); // temp dir
  ```

### hoop-daemon/tests/multi_operator_concurrency.rs

Total errors: 72

#### assert (7 occurrences)

- Line 187: `last_autosave_at should be set`
  ```rust
  assert!(fetched.last_autosave_at.is_some(), "last_autosave_at should be set");
  ```

- Line 236: `abandoned_at should be set`
  ```rust
  assert!(fetched.abandoned_at.is_some(), "abandoned_at should be set");
  ```

- Line 287: `should detect similar existing draft`
  ```rust
  assert!(!similar.is_empty(), "should detect similar existing draft");
  ```

- Line 372: `proposal should be approved`
  ```rust
  assert!(approved, "proposal should be approved");
  ```

- Line 385: `should have approved entries`
  ```rust
  assert!(!approved_list.is_empty(), "should have approved entries");
  ```

- Line 422: `proposal should be rejected`
  ```rust
  assert!(rejected, "proposal should be rejected");
  ```

- Line 430: `rejected proposal should not appear in proposed list`
  ```rust
  assert!(proposal.is_none(), "rejected proposal should not appear in proposed list");
  ```

#### assert_eq (8 occurrences)

- Line 186: `version should NOT increment on autosave`
  ```rust
  assert_eq!(fetched.version, 1, "version should NOT increment on autosave");
  ```

- Line 318: `duplicate proposal should return the same ID`
  ```rust
  assert_eq!(id_a, id_b, "duplicate proposal should return the same ID");
  ```

- Line 324: `should have only one proposal`
  ```rust
  assert_eq!(proposals.len(), 1, "should have only one proposal");
  ```

- Line 325: `proposal ID should match first proposal`
  ```rust
  assert_eq!(proposals[0].id, id_a, "proposal ID should match first proposal");
  ```

- Line 330: `should have 3 merged source stitches`
  ```rust
  assert_eq!(stitches.len(), 3, "should have 3 merged source stitches");
  ```

- Line 485: `hidden presence should not be returned`
  ```rust
  assert_eq!(presence.len(), 0, "hidden presence should not be returned");
  ```

- Line 520: `stale presence should be filtered out`
  ```rust
  assert_eq!(presence.len(), 0, "stale presence should be filtered out");
  ```

- Line 625: `both operator sessions should coexist`
  ```rust
  assert_eq!(active_sessions.len(), 2, "both operator sessions should coexist");
  ```

#### expect (55 occurrences)

- Line 26: `create temp dir`
  ```rust
  let tmp = TempDir::new().expect("create temp dir");
  ```

- Line 28: `create .hoop dir`
  ```rust
  std::fs::create_dir_all(&hoop_dir).expect("create .hoop dir");
  ```

- Line 34: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 114: `insert draft_a`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft_a).expect("insert draft_a");
  ```

- Line 115: `insert draft_b`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft_b).expect("insert draft_b");
  ```

- Line 119: `get draft_a`
  ```rust
  .expect("get draft_a")
  ```

- Line 120: `draft_a exists`
  ```rust
  .expect("draft_a exists");
  ```

- Line 124: `get draft_b`
  ```rust
  .expect("get draft_b")
  ```

- Line 125: `draft_b exists`
  ```rust
  .expect("draft_b exists");
  ```

- Line 166: `insert draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");
  ```

- Line 176: `autosave draft`
  ```rust
  ).expect("autosave draft");
  ```

- Line 179: `get draft`
  ```rust
  .expect("get draft")
  ```

- Line 180: `draft exists`
  ```rust
  .expect("draft exists");
  ```

- Line 226: `insert draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");
  ```

- Line 229: `abandon draft`
  ```rust
  hoop_daemon::fleet::abandon_draft("draft-abandon-test").expect("abandon draft");
  ```

- Line 232: `get draft`
  ```rust
  .expect("get draft")
  ```

- Line 233: `draft exists`
  ```rust
  .expect("draft exists");
  ```

- Line 276: `insert existing draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&existing).expect("insert existing draft");
  ```

- Line 284: `detect similar drafts`
  ```rust
  ).expect("detect similar drafts");
  ```

- Line 307: `propose from operator A`
  ```rust
  ).expect("propose from operator A");
  ```

- Line 315: `propose from operator B`
  ```rust
  ).expect("propose from operator B");
  ```

- Line 322: `list proposals`
  ```rust
  .expect("list proposals");
  ```

- Line 329: `parse source_stitches`
  ```rust
  .expect("parse source_stitches");
  ```

- Line 364: `insert proposal`
  ```rust
  .expect("insert proposal");
  ```

- Line 370: `approve proposal`
  ```rust
  ).expect("approve proposal");
  ```

- Line 376: `get proposal`
  ```rust
  .expect("get proposal")
  ```

- Line 377: `proposal exists`
  ```rust
  .expect("proposal exists");
  ```

- Line 383: `list approved entries`
  ```rust
  .expect("list approved entries");
  ```

- Line 416: `insert proposal`
  ```rust
  .expect("insert proposal");
  ```

- Line 420: `reject proposal`
  ```rust
  .expect("reject proposal");
  ```

- Line 426: `get proposal`
  ```rust
  .expect("get proposal");
  ```

- Line 451: `update presence`
  ```rust
  ).expect("update presence");
  ```

- Line 457: `query presence`
  ```rust
  ).expect("query presence");
  ```

- Line 477: `update presence hidden`
  ```rust
  ).expect("update presence hidden");
  ```

- Line 483: `query presence`
  ```rust
  ).expect("query presence");
  ```

- Line 498: `_HOOP_FLEET_DB_PATH not set`
  ```rust
  std::env::var("_HOOP_FLEET_DB_PATH").expect("_HOOP_FLEET_DB_PATH not set")
  ```

- Line 500: `open db`
  ```rust
  let conn = rusqlite::Connection::open(&db_path).expect("open db");
  ```

- Line 512: `insert stale presence`
  ```rust
  ).expect("insert stale presence");
  ```

- Line 518: `query presence`
  ```rust
  ).expect("query presence");
  ```

- Line 535: `update presence`
  ```rust
  ).expect("update presence");
  ```

- Line 541: `query presence`
  ```rust
  ).expect("query presence");
  ```

- Line 549: `remove presence`
  ```rust
  ).expect("remove presence");
  ```

- Line 555: `query presence`
  ```rust
  ).expect("query presence");
  ```

- Line 591: `insert session A`
  ```rust
  .expect("insert session A");
  ```

- Line 613: `insert session B`
  ```rust
  .expect("insert session B");
  ```

- Line 617: `list agent sessions`
  ```rust
  .expect("list agent sessions");
  ```

- Line 665: `insert draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");
  ```

- Line 668: `get draft`
  ```rust
  .expect("get draft")
  ```

- Line 669: `draft exists`
  ```rust
  .expect("draft exists");
  ```

- Line 702: `create stitch A`
  ```rust
  ).expect("create stitch A");
  ```

- Line 713: `create stitch B`
  ```rust
  ).expect("create stitch B");
  ```

- Line 717: `load stitch A`
  ```rust
  .expect("load stitch A")
  ```

- Line 718: `stitch A exists`
  ```rust
  .expect("stitch A exists");
  ```

- Line 721: `load stitch B`
  ```rust
  .expect("load stitch B")
  ```

- Line 722: `stitch B exists`
  ```rust
  .expect("stitch B exists");
  ```

#### unwrap (2 occurrences)

- Line 24: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 41: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

### hoop-daemon/tests/mutation_handler_test.rs

Total errors: 26

#### assert (1 occurrences)

- Line 289: `Should reject empty title`
  ```rust
  assert!(result.is_err(), "Should reject empty title");
  ```

#### unwrap (25 occurrences)

- Line 169: `\.unwrap\(\)`
  ```rust
  let broadcast_state = rx.recv().await.unwrap();
  ```

- Line 178: `\.unwrap\(\)`
  ```rust
  let error = broadcast_state.error.unwrap();
  ```

- Line 207: `\.unwrap\(\)`
  ```rust
  assert!(reject.field.unwrap().starts_with("auth:"));
  ```

- Line 211: `\.unwrap\(\)`
  ```rust
  let broadcast_state = rx.recv().await.unwrap();
  ```

- Line 216: `\.unwrap\(\)`
  ```rust
  let error = broadcast_state.error.unwrap();
  ```

- Line 217: `\.unwrap\(\)`
  ```rust
  assert!(error.field.unwrap().starts_with("auth:"));
  ```

- Line 240: `\.unwrap\(\)`
  ```rust
  assert!(reject.field.unwrap().starts_with("contention:"));
  ```

- Line 244: `\.unwrap\(\)`
  ```rust
  let broadcast_state = rx.recv().await.unwrap();
  ```

- Line 250: `\.unwrap\(\)`
  ```rust
  let error = broadcast_state.error.unwrap();
  ```

- Line 251: `\.unwrap\(\)`
  ```rust
  assert!(error.field.unwrap().starts_with("contention:"));
  ```

- Line 277: `\.unwrap\(\)`
  ```rust
  service.approve_draft("draft-1", "Title A").await.unwrap();
  ```

- Line 278: `\.unwrap\(\)`
  ```rust
  let accept_event = rx.recv().await.unwrap();
  ```

- Line 290: `\.unwrap\(\)`
  ```rust
  let reject_event = rx.recv().await.unwrap();
  ```

- Line 318: `\.unwrap\(\)`
  ```rust
  let event1 = rx.recv().await.unwrap();
  ```

- Line 325: `\.unwrap\(\)`
  ```rust
  let event2 = rx.recv().await.unwrap();
  ```

- Line 333: `\.unwrap\(\)`
  ```rust
  let event3 = rx.recv().await.unwrap();
  ```

- Line 339: `\.unwrap\(\)`
  ```rust
  event1.error.as_ref().unwrap().message,
  ```

- Line 343: `\.unwrap\(\)`
  ```rust
  event2.error.as_ref().unwrap().message,
  ```

- Line 349: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 369: `\.unwrap\(\)`
  ```rust
  let reject_event = rx.recv().await.unwrap();
  ```

- Line 375: `\.unwrap\(\)`
  ```rust
  let accept_event = rx.recv().await.unwrap();
  ```

- Line 410: `\.unwrap\(\)`
  ```rust
  let event = rx.recv().await.unwrap();
  ```

- Line 411: `\.unwrap\(\)`
  ```rust
  let error = event.error.unwrap();
  ```

- Line 436: `\.unwrap\(\)`
  ```rust
  let event = rx.recv().await.unwrap();
  ```

- Line 437: `\.unwrap\(\)`
  ```rust
  let error = event.error.unwrap();
  ```

### hoop-daemon/tests/needle_events_roundtrip.rs

Total errors: 65

#### assert (18 occurrences)

- Line 115: `claim: worker must be non-empty`
  ```rust
  assert!(!worker.is_empty(), "claim: worker must be non-empty");
  ```

- Line 116: `claim: bead must start with `
  ```rust
  assert!(bead.starts_with("bd-"), "claim: bead must start with 'bd-'");
  ```

- Line 142: `dispatch: worker must be non-empty`
  ```rust
  assert!(!worker.is_empty(), "dispatch: worker must be non-empty");
  ```

- Line 151: `dispatch in fixture should include model`
  ```rust
  assert!(model.is_some(), "dispatch in fixture should include model");
  ```

- Line 174: `complete: worker must be non-empty`
  ```rust
  assert!(!worker.is_empty(), "complete: worker must be non-empty");
  ```

- Line 212: `fail: worker must be non-empty`
  ```rust
  assert!(!worker.is_empty(), "fail: worker must be non-empty");
  ```

- Line 213: `fail: bead must start with `
  ```rust
  assert!(bead.starts_with("bd-"), "fail: bead must start with 'bd-'");
  ```

- Line 214: `fail in fixture should include error`
  ```rust
  assert!(error.is_some(), "fail in fixture should include error");
  ```

- Line 234: `release: worker must be non-empty`
  ```rust
  assert!(!worker.is_empty(), "release: worker must be non-empty");
  ```

- Line 254: `timeout: worker must be non-empty`
  ```rust
  assert!(!worker.is_empty(), "timeout: worker must be non-empty");
  ```

- Line 279: `crash: worker must be non-empty`
  ```rust
  assert!(!worker.is_empty(), "crash: worker must be non-empty");
  ```

- Line 280: `crash: bead must start with `
  ```rust
  assert!(bead.starts_with("bd-"), "crash: bead must start with 'bd-'");
  ```

- Line 456: `heartbeat: worker must be non-empty`
  ```rust
  assert!(!hb.worker.is_empty(), "heartbeat: worker must be non-empty");
  ```

- Line 463: `executing: pid must be positive`
  ```rust
  assert!(pid > 0, "executing: pid must be positive");
  ```

- Line 464: `executing: adapter must be non-empty`
  ```rust
  assert!(!adapter.is_empty(), "executing: adapter must be non-empty");
  ```

- Line 481: `heartbeat: worker must be non-empty`
  ```rust
  assert!(!hb.worker.is_empty(), "heartbeat: worker must be non-empty");
  ```

- Line 496: `heartbeat: worker must be non-empty`
  ```rust
  assert!(!hb.worker.is_empty(), "heartbeat: worker must be non-empty");
  ```

- Line 499: `knot: reason must be non-empty`
  ```rust
  assert!(!reason.is_empty(), "knot: reason must be non-empty");
  ```

#### expect (16 occurrences)

- Line 25: `workspace root is parent of hoop-daemon/`
  ```rust
  .expect("workspace root is parent of hoop-daemon/")
  ```

- Line 64: `testrepo/.beads/events.jsonl must be readable`
  ```rust
  .expect("testrepo/.beads/events.jsonl must be readable");
  ```

- Line 85: `testrepo/.beads/heartbeats.jsonl must be readable`
  ```rust
  .expect("testrepo/.beads/heartbeats.jsonl must be readable");
  ```

- Line 106: `fixture must have a claim event`
  ```rust
  .expect("fixture must have a claim event");
  ```

- Line 132: `fixture must have a dispatch event`
  ```rust
  .expect("fixture must have a dispatch event");
  ```

- Line 163: `fixture must have a complete event`
  ```rust
  .expect("fixture must have a complete event");
  ```

- Line 202: `fixture must have a fail event`
  ```rust
  .expect("fixture must have a fail event");
  ```

- Line 230: `fixture must have a release event`
  ```rust
  .expect("fixture must have a release event");
  ```

- Line 250: `fixture must have a timeout event`
  ```rust
  .expect("fixture must have a timeout event");
  ```

- Line 270: `fixture must have a crash event`
  ```rust
  .expect("fixture must have a crash event");
  ```

- Line 451: `fixture must have an executing heartbeat`
  ```rust
  .expect("fixture must have an executing heartbeat");
  ```

- Line 454: `executing heartbeat must parse successfully`
  ```rust
  .expect("executing heartbeat must parse successfully");
  ```

- Line 476: `fixture must have an idle heartbeat`
  ```rust
  .expect("fixture must have an idle heartbeat");
  ```

- Line 479: `idle heartbeat must parse successfully`
  ```rust
  .expect("idle heartbeat must parse successfully");
  ```

- Line 491: `fixture must have a knot heartbeat`
  ```rust
  .expect("fixture must have a knot heartbeat");
  ```

- Line 494: `knot heartbeat must parse successfully`
  ```rust
  .expect("knot heartbeat must parse successfully");
  ```

#### panic (10 occurrences)

- Line 41: `Failed to parse event line: {e}\n  Line: {line}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to parse event line: {e}\n  Line: {line}"))
  ```

- Line 122: `Expected Claim, got {other:?}`
  ```rust
  other => panic!("Expected Claim, got {other:?}"),
  ```

- Line 153: `Expected Dispatch, got {other:?}`
  ```rust
  other => panic!("Expected Dispatch, got {other:?}"),
  ```

- Line 192: `Expected Complete, got {other:?}`
  ```rust
  other => panic!("Expected Complete, got {other:?}"),
  ```

- Line 220: `Expected Fail, got {other:?}`
  ```rust
  other => panic!("Expected Fail, got {other:?}"),
  ```

- Line 240: `Expected Release, got {other:?}`
  ```rust
  other => panic!("Expected Release, got {other:?}"),
  ```

- Line 260: `Expected Timeout, got {other:?}`
  ```rust
  other => panic!("Expected Timeout, got {other:?}"),
  ```

- Line 286: `Expected Crash, got {other:?}`
  ```rust
  other => panic!("Expected Crash, got {other:?}"),
  ```

- Line 466: `Expected Executing state, got {other:?}`
  ```rust
  other => panic!("Expected Executing state, got {other:?}"),
  ```

- Line 501: `Expected Knot state, got {other:?}`
  ```rust
  other => panic!("Expected Knot state, got {other:?}"),
  ```

#### unwrap (21 occurrences)

- Line 102: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(events_fixture_path()).unwrap();
  ```

- Line 128: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(events_fixture_path()).unwrap();
  ```

- Line 159: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(events_fixture_path()).unwrap();
  ```

- Line 198: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(events_fixture_path()).unwrap();
  ```

- Line 226: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(events_fixture_path()).unwrap();
  ```

- Line 246: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(events_fixture_path()).unwrap();
  ```

- Line 266: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(events_fixture_path()).unwrap();
  ```

- Line 294: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(events_fixture_path()).unwrap();
  ```

- Line 313: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(events_fixture_path()).unwrap();
  ```

- Line 328: `\.unwrap\(\)`
  ```rust
  let data = data.unwrap();
  ```

- Line 356: `\.unwrap\(\)`
  ```rust
  let data = BeadEventData::from_event(&event).unwrap();
  ```

- Line 372: `\.unwrap\(\)`
  ```rust
  let data = BeadEventData::from_event(&event).unwrap();
  ```

- Line 389: `\.unwrap\(\)`
  ```rust
  let data = BeadEventData::from_event(&event).unwrap();
  ```

- Line 407: `\.unwrap\(\)`
  ```rust
  let data = BeadEventData::from_event(&event).unwrap();
  ```

- Line 425: `\.unwrap\(\)`
  ```rust
  let data = BeadEventData::from_event(&event).unwrap();
  ```

- Line 447: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(heartbeats_fixture_path()).unwrap();
  ```

- Line 472: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(heartbeats_fixture_path()).unwrap();
  ```

- Line 487: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(heartbeats_fixture_path()).unwrap();
  ```

- Line 508: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(heartbeats_fixture_path()).unwrap();
  ```

- Line 526: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(heartbeats_fixture_path()).unwrap();
  ```

- Line 554: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(heartbeats_fixture_path()).unwrap();
  ```

### hoop-daemon/tests/observer_mode_integration.rs

Total errors: 8

#### unwrap (8 occurrences)

- Line 18: `\.unwrap\(\)`
  ```rust
  let primary_default: SocketAddr = "127.0.0.1:3000".parse().unwrap();
  ```

- Line 19: `\.unwrap\(\)`
  ```rust
  let observer_default: SocketAddr = "127.0.0.1:3001".parse().unwrap();
  ```

- Line 32: `\.unwrap\(\)`
  ```rust
  bind_addr: "127.0.0.1:3001".parse().unwrap(),
  ```

- Line 36: `\.unwrap\(\)`
  ```rust
  primary_addr: "127.0.0.1:3000".parse().unwrap(),
  ```

- Line 62: `\.unwrap\(\)`
  ```rust
  let primary_addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
  ```

- Line 75: `\.unwrap\(\)`
  ```rust
  let primary_addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
  ```

- Line 102: `\.unwrap\(\)`
  ```rust
  let primary_addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
  ```

- Line 118: `\.unwrap\(\)`
  ```rust
  event_tx.send(test_event).unwrap();
  ```

### hoop-daemon/tests/orphans_integration.rs

Total errors: 19

#### assert (4 occurrences)

- Line 151: `attach_orphan_to_stitch should succeed`
  ```rust
  assert!(result.is_ok(), "attach_orphan_to_stitch should succeed");
  ```

- Line 162: `stitch_beads link should exist with relationship=`
  ```rust
  assert!(link_exists, "stitch_beads link should exist with relationship='referenced'");
  ```

- Line 171: `duplicate attach should succeed (idempotent)`
  ```rust
  assert!(result2.is_ok(), "duplicate attach should succeed (idempotent)");
  ```

- Line 230: `attach should succeed when link already exists`
  ```rust
  assert!(result.is_ok(), "attach should succeed when link already exists");
  ```

#### assert_eq (2 occurrences)

- Line 182: `should have exactly one stitch_beads row`
  ```rust
  assert_eq!(count, 1, "should have exactly one stitch_beads row");
  ```

- Line 240: `existing relationship should be preserved`
  ```rust
  assert_eq!(relationship, "created-here", "existing relationship should be preserved");
  ```

#### unwrap (13 occurrences)

- Line 14: `\.unwrap\(\)`
  ```rust
  let tmp = TempDir::new().unwrap();
  ```

- Line 19: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&beads_dir).unwrap();
  ```

- Line 23: `\.unwrap\(\)`
  ```rust
  fs::write(&issues_path, "").unwrap();
  ```

- Line 112: `\.unwrap\(\)`
  ```rust
  let json = serde_json::to_string(&orphan).unwrap();
  ```

- Line 120: `\.unwrap\(\)`
  ```rust
  let tmp = TempDir::new().unwrap();
  ```

- Line 127: `\.unwrap\(\)`
  ```rust
  let conn = Connection::open(&db_path).unwrap();
  ```

- Line 138: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 180: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 187: `\.unwrap\(\)`
  ```rust
  let tmp = TempDir::new().unwrap();
  ```

- Line 194: `\.unwrap\(\)`
  ```rust
  let conn = Connection::open(&db_path).unwrap();
  ```

- Line 205: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 219: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 238: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

### hoop-daemon/tests/output_capture_helpers/mod.rs

Total errors: 28

#### assert (5 occurrences)

- Line 791: `Verification should pass when content matches`
  ```rust
  assert!(result.passed, "Verification should pass when content matches");
  ```

- Line 810: `Verification should fail when content differs`
  ```rust
  assert!(!result.passed, "Verification should fail when content differs");
  ```

- Line 831: `Verification should fail when lengths differ`
  ```rust
  assert!(!result.passed, "Verification should fail when lengths differ");
  ```

- Line 850: `Should handle unicode and special characters`
  ```rust
  assert!(result.passed, "Should handle unicode and special characters");
  ```

- Line 893: `Large output verification should pass`
  ```rust
  assert!(result.passed, "Large output verification should pass");
  ```

#### unwrap (23 occurrences)

- Line 90: `\.unwrap\(\)`
  ```rust
  io::stdout().flush().unwrap();
  ```

- Line 96: `\.unwrap\(\)`
  ```rust
  io::stderr().flush().unwrap();
  ```

- Line 103: `\.unwrap\(\)`
  ```rust
  OutputStream::Stdout => io::stdout().flush().unwrap(),
  ```

- Line 104: `\.unwrap\(\)`
  ```rust
  OutputStream::Stderr => io::stderr().flush().unwrap(),
  ```

- Line 132: `\.unwrap\(\)`
  ```rust
  io::stdout().flush().unwrap();
  ```

- Line 133: `\.unwrap\(\)`
  ```rust
  io::stderr().flush().unwrap();
  ```

- Line 143: `\.unwrap\(\)`
  ```rust
  io::stdout().flush().unwrap();
  ```

- Line 144: `\.unwrap\(\)`
  ```rust
  io::stderr().flush().unwrap();
  ```

- Line 193: `\.unwrap\(\)`
  ```rust
  io::stdout().flush().unwrap();
  ```

- Line 194: `\.unwrap\(\)`
  ```rust
  io::stderr().flush().unwrap();
  ```

- Line 448: `\.unwrap\(\)`
  ```rust
  io::stdout().flush().unwrap();
  ```

- Line 786: `\.unwrap\(\)`
  ```rust
  fs::write(&log_path, log_content).unwrap();
  ```

- Line 789: `\.unwrap\(\)`
  ```rust
  let result = verify_stdout_char_by_char(expected, &log_path).unwrap();
  ```

- Line 805: `\.unwrap\(\)`
  ```rust
  fs::write(&log_path, log_content).unwrap();
  ```

- Line 808: `\.unwrap\(\)`
  ```rust
  let result = verify_stdout_char_by_char(expected, &log_path).unwrap();
  ```

- Line 826: `\.unwrap\(\)`
  ```rust
  fs::write(&log_path, log_content).unwrap();
  ```

- Line 829: `\.unwrap\(\)`
  ```rust
  let result = verify_stdout_char_by_char(expected, &log_path).unwrap();
  ```

- Line 845: `\.unwrap\(\)`
  ```rust
  fs::write(&log_path, log_content).unwrap();
  ```

- Line 848: `\.unwrap\(\)`
  ```rust
  let result = verify_stdout_char_by_char(expected, &log_path).unwrap();
  ```

- Line 863: `\.unwrap\(\)`
  ```rust
  fs::write(&log_path, log_content).unwrap();
  ```

- Line 865: `\.unwrap\(\)`
  ```rust
  let extracted = extract_raw_stdout_from_log(&log_path).unwrap();
  ```

- Line 890: `\.unwrap\(\)`
  ```rust
  fs::write(&log_path, log_lines).unwrap();
  ```

- Line 892: `\.unwrap\(\)`
  ```rust
  let result = verify_large_stdout_output(&config, &log_path).unwrap();
  ```

### hoop-daemon/tests/panic_isolation.rs

Total errors: 5

#### unwrap (5 occurrences)

- Line 32: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&beads_dir).unwrap();
  ```

- Line 34: `\.unwrap\(\)`
  ```rust
  std::fs::write(&issues_path, b"").unwrap();
  ```

- Line 37: `\.unwrap\(\)`
  ```rust
  tempfile::TempDir::new().unwrap()
  ```

- Line 43: `\.unwrap\(\)`
  ```rust
  let project1_dir = tempfile::tempdir().unwrap();
  ```

- Line 47: `\.unwrap\(\)`
  ```rust
  let project2_dir = tempfile::tempdir().unwrap();
  ```

### hoop-daemon/tests/path_traversal_hardening.rs

Total errors: 11

#### expect (1 occurrences)

- Line 147: `allowlist construction must succeed`
  ```rust
  PathAllowlist::for_workspace(ws.path()).expect("allowlist construction must succeed");
  ```

#### unwrap (10 occurrences)

- Line 20: `\.unwrap\(\)`
  ```rust
  let tmp = TempDir::new().unwrap();
  ```

- Line 21: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(tmp.path().join(".beads").join("attachments")).unwrap();
  ```

- Line 242: `\.unwrap\(\)`
  ```rust
  let tmp = TempDir::new().unwrap();
  ```

- Line 244: `\.unwrap\(\)`
  ```rust
  let al = PathAllowlist::for_uploads(&uploads_dir).unwrap();
  ```

- Line 248: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&fake_upload).unwrap();
  ```

- Line 259: `\.unwrap\(\)`
  ```rust
  let tmp = TempDir::new().unwrap();
  ```

- Line 261: `\.unwrap\(\)`
  ```rust
  let al = PathAllowlist::for_uploads(&uploads_dir).unwrap();
  ```

- Line 283: `\.unwrap\(\)`
  ```rust
  let tmp = TempDir::new().unwrap();
  ```

- Line 292: `\.unwrap\(\)`
  ```rust
  let registry = UploadRegistry::new(config).unwrap();
  ```

- Line 301: `\.unwrap\(\)`
  ```rust
  let valid_id = ValidUploadId::parse(fake_uuid).unwrap();
  ```

### hoop-daemon/tests/pattern_query_evaluator_integration.rs

Total errors: 66

#### assert (13 occurrences)

- Line 172: `query should match the stitch title`
  ```rust
  assert!(results[0].matched, "query should match the stitch title");
  ```

- Line 173: `query should not be slow`
  ```rust
  assert!(!results[0].is_slow, "query should not be slow");
  ```

- Line 181: `first insert should succeed`
  ```rust
  assert!(inserted, "first insert should succeed");
  ```

- Line 189: `second insert should return false (idempotent)`
  ```rust
  assert!(!inserted_again, "second insert should return false (idempotent)");
  ```

- Line 367: `should parse query `
  ```rust
  assert!(result.is_ok(), "should parse query '{}': {:?}", query, result.err());
  ```

- Line 375: `AND query should match`
  ```rust
  assert!(matches, "AND query should match");
  ```

- Line 382: `NOT query should match`
  ```rust
  assert!(matches, "NOT query should match");
  ```

- Line 389: `OR query should match`
  ```rust
  assert!(matches, "OR query should match");
  ```

- Line 396: `non-matching query should not match`
  ```rust
  assert!(!matches, "non-matching query should not match");
  ```

- Line 420: `kind:operator should match operator stitch`
  ```rust
  assert!(matches_operator, "kind:operator should match operator stitch");
  ```

- Line 423: `kind:operator should not match worker stitch`
  ```rust
  assert!(!matches_worker, "kind:operator should not match worker stitch");
  ```

- Line 439: `standalone word should match as label`
  ```rust
  assert!(matches, "standalone word should match as label");
  ```

- Line 444: `non-matching standalone word should not match`
  ```rust
  assert!(!matches, "non-matching standalone word should not match");
  ```

#### assert_eq (4 occurrences)

- Line 170: `should have 1 pattern query result`
  ```rust
  assert_eq!(results.len(), 1, "should have 1 pattern query result");
  ```

- Line 199: `should have exactly 1 pattern member`
  ```rust
  assert_eq!(count, 1, "should have exactly 1 pattern member");
  ```

- Line 332: `should have 3 pattern query results`
  ```rust
  assert_eq!(results.len(), 3, "should have 3 pattern query results");
  ```

- Line 336: `should match 2 patterns`
  ```rust
  assert_eq!(matched.len(), 2, "should match 2 patterns");
  ```

#### unwrap (49 occurrences)

- Line 15: `\.unwrap\(\)`
  ```rust
  let tmp = TempDir::new().unwrap();
  ```

- Line 19: `\.unwrap\(\)`
  ```rust
  let mut conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 20: `\.unwrap\(\)`
  ```rust
  conn.pragma_update(None, "journal_mode", "WAL").unwrap();
  ```

- Line 39: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 54: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 60: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 76: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 82: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 88: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 111: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 127: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 130: `\.unwrap\(\)`
  ```rust
  std::env::set_var("_HOOP_FLEET_DB_PATH", db_path.to_str().unwrap());
  ```

- Line 138: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 145: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 156: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 168: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 180: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 188: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 198: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 206: `\.unwrap\(\)`
  ```rust
  let tmp = TempDir::new().unwrap();
  ```

- Line 209: `\.unwrap\(\)`
  ```rust
  let mut conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 210: `\.unwrap\(\)`
  ```rust
  conn.pragma_update(None, "journal_mode", "WAL").unwrap();
  ```

- Line 229: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 243: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 265: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 267: `\.unwrap\(\)`
  ```rust
  std::env::set_var("_HOOP_FLEET_DB_PATH", db_path.to_str().unwrap());
  ```

- Line 275: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 281: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 288: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 294: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 301: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 307: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 318: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 330: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 373: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 374: `\.unwrap\(\)`
  ```rust
  let matches = hoop_daemon::pattern_query_evaluator::evaluate_query(&and_expr, &ctx).unwrap();
  ```

- Line 380: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 381: `\.unwrap\(\)`
  ```rust
  let matches = hoop_daemon::pattern_query_evaluator::evaluate_query(&not_expr, &ctx).unwrap();
  ```

- Line 387: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 388: `\.unwrap\(\)`
  ```rust
  let matches = hoop_daemon::pattern_query_evaluator::evaluate_query(&or_expr, &ctx).unwrap();
  ```

- Line 394: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 395: `\.unwrap\(\)`
  ```rust
  let matches = hoop_daemon::pattern_query_evaluator::evaluate_query(&non_match_expr, &ctx).unwrap();
  ```

- Line 417: `\.unwrap\(\)`
  ```rust
  let expr = hoop_daemon::pattern_query_evaluator::parse_query("kind:operator").unwrap();
  ```

- Line 419: `\.unwrap\(\)`
  ```rust
  let matches_operator = hoop_daemon::pattern_query_evaluator::evaluate_query(&expr, &ctx_operator).unwrap();
  ```

- Line 422: `\.unwrap\(\)`
  ```rust
  let matches_worker = hoop_daemon::pattern_query_evaluator::evaluate_query(&expr, &ctx_worker).unwrap();
  ```

- Line 437: `\.unwrap\(\)`
  ```rust
  let expr = hoop_daemon::pattern_query_evaluator::parse_query("urgent").unwrap();
  ```

- Line 438: `\.unwrap\(\)`
  ```rust
  let matches = hoop_daemon::pattern_query_evaluator::evaluate_query(&expr, &ctx).unwrap();
  ```

- Line 442: `\.unwrap\(\)`
  ```rust
  let expr = hoop_daemon::pattern_query_evaluator::parse_query("p0").unwrap();
  ```

- Line 443: `\.unwrap\(\)`
  ```rust
  let matches = hoop_daemon::pattern_query_evaluator::evaluate_query(&expr, &ctx).unwrap();
  ```

### hoop-daemon/tests/per_project_redaction_integration.rs

Total errors: 13

#### assert (2 occurrences)

- Line 277: `customer-data should allow clean content`
  ```rust
  assert!(result.is_ok(), "customer-data should allow clean content");
  ```

- Line 307: `customer-data should block Anthropic keys`
  ```rust
  assert!(result.is_err(), "customer-data should block Anthropic keys");
  ```

#### panic (3 occurrences)

- Line 103: `Expected Variant0 project`
  ```rust
  panic!("Expected Variant0 project");
  ```

- Line 118: `Expected Variant0 project`
  ```rust
  panic!("Expected Variant0 project");
  ```

- Line 128: `Expected Variant0 project`
  ```rust
  panic!("Expected Variant0 project");
  ```

#### unwrap (8 occurrences)

- Line 97: `\.unwrap\(\)`
  ```rust
  let policy = redaction.as_ref().unwrap();
  ```

- Line 112: `\.unwrap\(\)`
  ```rust
  let policy = redaction.as_ref().unwrap();
  ```

- Line 138: `\.unwrap\(\)`
  ```rust
  let rt = tokio::runtime::Runtime::new().unwrap();
  ```

- Line 172: `\.unwrap\(\)`
  ```rust
  let rt = tokio::runtime::Runtime::new().unwrap();
  ```

- Line 240: `\.unwrap\(\)`
  ```rust
  let rt = tokio::runtime::Runtime::new().unwrap();
  ```

- Line 297: `\.unwrap\(\)`
  ```rust
  let rt = tokio::runtime::Runtime::new().unwrap();
  ```

- Line 357: `\.unwrap\(\)`
  ```rust
  let rt = tokio::runtime::Runtime::new().unwrap();
  ```

- Line 374: `\.unwrap\(\)`
  ```rust
  let rt = tokio::runtime::Runtime::new().unwrap();
  ```

### hoop-daemon/tests/performance_budget.rs

Total errors: 23

#### assert_eq (1 occurrences)

- Line 171: `Expected {} projects`
  ```rust
  assert_eq!(project_count, NUM_PROJECTS, "Expected {} projects", NUM_PROJECTS);
  ```

#### expect (13 occurrences)

- Line 64: `Failed to populate testrepo with load test data`
  ```rust
  .expect("Failed to populate testrepo with load test data");
  ```

- Line 81: `Failed to create project directory`
  ```rust
  fs::create_dir_all(&project_path).expect("Failed to create project directory");
  ```

- Line 98: `Failed to serialize projects.yaml`
  ```rust
  .expect("Failed to serialize projects.yaml");
  ```

- Line 100: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 111: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 125: `healthz request failed`
  ```rust
  .expect("healthz request failed");
  ```

- Line 141: `readyz request failed`
  ```rust
  .expect("readyz request failed");
  ```

- Line 157: `projects request failed`
  ```rust
  .expect("projects request failed");
  ```

- Line 181: `metrics request failed`
  ```rust
  .expect("metrics request failed");
  ```

- Line 248: `Failed to populate testrepo`
  ```rust
  .expect("Failed to populate testrepo");
  ```

- Line 261: `Failed to create project directory`
  ```rust
  fs::create_dir_all(&project_path).expect("Failed to create project directory");
  ```

- Line 279: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 288: `readyz request failed`
  ```rust
  .expect("readyz request failed");
  ```

#### unwrap (9 occurrences)

- Line 51: `\.unwrap\(\)`
  ```rust
  let hoop_dir = config.control_socket_path.parent().unwrap(); // .hoop
  ```

- Line 52: `\.unwrap\(\)`
  ```rust
  let temp_dir = hoop_dir.parent().unwrap(); // temp dir
  ```

- Line 169: `\.unwrap\(\)`
  ```rust
  let projects_json: Value = projects_resp.json().await.unwrap();
  ```

- Line 170: `\.unwrap\(\)`
  ```rust
  let project_count = projects_json.as_array().unwrap().len();
  ```

- Line 193: `\.unwrap\(\)`
  ```rust
  let metrics_text = metrics_resp.text().await.unwrap();
  ```

- Line 235: `\.unwrap\(\)`
  ```rust
  let hoop_dir = cfg.control_socket_path.parent().unwrap();
  ```

- Line 236: `\.unwrap\(\)`
  ```rust
  let temp_dir = hoop_dir.parent().unwrap();
  ```

- Line 275: `\.unwrap\(\)`
  ```rust
  let updated_yaml = serde_yaml::to_string(&existing_projects).unwrap();
  ```

- Line 276: `\.unwrap\(\)`
  ```rust
  fs::write(&projects_yaml_path, updated_yaml).unwrap();
  ```

### hoop-daemon/tests/phase2_exit_gate.rs

Total errors: 2

#### assert_eq (1 occurrences)

- Line 448: `Phase 2 must have exactly 13 core deliverables`
  ```rust
  assert_eq!(deliverables.len(), 13, "Phase 2 must have exactly 13 core deliverables");
  ```

#### expect (1 occurrences)

- Line 438: `Report must serialize to JSON`
  ```rust
  let json = serde_json::to_string(&report).expect("Report must serialize to JSON");
  ```

### hoop-daemon/tests/privacy_surface_audit.rs

Total errors: 2

#### assert (2 occurrences)

- Line 141: `should find secrets`
  ```rust
  assert!(!findings.is_empty(), "should find secrets");
  ```

- Line 152: `finding match_len must be > 0; got: {f:?}`
  ```rust
  assert!(f.match_len > 0, "finding match_len must be > 0; got: {f:?}");
  ```

### hoop-daemon/tests/projection_file_audit.rs

Total errors: 12

#### expect (4 occurrences)

- Line 195: `CARGO_MANIFEST_DIR not set`
  ```rust
  let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
  ```

- Line 198: `workspace root is the parent of hoop-daemon/`
  ```rust
  .expect("workspace root is the parent of hoop-daemon/")
  ```

- Line 216: `valid regex`
  ```rust
  .map(|p| Regex::new(p).expect("valid regex"))
  ```

- Line 436: `valid regex`
  ```rust
  .map(|p| Regex::new(p).expect("valid regex"))
  ```

#### panic (2 occurrences)

- Line 229: `failed to read {}: {}`
  ```rust
  Err(e) => panic!("failed to read {}: {}", file.display(), e),
  ```

- Line 262: `{}`
  ```rust
  panic!("{}", msg);
  ```

#### unwrap (6 occurrences)

- Line 305: `\.unwrap\(\)`
  ```rust
  let code = r#"std::fs::write("fleet_status.json", &encoded).unwrap();"#;
  ```

- Line 318: `\.unwrap\(\)`
  ```rust
  let code = r#"std::fs::write("live-workers.json", &bytes).unwrap();"#;
  ```

- Line 331: `\.unwrap\(\)`
  ```rust
  let code = r#"std::fs::File::create("fleet_state.json").unwrap();"#;
  ```

- Line 344: `\.unwrap\(\)`
  ```rust
  let code = r#"std::fs::write("fleet_state.yaml", &serialized).unwrap();"#;
  ```

- Line 357: `\.unwrap\(\)`
  ```rust
  let code = r#"std::fs::write("live-fleet.json", data).unwrap();"#;
  ```

- Line 423: `\.unwrap\(\)`
  ```rust
  let code = r#"std::fs::write("worker_status.json", &data).unwrap();"#;
  ```

### hoop-daemon/tests/property_invariants.rs

Total errors: 46

#### assert_eq (8 occurrences)

- Line 270: `Event {} timestamp mismatch`
  ```rust
  assert_eq!(ts1, ts2, "Event {} timestamp mismatch", i);
  ```

- Line 273: `Event {} timestamp mismatch`
  ```rust
  assert_eq!(ts1, ts2, "Event {} timestamp mismatch", i);
  ```

- Line 276: `Event {} timestamp mismatch`
  ```rust
  assert_eq!(ts1, ts2, "Event {} timestamp mismatch", i);
  ```

- Line 379: `First and second calls differ`
  ```rust
  prop_assert_eq!(status1, status2, "First and second calls differ");
  ```

- Line 380: `Second and third calls differ`
  ```rust
  prop_assert_eq!(status2, status3, "Second and third calls differ");
  ```

- Line 579: `Status derivation is non-deterministic`
  ```rust
  assert_eq!(results.len(), 1, "Status derivation is non-deterministic");
  ```

- Line 853: `First and second replays differ`
  ```rust
  prop_assert_eq!(replay1, replay2, "First and second replays differ");
  ```

- Line 854: `Second and third replays differ`
  ```rust
  prop_assert_eq!(replay2, replay3, "Second and third replays differ");
  ```

#### panic (1 occurrences)

- Line 278: `Event type mismatch at index {}`
  ```rust
  _ => panic!("Event type mismatch at index {}", i),
  ```

#### unwrap (37 occurrences)

- Line 216: `\.unwrap\(\)`
  ```rust
  let tmp_dir = TempDir::new().unwrap();
  ```

- Line 220: `\.unwrap\(\)`
  ```rust
  let base_ts = Utc::now().with_nanosecond(0).unwrap();
  ```

- Line 247: `\.unwrap\(\)`
  ```rust
  let mut file = File::create(&events_path).unwrap();
  ```

- Line 249: `\.unwrap\(\)`
  ```rust
  let json = serde_json::to_string(event).unwrap();
  ```

- Line 250: `\.unwrap\(\)`
  ```rust
  writeln!(file, "{}", json).unwrap();
  ```

- Line 256: `\.unwrap\(\)`
  ```rust
  let file = File::open(&events_path).unwrap();
  ```

- Line 259: `\.unwrap\(\)`
  ```rust
  let line = line.unwrap();
  ```

- Line 666: `\.unwrap\(\)`
  ```rust
  let tmp_dir = TempDir::new().unwrap();
  ```

- Line 670: `\.unwrap\(\)`
  ```rust
  let mut file = File::create(&events_path).unwrap();
  ```

- Line 672: `\.unwrap\(\)`
  ```rust
  let json = serde_json::to_string(event).unwrap();
  ```

- Line 673: `\.unwrap\(\)`
  ```rust
  writeln!(file, "{}", json).unwrap();
  ```

- Line 678: `\.unwrap\(\)`
  ```rust
  let file = File::open(&events_path).unwrap();
  ```

- Line 681: `\.unwrap\(\)`
  ```rust
  let line = line.unwrap();
  ```

- Line 823: `\.unwrap\(\)`
  ```rust
  let tmp_dir = TempDir::new().unwrap();
  ```

- Line 828: `\.unwrap\(\)`
  ```rust
  let mut file = File::create(&events_path).unwrap();
  ```

- Line 830: `\.unwrap\(\)`
  ```rust
  let json = serde_json::to_string(event).unwrap();
  ```

- Line 831: `\.unwrap\(\)`
  ```rust
  writeln!(file, "{}", json).unwrap();
  ```

- Line 838: `\.unwrap\(\)`
  ```rust
  let file = File::open(path).unwrap();
  ```

- Line 841: `\.unwrap\(\)`
  ```rust
  let line = line.unwrap();
  ```

- Line 862: `\.unwrap\(\)`
  ```rust
  let tmp_dir = TempDir::new().unwrap();
  ```

- Line 867: `\.unwrap\(\)`
  ```rust
  let mut file = File::create(&events_path).unwrap();
  ```

- Line 868: `\.unwrap\(\)`
  ```rust
  writeln!(file, r#"{{"event":"claim","ts":"2026-04-21T18:42:10Z","worker":"alpha","bead":"bd-1"}}"#).unwrap();
  ```

- Line 869: `\.unwrap\(\)`
  ```rust
  writeln!(file, r#"{{"event":"dispatch","ts":"2026-04-21T18:42:11Z","worker":"alpha","bead":"bd-1"}}"#).unwrap();
  ```

- Line 874: `\.unwrap\(\)`
  ```rust
  let file = File::open(&events_path).unwrap();
  ```

- Line 876: `\.unwrap\(\)`
  ```rust
  reader.lines().map(|l| l.unwrap()).collect::<Vec<_>>()
  ```

- Line 881: `\.unwrap\(\)`
  ```rust
  let mut file = File::create(&events_path).unwrap();
  ```

- Line 882: `\.unwrap\(\)`
  ```rust
  writeln!(file, r#"{{"event":"claim","ts":"2026-04-21T18:43:10Z","worker":"beta","bead":"bd-2"}}"#).unwrap();
  ```

- Line 887: `\.unwrap\(\)`
  ```rust
  let file = File::open(&events_path).unwrap();
  ```

- Line 889: `\.unwrap\(\)`
  ```rust
  reader.lines().map(|l| l.unwrap()).collect::<Vec<_>>()
  ```

- Line 901: `\.unwrap\(\)`
  ```rust
  let tmp_dir = TempDir::new().unwrap();
  ```

- Line 906: `\.unwrap\(\)`
  ```rust
  let mut file = File::create(&events_path).unwrap();
  ```

- Line 907: `\.unwrap\(\)`
  ```rust
  writeln!(file, r#"{{"event":"claim","ts":"2026-04-21T18:42:10Z","worker":"alpha","bead":"bd-1"}}"#).unwrap();
  ```

- Line 908: `\.unwrap\(\)`
  ```rust
  writeln!(file, r#"{{"event":"invalid"#).unwrap(); // Malformed: missing closing brace
  ```

- Line 909: `\.unwrap\(\)`
  ```rust
  writeln!(file, r#"not json at all"#).unwrap(); // Malformed: not JSON
  ```

- Line 910: `\.unwrap\(\)`
  ```rust
  writeln!(file, r#"{{"event":"dispatch","ts":"2026-04-21T18:42:11Z","worker":"alpha","bead":"bd-1"}}"#).unwrap();
  ```

- Line 916: `\.unwrap\(\)`
  ```rust
  let file = File::open(&events_path).unwrap();
  ```

- Line 919: `\.unwrap\(\)`
  ```rust
  let line = line.unwrap();
  ```

### hoop-daemon/tests/protocol_contract.rs

Total errors: 59

#### assert (1 occurrences)

- Line 698: `fixture {} must be a JSON object`
  ```rust
  assert!(val.is_object(), "fixture {} must be a JSON object", path);
  ```

#### assert_eq (1 occurrences)

- Line 87: `field `
  ```rust
  assert_eq!(actual, expected, "field '{}' value mismatch", key);
  ```

#### expect (2 occurrences)

- Line 24: `workspace root`
  ```rust
  .expect("workspace root")
  ```

- Line 47: `CreateDraftRequest must deserialize from fixture (daemon side)`
  ```rust
  .expect("CreateDraftRequest must deserialize from fixture (daemon side)");
  ```

#### panic (5 occurrences)

- Line 28: `fixture file missing: {}`
  ```rust
  .unwrap_or_else(|_| panic!("fixture file missing: {}", path.display()));
  ```

- Line 30: `invalid JSON in fixture {}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("invalid JSON in fixture {}: {}", path.display(), e))
  ```

- Line 268: `expected ControlResponse::Status`
  ```rust
  _ => panic!("expected ControlResponse::Status"),
  ```

- Line 288: `expected ControlResponse::Error`
  ```rust
  _ => panic!("expected ControlResponse::Error"),
  ```

- Line 654: `fixture {} must deserialize as WsEvent: {}`
  ```rust
  .unwrap_or_else(|e| panic!("fixture {} must deserialize as WsEvent: {}", path, e));
  ```

#### unwrap (50 occurrences)

- Line 49: `\.unwrap\(\)`
  ```rust
  assert_eq!(req.project, fixture["project"].as_str().unwrap());
  ```

- Line 50: `\.unwrap\(\)`
  ```rust
  assert_eq!(req.title, fixture["title"].as_str().unwrap());
  ```

- Line 51: `\.unwrap\(\)`
  ```rust
  assert_eq!(req.kind, fixture["kind"].as_str().unwrap());
  ```

- Line 52: `\.unwrap\(\)`
  ```rust
  assert_eq!(req.source, fixture["source"].as_str().unwrap());
  ```

- Line 74: `\.unwrap\(\)`
  ```rust
  draft_id: fixture["draft_id"].as_str().unwrap().to_string(),
  ```

- Line 75: `\.unwrap\(\)`
  ```rust
  status: fixture["status"].as_str().unwrap().to_string(),
  ```

- Line 78: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&resp).unwrap();
  ```

- Line 80: `\.unwrap\(\)`
  ```rust
  for (key, expected) in fixture.as_object().unwrap() {
  ```

- Line 109: `\.unwrap\(\)`
  ```rust
  id: fixture_stitch["id"].as_str().unwrap().to_string(),
  ```

- Line 110: `\.unwrap\(\)`
  ```rust
  project: fixture_stitch["project"].as_str().unwrap().to_string(),
  ```

- Line 111: `\.unwrap\(\)`
  ```rust
  kind: fixture_stitch["kind"].as_str().unwrap().to_string(),
  ```

- Line 112: `\.unwrap\(\)`
  ```rust
  title: fixture_stitch["title"].as_str().unwrap().to_string(),
  ```

- Line 113: `\.unwrap\(\)`
  ```rust
  created_by: fixture_stitch["created_by"].as_str().unwrap().to_string(),
  ```

- Line 114: `\.unwrap\(\)`
  ```rust
  created_at: fixture_stitch["created_at"].as_str().unwrap().to_string(),
  ```

- Line 117: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 124: `\.unwrap\(\)`
  ```rust
  id: fixture_msg["id"].as_str().unwrap().to_string(),
  ```

- Line 125: `\.unwrap\(\)`
  ```rust
  ts: fixture_msg["ts"].as_str().unwrap().to_string(),
  ```

- Line 126: `\.unwrap\(\)`
  ```rust
  role: fixture_msg["role"].as_str().unwrap().to_string(),
  ```

- Line 127: `\.unwrap\(\)`
  ```rust
  content: fixture_msg["content"].as_str().unwrap().to_string(),
  ```

- Line 133: `\.unwrap\(\)`
  ```rust
  total_tokens: fixture_cost["total_tokens"].as_i64().unwrap(),
  ```

- Line 134: `\.unwrap\(\)`
  ```rust
  message_count: fixture_cost["message_count"].as_u64().unwrap() as usize,
  ```

- Line 135: `\.unwrap\(\)`
  ```rust
  wall_clock: fixture_cost["wall_clock"].as_str().unwrap().to_string(),
  ```

- Line 150: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&resp).unwrap();
  ```

- Line 153: `\.unwrap\(\)`
  ```rust
  for key in fixture.as_object().unwrap().keys() {
  ```

- Line 163: `\.unwrap\(\)`
  ```rust
  for key in fixture_stitch.as_object().unwrap().keys() {
  ```

- Line 173: `\.unwrap\(\)`
  ```rust
  for key in fixture_msg.as_object().unwrap().keys() {
  ```

- Line 183: `\.unwrap\(\)`
  ```rust
  for key in fixture_cost.as_object().unwrap().keys() {
  ```

- Line 207: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_string(&req).unwrap();
  ```

- Line 208: `\.unwrap\(\)`
  ```rust
  let parsed: ControlRequest = serde_json::from_str(&serialized).unwrap();
  ```

- Line 223: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_string(&req).unwrap();
  ```

- Line 224: `\.unwrap\(\)`
  ```rust
  let parsed: ControlRequest = serde_json::from_str(&serialized).unwrap();
  ```

- Line 251: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_string(&resp).unwrap();
  ```

- Line 252: `\.unwrap\(\)`
  ```rust
  let parsed: ControlResponse = serde_json::from_str(&serialized).unwrap();
  ```

- Line 281: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_string(&resp).unwrap();
  ```

- Line 282: `\.unwrap\(\)`
  ```rust
  let parsed: ControlResponse = serde_json::from_str(&serialized).unwrap();
  ```

- Line 308: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&event).unwrap();
  ```

- Line 337: `\.unwrap\(\)`
  ```rust
  last_heartbeat: fixture["worker"]["last_heartbeat"].as_str().unwrap().parse().unwrap(),
  ```

- Line 337: `\.unwrap\(\)`
  ```rust
  last_heartbeat: fixture["worker"]["last_heartbeat"].as_str().unwrap().parse().unwrap(),
  ```

- Line 367: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&event).unwrap();
  ```

- Line 397: `\.unwrap\(\)`
  ```rust
  last_heartbeat: "2026-04-26T10:00:00Z".parse().unwrap(),
  ```

- Line 403: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&event).unwrap();
  ```

- Line 432: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&event).unwrap();
  ```

- Line 455: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&event).unwrap();
  ```

- Line 482: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&event).unwrap();
  ```

- Line 507: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&event).unwrap();
  ```

- Line 537: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&event).unwrap();
  ```

- Line 564: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&event).unwrap();
  ```

- Line 589: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&event).unwrap();
  ```

- Line 622: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&event).unwrap();
  ```

- Line 657: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&event).unwrap();
  ```

### hoop-daemon/tests/pure_functions.rs

Total errors: 27

#### assert (7 occurrences)

- Line 351: `ANSI strip too slow: {:?}`
  ```rust
  assert!(ansi_time.as_millis() < 100, "ANSI strip too slow: {:?}", ansi_time);
  ```

- Line 359: `Cost functions too slow: {:?}`
  ```rust
  assert!(cost_time.as_millis() < 10, "Cost functions too slow: {:?}", cost_time);
  ```

- Line 368: `Embedding too slow: {:?}`
  ```rust
  assert!(embed_time.as_millis() < 500, "Embedding too slow: {:?}", embed_time);
  ```

- Line 376: `Similarity too slow: {:?}`
  ```rust
  assert!(similarity_time.as_millis() < 50, "Similarity too slow: {:?}", similarity_time);
  ```

- Line 398: `Status derivation too slow: {:?}`
  ```rust
  assert!(status_time.as_millis() < 1000, "Status derivation too slow: {:?}", status_time);
  ```

- Line 406: `Tag join too slow: {:?}`
  ```rust
  assert!(tag_time.as_millis() < 100, "Tag join too slow: {:?}", tag_time);
  ```

- Line 415: `Prompt substitute too slow: {:?}`
  ```rust
  assert!(sub_time.as_millis() < 100, "Prompt substitute too slow: {:?}", sub_time);
  ```

#### assert_eq (4 occurrences)

- Line 193: `world`
  ```rust
  assert_eq!(tokens, vec!["hello", "world", "foo", "bar"]);
  ```

- Line 199: `world`
  ```rust
  assert_eq!(tokens, vec!["hello", "world", "foo", "bar"]);
  ```

- Line 294: `file`
  ```rust
  assert_eq!(vars, vec!["custom", "file", "project"]);
  ```

- Line 307: `{\`
  ```rust
  assert_eq!(lines, vec!["{\"a\":1}", "{\"b\":2}"]);
  ```

#### expect (4 occurrences)

- Line 238: `sanitize should not fail`
  ```rust
  let result = svg_sanitize::sanitize(svg.as_bytes()).expect("sanitize should not fail");
  ```

- Line 247: `sanitize should not fail`
  ```rust
  let result = svg_sanitize::sanitize(svg.as_bytes()).expect("sanitize should not fail");
  ```

- Line 260: `sanitize should not fail`
  ```rust
  let result = pdf_sanitize::sanitize(pdf).expect("sanitize should not fail");
  ```

- Line 269: `sanitize should not fail`
  ```rust
  let result = pdf_sanitize::sanitize(pdf).expect("sanitize should not fail");
  ```

#### panic (1 occurrences)

- Line 544: `Expected Quiet with 999 days`
  ```rust
  _ => panic!("Expected Quiet with 999 days"),
  ```

#### unwrap (11 occurrences)

- Line 149: `\.unwrap\(\)`
  ```rust
  assert_eq!(result.binding.unwrap().worker, "alpha");
  ```

- Line 240: `\.unwrap\(\)`
  ```rust
  let out = String::from_utf8(result.safe_bytes).unwrap();
  ```

- Line 249: `\.unwrap\(\)`
  ```rust
  let out = String::from_utf8(result.safe_bytes).unwrap();
  ```

- Line 262: `\.unwrap\(\)`
  ```rust
  let out = String::from_utf8(result.safe_bytes).unwrap();
  ```

- Line 280: `\.unwrap\(\)`
  ```rust
  let result = prompt_substitute::substitute("Working on {{project}}", &ctx).unwrap();
  ```

- Line 473: `\.unwrap\(\)`
  ```rust
  assert_eq!(result.unwrap(), "p p p");
  ```

- Line 499: `\.unwrap\(\)`
  ```rust
  let result = svg_sanitize::sanitize(b"").unwrap();
  ```

- Line 503: `\.unwrap\(\)`
  ```rust
  let result = svg_sanitize::sanitize(b"   \n\t  ").unwrap();
  ```

- Line 508: `\.unwrap\(\)`
  ```rust
  let result = svg_sanitize::sanitize(svg).unwrap();
  ```

- Line 516: `\.unwrap\(\)`
  ```rust
  let result = pdf_sanitize::sanitize(b"%PDF-1.4\n%%EOF\n").unwrap();
  ```

- Line 521: `\.unwrap\(\)`
  ```rust
  let result = pdf_sanitize::sanitize(pdf).unwrap();
  ```

### hoop-daemon/tests/quarantine_integration.rs

Total errors: 59

#### assert (1 occurrences)

- Line 62: `quarantine dir should exist`
  ```rust
  assert!(quarantine_dir.exists(), "quarantine dir should exist");
  ```

#### assert_eq (11 occurrences)

- Line 57: `should parse 3 good lines`
  ```rust
  assert_eq!(good.len(), 3, "should parse 3 good lines");
  ```

- Line 58: `should quarantine 1 bad line`
  ```rust
  assert_eq!(quarantined, 1, "should quarantine 1 bad line");
  ```

- Line 59: `should skip 1 empty line`
  ```rust
  assert_eq!(empty, 1, "should skip 1 empty line");
  ```

- Line 67: `should have one date directory`
  ```rust
  assert_eq!(date_dirs.len(), 1, "should have one date directory");
  ```

- Line 73: `should have one quarantined entry`
  ```rust
  assert_eq!(entries.len(), 1, "should have one quarantined entry");
  ```

- Line 218: `Codex should parse 4 good lines`
  ```rust
  assert_eq!(codex_good, 4, "Codex should parse 4 good lines");
  ```

- Line 219: `Codex should quarantine 1 bad line`
  ```rust
  assert_eq!(codex_quarantined, 1, "Codex should quarantine 1 bad line");
  ```

- Line 220: `Gemini should parse 3 good lines`
  ```rust
  assert_eq!(gemini_good, 3, "Gemini should parse 3 good lines");
  ```

- Line 221: `Gemini should quarantine 1 bad line`
  ```rust
  assert_eq!(gemini_quarantined, 1, "Gemini should quarantine 1 bad line");
  ```

- Line 228: `should have one date directory`
  ```rust
  assert_eq!(date_dirs.len(), 1, "should have one date directory");
  ```

- Line 234: `should have two quarantined entries (one per adapter)`
  ```rust
  assert_eq!(entries.len(), 2, "should have two quarantined entries (one per adapter)");
  ```

#### unwrap (47 occurrences)

- Line 22: `\.unwrap\(\)`
  ```rust
  let _guard = QUARANTINE_TEST_LOCK.lock().unwrap();
  ```

- Line 23: `\.unwrap\(\)`
  ```rust
  let tmp = TempDir::new().unwrap();
  ```

- Line 27: `\.unwrap\(\)`
  ```rust
  std::env::set_var("HOOP_QUARANTINE_DIR", quarantine_dir.to_str().unwrap());
  ```

- Line 30: `\.unwrap\(\)`
  ```rust
  let mut f = fs::File::create(&jsonl_path).unwrap();
  ```

- Line 31: `\.unwrap\(\)`
  ```rust
  writeln!(f, r#"{{"key": "good1"}}"#).unwrap();
  ```

- Line 32: `\.unwrap\(\)`
  ```rust
  writeln!(f, "THIS IS NOT JSON AT ALL").unwrap();
  ```

- Line 33: `\.unwrap\(\)`
  ```rust
  writeln!(f, r#"{{"key": "good2"}}"#).unwrap();
  ```

- Line 34: `\.unwrap\(\)`
  ```rust
  writeln!(f, "").unwrap();
  ```

- Line 35: `\.unwrap\(\)`
  ```rust
  writeln!(f, r#"{{"key": "good3"}}"#).unwrap();
  ```

- Line 38: `\.unwrap\(\)`
  ```rust
  let content = fs::read_to_string(&jsonl_path).unwrap();
  ```

- Line 64: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 66: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 70: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 72: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 77: `\.unwrap\(\)`
  ```rust
  serde_json::from_str(&fs::read_to_string(entries[0].path()).unwrap()).unwrap();
  ```

- Line 77: `\.unwrap\(\)`
  ```rust
  serde_json::from_str(&fs::read_to_string(entries[0].path()).unwrap()).unwrap();
  ```

- Line 79: `\.unwrap\(\)`
  ```rust
  assert!(entry["line"].as_str().unwrap().contains("NOT JSON"));
  ```

- Line 82: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 91: `\.unwrap\(\)`
  ```rust
  let _guard = QUARANTINE_TEST_LOCK.lock().unwrap();
  ```

- Line 92: `\.unwrap\(\)`
  ```rust
  let tmp = TempDir::new().unwrap();
  ```

- Line 95: `\.unwrap\(\)`
  ```rust
  std::env::set_var("HOOP_QUARANTINE_DIR", quarantine_dir.to_str().unwrap());
  ```

- Line 111: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 113: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 117: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 119: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 123: `\.unwrap\(\)`
  ```rust
  serde_json::from_str(&fs::read_to_string(entries[0].path()).unwrap()).unwrap();
  ```

- Line 123: `\.unwrap\(\)`
  ```rust
  serde_json::from_str(&fs::read_to_string(entries[0].path()).unwrap()).unwrap();
  ```

- Line 126: `\.unwrap\(\)`
  ```rust
  assert!(entry["reason"].as_str().unwrap().contains("timestamp"));
  ```

- Line 141: `\.unwrap\(\)`
  ```rust
  let tmp = TempDir::new().unwrap();
  ```

- Line 144: `\.unwrap\(\)`
  ```rust
  std::env::set_var("HOOP_QUARANTINE_DIR", quarantine_dir.to_str().unwrap());
  ```

- Line 148: `\.unwrap\(\)`
  ```rust
  let mut f = fs::File::create(&codex_file).unwrap();
  ```

- Line 149: `\.unwrap\(\)`
  ```rust
  writeln!(f, r#"{{"type":"session_start","session_id":"codex-123","cwd":"/tmp"}}"#).unwrap();
  ```

- Line 150: `\.unwrap\(\)`
  ```rust
  writeln!(f, r#"{{"type":"message","role":"user","content":"Hello","timestamp":"2025-01-01T00:00:00Z"}}"#).unwrap();
  ```

- Line 151: `\.unwrap\(\)`
  ```rust
  writeln!(f, "THIS IS NOT JSON - CORRUPT CODEX LINE").unwrap();
  ```

- Line 152: `\.unwrap\(\)`
  ```rust
  writeln!(f, r#"{{"type":"message","role":"assistant","content":"Hi","timestamp":"2025-01-01T00:00:01Z"}}"#).unwrap();
  ```

- Line 153: `\.unwrap\(\)`
  ```rust
  writeln!(f, r#"{{"type":"session_end","end_time":"2025-01-01T00:01:00Z"}}"#).unwrap();
  ```

- Line 158: `\.unwrap\(\)`
  ```rust
  let mut f = fs::File::create(&gemini_file).unwrap();
  ```

- Line 159: `\.unwrap\(\)`
  ```rust
  writeln!(f, r#"{{"type":"metadata","session_id":"gemini-456","cwd":"/tmp"}}"#).unwrap();
  ```

- Line 160: `\.unwrap\(\)`
  ```rust
  writeln!(f, r#"{{"type":"message","role":"user","content":"Hello Gemini","timestamp":"2025-01-01T00:00:00Z"}}"#).unwrap();
  ```

- Line 161: `\.unwrap\(\)`
  ```rust
  writeln!(f, "INVALID JSON IN GEMHI {{{{").unwrap();
  ```

- Line 162: `\.unwrap\(\)`
  ```rust
  writeln!(f, r#"{{"type":"message","role":"assistant","content":"Hi from Gemini","timestamp":"2025-01-01T00:00:01Z"}}"#).unwrap();
  ```

- Line 174: `\.unwrap\(\)`
  ```rust
  let codex_content = fs::read_to_string(&codex_file).unwrap();
  ```

- Line 195: `\.unwrap\(\)`
  ```rust
  let gemini_content = fs::read_to_string(&gemini_file).unwrap();
  ```

- Line 225: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 227: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 231: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 233: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

### hoop-daemon/tests/reflection_detector_integration.rs

Total errors: 63

#### assert (3 occurrences)

- Line 168: `run_detection should succeed`
  ```rust
  assert!(result.is_ok(), "run_detection should succeed");
  ```

- Line 554: `build_reflection_rules_with_audit should succeed`
  ```rust
  assert!(result.is_ok(), "build_reflection_rules_with_audit should succeed");
  ```

- Line 605: `last_applied should be set`
  ```rust
  assert!(last_applied.is_some(), "last_applied should be set");
  ```

#### assert_eq (11 occurrences)

- Line 171: `Should propose 1 pattern from 3 similar negatives`
  ```rust
  assert_eq!(proposed, 1, "Should propose 1 pattern from 3 similar negatives");
  ```

- Line 186: `Should have 1 reflection ledger entry`
  ```rust
  assert_eq!(entries.len(), 1, "Should have 1 reflection ledger entry");
  ```

- Line 196: `Should have 3 source stitches`
  ```rust
  assert_eq!(source_stitches.len(), 3, "Should have 3 source stitches");
  ```

- Line 235: `Should propose 1 preference pattern`
  ```rust
  assert_eq!(proposed, 1, "Should propose 1 preference pattern");
  ```

- Line 273: `Should propose 1 correction pattern`
  ```rust
  assert_eq!(proposed, 1, "Should propose 1 correction pattern");
  ```

- Line 326: `Should not propose patterns: worker stitches ignored, operator below threshold`
  ```rust
  assert_eq!(proposed, 0, "Should not propose patterns: worker stitches ignored, operator below threshold");
  ```

- Line 446: `Should not propose patterns: old stitches outside window`
  ```rust
  assert_eq!(proposed, 0, "Should not propose patterns: old stitches outside window");
  ```

- Line 572: `Should have 2 audit rows, one per injected rule`
  ```rust
  assert_eq!(audit_rows.len(), 2, "Should have 2 audit rows, one per injected rule");
  ```

- Line 606: `applied_count should be 1 after injection`
  ```rust
  assert_eq!(applied_count, 1, "applied_count should be 1 after injection");
  ```

- Line 624: `applied_count should be 2 after second injection`
  ```rust
  assert_eq!(applied_count, 2, "applied_count should be 2 after second injection");
  ```

- Line 633: `Should have 4 audit rows total (2 per injection)`
  ```rust
  assert_eq!(count, 4, "Should have 4 audit rows total (2 per injection)");
  ```

#### unwrap (49 occurrences)

- Line 17: `\.unwrap\(\)`
  ```rust
  let conn = Connection::open(&db_path).unwrap();
  ```

- Line 36: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 47: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 57: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 72: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 77: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 82: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 104: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 113: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 119: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 170: `\.unwrap\(\)`
  ```rust
  let proposed = result.unwrap();
  ```

- Line 176: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 182: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 195: `\.unwrap\(\)`
  ```rust
  let source_stitches: Vec<String> = serde_json::from_str(source_stitches_json).unwrap();
  ```

- Line 205: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 234: `\.unwrap\(\)`
  ```rust
  let proposed = result.unwrap();
  ```

- Line 243: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 272: `\.unwrap\(\)`
  ```rust
  let proposed = result.unwrap();
  ```

- Line 281: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 325: `\.unwrap\(\)`
  ```rust
  let proposed = result.unwrap();
  ```

- Line 337: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 383: `\.unwrap\(\)`
  ```rust
  let proposed = result.unwrap();
  ```

- Line 393: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 398: `\.unwrap\(\)`
  ```rust
  Ok(serde_json::from_str::<Vec<String>>(&json).unwrap())
  ```

- Line 400: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 416: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 445: `\.unwrap\(\)`
  ```rust
  let proposed = result.unwrap();
  ```

- Line 454: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 469: `\.unwrap\(\)`
  ```rust
  hoop_daemon::fleet::is_operator_stitch("st-operator-1").unwrap(),
  ```

- Line 473: `\.unwrap\(\)`
  ```rust
  !hoop_daemon::fleet::is_operator_stitch("st-worker-1").unwrap(),
  ```

- Line 477: `\.unwrap\(\)`
  ```rust
  !hoop_daemon::fleet::is_operator_stitch("st-fleet-operator").unwrap(),
  ```

- Line 481: `\.unwrap\(\)`
  ```rust
  !hoop_daemon::fleet::is_operator_stitch("st-nonexistent").unwrap(),
  ```

- Line 496: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 508: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 514: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 535: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 545: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 555: `\.unwrap\(\)`
  ```rust
  let rules_string = result.unwrap();
  ```

- Line 562: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 568: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 578: `\.unwrap\(\)`
  ```rust
  let args: serde_json::Value = serde_json::from_str(args_json).unwrap();
  ```

- Line 582: `\.unwrap\(\)`
  ```rust
  let rule_id = args["rule_id"].as_str().unwrap();
  ```

- Line 592: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 598: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 615: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 619: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 630: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 632: `\.unwrap\(\)`
  ```rust
  let count: i64 = audit_stmt2.query_row([], |row| row.get(0)).unwrap();
  ```

- Line 643: `\.unwrap\(\)`
  ```rust
  write!(&mut result, "{:02x}", byte).unwrap();
  ```

### hoop-daemon/tests/risk_patterns_standalone.rs

Total errors: 11

#### assert_eq (10 occurrences)

- Line 21: `
`
  ```rust
  assert_eq!(lib.patterns().len(), expected_count,
  ```

- Line 84: `Should find exactly one match for `
  ```rust
  assert_eq!(matches.len(), 1, "Should find exactly one match for 'test' keyword");
  ```

- Line 85: `Matched pattern should have the expected ID`
  ```rust
  assert_eq!(matches[0].pattern.id, "test_pattern", "Matched pattern should have the expected ID");
  ```

- Line 139: `Library should contain exactly 2 patterns`
  ```rust
  assert_eq!(lib.patterns().len(), 2, "Library should contain exactly 2 patterns");
  ```

- Line 142: `Should find exactly one match for keyword1`
  ```rust
  assert_eq!(matches1.len(), 1, "Should find exactly one match for keyword1");
  ```

- Line 146: `Should find exactly one match for keyword2`
  ```rust
  assert_eq!(matches2.len(), 1, "Should find exactly one match for keyword2");
  ```

- Line 167: `Should find match via label keyword`
  ```rust
  assert_eq!(matches.len(), 1, "Should find match via label keyword");
  ```

- Line 188: `Should find match via title keyword`
  ```rust
  assert_eq!(matches_title.len(), 1, "Should find match via title keyword");
  ```

- Line 192: `Should find match via label keyword`
  ```rust
  assert_eq!(matches_label.len(), 1, "Should find match via label keyword");
  ```

- Line 196: `Should find match with both keywords`
  ```rust
  assert_eq!(matches_both.len(), 1, "Should find match with both keywords");
  ```

#### unwrap (1 occurrences)

- Line 63: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

### hoop-daemon/tests/s1_morning_review.rs

Total errors: 29

#### assert_eq (1 occurrences)

- Line 131: `Dashboard should return 200`
  ```rust
  assert_eq!(resp.status(), 200, "Dashboard should return 200");
  ```

#### expect (28 occurrences)

- Line 29: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 38: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 49: `Failed to parse dashboard response`
  ```rust
  .expect("Failed to parse dashboard response");
  ```

- Line 58: `total_workers must be a number`
  ```rust
  .expect("total_workers must be a number");
  ```

- Line 72: `total_spend_usd must be a number`
  ```rust
  .expect("total_spend_usd must be a number");
  ```

- Line 86: `longest_running must be an array`
  ```rust
  .expect("longest_running must be an array");
  ```

- Line 94: `Failed to fetch worker timeline`
  ```rust
  .expect("Failed to fetch worker timeline");
  ```

- Line 102: `Failed to parse timeline`
  ```rust
  let timeline: JsonValue = resp.json().await.expect("Failed to parse timeline");
  ```

- Line 117: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 127: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 150: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 159: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 167: `Failed to parse response`
  ```rust
  let dashboard: JsonValue = resp.json().await.expect("Failed to parse response");
  ```

- Line 188: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 197: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 199: `Failed to parse response`
  ```rust
  let dashboard1: JsonValue = resp1.json().await.expect("Failed to parse response");
  ```

- Line 209: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 211: `Failed to parse response`
  ```rust
  let dashboard2: JsonValue = resp2.json().await.expect("Failed to parse response");
  ```

- Line 229: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 237: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 239: `Failed to parse response`
  ```rust
  let dashboard: JsonValue = resp.json().await.expect("Failed to parse response");
  ```

- Line 244: `total_spend_usd must be present`
  ```rust
  .expect("total_spend_usd must be present");
  ```

- Line 254: `spend_by_project must be an array`
  ```rust
  .expect("spend_by_project must be an array");
  ```

- Line 279: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 287: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 289: `Failed to parse response`
  ```rust
  let dashboard: JsonValue = resp.json().await.expect("Failed to parse response");
  ```

- Line 293: `total_workers must be present`
  ```rust
  .expect("total_workers must be present");
  ```

- Line 297: `workers_by_project must be an array`
  ```rust
  .expect("workers_by_project must be an array");
  ```

### hoop-daemon/tests/s2_transcript_archaeology.rs

Total errors: 31

#### assert (1 occurrences)

- Line 74: `Events should be an array`
  ```rust
  assert!(events.is_array(), "Events should be an array");
  ```

#### expect (30 occurrences)

- Line 32: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 41: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 49: `Failed to parse beads`
  ```rust
  let beads: JsonValue = resp.json().await.expect("Failed to parse beads");
  ```

- Line 56: `Bead should have an id`
  ```rust
  .expect("Bead should have an id");
  ```

- Line 63: `Failed to fetch bead events`
  ```rust
  .expect("Failed to fetch bead events");
  ```

- Line 73: `Failed to parse events`
  ```rust
  let events: JsonValue = resp.json().await.expect("Failed to parse events");
  ```

- Line 90: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 99: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 101: `Failed to parse beads`
  ```rust
  let beads: JsonValue = resp.json().await.expect("Failed to parse beads");
  ```

- Line 107: `Bead should have an id`
  ```rust
  .expect("Bead should have an id");
  ```

- Line 116: `Failed to fetch bead events`
  ```rust
  .expect("Failed to fetch bead events");
  ```

- Line 139: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 148: `Failed to connect to stitch endpoint`
  ```rust
  .expect("Failed to connect to stitch endpoint");
  ```

- Line 167: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 184: `Failed to connect to endpoint`
  ```rust
  .expect("Failed to connect to endpoint");
  ```

- Line 203: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 212: `Failed to fetch conversations`
  ```rust
  .expect("Failed to fetch conversations");
  ```

- Line 220: `Failed to parse conversations`
  ```rust
  let conversations: JsonValue = resp.json().await.expect("Failed to parse conversations");
  ```

- Line 238: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 247: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 249: `Failed to parse beads`
  ```rust
  let beads: JsonValue = resp.json().await.expect("Failed to parse beads");
  ```

- Line 272: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 281: `Failed to fetch cost trends`
  ```rust
  .expect("Failed to fetch cost trends");
  ```

- Line 289: `Failed to parse cost data`
  ```rust
  let cost_data: JsonValue = resp.json().await.expect("Failed to parse cost data");
  ```

- Line 308: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 317: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 319: `Failed to parse beads`
  ```rust
  let beads: JsonValue = resp.json().await.expect("Failed to parse beads");
  ```

- Line 325: `Bead should have an id`
  ```rust
  .expect("Bead should have an id");
  ```

- Line 332: `Failed to fetch bead events`
  ```rust
  .expect("Failed to fetch bead events");
  ```

- Line 335: `Failed to parse events`
  ```rust
  let events: JsonValue = resp.json().await.expect("Failed to parse events");
  ```

### hoop-daemon/tests/s3_bead_creation_from_chat.rs

Total errors: 85

#### assert (8 occurrences)

- Line 179: `Draft should appear in the draft queue`
  ```rust
  assert!(found, "Draft should appear in the draft queue");
  ```

- Line 388: `Audit log should contain DraftCreated entry`
  ```rust
  assert!(draft_created.is_some(), "Audit log should contain DraftCreated entry");
  ```

- Line 401: `Audit log should contain DraftApproved entry`
  ```rust
  assert!(draft_approved.is_some(), "Audit log should contain DraftApproved entry");
  ```

- Line 413: `Operator identity should be present in audit log`
  ```rust
  assert!(!actor.is_empty(), "Operator identity should be present in audit log");
  ```

- Line 478: `Draft should be in queue`
  ```rust
  assert!(draft_in_queue, "Draft should be in queue");
  ```

- Line 522: `Audit should have DraftCreated`
  ```rust
  assert!(draft_created.is_some(), "Audit should have DraftCreated");
  ```

- Line 523: `Audit should have DraftApproved`
  ```rust
  assert!(draft_approved.is_some(), "Audit should have DraftApproved");
  ```

- Line 535: `operator identity should be present`
  ```rust
  assert!(!actor.is_empty(), "operator identity should be present");
  ```

#### assert_eq (13 occurrences)

- Line 164: `List drafts should return 200`
  ```rust
  assert_eq!(list_resp.status(), 200, "List drafts should return 200");
  ```

- Line 188: `Get draft should return 200`
  ```rust
  assert_eq!(get_resp.status(), 200, "Get draft should return 200");
  ```

- Line 195: `Draft title should match chat input`
  ```rust
  assert_eq!(draft["title"], chat_input, "Draft title should match chat input");
  ```

- Line 196: `Draft kind should be fix`
  ```rust
  assert_eq!(draft["kind"], "fix", "Draft kind should be fix");
  ```

- Line 197: `Draft source should be chat`
  ```rust
  assert_eq!(draft["source"], "chat", "Draft source should be chat");
  ```

- Line 198: `Draft project should be testrepo`
  ```rust
  assert_eq!(draft["project"], "testrepo", "Draft project should be testrepo");
  ```

- Line 199: `Draft status should be pending`
  ```rust
  assert_eq!(draft["status"], "pending", "Draft status should be pending");
  ```

- Line 299: `Draft status should be submitted`
  ```rust
  assert_eq!(draft["status"], "submitted", "Draft status should be submitted");
  ```

- Line 300: `Draft should have stitch_id`
  ```rust
  assert_eq!(draft["stitch_id"], stitch_id, "Draft should have stitch_id");
  ```

- Line 371: `Audit query should return 200`
  ```rust
  assert_eq!(audit_resp.status(), 200, "Audit query should return 200");
  ```

- Line 393: `DraftCreated source should be chat`
  ```rust
  assert_eq!(args["source"], "chat", "DraftCreated source should be chat");
  ```

- Line 527: `source should be chat`
  ```rust
  assert_eq!(dc_args["source"], "chat", "source should be chat");
  ```

- Line 531: `stitch_id should match`
  ```rust
  assert_eq!(da_args["stitch_id"], stitch_id, "stitch_id should match");
  ```

#### expect (57 occurrences)

- Line 41: `create temp dir`
  ```rust
  let bin_dir = TempDir::new().expect("create temp dir");
  ```

- Line 56: `create br script`
  ```rust
  let mut f = fs::File::create(&br_path).expect("create br script");
  ```

- Line 57: `write br script`
  ```rust
  f.write_all(script.as_bytes()).expect("write br script");
  ```

- Line 62: `chmod br script`
  ```rust
  .expect("chmod br script");
  ```

- Line 107: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 133: `Failed to create draft`
  ```rust
  .expect("Failed to create draft");
  ```

- Line 145: `Failed to parse draft response`
  ```rust
  .expect("Failed to parse draft response");
  ```

- Line 149: `draft_id should be present`
  ```rust
  .expect("draft_id should be present");
  ```

- Line 162: `Failed to list drafts`
  ```rust
  .expect("Failed to list drafts");
  ```

- Line 169: `Failed to parse list response`
  ```rust
  .expect("Failed to parse list response");
  ```

- Line 173: `drafts should be an array`
  ```rust
  .expect("drafts should be an array");
  ```

- Line 186: `Failed to get draft`
  ```rust
  .expect("Failed to get draft");
  ```

- Line 193: `Failed to parse draft`
  ```rust
  .expect("Failed to parse draft");
  ```

- Line 217: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 235: `Failed to create draft`
  ```rust
  .expect("Failed to create draft");
  ```

- Line 240: `Failed to parse draft response`
  ```rust
  .expect("Failed to parse draft response");
  ```

- Line 244: `draft_id should be present`
  ```rust
  .expect("draft_id should be present");
  ```

- Line 254: `Failed to approve draft`
  ```rust
  .expect("Failed to approve draft");
  ```

- Line 266: `Failed to parse approve response`
  ```rust
  .expect("Failed to parse approve response");
  ```

- Line 270: `stitch_id should be present`
  ```rust
  .expect("stitch_id should be present");
  ```

- Line 292: `Failed to get draft`
  ```rust
  .expect("Failed to get draft");
  ```

- Line 297: `Failed to parse draft`
  ```rust
  .expect("Failed to parse draft");
  ```

- Line 318: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 336: `Failed to create draft`
  ```rust
  .expect("Failed to create draft");
  ```

- Line 341: `Failed to parse draft response`
  ```rust
  .expect("Failed to parse draft response");
  ```

- Line 345: `draft_id should be present`
  ```rust
  .expect("draft_id should be present");
  ```

- Line 353: `Failed to approve draft`
  ```rust
  .expect("Failed to approve draft");
  ```

- Line 358: `Failed to parse approve response`
  ```rust
  .expect("Failed to parse approve response");
  ```

- Line 362: `stitch_id should be present`
  ```rust
  .expect("stitch_id should be present");
  ```

- Line 369: `Failed to query audit log`
  ```rust
  .expect("Failed to query audit log");
  ```

- Line 376: `Failed to parse audit response`
  ```rust
  .expect("Failed to parse audit response");
  ```

- Line 380: `audit_rows should be an array`
  ```rust
  .expect("audit_rows should be an array");
  ```

- Line 392: `args should be an object`
  ```rust
  let args = draft_created["args"].as_object().expect("args should be an object");
  ```

- Line 404: `args should be an object`
  ```rust
  let approved_args = draft_approved["args"].as_object().expect("args should be an object");
  ```

- Line 412: `actor should be present`
  ```rust
  let actor = draft_approved["actor"].as_str().expect("actor should be present");
  ```

- Line 434: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 459: `Failed to create draft`
  ```rust
  .expect("Failed to create draft");
  ```

- Line 464: `Failed to parse response`
  ```rust
  let create_response: serde_json::Value = create_resp.json().await.expect("Failed to parse response");
  ```

- Line 465: `draft_id present`
  ```rust
  let draft_id = create_response["draft_id"].as_str().expect("draft_id present");
  ```

- Line 472: `Failed to list drafts`
  ```rust
  .expect("Failed to list drafts");
  ```

- Line 474: `Failed to parse list`
  ```rust
  let list_response: serde_json::Value = list_resp.json().await.expect("Failed to parse list");
  ```

- Line 475: `drafts array`
  ```rust
  let drafts = list_response["drafts"].as_array().expect("drafts array");
  ```

- Line 488: `Failed to approve draft`
  ```rust
  .expect("Failed to approve draft");
  ```

- Line 493: `Failed to parse approve`
  ```rust
  let approve_response: serde_json::Value = approve_resp.json().await.expect("Failed to parse approve");
  ```

- Line 494: `stitch_id present`
  ```rust
  let stitch_id = approve_response["stitch_id"].as_str().expect("stitch_id present");
  ```

- Line 509: `Failed to query audit`
  ```rust
  .expect("Failed to query audit");
  ```

- Line 511: `Failed to parse audit`
  ```rust
  let audit_response: serde_json::Value = audit_resp.json().await.expect("Failed to parse audit");
  ```

- Line 512: `audit_rows array`
  ```rust
  let audit_rows = audit_response["audit_rows"].as_array().expect("audit_rows array");
  ```

- Line 526: `args object`
  ```rust
  let dc_args = draft_created.unwrap()["args"].as_object().expect("args object");
  ```

- Line 530: `args object`
  ```rust
  let da_args = draft_approved.unwrap()["args"].as_object().expect("args object");
  ```

- Line 534: `actor present`
  ```rust
  let actor = draft_approved.unwrap()["actor"].as_str().expect("actor present");
  ```

- Line 556: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 577: `Failed to create draft`
  ```rust
  .expect("Failed to create draft");
  ```

- Line 581: `Failed to parse`
  ```rust
  let create_response: serde_json::Value = create_resp.json().await.expect("Failed to parse");
  ```

- Line 582: `draft_id present`
  ```rust
  let draft_id = create_response["draft_id"].as_str().expect("draft_id present");
  ```

- Line 589: `Failed to get draft`
  ```rust
  .expect("Failed to get draft");
  ```

- Line 593: `Failed to parse draft`
  ```rust
  let draft: serde_json::Value = get_resp.json().await.expect("Failed to parse draft");
  ```

#### unwrap (7 occurrences)

- Line 46: `\.unwrap\(\)`
  ```rust
  let log_path_str = log_path.to_str().unwrap();
  ```

- Line 70: `\.unwrap\(\)`
  ```rust
  self.bin_dir.path().to_str().unwrap().to_string()
  ```

- Line 391: `\.unwrap\(\)`
  ```rust
  let draft_created = draft_created.unwrap();
  ```

- Line 403: `\.unwrap\(\)`
  ```rust
  let draft_approved = draft_approved.unwrap();
  ```

- Line 526: `\.unwrap\(\)`
  ```rust
  let dc_args = draft_created.unwrap()["args"].as_object().expect("args object");
  ```

- Line 530: `\.unwrap\(\)`
  ```rust
  let da_args = draft_approved.unwrap()["args"].as_object().expect("args object");
  ```

- Line 534: `\.unwrap\(\)`
  ```rust
  let actor = draft_approved.unwrap()["actor"].as_str().expect("actor present");
  ```

### hoop-daemon/tests/s4_daemon_restart.rs

Total errors: 48

#### assert_eq (3 occurrences)

- Line 366: `Should be able to fetch beads after rebuild`
  ```rust
  assert_eq!(resp.status(), 200, "Should be able to fetch beads after rebuild");
  ```

- Line 468: `Should see all beads including those created during restart`
  ```rust
  assert_eq!(resp.status(), 200, "Should see all beads including those created during restart");
  ```

- Line 527: `Should fetch beads in cycle {}`
  ```rust
  assert_eq!(resp.status(), 200, "Should fetch beads in cycle {}", cycle);
  ```

#### expect (40 occurrences)

- Line 33: `workspace root is parent of hoop-daemon/`
  ```rust
  .expect("workspace root is parent of hoop-daemon/")
  ```

- Line 107: `create temp dir for test HOOP home`
  ```rust
  let temp_dir = TempDir::new().expect("create temp dir for test HOOP home");
  ```

- Line 109: `create .hoop dir`
  ```rust
  fs::create_dir_all(&hoop_dir).expect("create .hoop dir");
  ```

- Line 124: `write projects.yaml`
  ```rust
  .expect("write projects.yaml");
  ```

- Line 133: `write config.yml`
  ```rust
  .expect("write config.yml");
  ```

- Line 135: `create data dir`
  ```rust
  fs::create_dir_all(hoop_dir.join("data")).expect("create data dir");
  ```

- Line 155: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 159: `write claim`
  ```rust
  worker.write_claim("bd-001").expect("write claim");
  ```

- Line 160: `write complete`
  ```rust
  worker.write_complete("bd-001").expect("write complete");
  ```

- Line 161: `write claim`
  ```rust
  worker.write_claim("bd-002").expect("write claim");
  ```

- Line 170: `Failed to spawn first daemon`
  ```rust
  .expect("Failed to spawn first daemon");
  ```

- Line 195: `Failed to fetch beads from first daemon`
  ```rust
  .expect("Failed to fetch beads from first daemon");
  ```

- Line 203: `Failed to parse beads`
  ```rust
  let beads1: serde_json::Value = resp1.json().await.expect("Failed to parse beads");
  ```

- Line 212: `write complete`
  ```rust
  worker.write_complete("bd-002").expect("write complete");
  ```

- Line 213: `write claim`
  ```rust
  worker.write_claim("bd-003").expect("write claim");
  ```

- Line 226: `Failed to spawn second daemon`
  ```rust
  .expect("Failed to spawn second daemon");
  ```

- Line 249: `Failed to fetch beads from second daemon`
  ```rust
  .expect("Failed to fetch beads from second daemon");
  ```

- Line 257: `Failed to parse beads`
  ```rust
  let beads2: serde_json::Value = resp2.json().await.expect("Failed to parse beads");
  ```

- Line 290: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 296: `write claim`
  ```rust
  worker.write_claim(&bead_id).expect("write claim");
  ```

- Line 298: `write complete`
  ```rust
  worker.write_complete(&bead_id).expect("write complete");
  ```

- Line 305: `Failed to spawn first daemon`
  ```rust
  .expect("Failed to spawn first daemon");
  ```

- Line 332: `Failed to spawn second daemon`
  ```rust
  .expect("Failed to spawn second daemon");
  ```

- Line 364: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 387: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 392: `Failed to spawn first daemon`
  ```rust
  .expect("Failed to spawn first daemon");
  ```

- Line 418: `write claim`
  ```rust
  worker.write_claim("bd-restart-1").expect("write claim");
  ```

- Line 419: `write complete`
  ```rust
  worker.write_complete("bd-restart-1").expect("write complete");
  ```

- Line 420: `write claim`
  ```rust
  worker.write_claim("bd-restart-2").expect("write claim");
  ```

- Line 432: `Failed to spawn second daemon`
  ```rust
  .expect("Failed to spawn second daemon");
  ```

- Line 451: `write complete`
  ```rust
  worker.write_complete("bd-restart-2").expect("write complete");
  ```

- Line 452: `write claim`
  ```rust
  worker.write_claim("bd-restart-3").expect("write claim");
  ```

- Line 466: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 488: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 495: `write claim`
  ```rust
  worker.write_claim("bd-s4-1").expect("write claim");
  ```

- Line 496: `write complete`
  ```rust
  worker.write_complete("bd-s4-1").expect("write complete");
  ```

- Line 502: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 525: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 529: `Failed to parse beads`
  ```rust
  let beads: serde_json::Value = resp.json().await.expect("Failed to parse beads");
  ```

- Line 552: `write claim`
  ```rust
  worker.write_claim(&format!("bd-s4-{}", cycle * 10 + 2)).expect("write claim");
  ```

#### unwrap (5 occurrences)

- Line 105: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 144: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 279: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 376: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 477: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

### hoop-daemon/tests/s5_workspace_deleted.rs

Total errors: 39

#### assert_eq (3 occurrences)

- Line 168: `Initial readyz should return 200`
  ```rust
  assert_eq!(status, 200, "Initial readyz should return 200");
  ```

- Line 169: `Initial readyz status should be ok`
  ```rust
  assert_eq!(readyz.status, "ok", "Initial readyz status should be ok");
  ```

- Line 277: `Projects endpoint should still work`
  ```rust
  assert_eq!(resp.status(), 200, "Projects endpoint should still work");
  ```

#### expect (26 occurrences)

- Line 30: `Failed to create .beads dir`
  ```rust
  fs::create_dir_all(&beads_dir).expect("Failed to create .beads dir");
  ```

- Line 32: `Failed to create issues.jsonl`
  ```rust
  fs::write(&issues_path, b"").expect("Failed to create issues.jsonl");
  ```

- Line 40: `Failed to create temp dir`
  ```rust
  let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
  ```

- Line 42: `Failed to create .hoop dir`
  ```rust
  fs::create_dir_all(&hoop_dir).expect("Failed to create .hoop dir");
  ```

- Line 71: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 79: `Failed to write config.yml`
  ```rust
  fs::write(hoop_dir.join("config.yml"), config_yaml).expect("Failed to write config.yml");
  ```

- Line 80: `Failed to create data dir`
  ```rust
  fs::create_dir_all(hoop_dir.join("data")).expect("Failed to create data dir");
  ```

- Line 120: `Failed to bind to random port`
  ```rust
  .expect("Failed to bind to random port");
  ```

- Line 121: `Failed to get local address`
  ```rust
  let addr = listener.local_addr().expect("Failed to get local address");
  ```

- Line 166: `Failed to get readyz status`
  ```rust
  .expect("Failed to get readyz status");
  ```

- Line 173: `Failed to remove .beads from project A`
  ```rust
  fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads from project A");
  ```

- Line 222: `Failed to bind to random port`
  ```rust
  .expect("Failed to bind to random port");
  ```

- Line 223: `Failed to get local address`
  ```rust
  let addr = listener.local_addr().expect("Failed to get local address");
  ```

- Line 265: `Failed to remove .beads from project A`
  ```rust
  fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads from project A");
  ```

- Line 275: `Failed to fetch projects`
  ```rust
  .expect("Failed to fetch projects");
  ```

- Line 279: `Failed to parse projects`
  ```rust
  let projects: JsonValue = resp.json().await.expect("Failed to parse projects");
  ```

- Line 292: `Failed to check health`
  ```rust
  .expect("Failed to check health");
  ```

- Line 323: `Failed to bind to random port`
  ```rust
  .expect("Failed to bind to random port");
  ```

- Line 324: `Failed to get local address`
  ```rust
  let addr = listener.local_addr().expect("Failed to get local address");
  ```

- Line 367: `Failed to get readyz status`
  ```rust
  .expect("Failed to get readyz status");
  ```

- Line 372: `Failed to remove .beads from project A`
  ```rust
  fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads from project A");
  ```

- Line 379: `Failed to get readyz status after deletion`
  ```rust
  .expect("Failed to get readyz status after deletion");
  ```

- Line 427: `Failed to bind to random port`
  ```rust
  .expect("Failed to bind to random port");
  ```

- Line 428: `Failed to get local address`
  ```rust
  let addr = listener.local_addr().expect("Failed to get local address");
  ```

- Line 470: `Failed to remove .beads`
  ```rust
  fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads");
  ```

- Line 479: `Failed to check health`
  ```rust
  .expect("Failed to check health");
  ```

#### unwrap (10 occurrences)

- Line 103: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 107: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 111: `\.unwrap\(\)`
  ```rust
  let project_c_dir = tempfile::tempdir().unwrap();
  ```

- Line 206: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 210: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 214: `\.unwrap\(\)`
  ```rust
  let project_c_dir = tempfile::tempdir().unwrap();
  ```

- Line 307: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 311: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 315: `\.unwrap\(\)`
  ```rust
  let project_c_dir = tempfile::tempdir().unwrap();
  ```

- Line 419: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

### hoop-daemon/tests/secrets_scanner_integration.rs

Total errors: 7

#### assert (5 occurrences)

- Line 252: `High-entropy string should be detected`
  ```rust
  assert!(!findings.is_empty(), "High-entropy string should be detected");
  ```

- Line 302: `high`
  ```rust
  assert!(matches!(finding.severity.as_str(), "high" | "medium" | "low"));
  ```

- Line 341: `API key should have high entropy: {}`
  ```rust
  assert!(e > 4.5, "API key should have high entropy: {}", e);
  ```

- Line 346: `Normal text should have low entropy: {}`
  ```rust
  assert!(e < 4.5, "Normal text should have low entropy: {}", e);
  ```

- Line 360: `Very short strings should not be flagged`
  ```rust
  assert!(findings.is_empty(), "Very short strings should not be flagged");
  ```

#### unwrap (2 occurrences)

- Line 382: `\.unwrap\(\)`
  ```rust
  let json = serde_json::to_string(&findings).unwrap();
  ```

- Line 387: `\.unwrap\(\)`
  ```rust
  serde_json::from_str(&json).unwrap();
  ```

### hoop-daemon/tests/secrets_scanner_parity.rs

Total errors: 8

#### assert (6 occurrences)

- Line 212: `Default patterns should not be empty`
  ```rust
  assert!(!patterns.is_empty(), "Default patterns should not be empty");
  ```

- Line 216: `Pattern ID should not be empty`
  ```rust
  assert!(!pattern.id.is_empty(), "Pattern ID should not be empty");
  ```

- Line 217: `Pattern name should not be empty`
  ```rust
  assert!(!pattern.name.is_empty(), "Pattern name should not be empty");
  ```

- Line 224: `Pattern `
  ```rust
  assert!(!pattern.patterns.is_empty(), "Pattern '{}' should have at least one regex", pattern.id);
  ```

- Line 279: `Should detect Anthropic API key`
  ```rust
  assert!(!findings.is_empty(), "Should detect Anthropic API key");
  ```

- Line 308: `Custom pattern should detect test secret`
  ```rust
  assert!(!findings.is_empty(), "Custom pattern should detect test secret");
  ```

#### expect (2 occurrences)

- Line 238: `Pattern should serialize to JSON`
  ```rust
  let json = serde_json::to_string(pattern).expect("Pattern should serialize to JSON");
  ```

- Line 239: `Serialized pattern should deserialize`
  ```rust
  let parsed: SecretPattern = serde_json::from_str(&json).expect("Serialized pattern should deserialize");
  ```

### hoop-daemon/tests/session_redaction.rs

Total errors: 13

#### assert (4 occurrences)

- Line 172: `must be redacted: {r1}`
  ```rust
  assert!(r1.contains("[REDACTED]"), "must be redacted: {r1}");
  ```

- Line 173: `raw key must not appear: {r1}`
  ```rust
  assert!(!r1.contains("CACHETEST"), "raw key must not appear: {r1}");
  ```

- Line 245: `JWT must be redacted: {out}`
  ```rust
  assert!(out.contains("[REDACTED]"), "JWT must be redacted: {out}");
  ```

- Line 246: `raw JWT must not appear: {out}`
  ```rust
  assert!(!out.contains("eyJhbGci"), "raw JWT must not appear: {out}");
  ```

#### assert_eq (3 occurrences)

- Line 160: `clean content must pass through unchanged`
  ```rust
  assert_eq!(out, raw, "clean content must pass through unchanged");
  ```

- Line 170: `cache must return same result`
  ```rust
  assert_eq!(r1, r2, "cache must return same result");
  ```

- Line 171: `cache must return same result`
  ```rust
  assert_eq!(r2, r3, "cache must return same result");
  ```

#### expect (1 occurrences)

- Line 216: `valid JSON`
  ```rust
  let v: Value = serde_json::from_str(line).expect("valid JSON");
  ```

#### unwrap (5 occurrences)

- Line 66: `\.unwrap\(\)`
  ```rust
  let mut tmp = NamedTempFile::new().unwrap();
  ```

- Line 68: `\.unwrap\(\)`
  ```rust
  fs::write(tmp.path(), &original_content).unwrap();
  ```

- Line 87: `\.unwrap\(\)`
  ```rust
  let after = fs::read_to_string(tmp.path()).unwrap();
  ```

- Line 139: `\.unwrap\(\)`
  ```rust
  let text0 = out[0]["text"].as_str().unwrap();
  ```

- Line 149: `\.unwrap\(\)`
  ```rust
  let text1 = out[1]["text"].as_str().unwrap();
  ```

### hoop-daemon/tests/skills_integration.rs

Total errors: 32

#### assert_eq (1 occurrences)

- Line 374: `project-b`
  ```rust
  assert_eq!(skills[0].manifest.projects, vec!["project-a", "project-b"]);
  ```

#### expect (30 occurrences)

- Line 18: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 21: `Failed to create skill dir`
  ```rust
  fs::create_dir(&skill_dir).expect("Failed to create skill dir");
  ```

- Line 44: `Failed to write manifest`
  ```rust
  .expect("Failed to write manifest");
  ```

- Line 56: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 59: `Failed to create skill dir`
  ```rust
  fs::create_dir(&skill_dir).expect("Failed to create skill dir");
  ```

- Line 77: `Failed to write manifest`
  ```rust
  .expect("Failed to write manifest");
  ```

- Line 86: `Failed to write run script`
  ```rust
  .expect("Failed to write run script");
  ```

- Line 90: `Failed to get metadata`
  ```rust
  .expect("Failed to get metadata")
  ```

- Line 94: `Failed to set permissions`
  ```rust
  .expect("Failed to set permissions");
  ```

- Line 117: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 120: `Failed to create skill dir`
  ```rust
  fs::create_dir(&skill_dir).expect("Failed to create skill dir");
  ```

- Line 140: `Failed to write manifest`
  ```rust
  .expect("Failed to write manifest");
  ```

- Line 162: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 165: `Failed to create skill dir`
  ```rust
  fs::create_dir(&skill_dir).expect("Failed to create skill dir");
  ```

- Line 187: `Failed to write manifest`
  ```rust
  .expect("Failed to write manifest");
  ```

- Line 211: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 214: `Failed to create skill dir`
  ```rust
  fs::create_dir(&skill_dir).expect("Failed to create skill dir");
  ```

- Line 233: `Failed to write manifest`
  ```rust
  .expect("Failed to write manifest");
  ```

- Line 252: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 255: `Failed to create skill dir`
  ```rust
  fs::create_dir(&skill_dir).expect("Failed to create skill dir");
  ```

- Line 275: `Failed to write manifest`
  ```rust
  .expect("Failed to write manifest");
  ```

- Line 350: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 353: `Failed to create skill dir`
  ```rust
  fs::create_dir(&skill_dir).expect("Failed to create skill dir");
  ```

- Line 369: `Failed to write manifest`
  ```rust
  .expect("Failed to write manifest");
  ```

- Line 379: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 382: `Failed to create skill dir`
  ```rust
  fs::create_dir(&skill_dir).expect("Failed to create skill dir");
  ```

- Line 396: `Failed to write manifest`
  ```rust
  .expect("Failed to write manifest");
  ```

- Line 406: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 409: `Failed to create skill dir`
  ```rust
  fs::create_dir(&skill_dir).expect("Failed to create skill dir");
  ```

- Line 422: `Failed to write manifest`
  ```rust
  .expect("Failed to write manifest");
  ```

#### unwrap (1 occurrences)

- Line 109: `\.unwrap\(\)`
  ```rust
  let response = result.unwrap();
  ```

### hoop-daemon/tests/skills_quarantine_integration.rs

Total errors: 65

#### expect (11 occurrences)

- Line 56: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 83: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 111: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 138: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 162: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 206: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 234: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 264: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 289: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 307: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 326: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

#### unwrap (54 occurrences)

- Line 59: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&pending_dir).unwrap();
  ```

- Line 62: `\.unwrap\(\)`
  ```rust
  fs::create_dir(&source_dir).unwrap();
  ```

- Line 63: `\.unwrap\(\)`
  ```rust
  let skill_path = create_test_skill(&source_dir, "test-skill", "A test skill").unwrap();
  ```

- Line 69: `\.unwrap\(\)`
  ```rust
  fs_extra::dir::copy(&skill_path, &pending_skill, &options).unwrap();
  ```

- Line 86: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&pending_dir).unwrap();
  ```

- Line 90: `\.unwrap\(\)`
  ```rust
  fs::create_dir(&source_dir).unwrap();
  ```

- Line 91: `\.unwrap\(\)`
  ```rust
  let skill_path = create_test_skill(&source_dir, "enable-test", "Test enable").unwrap();
  ```

- Line 96: `\.unwrap\(\)`
  ```rust
  fs_extra::dir::copy(&skill_path, &pending_skill, &options).unwrap();
  ```

- Line 100: `\.unwrap\(\)`
  ```rust
  fs::rename(&pending_skill, &active_path).unwrap();
  ```

- Line 114: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&skills_base).unwrap();
  ```

- Line 115: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&pending_dir).unwrap();
  ```

- Line 119: `\.unwrap\(\)`
  ```rust
  fs::create_dir(&source_dir).unwrap();
  ```

- Line 120: `\.unwrap\(\)`
  ```rust
  let skill_path = create_test_skill(&source_dir, "disable-test", "Test disable").unwrap();
  ```

- Line 125: `\.unwrap\(\)`
  ```rust
  fs_extra::dir::copy(&skill_path, &active_skill, &options).unwrap();
  ```

- Line 129: `\.unwrap\(\)`
  ```rust
  fs::rename(&active_skill, &pending_path).unwrap();
  ```

- Line 140: `\.unwrap\(\)`
  ```rust
  fs::create_dir(&source_dir).unwrap();
  ```

- Line 143: `\.unwrap\(\)`
  ```rust
  fs::create_dir(&skill_dir).unwrap();
  ```

- Line 154: `\.unwrap\(\)`
  ```rust
  fs::write(skill_dir.join("manifest.yml"), manifest).unwrap();
  ```

- Line 165: `\.unwrap\(\)`
  ```rust
  fs::create_dir(&skill_dir).unwrap();
  ```

- Line 176: `\.unwrap\(\)`
  ```rust
  fs::write(skill_dir.join("manifest.yml"), manifest).unwrap();
  ```

- Line 182: `\.unwrap\(\)`
  ```rust
  fs::write(skill_dir.join("run"), run_content).unwrap();
  ```

- Line 185: `\.unwrap\(\)`
  ```rust
  let mut perms = fs::metadata(skill_dir.join("run")).unwrap().permissions();
  ```

- Line 187: `\.unwrap\(\)`
  ```rust
  fs::set_permissions(skill_dir.join("run"), perms).unwrap();
  ```

- Line 190: `\.unwrap\(\)`
  ```rust
  let metadata = fs::metadata(skill_dir.join("run")).unwrap();
  ```

- Line 195: `\.unwrap\(\)`
  ```rust
  let content = fs::read(skill_dir.join("run")).unwrap();
  ```

- Line 209: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&skills_base).unwrap();
  ```

- Line 210: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&pending_dir).unwrap();
  ```

- Line 213: `\.unwrap\(\)`
  ```rust
  fs::create_dir(&source_dir).unwrap();
  ```

- Line 216: `\.unwrap\(\)`
  ```rust
  let active_skill = create_test_skill(&source_dir, "active-skill", "Active skill").unwrap();
  ```

- Line 219: `\.unwrap\(\)`
  ```rust
  fs_extra::dir::copy(&active_skill, skills_base.join("active-skill"), &options).unwrap();
  ```

- Line 222: `\.unwrap\(\)`
  ```rust
  let pending_skill = create_test_skill(&source_dir, "pending-skill", "Pending skill").unwrap();
  ```

- Line 223: `\.unwrap\(\)`
  ```rust
  fs_extra::dir::copy(&pending_skill, pending_dir.join("pending-skill"), &options).unwrap();
  ```

- Line 237: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&skills_base).unwrap();
  ```

- Line 238: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&pending_dir).unwrap();
  ```

- Line 241: `\.unwrap\(\)`
  ```rust
  fs::create_dir(&source_dir).unwrap();
  ```

- Line 244: `\.unwrap\(\)`
  ```rust
  let pending_skill = create_test_skill(&source_dir, "remove-pending", "Remove pending").unwrap();
  ```

- Line 247: `\.unwrap\(\)`
  ```rust
  fs_extra::dir::copy(&pending_skill, pending_dir.join("remove-pending"), &options).unwrap();
  ```

- Line 250: `\.unwrap\(\)`
  ```rust
  fs::remove_dir_all(pending_dir.join("remove-pending")).unwrap();
  ```

- Line 254: `\.unwrap\(\)`
  ```rust
  let active_skill = create_test_skill(&source_dir, "remove-active", "Remove active").unwrap();
  ```

- Line 255: `\.unwrap\(\)`
  ```rust
  fs_extra::dir::copy(&active_skill, skills_base.join("remove-active"), &options).unwrap();
  ```

- Line 258: `\.unwrap\(\)`
  ```rust
  fs::remove_dir_all(skills_base.join("remove-active")).unwrap();
  ```

- Line 267: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&pending_dir).unwrap();
  ```

- Line 270: `\.unwrap\(\)`
  ```rust
  fs::create_dir(&source_dir).unwrap();
  ```

- Line 273: `\.unwrap\(\)`
  ```rust
  let skill_path = create_test_skill(&source_dir, "duplicate-test", "Duplicate test").unwrap();
  ```

- Line 278: `\.unwrap\(\)`
  ```rust
  fs_extra::dir::copy(&skill_path, &pending_skill, &options).unwrap();
  ```

- Line 292: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&skills_base).unwrap();
  ```

- Line 293: `\.unwrap\(\)`
  ```rust
  fs::create_dir_all(&pending_dir).unwrap();
  ```

- Line 309: `\.unwrap\(\)`
  ```rust
  fs::create_dir(&source_dir).unwrap();
  ```

- Line 311: `\.unwrap\(\)`
  ```rust
  let skill_path = create_test_skill(&source_dir, "yaml-show-test", "Show YAML test").unwrap();
  ```

- Line 314: `\.unwrap\(\)`
  ```rust
  let manifest_content = fs::read_to_string(skill_path.join("manifest.yml")).unwrap();
  ```

- Line 328: `\.unwrap\(\)`
  ```rust
  fs::create_dir(&skill_dir).unwrap();
  ```

- Line 331: `\.unwrap\(\)`
  ```rust
  fs::write(skill_dir.join("run"), run_content).unwrap();
  ```

- Line 339: `\.unwrap\(\)`
  ```rust
  let mut file = fs::File::open(skill_dir.join("run")).unwrap();
  ```

- Line 341: `\.unwrap\(\)`
  ```rust
  file.read_to_end(&mut buffer).unwrap();
  ```

### hoop-daemon/tests/state_projections.rs

Total errors: 78

#### assert (24 occurrences)

- Line 163: `Health check should return 200`
  ```rust
  assert!(resp.status().is_success(), "Health check should return 200");
  ```

- Line 226: `Must receive workers_snapshot`
  ```rust
  assert!(snapshots.workers_received, "Must receive workers_snapshot");
  ```

- Line 227: `Must receive beads_snapshot`
  ```rust
  assert!(snapshots.beads_received, "Must receive beads_snapshot");
  ```

- Line 228: `Must receive conversations_snapshot`
  ```rust
  assert!(snapshots.conversations_received, "Must receive conversations_snapshot");
  ```

- Line 229: `Must receive projects_snapshot`
  ```rust
  assert!(snapshots.projects_received, "Must receive projects_snapshot");
  ```

- Line 230: `Must receive config_status`
  ```rust
  assert!(snapshots.config_received, "Must receive config_status");
  ```

- Line 344: `Should receive messages after subscribe/unsubscribe`
  ```rust
  assert!(snapshot_msg.is_ok(), "Should receive messages after subscribe/unsubscribe");
  ```

- Line 390: `Beads must be an array`
  ```rust
  assert!(beads.is_array(), "Beads must be an array");
  ```

- Line 394: `Each bead must have an id`
  ```rust
  assert!(bead.get("id").is_some(), "Each bead must have an id");
  ```

- Line 395: `Each bead must have a title`
  ```rust
  assert!(bead.get("title").is_some(), "Each bead must have a title");
  ```

- Line 396: `Each bead must have a status`
  ```rust
  assert!(bead.get("status").is_some(), "Each bead must have a status");
  ```

- Line 419: `Workers response is valid array`
  ```rust
  assert!(!workers.is_empty() || workers.is_empty(), "Workers response is valid array");
  ```

- Line 441: `Projects response is valid array`
  ```rust
  assert!(!projects.is_empty() || projects.is_empty(), "Projects response is valid array");
  ```

- Line 488: `Connection should receive init`
  ```rust
  assert!(handle.await.expect("Task failed"), "Connection should receive init");
  ```

- Line 572: `Reconnect should receive init event`
  ```rust
  assert!(received_init, "Reconnect should receive init event");
  ```

- Line 573: `Reconnect should receive beads_snapshot`
  ```rust
  assert!(received_beads_snapshot, "Reconnect should receive beads_snapshot");
  ```

- Line 653: `Should receive all snapshot events`
  ```rust
  assert!(received_all, "Should receive all snapshot events");
  ```

- Line 662: `global should be valid`
  ```rust
  assert!(WsTopic::parse("global").is_some(), "global should be valid");
  ```

- Line 663: `project:testrepo should be valid`
  ```rust
  assert!(WsTopic::parse("project:testrepo").is_some(), "project:testrepo should be valid");
  ```

- Line 664: `project with colons should be valid`
  ```rust
  assert!(WsTopic::parse("project:ns:name").is_some(), "project with colons should be valid");
  ```

- Line 667: `empty project name should be invalid`
  ```rust
  assert!(WsTopic::parse("project:").is_none(), "empty project name should be invalid");
  ```

- Line 668: `fleet: prefix should be invalid`
  ```rust
  assert!(WsTopic::parse("fleet:alpha").is_none(), "fleet: prefix should be invalid");
  ```

- Line 669: `empty string should be invalid`
  ```rust
  assert!(WsTopic::parse("").is_none(), "empty string should be invalid");
  ```

- Line 670: `GLOBAL (uppercase) should be invalid`
  ```rust
  assert!(WsTopic::parse("GLOBAL").is_none(), "GLOBAL (uppercase) should be invalid");
  ```

#### assert_eq (1 occurrences)

- Line 193: `First message must be init`
  ```rust
  assert_eq!(event["type"], "init", "First message must be init");
  ```

#### expect (50 occurrences)

- Line 153: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 161: `Health check request failed`
  ```rust
  .expect("Health check request failed");
  ```

- Line 171: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 178: `Failed to connect to WebSocket`
  ```rust
  .expect("Failed to connect to WebSocket");
  ```

- Line 184: `Timeout waiting for first message`
  ```rust
  .expect("Timeout waiting for first message")
  ```

- Line 185: `WebSocket stream ended`
  ```rust
  .expect("WebSocket stream ended");
  ```

- Line 187: `Failed to receive first message`
  ```rust
  let first_msg = first_msg.expect("Failed to receive first message");
  ```

- Line 191: `Failed to parse init event`
  ```rust
  .expect("Failed to parse init event");
  ```

- Line 220: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 224: `Failed to collect snapshots`
  ```rust
  .expect("Failed to collect snapshots");
  ```

- Line 238: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 243: `Failed to collect WS snapshots`
  ```rust
  .expect("Failed to collect WS snapshots");
  ```

- Line 253: `REST workers request failed`
  ```rust
  .expect("REST workers request failed")
  ```

- Line 256: `Failed to parse REST workers response`
  ```rust
  .expect("Failed to parse REST workers response");
  ```

- Line 263: `REST beads request failed`
  ```rust
  .expect("REST beads request failed")
  ```

- Line 266: `Failed to parse REST beads response`
  ```rust
  .expect("Failed to parse REST beads response");
  ```

- Line 273: `REST projects request failed`
  ```rust
  .expect("REST projects request failed")
  ```

- Line 276: `Failed to parse REST projects response`
  ```rust
  .expect("Failed to parse REST projects response");
  ```

- Line 298: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 305: `Failed to connect`
  ```rust
  .expect("Failed to connect");
  ```

- Line 312: `Timeout waiting for init`
  ```rust
  .expect("Timeout waiting for init")
  ```

- Line 313: `Stream ended`
  ```rust
  .expect("Stream ended")
  ```

- Line 314: `Failed to receive init`
  ```rust
  .expect("Failed to receive init");
  ```

- Line 329: `Failed to send subscribe`
  ```rust
  .expect("Failed to send subscribe");
  ```

- Line 339: `Failed to send unsubscribe`
  ```rust
  .expect("Failed to send unsubscribe");
  ```

- Line 352: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 360: `Config status request failed`
  ```rust
  .expect("Config status request failed")
  ```

- Line 363: `Failed to parse config status`
  ```rust
  .expect("Failed to parse config status");
  ```

- Line 376: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 384: `Beads request failed`
  ```rust
  .expect("Beads request failed")
  ```

- Line 387: `Failed to parse beads response`
  ```rust
  .expect("Failed to parse beads response");
  ```

- Line 405: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 413: `Workers request failed`
  ```rust
  .expect("Workers request failed")
  ```

- Line 416: `Failed to parse workers response`
  ```rust
  .expect("Failed to parse workers response");
  ```

- Line 427: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 435: `Projects request failed`
  ```rust
  .expect("Projects request failed")
  ```

- Line 438: `Failed to parse projects response`
  ```rust
  .expect("Failed to parse projects response");
  ```

- Line 449: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 469: `Stream ended`
  ```rust
  .expect("Stream ended");
  ```

- Line 475: `Failed to parse`
  ```rust
  .expect("Failed to parse");
  ```

- Line 488: `Task failed`
  ```rust
  assert!(handle.await.expect("Task failed"), "Connection should receive init");
  ```

- Line 497: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 505: `Failed to connect first time`
  ```rust
  .expect("Failed to connect first time");
  ```

- Line 535: `Failed to reconnect`
  ```rust
  .expect("Failed to reconnect");
  ```

- Line 590: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 599: `Failed to connect`
  ```rust
  .expect("Failed to connect");
  ```

- Line 679: `Failed to spawn first daemon`
  ```rust
  .expect("Failed to spawn first daemon");
  ```

- Line 683: `Failed to spawn second daemon`
  ```rust
  .expect("Failed to spawn second daemon");
  ```

- Line 698: `First daemon health check failed`
  ```rust
  .expect("First daemon health check failed");
  ```

- Line 704: `Second daemon health check failed`
  ```rust
  .expect("Second daemon health check failed");
  ```

#### panic (1 occurrences)

- Line 211: `Expected text message for init, got {:?}`
  ```rust
  panic!("Expected text message for init, got {:?}", first_msg);
  ```

#### unwrap (2 occurrences)

- Line 201: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 317: `\.unwrap\(\)`
  ```rust
  let event: serde_json::Value = serde_json::from_str(&text).unwrap();
  ```

### hoop-daemon/tests/stderr_stdout_capture.rs

Total errors: 6

#### assert (1 occurrences)

- Line 166: `Large config should generate more output than default`
  ```rust
  assert!(bytes2 > bytes1, "Large config should generate more output than default");
  ```

#### assert_eq (1 occurrences)

- Line 171: `Same configuration should produce identical output size`
  ```rust
  assert_eq!(bytes2, bytes3, "Same configuration should produce identical output size");
  ```

#### unwrap (4 occurrences)

- Line 55: `\.unwrap\(\)`
  ```rust
  io::stdout().flush().unwrap();
  ```

- Line 56: `\.unwrap\(\)`
  ```rust
  io::stderr().flush().unwrap();
  ```

- Line 119: `\.unwrap\(\)`
  ```rust
  io::stdout().flush().unwrap();
  ```

- Line 121: `\.unwrap\(\)`
  ```rust
  io::stderr().flush().unwrap();
  ```

### hoop-daemon/tests/stdout_generation_test.rs

Total errors: 26

#### assert (14 occurrences)

- Line 283: `Subprocess should succeed`
  ```rust
  assert!(result.succeeded(), "Subprocess should succeed");
  ```

- Line 284: `Should have stdout output`
  ```rust
  assert!(!result.stdout.is_empty(), "Should have stdout output");
  ```

- Line 296: `Stderr subprocess should succeed`
  ```rust
  assert!(result.succeeded(), "Stderr subprocess should succeed");
  ```

- Line 308: `Mixed subprocess should succeed`
  ```rust
  assert!(result.succeeded(), "Mixed subprocess should succeed");
  ```

- Line 309: `Should have stdout output`
  ```rust
  assert!(!result.stdout.is_empty(), "Should have stdout output");
  ```

- Line 310: `Should have stderr output`
  ```rust
  assert!(!result.stderr.is_empty(), "Should have stderr output");
  ```

- Line 322: `Multi-line subprocess should succeed`
  ```rust
  assert!(result.succeeded(), "Multi-line subprocess should succeed");
  ```

- Line 323: `Should have stdout output`
  ```rust
  assert!(!result.stdout.is_empty(), "Should have stdout output");
  ```

- Line 324: `Should have stderr output`
  ```rust
  assert!(!result.stderr.is_empty(), "Should have stderr output");
  ```

- Line 351: `Configured subprocess should succeed`
  ```rust
  assert!(result.succeeded(), "Configured subprocess should succeed");
  ```

- Line 365: `Should have exit code`
  ```rust
  assert!(result.exit_code.is_some(), "Should have exit code");
  ```

- Line 366: `Should succeed`
  ```rust
  assert!(result.succeeded(), "Should succeed");
  ```

- Line 367: `Should have stdout`
  ```rust
  assert!(!result.stdout.is_empty(), "Should have stdout");
  ```

- Line 380: `Path should be in target directory`
  ```rust
  assert!(path.to_string_lossy().contains("target"), "Path should be in target directory");
  ```

#### assert_eq (2 occurrences)

- Line 330: `Should have 5 stdout lines`
  ```rust
  assert_eq!(stdout_lines.len(), 5, "Should have 5 stdout lines");
  ```

- Line 331: `Should have 5 stderr lines`
  ```rust
  assert_eq!(stderr_lines.len(), 5, "Should have 5 stderr lines");
  ```

#### expect (3 occurrences)

- Line 150: `Failed to execute subprocess`
  ```rust
  let output = command.output().expect("Failed to execute subprocess");
  ```

- Line 183: `Failed to execute test binary`
  ```rust
  let output = command.output().expect("Failed to execute test binary");
  ```

- Line 266: `Failed to execute multi-line subprocess`
  ```rust
  let output = command.output().expect("Failed to execute multi-line subprocess");
  ```

#### unwrap (7 occurrences)

- Line 287: `\.unwrap\(\)`
  ```rust
  io::stdout().flush().unwrap();
  ```

- Line 299: `\.unwrap\(\)`
  ```rust
  io::stdout().flush().unwrap();
  ```

- Line 313: `\.unwrap\(\)`
  ```rust
  io::stdout().flush().unwrap();
  ```

- Line 334: `\.unwrap\(\)`
  ```rust
  io::stdout().flush().unwrap();
  ```

- Line 354: `\.unwrap\(\)`
  ```rust
  io::stdout().flush().unwrap();
  ```

- Line 370: `\.unwrap\(\)`
  ```rust
  io::stdout().flush().unwrap();
  ```

- Line 383: `\.unwrap\(\)`
  ```rust
  io::stdout().flush().unwrap();
  ```

### hoop-daemon/tests/stdout_verification.rs

Total errors: 10

#### assert (3 occurrences)

- Line 88: `In-memory verification should pass`
  ```rust
  assert!(result.passed, "In-memory verification should pass");
  ```

- Line 117: `Verification should fail for mismatched content`
  ```rust
  assert!(!result.passed, "Verification should fail for mismatched content");
  ```

- Line 149: `Unicode verification should pass`
  ```rust
  assert!(result.passed, "Unicode verification should pass");
  ```

#### unwrap (7 occurrences)

- Line 83: `\.unwrap\(\)`
  ```rust
  fs::write(&log_path, log_content).unwrap();
  ```

- Line 86: `\.unwrap\(\)`
  ```rust
  let result = verify_stdout_char_by_char(expected_content, &log_path).unwrap();
  ```

- Line 113: `\.unwrap\(\)`
  ```rust
  fs::write(&log_path, mismatched_content).unwrap();
  ```

- Line 115: `\.unwrap\(\)`
  ```rust
  let result = verify_stdout_char_by_char(expected_content, &log_path).unwrap();
  ```

- Line 145: `\.unwrap\(\)`
  ```rust
  fs::write(&log_path, log_lines).unwrap();
  ```

- Line 147: `\.unwrap\(\)`
  ```rust
  let result = verify_stdout_char_by_char(unicode_content, &log_path).unwrap();
  ```

- Line 177: `\.unwrap\(\)`
  ```rust
  io::stdout().flush().unwrap();
  ```

### hoop-daemon/tests/stitch_percentile_index_integration.rs

Total errors: 54

#### assert (4 occurrences)

- Line 168: `stitch_percentile_index table should exist`
  ```rust
  assert!(table_exists, "stitch_percentile_index table should exist");
  ```

- Line 179: `stitch_percentile_index_meta table should exist`
  ```rust
  assert!(meta_exists, "stitch_percentile_index_meta table should exist");
  ```

- Line 383: `Cost p50 should be positive`
  ```rust
  assert!(cost_p50 > 0.0, "Cost p50 should be positive");
  ```

- Line 384: `Cost p90 should be >= p50`
  ```rust
  assert!(cost_p90 > cost_p50, "Cost p90 should be >= p50");
  ```

#### assert_eq (6 occurrences)

- Line 371: `Should have one bucket for 3 similar stitches`
  ```rust
  assert_eq!(count, 1, "Should have one bucket for 3 similar stitches");
  ```

- Line 385: `Should have 3 samples`
  ```rust
  assert_eq!(sample_count, 3, "Should have 3 samples");
  ```

- Line 546: `Should take first 5 tokens`
  ```rust
  assert_eq!(tokens.len(), 5, "Should take first 5 tokens");
  ```

- Line 627: `Should have one bucket`
  ```rust
  assert_eq!(count_before, 1, "Should have one bucket");
  ```

- Line 653: `Should have two buckets after rebuild`
  ```rust
  assert_eq!(count_after, 2, "Should have two buckets after rebuild");
  ```

- Line 689: `Should have 5 samples`
  ```rust
  assert_eq!(sample_count, 5, "Should have 5 samples");
  ```

#### expect (44 occurrences)

- Line 22: `Failed to open test DB`
  ```rust
  let mut conn = Connection::open(&db_path).expect("Failed to open test DB");
  ```

- Line 41: `Failed to create stitches table`
  ```rust
  .expect("Failed to create stitches table");
  ```

- Line 57: `Failed to create stitch_messages table`
  ```rust
  .expect("Failed to create stitch_messages table");
  ```

- Line 72: `Failed to create actions table`
  ```rust
  .expect("Failed to create actions table");
  ```

- Line 76: `Failed to initialize percentile index`
  ```rust
  .expect("Failed to initialize percentile index");
  ```

- Line 113: `Failed to insert stitch`
  ```rust
  .expect("Failed to insert stitch");
  ```

- Line 131: `Failed to insert message`
  ```rust
  .expect("Failed to insert message");
  ```

- Line 150: `Failed to insert action`
  ```rust
  .expect("Failed to insert action");
  ```

- Line 156: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 166: `Failed to check table existence`
  ```rust
  .expect("Failed to check table existence");
  ```

- Line 177: `Failed to check metadata table existence`
  ```rust
  .expect("Failed to check metadata table existence");
  ```

- Line 188: `Failed to get schema version`
  ```rust
  .expect("Failed to get schema version");
  ```

- Line 198: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 204: `Failed to check schema version`
  ```rust
  .expect("Failed to check schema version")
  ```

- Line 208: `Failed to check rebuild needed`
  ```rust
  .expect("Failed to check rebuild needed")
  ```

- Line 220: `Failed to corrupt schema version`
  ```rust
  .expect("Failed to corrupt schema version");
  ```

- Line 225: `Failed to check schema version`
  ```rust
  .expect("Failed to check schema version")
  ```

- Line 229: `Failed to check rebuild needed`
  ```rust
  .expect("Failed to check rebuild needed")
  ```

- Line 324: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 360: `Failed to rebuild index`
  ```rust
  .expect("Failed to rebuild index");
  ```

- Line 369: `Failed to count index entries`
  ```rust
  .expect("Failed to count index entries");
  ```

- Line 380: `Failed to query bucket`
  ```rust
  .expect("Failed to query bucket");
  ```

- Line 390: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 421: `Failed to rebuild index`
  ```rust
  .expect("Failed to rebuild index");
  ```

- Line 430: `Failed to count buckets`
  ```rust
  .expect("Failed to count buckets");
  ```

- Line 446: `Query should succeed`
  ```rust
  .expect("Query should succeed");
  ```

- Line 467: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 482: `Failed to rebuild index`
  ```rust
  .expect("Failed to rebuild index");
  ```

- Line 492: `Query should succeed`
  ```rust
  .expect("Query should succeed");
  ```

- Line 506: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 514: `Failed to rebuild index`
  ```rust
  .expect("Failed to rebuild index");
  ```

- Line 523: `Query should succeed`
  ```rust
  .expect("Query should succeed");
  ```

- Line 557: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 572: `Failed to rebuild index`
  ```rust
  .expect("Failed to rebuild index");
  ```

- Line 582: `Query should succeed`
  ```rust
  .expect("Query should succeed");
  ```

- Line 598: `Query should succeed`
  ```rust
  .expect("Query should succeed");
  ```

- Line 609: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 617: `Failed to rebuild index`
  ```rust
  .expect("Failed to rebuild index");
  ```

- Line 625: `Failed to count`
  ```rust
  .expect("Failed to count");
  ```

- Line 642: `Failed to rebuild index`
  ```rust
  .expect("Failed to rebuild index");
  ```

- Line 650: `Failed to count`
  ```rust
  .expect("Failed to count");
  ```

- Line 658: `Failed to create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  ```

- Line 678: `Failed to rebuild index`
  ```rust
  .expect("Failed to rebuild index");
  ```

- Line 687: `Failed to query bucket`
  ```rust
  .expect("Failed to query bucket");
  ```

### hoop-daemon/tests/supervisor_health.rs

Total errors: 33

#### assert (7 occurrences)

- Line 186: `Should receive status update`
  ```rust
  assert!(received, "Should receive status update");
  ```

- Line 246: `Should not be ready with no runtimes`
  ```rust
  assert!(!is_ready(&snapshot), "Should not be ready with no runtimes");
  ```

- Line 263: `Should be ready with healthy runtime`
  ```rust
  assert!(is_ready(&snapshot), "Should be ready with healthy runtime");
  ```

- Line 309: `Should not be ready when all failed`
  ```rust
  assert!(!is_ready(&all_failed), "Should not be ready when all failed");
  ```

- Line 323: `Should not be ready when all in error state`
  ```rust
  assert!(!is_ready(&all_error), "Should not be ready when all in error state");
  ```

- Line 337: `Should not be ready when all abandoned`
  ```rust
  assert!(!is_ready(&all_abandoned), "Should not be ready when all abandoned");
  ```

- Line 378: `Should be ready with at least one healthy`
  ```rust
  assert!(is_ready(&mixed_states), "Should be ready with at least one healthy");
  ```

#### expect (10 occurrences)

- Line 57: `Failed to create CostAggregator`
  ```rust
  .expect("Failed to create CostAggregator"),
  ```

- Line 141: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 178: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 216: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 257: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 284: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 429: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 467: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 508: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 579: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

#### unwrap (16 occurrences)

- Line 38: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&beads_dir).unwrap();
  ```

- Line 40: `\.unwrap\(\)`
  ```rust
  std::fs::write(&issues_path, b"").unwrap();
  ```

- Line 43: `\.unwrap\(\)`
  ```rust
  tempfile::TempDir::new().unwrap()
  ```

- Line 113: `\.unwrap\(\)`
  ```rust
  let project1_dir = tempfile::tempdir().unwrap();
  ```

- Line 117: `\.unwrap\(\)`
  ```rust
  let project2_dir = tempfile::tempdir().unwrap();
  ```

- Line 121: `\.unwrap\(\)`
  ```rust
  let project3_dir = tempfile::tempdir().unwrap();
  ```

- Line 160: `\.unwrap\(\)`
  ```rust
  let project_dir = tempfile::tempdir().unwrap();
  ```

- Line 196: `\.unwrap\(\)`
  ```rust
  let project_dir = tempfile::tempdir().unwrap();
  ```

- Line 238: `\.unwrap\(\)`
  ```rust
  let project_dir = tempfile::tempdir().unwrap();
  ```

- Line 269: `\.unwrap\(\)`
  ```rust
  let project_dir = tempfile::tempdir().unwrap();
  ```

- Line 414: `\.unwrap\(\)`
  ```rust
  let project_dir = tempfile::tempdir().unwrap();
  ```

- Line 446: `\.unwrap\(\)`
  ```rust
  let project_dir = tempfile::tempdir().unwrap();
  ```

- Line 493: `\.unwrap\(\)`
  ```rust
  let project_dir = tempfile::tempdir().unwrap();
  ```

- Line 523: `\.unwrap\(\)`
  ```rust
  let project_dir = tempfile::tempdir().unwrap();
  ```

- Line 526: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&project_path1).unwrap();
  ```

- Line 527: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&project_path2).unwrap();
  ```

### hoop-daemon/tests/supervisor_hotreload.rs

Total errors: 24

#### assert_eq (5 occurrences)

- Line 118: `Should have no runtimes initially`
  ```rust
  assert_eq!(snapshot.len(), 0, "Should have no runtimes initially");
  ```

- Line 135: `Should have one runtime`
  ```rust
  assert_eq!(snapshot.len(), 1, "Should have one runtime");
  ```

- Line 172: `Should have three runtimes`
  ```rust
  assert_eq!(snapshot.len(), 3, "Should have three runtimes");
  ```

- Line 207: `Should have two runtimes initially`
  ```rust
  assert_eq!(snapshot.len(), 2, "Should have two runtimes initially");
  ```

- Line 220: `Should have one runtime after removal`
  ```rust
  assert_eq!(snapshot.len(), 1, "Should have one runtime after removal");
  ```

#### expect (8 occurrences)

- Line 115: `Empty reconcile should succeed`
  ```rust
  .expect("Empty reconcile should succeed");
  ```

- Line 129: `Reconcile with new project should succeed`
  ```rust
  .expect("Reconcile with new project should succeed");
  ```

- Line 166: `Reconcile with multiple projects should succeed`
  ```rust
  .expect("Reconcile with multiple projects should succeed");
  ```

- Line 202: `Reconcile with two projects should succeed`
  ```rust
  .expect("Reconcile with two projects should succeed");
  ```

- Line 215: `Reconcile after removal should succeed`
  ```rust
  .expect("Reconcile after removal should succeed");
  ```

- Line 242: `Initial reconcile should succeed`
  ```rust
  .expect("Initial reconcile should succeed");
  ```

- Line 253: `No-op reconcile should succeed`
  ```rust
  .expect("No-op reconcile should succeed");
  ```

- Line 317: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

#### unwrap (11 occurrences)

- Line 39: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&beads_dir).unwrap();
  ```

- Line 41: `\.unwrap\(\)`
  ```rust
  std::fs::write(&issues_path, b"").unwrap();
  ```

- Line 44: `\.unwrap\(\)`
  ```rust
  tempfile::TempDir::new().unwrap()
  ```

- Line 104: `\.unwrap\(\)`
  ```rust
  let project_dir = tempfile::tempdir().unwrap();
  ```

- Line 142: `\.unwrap\(\)`
  ```rust
  let project1_dir = tempfile::tempdir().unwrap();
  ```

- Line 146: `\.unwrap\(\)`
  ```rust
  let project2_dir = tempfile::tempdir().unwrap();
  ```

- Line 150: `\.unwrap\(\)`
  ```rust
  let project3_dir = tempfile::tempdir().unwrap();
  ```

- Line 183: `\.unwrap\(\)`
  ```rust
  let project1_dir = tempfile::tempdir().unwrap();
  ```

- Line 187: `\.unwrap\(\)`
  ```rust
  let project2_dir = tempfile::tempdir().unwrap();
  ```

- Line 227: `\.unwrap\(\)`
  ```rust
  let project_dir = tempfile::tempdir().unwrap();
  ```

- Line 299: `\.unwrap\(\)`
  ```rust
  let project_dir = tempfile::tempdir().unwrap();
  ```

### hoop-daemon/tests/supervisor_isolation.rs

Total errors: 33

#### assert_eq (2 occurrences)

- Line 144: `Should have two runtimes`
  ```rust
  assert_eq!(snapshot.len(), 2, "Should have two runtimes");
  ```

- Line 204: `Both runtimes should still exist`
  ```rust
  assert_eq!(snapshot_after.len(), 2, "Both runtimes should still exist");
  ```

#### expect (12 occurrences)

- Line 59: `CostAggregator creation should succeed`
  ```rust
  .expect("CostAggregator creation should succeed"),
  ```

- Line 138: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 181: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 210: `project-a should exist`
  ```rust
  .expect("project-a should exist");
  ```

- Line 215: `project-b should exist`
  ```rust
  .expect("project-b should exist");
  ```

- Line 260: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 288: `project-b should exist`
  ```rust
  .expect("project-b should exist");
  ```

- Line 293: `project-c should exist`
  ```rust
  .expect("project-c should exist");
  ```

- Line 383: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 428: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 475: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 507: `project-b should exist`
  ```rust
  .expect("project-b should exist");
  ```

#### unwrap (19 occurrences)

- Line 40: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&beads_dir).unwrap();
  ```

- Line 42: `\.unwrap\(\)`
  ```rust
  std::fs::write(&issues_path, b"").unwrap();
  ```

- Line 45: `\.unwrap\(\)`
  ```rust
  tempfile::TempDir::new().unwrap()
  ```

- Line 119: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 123: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 159: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 163: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 194: `\.unwrap\(\)`
  ```rust
  std::fs::remove_dir_all(&beads_path).unwrap();
  ```

- Line 236: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 240: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 244: `\.unwrap\(\)`
  ```rust
  let project_c_dir = tempfile::tempdir().unwrap();
  ```

- Line 315: `\.unwrap\(\)`
  ```rust
  let project_dir = tempfile::tempdir().unwrap();
  ```

- Line 364: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 368: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 409: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 413: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 453: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 457: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 490: `\.unwrap\(\)`
  ```rust
  std::fs::remove_dir_all(&beads_path).unwrap();
  ```

### hoop-daemon/tests/supervisor_restart.rs

Total errors: 4

#### expect (1 occurrences)

- Line 57: `Failed to create cost aggregator`
  ```rust
  .expect("Failed to create cost aggregator"),
  ```

#### unwrap (3 occurrences)

- Line 39: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&beads_dir).unwrap();
  ```

- Line 41: `\.unwrap\(\)`
  ```rust
  std::fs::write(&issues_path, b"").unwrap();
  ```

- Line 44: `\.unwrap\(\)`
  ```rust
  tempfile::TempDir::new().unwrap()
  ```

### hoop-daemon/tests/supervisor_shutdown.rs

Total errors: 26

#### assert (1 occurrences)

- Line 130: `Runtime should be running`
  ```rust
  assert!(is_running, "Runtime should be running");
  ```

#### assert_eq (6 occurrences)

- Line 125: `Should have one runtime`
  ```rust
  assert_eq!(snapshot.len(), 1, "Should have one runtime");
  ```

- Line 136: `Runtime should still exist`
  ```rust
  assert_eq!(snapshot_after.len(), 1, "Runtime should still exist");
  ```

- Line 166: `Should have two runtimes`
  ```rust
  assert_eq!(snapshot.len(), 2, "Should have two runtimes");
  ```

- Line 179: `Should have one runtime after removal`
  ```rust
  assert_eq!(snapshot.len(), 1, "Should have one runtime after removal");
  ```

- Line 215: `Should have three runtimes`
  ```rust
  assert_eq!(snapshot.len(), 3, "Should have three runtimes");
  ```

- Line 228: `Should have no runtimes after shutdown`
  ```rust
  assert_eq!(snapshot.len(), 0, "Should have no runtimes after shutdown");
  ```

#### expect (8 occurrences)

- Line 119: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 161: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 174: `Reconcile after removal should succeed`
  ```rust
  .expect("Reconcile after removal should succeed");
  ```

- Line 210: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 223: `Reconcile to empty should succeed`
  ```rust
  .expect("Reconcile to empty should succeed");
  ```

- Line 252: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 301: `Reconcile should succeed`
  ```rust
  .expect("Reconcile should succeed");
  ```

- Line 318: `Reconcile to empty should succeed`
  ```rust
  .expect("Reconcile to empty should succeed");
  ```

#### unwrap (11 occurrences)

- Line 39: `\.unwrap\(\)`
  ```rust
  std::fs::create_dir_all(&beads_dir).unwrap();
  ```

- Line 41: `\.unwrap\(\)`
  ```rust
  std::fs::write(&issues_path, b"").unwrap();
  ```

- Line 44: `\.unwrap\(\)`
  ```rust
  tempfile::TempDir::new().unwrap()
  ```

- Line 104: `\.unwrap\(\)`
  ```rust
  let project_dir = tempfile::tempdir().unwrap();
  ```

- Line 142: `\.unwrap\(\)`
  ```rust
  let project1_dir = tempfile::tempdir().unwrap();
  ```

- Line 146: `\.unwrap\(\)`
  ```rust
  let project2_dir = tempfile::tempdir().unwrap();
  ```

- Line 186: `\.unwrap\(\)`
  ```rust
  let project1_dir = tempfile::tempdir().unwrap();
  ```

- Line 190: `\.unwrap\(\)`
  ```rust
  let project2_dir = tempfile::tempdir().unwrap();
  ```

- Line 194: `\.unwrap\(\)`
  ```rust
  let project3_dir = tempfile::tempdir().unwrap();
  ```

- Line 234: `\.unwrap\(\)`
  ```rust
  let project_dir = tempfile::tempdir().unwrap();
  ```

- Line 284: `\.unwrap\(\)`
  ```rust
  let project_dir = tempfile::tempdir().unwrap();
  ```

### hoop-daemon/tests/testrepo_harness_integration.rs

Total errors: 72

#### assert (8 occurrences)

- Line 377: `Beads response should be an array`
  ```rust
  assert!(true, "Beads response should be an array");
  ```

- Line 382: `Workers response should be an array`
  ```rust
  assert!(true, "Workers response should be an array");
  ```

- Line 387: `Conversations response should be an array`
  ```rust
  assert!(true, "Conversations response should be an array");
  ```

- Line 392: `Projects response should be an array`
  ```rust
  assert!(true, "Projects response should be an array");
  ```

- Line 396: `Config status must include `
  ```rust
  assert!(config.get("valid").is_some(), "Config status must include 'valid' field");
  ```

- Line 401: `Capacity should be object or array`
  ```rust
  assert!(capacity.is_object() || capacity.is_array(), "Capacity should be object or array");
  ```

- Line 488: `Should receive messages after subscribe/unsubscribe`
  ```rust
  assert!(snapshot_msg.is_ok(), "Should receive messages after subscribe/unsubscribe");
  ```

- Line 535: `Connection should receive init`
  ```rust
  assert!(handle.await.expect("Task failed"), "Connection should receive init");
  ```

#### assert_eq (3 occurrences)

- Line 264: `Health check should return ok`
  ```rust
  assert_eq!(health["status"], "ok", "Health check should return ok");
  ```

- Line 268: `Ready check should return ok`
  ```rust
  assert_eq!(ready["status"], "ok", "Ready check should return ok");
  ```

- Line 300: `First message must be init`
  ```rust
  assert_eq!(event["type"], "init", "First message must be init");
  ```

#### bail (1 occurrences)

- Line 57: `Daemon did not become ready`
  ```rust
  anyhow::bail!("Daemon did not become ready");
  ```

#### expect (55 occurrences)

- Line 258: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 260: `Failed to create test client`
  ```rust
  let client = TestClient::new(base_url.clone()).await.expect("Failed to create test client");
  ```

- Line 263: `Health check failed`
  ```rust
  let health = client.healthz().await.expect("Health check failed");
  ```

- Line 267: `Ready check failed`
  ```rust
  let ready = client.readyz().await.expect("Ready check failed");
  ```

- Line 276: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 278: `Failed to create test client`
  ```rust
  let client = TestClient::new(base_url.clone()).await.expect("Failed to create test client");
  ```

- Line 285: `Failed to connect to WebSocket`
  ```rust
  .expect("Failed to connect to WebSocket");
  ```

- Line 291: `Timeout waiting for first message`
  ```rust
  .expect("Timeout waiting for first message")
  ```

- Line 292: `WebSocket stream ended`
  ```rust
  .expect("WebSocket stream ended");
  ```

- Line 294: `Failed to receive first message`
  ```rust
  let first_msg = first_msg.expect("Failed to receive first message");
  ```

- Line 298: `Failed to parse init event`
  ```rust
  .expect("Failed to parse init event");
  ```

- Line 309: `subscriptions should be array`
  ```rust
  .expect("subscriptions should be array")
  ```

- Line 317: `subscriptions should be array`
  ```rust
  .expect("subscriptions should be array")
  ```

- Line 336: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 338: `Failed to create test client`
  ```rust
  let client = TestClient::new(base_url.clone()).await.expect("Failed to create test client");
  ```

- Line 340: `Failed to collect snapshots`
  ```rust
  let snapshots = client.collect_ws_snapshots().await.expect("Failed to collect snapshots");
  ```

- Line 370: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 372: `Failed to create test client`
  ```rust
  let client = TestClient::new(base_url.clone()).await.expect("Failed to create test client");
  ```

- Line 375: `Failed to fetch beads`
  ```rust
  let beads = client.get_beads().await.expect("Failed to fetch beads");
  ```

- Line 380: `Failed to fetch workers`
  ```rust
  let workers = client.get_workers_timeline().await.expect("Failed to fetch workers");
  ```

- Line 385: `Failed to fetch conversations`
  ```rust
  let conversations = client.get_conversations().await.expect("Failed to fetch conversations");
  ```

- Line 390: `Failed to fetch projects`
  ```rust
  let projects = client.get_projects().await.expect("Failed to fetch projects");
  ```

- Line 395: `Failed to fetch config status`
  ```rust
  let config = client.get_config_status().await.expect("Failed to fetch config status");
  ```

- Line 399: `Failed to fetch capacity`
  ```rust
  let capacity = client.get_capacity().await.expect("Failed to fetch capacity");
  ```

- Line 409: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 411: `Failed to create test client`
  ```rust
  let client = TestClient::new(base_url.clone()).await.expect("Failed to create test client");
  ```

- Line 413: `Failed to fetch metrics`
  ```rust
  let metrics = client.get_metrics().await.expect("Failed to fetch metrics");
  ```

- Line 444: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 451: `Failed to connect`
  ```rust
  .expect("Failed to connect");
  ```

- Line 458: `Timeout waiting for init`
  ```rust
  .expect("Timeout waiting for init")
  ```

- Line 459: `Stream ended`
  ```rust
  .expect("Stream ended")
  ```

- Line 460: `Failed to receive init`
  ```rust
  .expect("Failed to receive init");
  ```

- Line 474: `Failed to send subscribe`
  ```rust
  .expect("Failed to send subscribe");
  ```

- Line 483: `Failed to send unsubscribe`
  ```rust
  .expect("Failed to send unsubscribe");
  ```

- Line 496: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 516: `Stream ended`
  ```rust
  .expect("Stream ended");
  ```

- Line 522: `Failed to parse`
  ```rust
  .expect("Failed to parse");
  ```

- Line 535: `Task failed`
  ```rust
  assert!(handle.await.expect("Task failed"), "Connection should receive init");
  ```

- Line 544: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 553: `Failed to connect first time`
  ```rust
  .expect("Failed to connect first time");
  ```

- Line 560: `Timeout on first connection`
  ```rust
  .expect("Timeout on first connection")
  ```

- Line 561: `Stream ended`
  ```rust
  .expect("Stream ended")
  ```

- Line 562: `No init on first connection`
  ```rust
  .expect("No init on first connection");
  ```

- Line 573: `Failed to reconnect`
  ```rust
  .expect("Failed to reconnect");
  ```

- Line 580: `Timeout on reconnection`
  ```rust
  .expect("Timeout on reconnection")
  ```

- Line 581: `Stream ended`
  ```rust
  .expect("Stream ended")
  ```

- Line 582: `No init on reconnection`
  ```rust
  .expect("No init on reconnection");
  ```

- Line 592: `Timeout waiting for snapshots after reconnect`
  ```rust
  .expect("Timeout waiting for snapshots after reconnect")
  ```

- Line 593: `Stream ended`
  ```rust
  .expect("Stream ended")
  ```

- Line 594: `No snapshots after reconnect`
  ```rust
  .expect("No snapshots after reconnect");
  ```

- Line 613: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 615: `Failed to create test client`
  ```rust
  let client = TestClient::new(base_url.clone()).await.expect("Failed to create test client");
  ```

- Line 618: `Failed to fetch beads`
  ```rust
  let beads = client.get_beads().await.expect("Failed to fetch beads");
  ```

- Line 635: `Failed to fetch workers`
  ```rust
  let workers = client.get_workers_timeline().await.expect("Failed to fetch workers");
  ```

- Line 645: `Failed to fetch projects`
  ```rust
  let projects = client.get_projects().await.expect("Failed to fetch projects");
  ```

#### panic (1 occurrences)

- Line 327: `First message must be text, got {:?}`
  ```rust
  panic!("First message must be text, got {:?}", first_msg);
  ```

#### unwrap (4 occurrences)

- Line 463: `\.unwrap\(\)`
  ```rust
  let event: serde_json::Value = serde_json::from_str(&text).unwrap();
  ```

- Line 565: `\.unwrap\(\)`
  ```rust
  let event: serde_json::Value = serde_json::from_str(&text).unwrap();
  ```

- Line 585: `\.unwrap\(\)`
  ```rust
  let event: serde_json::Value = serde_json::from_str(&text).unwrap();
  ```

- Line 597: `\.unwrap\(\)`
  ```rust
  let event: serde_json::Value = serde_json::from_str(&text).unwrap();
  ```

### hoop-daemon/tests/testrepo_integration.rs

Total errors: 77

#### assert (8 occurrences)

- Line 396: `Beads response should not be empty`
  ```rust
  assert!(!beads.is_empty(), "Beads response should not be empty");
  ```

- Line 400: `Workers response should not be empty`
  ```rust
  assert!(!workers.is_empty(), "Workers response should not be empty");
  ```

- Line 404: `Projects response should not be empty`
  ```rust
  assert!(!projects.is_empty(), "Projects response should not be empty");
  ```

- Line 410: `testrepo should be in projects list`
  ```rust
  assert!(testrepo_found, "testrepo should be in projects list");
  ```

- Line 414: `Config status must include `
  ```rust
  assert!(config.get("valid").is_some(), "Config status must include 'valid' field");
  ```

- Line 418: `Capacity should be object or array`
  ```rust
  assert!(capacity.is_object() || capacity.is_array(), "Capacity should be object or array");
  ```

- Line 505: `Should receive messages after subscribe/unsubscribe`
  ```rust
  assert!(snapshot_msg.is_ok(), "Should receive messages after subscribe/unsubscribe");
  ```

- Line 552: `Connection should receive init`
  ```rust
  assert!(handle.await.expect("Task failed"), "Connection should receive init");
  ```

#### assert_eq (3 occurrences)

- Line 244: `Health check should return ok`
  ```rust
  assert_eq!(health["status"], "ok", "Health check should return ok");
  ```

- Line 248: `Ready check should return ok`
  ```rust
  assert_eq!(ready["status"], "ok", "Ready check should return ok");
  ```

- Line 280: `First message must be init`
  ```rust
  assert_eq!(event["type"], "init", "First message must be init");
  ```

#### bail (1 occurrences)

- Line 67: `Daemon did not become ready`
  ```rust
  anyhow::bail!("Daemon did not become ready");
  ```

#### expect (60 occurrences)

- Line 238: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 240: `Failed to create test client`
  ```rust
  let client = TestClient::new(base_url).await.expect("Failed to create test client");
  ```

- Line 243: `Health check failed`
  ```rust
  let health = client.healthz().await.expect("Health check failed");
  ```

- Line 247: `Ready check failed`
  ```rust
  let ready = client.readyz().await.expect("Ready check failed");
  ```

- Line 256: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 258: `Failed to create test client`
  ```rust
  let client = TestClient::new(base_url).await.expect("Failed to create test client");
  ```

- Line 265: `Failed to connect to WebSocket`
  ```rust
  .expect("Failed to connect to WebSocket");
  ```

- Line 271: `Timeout waiting for first message`
  ```rust
  .expect("Timeout waiting for first message")
  ```

- Line 272: `WebSocket stream ended`
  ```rust
  .expect("WebSocket stream ended");
  ```

- Line 274: `Failed to receive first message`
  ```rust
  let first_msg = first_msg.expect("Failed to receive first message");
  ```

- Line 278: `Failed to parse init event`
  ```rust
  .expect("Failed to parse init event");
  ```

- Line 289: `subscriptions should be array`
  ```rust
  .expect("subscriptions should be array")
  ```

- Line 308: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 310: `Failed to create test client`
  ```rust
  let client = TestClient::new(base_url).await.expect("Failed to create test client");
  ```

- Line 312: `Failed to collect snapshots`
  ```rust
  let snapshots = client.collect_ws_snapshots().await.expect("Failed to collect snapshots");
  ```

- Line 342: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 344: `Failed to create test client`
  ```rust
  let client = TestClient::new(base_url).await.expect("Failed to create test client");
  ```

- Line 347: `Failed to collect WS snapshots`
  ```rust
  let ws_snapshots = client.collect_ws_snapshots().await.expect("Failed to collect WS snapshots");
  ```

- Line 350: `Failed to fetch beads via REST`
  ```rust
  let rest_beads = client.get_beads().await.expect("Failed to fetch beads via REST");
  ```

- Line 351: `Failed to fetch workers via REST`
  ```rust
  let rest_workers = client.get_workers().await.expect("Failed to fetch workers via REST");
  ```

- Line 352: `Failed to fetch projects via REST`
  ```rust
  let rest_projects = client.get_projects().await.expect("Failed to fetch projects via REST");
  ```

- Line 353: `Failed to fetch config via REST`
  ```rust
  let rest_config = client.get_config_status().await.expect("Failed to fetch config via REST");
  ```

- Line 390: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 392: `Failed to create test client`
  ```rust
  let client = TestClient::new(base_url).await.expect("Failed to create test client");
  ```

- Line 395: `Failed to fetch beads`
  ```rust
  let beads = client.get_beads().await.expect("Failed to fetch beads");
  ```

- Line 399: `Failed to fetch workers`
  ```rust
  let workers = client.get_workers().await.expect("Failed to fetch workers");
  ```

- Line 403: `Failed to fetch projects`
  ```rust
  let projects = client.get_projects().await.expect("Failed to fetch projects");
  ```

- Line 413: `Failed to fetch config status`
  ```rust
  let config = client.get_config_status().await.expect("Failed to fetch config status");
  ```

- Line 417: `Failed to fetch capacity`
  ```rust
  let capacity = client.get_capacity().await.expect("Failed to fetch capacity");
  ```

- Line 426: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 428: `Failed to create test client`
  ```rust
  let client = TestClient::new(base_url).await.expect("Failed to create test client");
  ```

- Line 430: `Failed to fetch metrics`
  ```rust
  let metrics = client.get_metrics().await.expect("Failed to fetch metrics");
  ```

- Line 461: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 468: `Failed to connect`
  ```rust
  .expect("Failed to connect");
  ```

- Line 475: `Timeout waiting for init`
  ```rust
  .expect("Timeout waiting for init")
  ```

- Line 476: `Stream ended`
  ```rust
  .expect("Stream ended")
  ```

- Line 477: `Failed to receive init`
  ```rust
  .expect("Failed to receive init");
  ```

- Line 491: `Failed to send subscribe`
  ```rust
  .expect("Failed to send subscribe");
  ```

- Line 500: `Failed to send unsubscribe`
  ```rust
  .expect("Failed to send unsubscribe");
  ```

- Line 513: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 533: `Stream ended`
  ```rust
  .expect("Stream ended");
  ```

- Line 539: `Failed to parse`
  ```rust
  .expect("Failed to parse");
  ```

- Line 552: `Task failed`
  ```rust
  assert!(handle.await.expect("Task failed"), "Connection should receive init");
  ```

- Line 561: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 570: `Failed to connect first time`
  ```rust
  .expect("Failed to connect first time");
  ```

- Line 577: `Timeout on first connection`
  ```rust
  .expect("Timeout on first connection")
  ```

- Line 578: `Stream ended`
  ```rust
  .expect("Stream ended")
  ```

- Line 579: `No init on first connection`
  ```rust
  .expect("No init on first connection");
  ```

- Line 590: `Failed to reconnect`
  ```rust
  .expect("Failed to reconnect");
  ```

- Line 597: `Timeout on reconnection`
  ```rust
  .expect("Timeout on reconnection")
  ```

- Line 598: `Stream ended`
  ```rust
  .expect("Stream ended")
  ```

- Line 599: `No init on reconnection`
  ```rust
  .expect("No init on reconnection");
  ```

- Line 609: `Timeout waiting for snapshots after reconnect`
  ```rust
  .expect("Timeout waiting for snapshots after reconnect")
  ```

- Line 610: `Stream ended`
  ```rust
  .expect("Stream ended")
  ```

- Line 611: `No snapshots after reconnect`
  ```rust
  .expect("No snapshots after reconnect");
  ```

- Line 630: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 632: `Failed to create test client`
  ```rust
  let client = TestClient::new(base_url).await.expect("Failed to create test client");
  ```

- Line 635: `Failed to fetch beads`
  ```rust
  let beads = client.get_beads().await.expect("Failed to fetch beads");
  ```

- Line 652: `Failed to fetch workers`
  ```rust
  let workers = client.get_workers().await.expect("Failed to fetch workers");
  ```

- Line 665: `Failed to fetch projects`
  ```rust
  let projects = client.get_projects().await.expect("Failed to fetch projects");
  ```

#### panic (1 occurrences)

- Line 299: `First message must be text, got {:?}`
  ```rust
  panic!("First message must be text, got {:?}", first_msg);
  ```

#### unwrap (4 occurrences)

- Line 480: `\.unwrap\(\)`
  ```rust
  let event: serde_json::Value = serde_json::from_str(&text).unwrap();
  ```

- Line 582: `\.unwrap\(\)`
  ```rust
  let event: serde_json::Value = serde_json::from_str(&text).unwrap();
  ```

- Line 602: `\.unwrap\(\)`
  ```rust
  let event: serde_json::Value = serde_json::from_str(&text).unwrap();
  ```

- Line 614: `\.unwrap\(\)`
  ```rust
  let event: serde_json::Value = serde_json::from_str(&text).unwrap();
  ```

### hoop-daemon/tests/upload_secrets_scan.rs

Total errors: 35

#### assert (7 occurrences)

- Line 42: `Should detect secret in attachment`
  ```rust
  assert!(!findings.is_empty(), "Should detect secret in attachment");
  ```

- Line 67: `Should detect at least 3 secrets`
  ```rust
  assert!(findings.len() >= 3, "Should detect at least 3 secrets");
  ```

- Line 94: `Clean attachment should have no findings`
  ```rust
  assert!(findings.is_empty(), "Clean attachment should have no findings");
  ```

- Line 138: `Binary files should not be scanned`
  ```rust
  assert!(findings.is_empty(), "Binary files should not be scanned");
  ```

- Line 165: `Should detect secrets in JSON`
  ```rust
  assert!(!findings.is_empty(), "Should detect secrets in JSON");
  ```

- Line 198: `Should detect at least 2 env var secrets`
  ```rust
  assert!(findings.len() >= 2, "Should detect at least 2 env var secrets");
  ```

- Line 218: `Large files should be skipped`
  ```rust
  assert!(findings.is_empty(), "Large files should be skipped");
  ```

#### assert_eq (1 occurrences)

- Line 264: `Should write one audit entry`
  ```rust
  assert_eq!(written, 1, "Should write one audit entry");
  ```

#### unwrap (27 occurrences)

- Line 16: `\.unwrap\(\)`
  ```rust
  let mut file = std::fs::File::create(&path).unwrap();
  ```

- Line 17: `\.unwrap\(\)`
  ```rust
  file.write_all(content.as_bytes()).unwrap();
  ```

- Line 31: `\.unwrap\(\)`
  ```rust
  let dir = TempDir::new().unwrap();
  ```

- Line 40: `\.unwrap\(\)`
  ```rust
  let findings = hoop_daemon::redaction::scan_attachment(&attachment_path).unwrap();
  ```

- Line 54: `\.unwrap\(\)`
  ```rust
  let dir = TempDir::new().unwrap();
  ```

- Line 65: `\.unwrap\(\)`
  ```rust
  let findings = hoop_daemon::redaction::scan_attachment(&attachment_path).unwrap();
  ```

- Line 82: `\.unwrap\(\)`
  ```rust
  let dir = TempDir::new().unwrap();
  ```

- Line 92: `\.unwrap\(\)`
  ```rust
  let findings = hoop_daemon::redaction::scan_attachment(&attachment_path).unwrap();
  ```

- Line 102: `\.unwrap\(\)`
  ```rust
  let dir = TempDir::new().unwrap();
  ```

- Line 112: `\.unwrap\(\)`
  ```rust
  let findings = hoop_daemon::redaction::scan_attachment(&attachment_path).unwrap();
  ```

- Line 127: `\.unwrap\(\)`
  ```rust
  let dir = TempDir::new().unwrap();
  ```

- Line 131: `\.unwrap\(\)`
  ```rust
  let mut file = std::fs::File::create(&path).unwrap();
  ```

- Line 132: `\.unwrap\(\)`
  ```rust
  file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A]).unwrap(); // PNG header
  ```

- Line 133: `\.unwrap\(\)`
  ```rust
  file.write_all(b"sk-ant-api03-FAKESECRET").unwrap(); // Secret-like data
  ```

- Line 135: `\.unwrap\(\)`
  ```rust
  let findings = hoop_daemon::redaction::scan_attachment(&path).unwrap();
  ```

- Line 146: `\.unwrap\(\)`
  ```rust
  let dir = TempDir::new().unwrap();
  ```

- Line 163: `\.unwrap\(\)`
  ```rust
  let findings = hoop_daemon::redaction::scan_attachment(&attachment_path).unwrap();
  ```

- Line 186: `\.unwrap\(\)`
  ```rust
  let dir = TempDir::new().unwrap();
  ```

- Line 196: `\.unwrap\(\)`
  ```rust
  let findings = hoop_daemon::redaction::scan_attachment(&attachment_path).unwrap();
  ```

- Line 206: `\.unwrap\(\)`
  ```rust
  let dir = TempDir::new().unwrap();
  ```

- Line 211: `\.unwrap\(\)`
  ```rust
  let mut file = std::fs::File::create(&path).unwrap();
  ```

- Line 213: `\.unwrap\(\)`
  ```rust
  file.write_all(large_data.as_bytes()).unwrap();
  ```

- Line 215: `\.unwrap\(\)`
  ```rust
  let findings = hoop_daemon::redaction::scan_attachment(&path).unwrap();
  ```

- Line 226: `\.unwrap\(\)`
  ```rust
  let dir = TempDir::new().unwrap();
  ```

- Line 231: `\.unwrap\(\)`
  ```rust
  let findings = hoop_daemon::redaction::scan_attachment(&attachment_path).unwrap();
  ```

- Line 272: `\.unwrap\(\)`
  ```rust
  let dir = TempDir::new().unwrap();
  ```

- Line 285: `\.unwrap\(\)`
  ```rust
  let findings = hoop_daemon::redaction::scan_attachment(&attachment_path).unwrap();
  ```

### hoop-daemon/tests/zero_write_invariant.rs

Total errors: 15

#### panic (1 occurrences)

- Line 221: `invariant violated: this test should not run under create-only-write`
  ```rust
  panic!("invariant violated: this test should not run under create-only-write");
  ```

#### should_panic (14 occurrences)

- Line 98: `expected = "zero-write invariant violated: br create"`
  ```rust
  #[should_panic(expected = "zero-write invariant violated: br create")]
  ```

- Line 104: `expected = "zero-write invariant violated: br close"`
  ```rust
  #[should_panic(expected = "zero-write invariant violated: br close")]
  ```

- Line 110: `expected = "zero-write invariant violated: br update"`
  ```rust
  #[should_panic(expected = "zero-write invariant violated: br update")]
  ```

- Line 116: `expected = "zero-write invariant violated: br release"`
  ```rust
  #[should_panic(expected = "zero-write invariant violated: br release")]
  ```

- Line 122: `expected = "zero-write invariant violated: br claim"`
  ```rust
  #[should_panic(expected = "zero-write invariant violated: br claim")]
  ```

- Line 128: `expected = "zero-write invariant violated: br depend"`
  ```rust
  #[should_panic(expected = "zero-write invariant violated: br depend")]
  ```

- Line 158: `expected = "create-only invariant violated"`
  ```rust
  #[should_panic(expected = "create-only invariant violated")]
  ```

- Line 164: `expected = "create-only invariant violated"`
  ```rust
  #[should_panic(expected = "create-only invariant violated")]
  ```

- Line 170: `expected = "create-only invariant violated"`
  ```rust
  #[should_panic(expected = "create-only invariant violated")]
  ```

- Line 176: `expected = "create-only invariant violated"`
  ```rust
  #[should_panic(expected = "create-only invariant violated")]
  ```

- Line 182: `expected = "create-only invariant violated"`
  ```rust
  #[should_panic(expected = "create-only invariant violated")]
  ```

- Line 210: `expected = "invariant violated"`
  ```rust
  #[should_panic(expected = "invariant violated")]
  ```

- Line 225: `expected = "invariant violated"`
  ```rust
  #[should_panic(expected = "invariant violated")]
  ```

- Line 231: `expected = "invariant violated"`
  ```rust
  #[should_panic(expected = "invariant violated")]
  ```

### hoop-daemon/tests_phase5/adapter_failover.rs

Total errors: 73

#### assert (6 occurrences)

- Line 99: `Adapter build should succeed`
  ```rust
  assert!(adapter_result.is_ok(), "Adapter build should succeed");
  ```

- Line 120: `ZAI adapter build should succeed after Anthropic`
  ```rust
  assert!(adapter_result2.is_ok(), "ZAI adapter build should succeed after Anthropic");
  ```

- Line 452: `Global rule should be preserved`
  ```rust
  assert!(scopes.contains(&"global"), "Global rule should be preserved");
  ```

- Line 723: `Multi-line content should be preserved`
  ```rust
  assert!(messages[1].1.contains('\n'), "Multi-line content should be preserved");
  ```

- Line 724: `Quotes should be preserved`
  ```rust
  assert!(messages[1].1.contains('"'), "Quotes should be preserved");
  ```

- Line 725: `Code blocks should be preserved`
  ```rust
  assert!(messages[3].1.contains("```rust"), "Code blocks should be preserved");
  ```

#### assert_eq (19 occurrences)

- Line 188: `Stitch should be in hoop-agent project`
  ```rust
  assert_eq!(stitch_project, "hoop-agent", "Stitch should be in hoop-agent project");
  ```

- Line 189: `Stitch should be kind=operator`
  ```rust
  assert_eq!(stitch_kind, "operator", "Stitch should be kind=operator");
  ```

- Line 204: `All history messages should be stored`
  ```rust
  assert_eq!(msg_count, 4, "All history messages should be stored");
  ```

- Line 268: `Session should be marked as switched`
  ```rust
  assert_eq!(status, "switched", "Session should be marked as switched");
  ```

- Line 336: `Only one session should be active`
  ```rust
  assert_eq!(active_count, 1, "Only one session should be active");
  ```

- Line 346: `Active adapter should be zai`
  ```rust
  assert_eq!(active_adapter, "zai", "Active adapter should be zai");
  ```

- Line 449: `Both Reflection Ledger entries should be preserved`
  ```rust
  assert_eq!(entries.len(), 2, "Both Reflection Ledger entries should be preserved");
  ```

- Line 526: `Should have exactly one active session`
  ```rust
  assert_eq!(active.len(), 1, "Should have exactly one active session");
  ```

- Line 529: `Active adapter should be zai`
  ```rust
  assert_eq!(active_session.adapter, "zai", "Active adapter should be zai");
  ```

- Line 530: `Active model should be glm-5`
  ```rust
  assert_eq!(active_session.model, "glm-5", "Active model should be glm-5");
  ```

- Line 531: `New session should have 0 turns`
  ```rust
  assert_eq!(active_session.turn_count, 0, "New session should have 0 turns");
  ```

- Line 540: `Should have one archived session`
  ```rust
  assert_eq!(archived_sessions.len(), 1, "Should have one archived session");
  ```

- Line 627: `Created by should be hoop:agent`
  ```rust
  assert_eq!(created_by, "hoop:agent", "Created by should be hoop:agent");
  ```

- Line 638: `All 4 messages should be stored`
  ```rust
  assert_eq!(messages.len(), 4, "All 4 messages should be stored");
  ```

- Line 646: `Tool message should be preserved`
  ```rust
  assert_eq!(tool_messages.len(), 1, "Tool message should be preserved");
  ```

- Line 715: `Message count should match`
  ```rust
  assert_eq!(messages.len(), history.len(), "Message count should match");
  ```

- Line 718: `Role mismatch at message {}`
  ```rust
  assert_eq!(orig.0, retrieved.0, "Role mismatch at message {}", i);
  ```

- Line 719: `Content mismatch at message {}`
  ```rust
  assert_eq!(orig.1, retrieved.1, "Content mismatch at message {}", i);
  ```

- Line 800: `Only approved entries should appear`
  ```rust
  assert_eq!(approved.len(), 2, "Only approved entries should appear");
  ```

#### expect (44 occurrences)

- Line 26: `create temp dir`
  ```rust
  let tmp = TempDir::new().expect("create temp dir");
  ```

- Line 28: `create .hoop dir`
  ```rust
  std::fs::create_dir_all(&hoop_dir).expect("create .hoop dir");
  ```

- Line 34: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 73: `write config.yml`
  ```rust
  std::fs::write(path, yaml).expect("write config.yml");
  ```

- Line 157: `insert session`
  ```rust
  hoop_daemon::fleet::insert_agent_session(&session_row).expect("insert session");
  ```

- Line 175: `archive session as stitch`
  ```rust
  .expect("archive session as stitch");
  ```

- Line 178: `open db`
  ```rust
  let conn = rusqlite::Connection::open(hoop_daemon::fleet::db_path()).expect("open db");
  ```

- Line 186: `query stitch`
  ```rust
  .expect("query stitch");
  ```

- Line 202: `count messages`
  ```rust
  .expect("count messages");
  ```

- Line 213: `query linked stitch`
  ```rust
  .expect("query linked stitch");
  ```

- Line 251: `insert session`
  ```rust
  hoop_daemon::fleet::insert_agent_session(&session_row).expect("insert session");
  ```

- Line 255: `archive session`
  ```rust
  .expect("archive session");
  ```

- Line 258: `open db`
  ```rust
  let conn = rusqlite::Connection::open(hoop_daemon::fleet::db_path()).expect("open db");
  ```

- Line 266: `query archived session`
  ```rust
  .expect("query archived session");
  ```

- Line 314: `insert session`
  ```rust
  hoop_daemon::fleet::insert_agent_session(&session_row).expect("insert session");
  ```

- Line 326: `open db`
  ```rust
  let conn = rusqlite::Connection::open(hoop_daemon::fleet::db_path()).expect("open db");
  ```

- Line 334: `count active`
  ```rust
  .expect("count active");
  ```

- Line 344: `get active adapter`
  ```rust
  .expect("get active adapter");
  ```

- Line 396: `insert entry 1`
  ```rust
  hoop_daemon::fleet::insert_reflection_entry(&entry1).expect("insert entry 1");
  ```

- Line 397: `insert entry 2`
  ```rust
  hoop_daemon::fleet::insert_reflection_entry(&entry2).expect("insert entry 2");
  ```

- Line 419: `insert session`
  ```rust
  hoop_daemon::fleet::insert_agent_session(&session_row).expect("insert session");
  ```

- Line 421: `archive session`
  ```rust
  .expect("archive session");
  ```

- Line 443: `insert new session`
  ```rust
  hoop_daemon::fleet::insert_agent_session(&new_session_row).expect("insert new session");
  ```

- Line 447: `list approved entries`
  ```rust
  .expect("list approved entries");
  ```

- Line 489: `insert old session`
  ```rust
  hoop_daemon::fleet::insert_agent_session(&old_session).expect("insert old session");
  ```

- Line 493: `archive old session`
  ```rust
  .expect("archive old session");
  ```

- Line 515: `insert new session`
  ```rust
  hoop_daemon::fleet::insert_agent_session(&new_session).expect("insert new session");
  ```

- Line 519: `list sessions`
  ```rust
  .expect("list sessions");
  ```

- Line 535: `list sessions`
  ```rust
  .expect("list sessions")
  ```

- Line 585: `insert session`
  ```rust
  hoop_daemon::fleet::insert_agent_session(&session_row).expect("insert session");
  ```

- Line 606: `archive as stitch`
  ```rust
  .expect("archive as stitch");
  ```

- Line 609: `open db`
  ```rust
  let conn = rusqlite::Connection::open(hoop_daemon::fleet::db_path()).expect("open db");
  ```

- Line 617: `query stitch metadata`
  ```rust
  .expect("query stitch metadata");
  ```

- Line 632: `prepare query`
  ```rust
  .expect("prepare query")
  ```

- Line 634: `query messages`
  ```rust
  .expect("query messages")
  ```

- Line 683: `insert session`
  ```rust
  hoop_daemon::fleet::insert_agent_session(&session_row).expect("insert session");
  ```

- Line 701: `archive as stitch`
  ```rust
  .expect("archive as stitch");
  ```

- Line 704: `open db`
  ```rust
  let conn = rusqlite::Connection::open(hoop_daemon::fleet::db_path()).expect("open db");
  ```

- Line 708: `prepare query`
  ```rust
  .expect("prepare query")
  ```

- Line 710: `query messages`
  ```rust
  .expect("query messages")
  ```

- Line 773: `insert entry 1`
  ```rust
  hoop_daemon::fleet::insert_reflection_entry(&entry1).expect("insert entry 1");
  ```

- Line 774: `insert entry 2`
  ```rust
  hoop_daemon::fleet::insert_reflection_entry(&entry2).expect("insert entry 2");
  ```

- Line 794: `insert rejected`
  ```rust
  hoop_daemon::fleet::insert_reflection_entry(&rejected).expect("insert rejected");
  ```

- Line 798: `list approved`
  ```rust
  .expect("list approved");
  ```

#### unwrap (4 occurrences)

- Line 24: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 41: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 101: `\.unwrap\(\)`
  ```rust
  let adapter = adapter_result.unwrap();
  ```

- Line 122: `\.unwrap\(\)`
  ```rust
  let adapter2 = adapter_result2.unwrap();
  ```

### hoop-daemon/tests_phase5/adapter_failover_integration.rs

Total errors: 77

#### assert (2 occurrences)

- Line 74: `Adapter build should succeed`
  ```rust
  assert!(adapter_result.is_ok(), "Adapter build should succeed");
  ```

- Line 95: `ZAI adapter build should succeed after Anthropic`
  ```rust
  assert!(adapter_result2.is_ok(), "ZAI adapter build should succeed after Anthropic");
  ```

#### assert_eq (10 occurrences)

- Line 176: `Stitch should be created`
  ```rust
  assert_eq!(stitch_count, 1, "Stitch should be created");
  ```

- Line 187: `Stitch should be in hoop-agent project`
  ```rust
  assert_eq!(stitch_project, "hoop-agent", "Stitch should be in hoop-agent project");
  ```

- Line 188: `Stitch should be kind=operator`
  ```rust
  assert_eq!(stitch_kind, "operator", "Stitch should be kind=operator");
  ```

- Line 216: `Session should be marked as switched`
  ```rust
  assert_eq!(status, "switched", "Session should be marked as switched");
  ```

- Line 370: `Cost should be preserved`
  ```rust
  assert_eq!(cost_usd, 0.125, "Cost should be preserved");
  ```

- Line 371: `Input tokens should be preserved`
  ```rust
  assert_eq!(input_tokens, 5000, "Input tokens should be preserved");
  ```

- Line 372: `Output tokens should be preserved`
  ```rust
  assert_eq!(output_tokens, 2000, "Output tokens should be preserved");
  ```

- Line 373: `Turn count should be preserved`
  ```rust
  assert_eq!(turn_count, 7, "Turn count should be preserved");
  ```

- Line 547: `All approved rules should be preserved`
  ```rust
  assert_eq!(entries.len(), 2, "All approved rules should be preserved");
  ```

- Line 675: `Only approved rules should be returned`
  ```rust
  assert_eq!(entries.len(), 2, "Only approved rules should be returned");
  ```

#### expect (15 occurrences)

- Line 27: `create temp dir`
  ```rust
  let tmp = TempDir::new().expect("create temp dir");
  ```

- Line 29: `create .hoop dir`
  ```rust
  fs::create_dir_all(&hoop_dir).expect("create .hoop dir");
  ```

- Line 35: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 143: `load active session`
  ```rust
  .expect("load active session")
  ```

- Line 144: `should have active session`
  ```rust
  .expect("should have active session");
  ```

- Line 164: `archive session as stitch`
  ```rust
  fleet::archive_session_as_stitch(&session_row, &history).expect("archive session as stitch");
  ```

- Line 168: `archive agent session`
  ```rust
  .expect("archive agent session");
  ```

- Line 359: `archive session`
  ```rust
  .expect("archive session");
  ```

- Line 544: `list approved entries`
  ```rust
  let entries = fleet::list_approved_reflection_entries(None).expect("list approved entries");
  ```

- Line 611: `load active session should succeed`
  ```rust
  .expect("load active session should succeed")
  ```

- Line 612: `should have an active session`
  ```rust
  .expect("should have an active session");
  ```

- Line 672: `list approved entries`
  ```rust
  let entries = fleet::list_approved_reflection_entries(None).expect("list approved entries");
  ```

- Line 710: `load active session`
  ```rust
  .expect("load active session")
  ```

- Line 711: `should have active session`
  ```rust
  .expect("should have active session");
  ```

- Line 715: `archive as stitch`
  ```rust
  fleet::archive_session_as_stitch(&session_row, &history).expect("archive as stitch");
  ```

#### unwrap (50 occurrences)

- Line 25: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 42: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 76: `\.unwrap\(\)`
  ```rust
  let adapter = adapter_result.unwrap();
  ```

- Line 97: `\.unwrap\(\)`
  ```rust
  let adapter2 = adapter_result2.unwrap();
  ```

- Line 122: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 130: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 139: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 175: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 185: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 201: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 214: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 229: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 244: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 266: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 274: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 292: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 301: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 315: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 327: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 347: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 355: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 368: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 388: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 400: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 409: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 414: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 426: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 435: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 440: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 452: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 461: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 467: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 477: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 487: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 504: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 514: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 522: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 532: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 541: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 576: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 588: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 606: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 628: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 645: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 654: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 661: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 669: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 698: `\.unwrap\(\)`
  ```rust
  let conn = rusqlite::Connection::open(&db_path).unwrap();
  ```

- Line 706: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 724: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

### hoop-daemon/tests_phase5/adapter_failover_test.rs

Total errors: 114

#### assert (1 occurrences)

- Line 902: `Should have performed at least 6 health checks over 30s`
  ```rust
  assert!(checks >= 6, "Should have performed at least 6 health checks over 30s");
  ```

#### assert_eq (26 occurrences)

- Line 158: `Daemon should be healthy`
  ```rust
  assert_eq!(health["status"], "ok", "Daemon should be healthy");
  ```

- Line 162: `Agent spawn should succeed`
  ```rust
  assert_eq!(spawn_resp["status"], "ok", "Agent spawn should succeed");
  ```

- Line 170: `Agent should be active`
  ```rust
  assert_eq!(status["active"], true, "Agent should be active");
  ```

- Line 174: `Daemon should remain healthy after 5xx`
  ```rust
  assert_eq!(health["status"], "ok", "Daemon should remain healthy after 5xx");
  ```

- Line 190: `Agent spawn should succeed`
  ```rust
  assert_eq!(spawn_resp["status"], "ok", "Agent spawn should succeed");
  ```

- Line 201: `Agent should be active`
  ```rust
  assert_eq!(status["active"], true, "Agent should be active");
  ```

- Line 213: `Adapter switch should succeed`
  ```rust
  assert_eq!(switch_resp["status"], "ok", "Adapter switch should succeed");
  ```

- Line 242: `Should have exactly 1 active session`
  ```rust
  assert_eq!(active_count, 1, "Should have exactly 1 active session");
  ```

- Line 243: `Should have 1 switched (archived) session`
  ```rust
  assert_eq!(archived_count, 1, "Should have 1 switched (archived) session");
  ```

- Line 250: `Agent should still be active`
  ```rust
  assert_eq!(status["active"], true, "Agent should still be active");
  ```

- Line 251: `Adapter should be zai`
  ```rust
  assert_eq!(status["adapter"], "zai", "Adapter should be zai");
  ```

- Line 252: `Model should be glm-5`
  ```rust
  assert_eq!(status["model"], "glm-5", "Model should be glm-5");
  ```

- Line 441: `Should have 2 switched sessions`
  ```rust
  assert_eq!(archived_count, 2, "Should have 2 switched sessions");
  ```

- Line 581: `Daemon should remain healthy`
  ```rust
  assert_eq!(health["status"], "ok", "Daemon should remain healthy");
  ```

- Line 603: `Agent spawn should succeed`
  ```rust
  assert_eq!(spawn_resp["status"], "ok", "Agent spawn should succeed");
  ```

- Line 614: `Agent should be active`
  ```rust
  assert_eq!(status["active"], true, "Agent should be active");
  ```

- Line 651: `Agent should still be active`
  ```rust
  assert_eq!(status["active"], true, "Agent should still be active");
  ```

- Line 657: `Model should be glm-5`
  ```rust
  assert_eq!(status["model"], "glm-5", "Model should be glm-5");
  ```

- Line 676: `Should have exactly 1 active session`
  ```rust
  assert_eq!(active_count, 1, "Should have exactly 1 active session");
  ```

- Line 677: `Should have 1 switched (archived) session`
  ```rust
  assert_eq!(archived_count, 1, "Should have 1 switched (archived) session");
  ```

- Line 723: `Daemon should remain healthy after hot-reload`
  ```rust
  assert_eq!(health["status"], "ok", "Daemon should remain healthy after hot-reload");
  ```

- Line 821: `Daemon should be healthy initially`
  ```rust
  assert_eq!(health["status"], "ok", "Daemon should be healthy initially");
  ```

- Line 955: `Switch to ZAI should succeed`
  ```rust
  assert_eq!(switch_resp["status"], "ok", "Switch to ZAI should succeed");
  ```

- Line 962: `Agent should be active after switch`
  ```rust
  assert_eq!(status["active"], true, "Agent should be active after switch");
  ```

- Line 963: `Should be using ZAI adapter`
  ```rust
  assert_eq!(status["adapter"], "zai", "Should be using ZAI adapter");
  ```

- Line 967: `Daemon should be healthy after recovery`
  ```rust
  assert_eq!(final_health["status"], "ok", "Daemon should be healthy after recovery");
  ```

#### bail (1 occurrences)

- Line 46: `Daemon did not become ready`
  ```rust
  anyhow::bail!("Daemon did not become ready");
  ```

#### expect (82 occurrences)

- Line 152: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 154: `Failed to create client`
  ```rust
  let client = FailoverClient::new(_base_url.clone()).await.expect("Failed to create client");
  ```

- Line 157: `Health check failed`
  ```rust
  let health = client.healthz().await.expect("Health check failed");
  ```

- Line 161: `Failed to spawn agent`
  ```rust
  let spawn_resp = client.spawn_agent().await.expect("Failed to spawn agent");
  ```

- Line 169: `Failed to get agent status`
  ```rust
  .expect("Failed to get agent status");
  ```

- Line 173: `Health check failed`
  ```rust
  let health = client.healthz().await.expect("Health check failed");
  ```

- Line 184: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 186: `Failed to create client`
  ```rust
  let client = FailoverClient::new(_base_url.clone()).await.expect("Failed to create client");
  ```

- Line 189: `Failed to spawn agent`
  ```rust
  let spawn_resp = client.spawn_agent().await.expect("Failed to spawn agent");
  ```

- Line 194: `Should have session_db_id`
  ```rust
  .expect("Should have session_db_id");
  ```

- Line 200: `Failed to get agent status`
  ```rust
  .expect("Failed to get agent status");
  ```

- Line 212: `Failed to switch adapter`
  ```rust
  .expect("Failed to switch adapter");
  ```

- Line 217: `Should have new session_db_id`
  ```rust
  .expect("Should have new session_db_id");
  ```

- Line 229: `Failed to list sessions`
  ```rust
  .expect("Failed to list sessions");
  ```

- Line 249: `Failed to get agent status`
  ```rust
  .expect("Failed to get agent status");
  ```

- Line 262: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 264: `Failed to create client`
  ```rust
  let client = FailoverClient::new(_base_url.clone()).await.expect("Failed to create client");
  ```

- Line 267: `Failed to spawn agent`
  ```rust
  let spawn_resp = client.spawn_agent().await.expect("Failed to spawn agent");
  ```

- Line 272: `Should have session_db_id`
  ```rust
  .expect("Should have session_db_id");
  ```

- Line 278: `Failed to switch adapter`
  ```rust
  .expect("Failed to switch adapter");
  ```

- Line 284: `Failed to list sessions`
  ```rust
  .expect("Failed to list sessions");
  ```

- Line 290: `Should find archived session`
  ```rust
  .expect("Should find archived session");
  ```

- Line 308: `Failed to query stitch from fleet.db`
  ```rust
  .expect("Failed to query stitch from fleet.db");
  ```

- Line 341: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 343: `Failed to create client`
  ```rust
  let client = FailoverClient::new(_base_url.clone()).await.expect("Failed to create client");
  ```

- Line 364: `Failed to insert reflection entry`
  ```rust
  .expect("Failed to insert reflection entry");
  ```

- Line 367: `Failed to spawn agent`
  ```rust
  let _spawn_resp = client.spawn_agent().await.expect("Failed to spawn agent");
  ```

- Line 371: `Failed to switch adapter`
  ```rust
  .expect("Failed to switch adapter");
  ```

- Line 375: `Failed to list reflection entries`
  ```rust
  .expect("Failed to list reflection entries");
  ```

- Line 386: `Entry should exist`
  ```rust
  .expect("Entry should exist");
  ```

- Line 399: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 401: `Failed to create client`
  ```rust
  let client = FailoverClient::new(_base_url.clone()).await.expect("Failed to create client");
  ```

- Line 404: `Failed to spawn agent`
  ```rust
  let spawn_resp = client.spawn_agent().await.expect("Failed to spawn agent");
  ```

- Line 408: `Should have session_db_id`
  ```rust
  .expect("Should have session_db_id");
  ```

- Line 414: `Failed to switch adapter`
  ```rust
  .expect("Failed to switch adapter");
  ```

- Line 420: `Failed to switch adapter back`
  ```rust
  .expect("Failed to switch adapter back");
  ```

- Line 424: `Should have second session_db_id`
  ```rust
  .expect("Should have second session_db_id");
  ```

- Line 430: `Failed to list sessions`
  ```rust
  .expect("Failed to list sessions");
  ```

- Line 447: `Should find first archived session`
  ```rust
  .expect("Should find first archived session");
  ```

- Line 451: `Should find second archived session`
  ```rust
  .expect("Should find second archived session");
  ```

- Line 477: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 479: `Failed to create client`
  ```rust
  let client = FailoverClient::new(_base_url.clone()).await.expect("Failed to create client");
  ```

- Line 482: `Failed to spawn agent`
  ```rust
  let _spawn_resp = client.spawn_agent().await.expect("Failed to spawn agent");
  ```

- Line 503: `Failed to insert reflection entry`
  ```rust
  .expect("Failed to insert reflection entry");
  ```

- Line 509: `Failed to switch adapter`
  ```rust
  .expect("Failed to switch adapter");
  ```

- Line 515: `Failed to get agent status`
  ```rust
  .expect("Failed to get agent status");
  ```

- Line 521: `Failed to list reflection entries`
  ```rust
  .expect("Failed to list reflection entries");
  ```

- Line 536: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 538: `Failed to create client`
  ```rust
  let client = FailoverClient::new(_base_url.clone()).await.expect("Failed to create client");
  ```

- Line 541: `Failed to spawn agent`
  ```rust
  let _spawn_resp = client.spawn_agent().await.expect("Failed to spawn agent");
  ```

- Line 568: `Switch 1 should complete`
  ```rust
  .expect("Switch 1 should complete");
  ```

- Line 571: `Switch 2 should complete`
  ```rust
  .expect("Switch 2 should complete");
  ```

- Line 580: `Health check failed`
  ```rust
  let health = client.healthz().await.expect("Health check failed");
  ```

- Line 597: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 599: `Failed to create client`
  ```rust
  let client = FailoverClient::new(base_url.clone()).await.expect("Failed to create client");
  ```

- Line 602: `Failed to spawn agent`
  ```rust
  let spawn_resp = client.spawn_agent().await.expect("Failed to spawn agent");
  ```

- Line 607: `Should have session_db_id`
  ```rust
  .expect("Should have session_db_id");
  ```

- Line 613: `Failed to get agent status`
  ```rust
  .expect("Failed to get agent status");
  ```

- Line 639: `Failed to write updated config.yml`
  ```rust
  .expect("Failed to write updated config.yml");
  ```

- Line 650: `Failed to get agent status after config reload`
  ```rust
  .expect("Failed to get agent status after config reload");
  ```

- Line 663: `Failed to list sessions`
  ```rust
  .expect("Failed to list sessions");
  ```

- Line 683: `Should find original archived session`
  ```rust
  .expect("Should find original archived session");
  ```

- Line 700: `Failed to query stitch from fleet.db`
  ```rust
  .expect("Failed to query stitch from fleet.db");
  ```

- Line 722: `Health check failed`
  ```rust
  let health = client.healthz().await.expect("Health check failed");
  ```

- Line 808: `Failed to start mock Anthropic server`
  ```rust
  .expect("Failed to start mock Anthropic server");
  ```

- Line 815: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 817: `Failed to create client`
  ```rust
  let client = FailoverClient::new(_base_url.clone()).await.expect("Failed to create client");
  ```

- Line 820: `Health check failed`
  ```rust
  let health = client.healthz().await.expect("Health check failed");
  ```

- Line 840: `Failed to write config with mock server URL`
  ```rust
  .expect("Failed to write config with mock server URL");
  ```

- Line 856: `Health check failed`
  ```rust
  let health_after = client.healthz().await.expect("Health check failed");
  ```

- Line 868: `Ready endpoint request failed`
  ```rust
  .expect("Ready endpoint request failed");
  ```

- Line 885: `Health check failed`
  ```rust
  let health = client.healthz().await.expect("Health check failed");
  ```

- Line 895: `Health check failed`
  ```rust
  let final_health = client.healthz().await.expect("Health check failed");
  ```

- Line 913: `Failed to start mock Anthropic server`
  ```rust
  .expect("Failed to start mock Anthropic server");
  ```

- Line 920: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 922: `Failed to create client`
  ```rust
  let client = FailoverClient::new(_base_url.clone()).await.expect("Failed to create client");
  ```

- Line 925: `Health check failed`
  ```rust
  let health = client.healthz().await.expect("Health check failed");
  ```

- Line 940: `Failed to write config`
  ```rust
  std::fs::write(&config_path, mock_config).expect("Failed to write config");
  ```

- Line 946: `Health check failed`
  ```rust
  let health_after_503 = client.healthz().await.expect("Health check failed");
  ```

- Line 953: `Adapter switch should succeed`
  ```rust
  .expect("Adapter switch should succeed");
  ```

- Line 961: `Failed to get agent status`
  ```rust
  .expect("Failed to get agent status");
  ```

- Line 966: `Health check failed`
  ```rust
  let final_health = client.healthz().await.expect("Health check failed");
  ```

#### unwrap (4 occurrences)

- Line 307: `\.unwrap\(\)`
  ```rust
  let stitch_row_opt = fleet::load_stitch_by_id(stitch_id.as_ref().unwrap())
  ```

- Line 315: `\.unwrap\(\)`
  ```rust
  let stitch_row = stitch_row_opt.unwrap();
  ```

- Line 699: `\.unwrap\(\)`
  ```rust
  let stitch_row_opt = fleet::load_stitch_by_id(stitch_id.as_ref().unwrap())
  ```

- Line 707: `\.unwrap\(\)`
  ```rust
  let stitch_row = stitch_row_opt.unwrap();
  ```

### hoop-daemon/tests_phase5/agent_turn_audit_trail.rs

Total errors: 30

#### assert (1 occurrences)

- Line 167: `System message should reference the turn_id`
  ```rust
  assert!(message_content.contains(turn_id), "System message should reference the turn_id");
  ```

#### assert_eq (7 occurrences)

- Line 142: `created_by_actor should be set`
  ```rust
  assert_eq!(stitch_row.1, Some(actor.to_string()), "created_by_actor should be set");
  ```

- Line 143: `created_by_session_id should be set`
  ```rust
  assert_eq!(stitch_row.2, Some(session_id.to_string()), "created_by_session_id should be set");
  ```

- Line 144: `created_by_adapter should be set`
  ```rust
  assert_eq!(stitch_row.3, Some(adapter.to_string()), "created_by_adapter should be set");
  ```

- Line 145: `created_by_model should be set`
  ```rust
  assert_eq!(stitch_row.4, Some(model.to_string()), "created_by_model should be set");
  ```

- Line 146: `turn_id should be set`
  ```rust
  assert_eq!(stitch_row.5, Some(turn_id.to_string()), "turn_id should be set");
  ```

- Line 157: `Should have one system note with turn reference`
  ```rust
  assert_eq!(message_count, 1, "Should have one system note with turn reference");
  ```

- Line 332: `session_id, turn_id));
`
  ```rust
  assert_eq!(turn_url, format!("/agent?session={}&turn={}", session_id, turn_id));
  ```

#### expect (18 occurrences)

- Line 25: `create temp dir`
  ```rust
  let tmp = TempDir::new().expect("create temp dir");
  ```

- Line 27: `create .hoop dir`
  ```rust
  std::fs::create_dir_all(&hoop_dir).expect("create .hoop dir");
  ```

- Line 33: `init fleet.db`
  ```rust
  hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");
  ```

- Line 83: `insert draft`
  ```rust
  hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");
  ```

- Line 87: `get draft`
  ```rust
  .expect("get draft")
  ```

- Line 88: `draft exists`
  ```rust
  .expect("draft exists");
  ```

- Line 117: `create stitch with audit`
  ```rust
  .expect("create stitch with audit");
  ```

- Line 121: `open fleet.db`
  ```rust
  .expect("open fleet.db");
  ```

- Line 139: `query stitch`
  ```rust
  .expect("query stitch");
  ```

- Line 155: `count system messages`
  ```rust
  .expect("count system messages");
  ```

- Line 165: `get system message content`
  ```rust
  .expect("get system message content");
  ```

- Line 211: `write audit row`
  ```rust
  .expect("write audit row");
  ```

- Line 215: `query audit rows`
  ```rust
  .expect("query audit rows");
  ```

- Line 220: `should find audit row for our stitch`
  ```rust
  .expect("should find audit row for our stitch");
  ```

- Line 230: `args_json should be valid JSON`
  ```rust
  .expect("args_json should be valid JSON");
  ```

- Line 298: `create stitch for reconstruction`
  ```rust
  .expect("create stitch for reconstruction");
  ```

- Line 302: `open fleet.db`
  ```rust
  .expect("open fleet.db");
  ```

- Line 317: `query stitch for reconstruction`
  ```rust
  .expect("query stitch for reconstruction");
  ```

#### unwrap (4 occurrences)

- Line 23: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 40: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 229: `\.unwrap\(\)`
  ```rust
  .map(|s| serde_json::from_str(s).unwrap())
  ```

- Line 254: `\.unwrap\(\)`
  ```rust
  let extracted_session_id = actor.strip_prefix("hoop:agent:").unwrap();
  ```

### hoop-daemon/tests_phase5/reflection_detector_integration.rs

Total errors: 63

#### assert (3 occurrences)

- Line 168: `run_detection should succeed`
  ```rust
  assert!(result.is_ok(), "run_detection should succeed");
  ```

- Line 554: `build_reflection_rules_with_audit should succeed`
  ```rust
  assert!(result.is_ok(), "build_reflection_rules_with_audit should succeed");
  ```

- Line 605: `last_applied should be set`
  ```rust
  assert!(last_applied.is_some(), "last_applied should be set");
  ```

#### assert_eq (11 occurrences)

- Line 171: `Should propose 1 pattern from 3 similar negatives`
  ```rust
  assert_eq!(proposed, 1, "Should propose 1 pattern from 3 similar negatives");
  ```

- Line 186: `Should have 1 reflection ledger entry`
  ```rust
  assert_eq!(entries.len(), 1, "Should have 1 reflection ledger entry");
  ```

- Line 196: `Should have 3 source stitches`
  ```rust
  assert_eq!(source_stitches.len(), 3, "Should have 3 source stitches");
  ```

- Line 235: `Should propose 1 preference pattern`
  ```rust
  assert_eq!(proposed, 1, "Should propose 1 preference pattern");
  ```

- Line 273: `Should propose 1 correction pattern`
  ```rust
  assert_eq!(proposed, 1, "Should propose 1 correction pattern");
  ```

- Line 326: `Should not propose patterns: worker stitches ignored, operator below threshold`
  ```rust
  assert_eq!(proposed, 0, "Should not propose patterns: worker stitches ignored, operator below threshold");
  ```

- Line 446: `Should not propose patterns: old stitches outside window`
  ```rust
  assert_eq!(proposed, 0, "Should not propose patterns: old stitches outside window");
  ```

- Line 572: `Should have 2 audit rows, one per injected rule`
  ```rust
  assert_eq!(audit_rows.len(), 2, "Should have 2 audit rows, one per injected rule");
  ```

- Line 606: `applied_count should be 1 after injection`
  ```rust
  assert_eq!(applied_count, 1, "applied_count should be 1 after injection");
  ```

- Line 624: `applied_count should be 2 after second injection`
  ```rust
  assert_eq!(applied_count, 2, "applied_count should be 2 after second injection");
  ```

- Line 633: `Should have 4 audit rows total (2 per injection)`
  ```rust
  assert_eq!(count, 4, "Should have 4 audit rows total (2 per injection)");
  ```

#### unwrap (49 occurrences)

- Line 17: `\.unwrap\(\)`
  ```rust
  let conn = Connection::open(&db_path).unwrap();
  ```

- Line 36: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 47: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 57: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 72: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 77: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 82: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 104: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 113: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 119: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 170: `\.unwrap\(\)`
  ```rust
  let proposed = result.unwrap();
  ```

- Line 176: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 182: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 195: `\.unwrap\(\)`
  ```rust
  let source_stitches: Vec<String> = serde_json::from_str(source_stitches_json).unwrap();
  ```

- Line 205: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 234: `\.unwrap\(\)`
  ```rust
  let proposed = result.unwrap();
  ```

- Line 243: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 272: `\.unwrap\(\)`
  ```rust
  let proposed = result.unwrap();
  ```

- Line 281: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 325: `\.unwrap\(\)`
  ```rust
  let proposed = result.unwrap();
  ```

- Line 337: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 383: `\.unwrap\(\)`
  ```rust
  let proposed = result.unwrap();
  ```

- Line 393: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 398: `\.unwrap\(\)`
  ```rust
  Ok(serde_json::from_str::<Vec<String>>(&json).unwrap())
  ```

- Line 400: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 416: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 445: `\.unwrap\(\)`
  ```rust
  let proposed = result.unwrap();
  ```

- Line 454: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 469: `\.unwrap\(\)`
  ```rust
  hoop_daemon::fleet::is_operator_stitch("st-operator-1").unwrap(),
  ```

- Line 473: `\.unwrap\(\)`
  ```rust
  !hoop_daemon::fleet::is_operator_stitch("st-worker-1").unwrap(),
  ```

- Line 477: `\.unwrap\(\)`
  ```rust
  !hoop_daemon::fleet::is_operator_stitch("st-fleet-operator").unwrap(),
  ```

- Line 481: `\.unwrap\(\)`
  ```rust
  !hoop_daemon::fleet::is_operator_stitch("st-nonexistent").unwrap(),
  ```

- Line 496: `\.unwrap\(\)`
  ```rust
  let temp_dir = TempDir::new().unwrap();
  ```

- Line 508: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 514: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 535: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 545: `\.unwrap\(\)`
  ```rust
  ).unwrap();
  ```

- Line 555: `\.unwrap\(\)`
  ```rust
  let rules_string = result.unwrap();
  ```

- Line 562: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 568: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 578: `\.unwrap\(\)`
  ```rust
  let args: serde_json::Value = serde_json::from_str(args_json).unwrap();
  ```

- Line 582: `\.unwrap\(\)`
  ```rust
  let rule_id = args["rule_id"].as_str().unwrap();
  ```

- Line 592: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 598: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 615: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 619: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 630: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 632: `\.unwrap\(\)`
  ```rust
  let count: i64 = audit_stmt2.query_row([], |row| row.get(0)).unwrap();
  ```

- Line 643: `\.unwrap\(\)`
  ```rust
  write!(&mut result, "{:02x}", byte).unwrap();
  ```

### hoop-mcp/tests/create_only_stub.rs

Total errors: 11

#### assert (1 occurrences)

- Line 97: `fake br should succeed`
  ```rust
  assert!(output.status.success(), "fake br should succeed");
  ```

#### assert_eq (2 occurrences)

- Line 137: `expected 3 invocations, got {:?}`
  ```rust
  assert_eq!(verbs.len(), 3, "expected 3 invocations, got {:?}", verbs);
  ```

- Line 275: `expected 3 invocations, got {:?}`
  ```rust
  assert_eq!(verbs.len(), 3, "expected 3 invocations, got {:?}", verbs);
  ```

#### expect (6 occurrences)

- Line 20: `create temp dir`
  ```rust
  let bin_dir = tempfile::TempDir::new().expect("create temp dir");
  ```

- Line 37: `create br script`
  ```rust
  let mut f = fs::File::create(&br_path).expect("create br script");
  ```

- Line 38: `write br script`
  ```rust
  f.write_all(script.as_bytes()).expect("write br script");
  ```

- Line 43: `chmod br script`
  ```rust
  .expect("chmod br script");
  ```

- Line 96: `run fake br`
  ```rust
  let output = cmd.output().expect("run fake br");
  ```

- Line 266: `run fake br`
  ```rust
  let output = cmd.output().expect("run fake br");
  ```

#### unwrap (2 occurrences)

- Line 24: `\.unwrap\(\)`
  ```rust
  let log_path_str = log_path.to_str().unwrap();
  ```

- Line 50: `\.unwrap\(\)`
  ```rust
  self.bin_dir.path().to_str().unwrap().to_string()
  ```

### hoop-mcp/tests/forbidden_worker_steering.rs

Total errors: 3

#### expect (2 occurrences)

- Line 116: `Failed to create McpServerState for test`
  ```rust
  .expect("Failed to create McpServerState for test");
  ```

- Line 149: `Failed to create McpServerState for test`
  ```rust
  .expect("Failed to create McpServerState for test");
  ```

#### unwrap (1 occurrences)

- Line 153: `\.unwrap\(\)`
  ```rust
  json!({"message": "test"}).as_object().unwrap().clone();
  ```

### hoop-mcp/tests/protocol_contract.rs

Total errors: 42

#### assert (1 occurrences)

- Line 586: `fixture {} must be a JSON object`
  ```rust
  assert!(val.is_object(), "fixture {} must be a JSON object", path);
  ```

#### expect (7 occurrences)

- Line 22: `workspace root`
  ```rust
  .expect("workspace root")
  ```

- Line 43: `JsonRpcRequest must deserialize from initialize fixture`
  ```rust
  .expect("JsonRpcRequest must deserialize from initialize fixture");
  ```

- Line 74: `JsonRpcRequest must deserialize from tools/list fixture`
  ```rust
  .expect("JsonRpcRequest must deserialize from tools/list fixture");
  ```

- Line 145: `JsonRpcRequest must deserialize from prompts/list fixture`
  ```rust
  .expect("JsonRpcRequest must deserialize from prompts/list fixture");
  ```

- Line 197: `JsonRpcRequest must deserialize from resources/list fixture`
  ```rust
  .expect("JsonRpcRequest must deserialize from resources/list fixture");
  ```

- Line 249: `JsonRpcRequest must deserialize from shutdown fixture`
  ```rust
  .expect("JsonRpcRequest must deserialize from shutdown fixture");
  ```

- Line 376: `JsonRpcRequest must deserialize from tools_call fixture`
  ```rust
  .expect("JsonRpcRequest must deserialize from tools_call fixture");
  ```

#### panic (8 occurrences)

- Line 26: `fixture file missing: {}`
  ```rust
  .unwrap_or_else(|_| panic!("fixture file missing: {}", path.display()));
  ```

- Line 28: `invalid JSON in fixture {}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("invalid JSON in fixture {}: {}", path.display(), e))
  ```

- Line 58: `expected Method::Initialize`
  ```rust
  _ => panic!("expected Method::Initialize"),
  ```

- Line 80: `expected Method::ToolsList`
  ```rust
  _ => panic!("expected Method::ToolsList"),
  ```

- Line 151: `expected Method::PromptsList`
  ```rust
  _ => panic!("expected Method::PromptsList"),
  ```

- Line 203: `expected Method::ResourcesList`
  ```rust
  _ => panic!("expected Method::ResourcesList"),
  ```

- Line 255: `expected Method::Shutdown`
  ```rust
  _ => panic!("expected Method::Shutdown"),
  ```

- Line 396: `expected Method::ToolsCall`
  ```rust
  _ => panic!("expected Method::ToolsCall"),
  ```

#### unwrap (26 occurrences)

- Line 47: `\.unwrap\(\)`
  ```rust
  let expected_version = fixture["params"]["protocol_version"].as_str().unwrap();
  ```

- Line 50: `\.unwrap\(\)`
  ```rust
  let expected_name = fixture["params"]["client_info"]["name"].as_str().unwrap();
  ```

- Line 55: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 110: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 120: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&resp).unwrap();
  ```

- Line 172: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&resp).unwrap();
  ```

- Line 224: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&resp).unwrap();
  ```

- Line 276: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&resp).unwrap();
  ```

- Line 305: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 317: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 321: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 328: `\.unwrap\(\)`
  ```rust
  serde_json::to_value(result).unwrap(),
  ```

- Line 331: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&resp).unwrap();
  ```

- Line 340: `\.unwrap\(\)`
  ```rust
  for key in fixture_result.as_object().unwrap().keys() {
  ```

- Line 351: `\.unwrap\(\)`
  ```rust
  for key in fixture_server.as_object().unwrap().keys() {
  ```

- Line 380: `\.unwrap\(\)`
  ```rust
  let expected_name = fixture["params"]["name"].as_str().unwrap();
  ```

- Line 384: `\.unwrap\(\)`
  ```rust
  let fixture_params = fixture["params"].as_object().unwrap();
  ```

- Line 416: `\.unwrap\(\)`
  ```rust
  .unwrap()
  ```

- Line 426: `\.unwrap\(\)`
  ```rust
  serde_json::to_value(result).unwrap(),
  ```

- Line 429: `\.unwrap\(\)`
  ```rust
  let serialized = serde_json::to_value(&resp).unwrap();
  ```

- Line 440: `\.unwrap\(\)`
  ```rust
  let serialized_content = serialized_result["content"].as_array().unwrap();
  ```

- Line 477: `\.unwrap\(\)`
  ```rust
  let project = fixture["project"].as_str().unwrap();
  ```

- Line 478: `\.unwrap\(\)`
  ```rust
  let title = fixture["title"].as_str().unwrap();
  ```

- Line 480: `\.unwrap\(\)`
  ```rust
  let kind = fixture["kind"].as_str().unwrap();
  ```

- Line 496: `\.unwrap\(\)`
  ```rust
  for (key, expected_val) in fixture.as_object().unwrap() {
  ```

- Line 550: `\.unwrap\(\)`
  ```rust
  if let Some(msg) = fixture["messages"].as_array().unwrap().first() {
  ```

### hoop-mcp/tests/socket_permissions.rs

Total errors: 8

#### expect (8 occurrences)

- Line 14: `temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("temp dir");
  ```

- Line 18: `bind socket`
  ```rust
  let listener = std::os::unix::net::UnixListener::bind(&socket_path).expect("bind socket");
  ```

- Line 21: `set permissions`
  ```rust
  fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600)).expect("set permissions");
  ```

- Line 24: `metadata`
  ```rust
  let metadata = fs::metadata(&socket_path).expect("metadata");
  ```

- Line 127: `temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("temp dir");
  ```

- Line 130: `bind socket`
  ```rust
  let _listener = std::os::unix::net::UnixListener::bind(&socket_path).expect("bind socket");
  ```

- Line 132: `set permissions`
  ```rust
  fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600)).expect("set permissions");
  ```

- Line 135: `metadata`
  ```rust
  let metadata = fs::metadata(&socket_path).expect("metadata");
  ```

### hoop-schema/tests/schema_drift.rs

Total errors: 86

#### expect (4 occurrences)

- Line 756: `Failed to create fixture directory`
  ```rust
  .expect("Failed to create fixture directory");
  ```

- Line 778: `Failed to write index`
  ```rust
  .expect("Failed to write index");
  ```

- Line 885: `Failed to read index.json`
  ```rust
  let index_content = fs::read_to_string(&index_path).expect("Failed to read index.json");
  ```

- Line 887: `Failed to parse index.json`
  ```rust
  serde_json::from_str(&index_content).expect("Failed to parse index.json");
  ```

#### panic (4 occurrences)

- Line 860: `Failed to read fixture {}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", fixture_file, e));
  ```

- Line 864: `Failed to parse fixture {} as JSON: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to parse fixture {} as JSON: {}", fixture_file, e));
  ```

- Line 868: `Failed to serialize fixture {}: {}`
  ```rust
  .unwrap_or_else(|e| panic!("Failed to serialize fixture {}: {}", fixture_file, e));
  ```

- Line 872: `Failed to parse normalized JSON for {}: {}`
  ```rust
  panic!("Failed to parse normalized JSON for {}: {}", fixture_file, e)
  ```

#### unwrap (78 occurrences)

- Line 23: `\.unwrap\(\)`
  ```rust
  let ts: DateTime<Utc> = "2024-01-01T00:00:00Z".parse().unwrap();
  ```

- Line 24: `\.unwrap\(\)`
  ```rust
  let uuid = uuid::Uuid::parse_str("01234567-89ab-cdef-0123-456789abcdef").unwrap();
  ```

- Line 30: `\.unwrap\(\)`
  ```rust
  serde_json::to_string_pretty(&WorkerLiveness::Live).unwrap(),
  ```

- Line 42: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 51: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 69: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 86: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 101: `\.unwrap\(\)`
  ```rust
  schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
  ```

- Line 103: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 114: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 124: `\.unwrap\(\)`
  ```rust
  }).unwrap(),
  ```

- Line 134: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 162: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 181: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 191: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 214: `\.unwrap\(\)`
  ```rust
  schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
  ```

- Line 216: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 227: `\.unwrap\(\)`
  ```rust
  schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
  ```

- Line 229: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 240: `\.unwrap\(\)`
  ```rust
  schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
  ```

- Line 242: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 255: `\.unwrap\(\)`
  ```rust
  schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
  ```

- Line 257: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 262: `\.unwrap\(\)`
  ```rust
  schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
  ```

- Line 268: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 288: `\.unwrap\(\)`
  ```rust
  schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
  ```

- Line 290: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 299: `\.unwrap\(\)`
  ```rust
  schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
  ```

- Line 301: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 309: `\.unwrap\(\)`
  ```rust
  schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
  ```

- Line 311: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 330: `\.unwrap\(\)`
  ```rust
  schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
  ```

- Line 332: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 339: `\.unwrap\(\)`
  ```rust
  schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
  ```

- Line 357: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 372: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 384: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 393: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 402: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 409: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 418: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 426: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 435: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 440: `\.unwrap\(\)`
  ```rust
  idle_timeout_secs: std::num::NonZero::new(180).unwrap(),
  ```

- Line 441: `\.unwrap\(\)`
  ```rust
  max_runtime_secs: std::num::NonZero::new(3600).unwrap(),
  ```

- Line 442: `\.unwrap\(\)`
  ```rust
  content_seen_grace_secs: std::num::NonZero::new(600).unwrap(),
  ```

- Line 443: `\.unwrap\(\)`
  ```rust
  heartbeat_transition_threshold_secs: std::num::NonZero::new(300).unwrap(),
  ```

- Line 444: `\.unwrap\(\)`
  ```rust
  retry_threshold: std::num::NonZero::new(3).unwrap(),
  ```

- Line 446: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 457: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 469: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 476: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 491: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 504: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 533: `\.unwrap\(\)`
  ```rust
  schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
  ```

- Line 535: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 555: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 570: `\.unwrap\(\)`
  ```rust
  hash_prev: "0".repeat(64).parse().unwrap(),
  ```

- Line 571: `\.unwrap\(\)`
  ```rust
  hash_self: "0".repeat(64).parse().unwrap(),
  ```

- Line 574: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 604: `\.unwrap\(\)`
  ```rust
  schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
  ```

- Line 606: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 623: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 632: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 647: `\.unwrap\(\)`
  ```rust
  schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
  ```

- Line 651: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 669: `\.unwrap\(\)`
  ```rust
  schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
  ```

- Line 671: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 679: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 694: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 704: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 715: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 730: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 738: `\.unwrap\(\)`
  ```rust
  schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
  ```

- Line 740: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 748: `\.unwrap\(\)`
  ```rust
  schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
  ```

- Line 750: `\.unwrap\(\)`
  ```rust
  .unwrap(),
  ```

- Line 777: `\.unwrap\(\)`
  ```rust
  fs::write(&index_path, serde_json::to_string_pretty(&index).unwrap())
  ```

### testrepo/tests/integration/test_01.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 01 passed`
  ```rust
  assert!(true, "Integration test 01 passed");
  ```

### testrepo/tests/integration/test_02.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 02 passed`
  ```rust
  assert!(true, "Integration test 02 passed");
  ```

### testrepo/tests/integration/test_03.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 03 passed`
  ```rust
  assert!(true, "Integration test 03 passed");
  ```

### testrepo/tests/integration/test_04.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 04 passed`
  ```rust
  assert!(true, "Integration test 04 passed");
  ```

### testrepo/tests/integration/test_05.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 05 passed`
  ```rust
  assert!(true, "Integration test 05 passed");
  ```

### testrepo/tests/integration/test_06.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 06 passed`
  ```rust
  assert!(true, "Integration test 06 passed");
  ```

### testrepo/tests/integration/test_07.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 07 passed`
  ```rust
  assert!(true, "Integration test 07 passed");
  ```

### testrepo/tests/integration/test_08.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 08 passed`
  ```rust
  assert!(true, "Integration test 08 passed");
  ```

### testrepo/tests/integration/test_09.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 09 passed`
  ```rust
  assert!(true, "Integration test 09 passed");
  ```

### testrepo/tests/integration/test_10.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 10 passed`
  ```rust
  assert!(true, "Integration test 10 passed");
  ```

### testrepo/tests/integration/test_11.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 11 passed`
  ```rust
  assert!(true, "Integration test 11 passed");
  ```

### testrepo/tests/integration/test_12.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 12 passed`
  ```rust
  assert!(true, "Integration test 12 passed");
  ```

### testrepo/tests/integration/test_13.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 13 passed`
  ```rust
  assert!(true, "Integration test 13 passed");
  ```

### testrepo/tests/integration/test_14.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 14 passed`
  ```rust
  assert!(true, "Integration test 14 passed");
  ```

### testrepo/tests/integration/test_15.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 15 passed`
  ```rust
  assert!(true, "Integration test 15 passed");
  ```

### testrepo/tests/integration/test_16.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 16 passed`
  ```rust
  assert!(true, "Integration test 16 passed");
  ```

### testrepo/tests/integration/test_17.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 17 passed`
  ```rust
  assert!(true, "Integration test 17 passed");
  ```

### testrepo/tests/integration/test_18.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 18 passed`
  ```rust
  assert!(true, "Integration test 18 passed");
  ```

### testrepo/tests/integration/test_19.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 19 passed`
  ```rust
  assert!(true, "Integration test 19 passed");
  ```

### testrepo/tests/integration/test_20.rs

Total errors: 1

#### assert (1 occurrences)

- Line 5: `Integration test 20 passed`
  ```rust
  assert!(true, "Integration test 20 passed");
  ```

### tests/acceptance/s1_morning_review.rs

Total errors: 35

#### anyhow (1 occurrences)

- Line 96: `Daemon failed to start within timeout`
  ```rust
  Err(anyhow::anyhow!("Daemon failed to start within timeout"))
  ```

#### assert (2 occurrences)

- Line 132: `total_spend_usd must be non-negative`
  ```rust
  assert!(total_cost >= 0.0, "total_spend_usd must be non-negative");
  ```

- Line 270: `Total cost must be non-negative`
  ```rust
  assert!(total_cost >= 0.0, "Total cost must be non-negative");
  ```

#### assert_eq (3 occurrences)

- Line 113: `Dashboard endpoint should return 200`
  ```rust
  assert_eq!(resp.status(), 200, "Dashboard endpoint should return 200");
  ```

- Line 148: `Worker timeline endpoint should return 200`
  ```rust
  assert_eq!(resp.status(), 200, "Worker timeline endpoint should return 200");
  ```

- Line 175: `Dashboard should return 200`
  ```rust
  assert_eq!(resp.status(), 200, "Dashboard should return 200");
  ```

#### expect (29 occurrences)

- Line 34: `workspace root`
  ```rust
  .expect("workspace root");
  ```

- Line 103: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 111: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 115: `Failed to parse dashboard`
  ```rust
  let dashboard: JsonValue = resp.json().await.expect("Failed to parse dashboard");
  ```

- Line 123: `total_workers must be a number`
  ```rust
  .expect("total_workers must be a number");
  ```

- Line 131: `total_spend_usd must be a number`
  ```rust
  .expect("total_spend_usd must be a number");
  ```

- Line 140: `longest_running must be an array`
  ```rust
  .expect("longest_running must be an array");
  ```

- Line 146: `Failed to fetch worker timeline`
  ```rust
  .expect("Failed to fetch worker timeline");
  ```

- Line 150: `Failed to parse timeline`
  ```rust
  let _timeline: JsonValue = resp.json().await.expect("Failed to parse timeline");
  ```

- Line 161: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 171: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 190: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 198: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 206: `Failed to parse response`
  ```rust
  let dashboard: JsonValue = resp.json().await.expect("Failed to parse response");
  ```

- Line 223: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 231: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 233: `Failed to parse response`
  ```rust
  let dashboard1: JsonValue = resp1.json().await.expect("Failed to parse response");
  ```

- Line 241: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 243: `Failed to parse response`
  ```rust
  let dashboard2: JsonValue = resp2.json().await.expect("Failed to parse response");
  ```

- Line 254: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 262: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 264: `Failed to parse response`
  ```rust
  let dashboard: JsonValue = resp.json().await.expect("Failed to parse response");
  ```

- Line 268: `total_spend_usd must be present`
  ```rust
  .expect("total_spend_usd must be present");
  ```

- Line 274: `spend_by_project must be an array`
  ```rust
  .expect("spend_by_project must be an array");
  ```

- Line 295: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 303: `Failed to fetch dashboard`
  ```rust
  .expect("Failed to fetch dashboard");
  ```

- Line 305: `Failed to parse response`
  ```rust
  let dashboard: JsonValue = resp.json().await.expect("Failed to parse response");
  ```

- Line 309: `total_workers must be present`
  ```rust
  .expect("total_workers must be present");
  ```

- Line 313: `workers_by_project must be an array`
  ```rust
  .expect("workers_by_project must be an array");
  ```

### tests/acceptance/s2_transcript_archaeology.rs

Total errors: 38

#### anyhow (1 occurrences)

- Line 97: `Daemon failed to start within timeout`
  ```rust
  Err(anyhow::anyhow!("Daemon failed to start within timeout"))
  ```

#### assert (3 occurrences)

- Line 137: `Events should be an array`
  ```rust
  assert!(events.is_array(), "Events should be an array");
  ```

- Line 265: `Conversations should be an array`
  ```rust
  assert!(conversations.is_array(), "Conversations should be an array");
  ```

- Line 314: `Cost data should be an object`
  ```rust
  assert!(cost_data.is_object(), "Cost data should be an object");
  ```

#### assert_eq (3 occurrences)

- Line 114: `Beads endpoint should return 200`
  ```rust
  assert_eq!(resp.status(), 200, "Beads endpoint should return 200");
  ```

- Line 261: `Conversations endpoint should return 200`
  ```rust
  assert_eq!(resp.status(), 200, "Conversations endpoint should return 200");
  ```

- Line 310: `Cost trends endpoint should return 200`
  ```rust
  assert_eq!(resp.status(), 200, "Cost trends endpoint should return 200");
  ```

#### expect (31 occurrences)

- Line 35: `workspace root`
  ```rust
  .expect("workspace root");
  ```

- Line 104: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 112: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 116: `Failed to parse beads`
  ```rust
  let beads: JsonValue = resp.json().await.expect("Failed to parse beads");
  ```

- Line 122: `Bead should have an id`
  ```rust
  .expect("Bead should have an id");
  ```

- Line 128: `Failed to fetch bead events`
  ```rust
  .expect("Failed to fetch bead events");
  ```

- Line 136: `Failed to parse events`
  ```rust
  let events: JsonValue = resp.json().await.expect("Failed to parse events");
  ```

- Line 152: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 160: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 162: `Failed to parse beads`
  ```rust
  let beads: JsonValue = resp.json().await.expect("Failed to parse beads");
  ```

- Line 168: `Bead should have an id`
  ```rust
  .expect("Bead should have an id");
  ```

- Line 176: `Failed to fetch bead events`
  ```rust
  .expect("Failed to fetch bead events");
  ```

- Line 197: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 205: `Failed to connect to stitch endpoint`
  ```rust
  .expect("Failed to connect to stitch endpoint");
  ```

- Line 219: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 235: `Failed to connect to endpoint`
  ```rust
  .expect("Failed to connect to endpoint");
  ```

- Line 251: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 259: `Failed to fetch conversations`
  ```rust
  .expect("Failed to fetch conversations");
  ```

- Line 263: `Failed to parse conversations`
  ```rust
  let conversations: JsonValue = resp.json().await.expect("Failed to parse conversations");
  ```

- Line 274: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 282: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 284: `Failed to parse beads`
  ```rust
  let beads: JsonValue = resp.json().await.expect("Failed to parse beads");
  ```

- Line 300: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 308: `Failed to fetch cost trends`
  ```rust
  .expect("Failed to fetch cost trends");
  ```

- Line 312: `Failed to parse cost data`
  ```rust
  let cost_data: JsonValue = resp.json().await.expect("Failed to parse cost data");
  ```

- Line 323: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 331: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 333: `Failed to parse beads`
  ```rust
  let beads: JsonValue = resp.json().await.expect("Failed to parse beads");
  ```

- Line 339: `Bead should have an id`
  ```rust
  .expect("Bead should have an id");
  ```

- Line 345: `Failed to fetch bead events`
  ```rust
  .expect("Failed to fetch bead events");
  ```

- Line 348: `Failed to parse events`
  ```rust
  let events: JsonValue = resp.json().await.expect("Failed to parse events");
  ```

### tests/acceptance/s3_bead_creation_from_chat.rs

Total errors: 24

#### anyhow (1 occurrences)

- Line 99: `Daemon failed to start within timeout`
  ```rust
  Err(anyhow::anyhow!("Daemon failed to start within timeout"))
  ```

#### assert (1 occurrences)

- Line 328: `Draft should appear in queue`
  ```rust
  assert!(found, "Draft should appear in queue");
  ```

#### assert_eq (1 occurrences)

- Line 194: `Bead list endpoint should return 200`
  ```rust
  assert_eq!(resp.status(), 200, "Bead list endpoint should return 200");
  ```

#### expect (21 occurrences)

- Line 37: `workspace root`
  ```rust
  .expect("workspace root");
  ```

- Line 106: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 124: `Failed to create draft`
  ```rust
  .expect("Failed to create draft");
  ```

- Line 140: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 148: `Failed to fetch drafts`
  ```rust
  .expect("Failed to fetch drafts");
  ```

- Line 162: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 170: `Failed to fetch audit log`
  ```rust
  .expect("Failed to fetch audit log");
  ```

- Line 184: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 192: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 205: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 225: `Failed to create draft`
  ```rust
  .expect("Failed to create draft");
  ```

- Line 228: `Failed to parse draft`
  ```rust
  let draft: JsonValue = resp.json().await.expect("Failed to parse draft");
  ```

- Line 248: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 256: `Failed to fetch audit log`
  ```rust
  .expect("Failed to fetch audit log");
  ```

- Line 259: `Failed to parse audit`
  ```rust
  let audit: JsonValue = resp.json().await.expect("Failed to parse audit");
  ```

- Line 279: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 298: `Failed to create draft`
  ```rust
  .expect("Failed to create draft");
  ```

- Line 302: `Failed to parse draft`
  ```rust
  let create_response: JsonValue = create_resp.json().await.expect("Failed to parse draft");
  ```

- Line 306: `draft_id should be present`
  ```rust
  .expect("draft_id should be present");
  ```

- Line 320: `Failed to list drafts`
  ```rust
  .expect("Failed to list drafts");
  ```

- Line 323: `Failed to parse list`
  ```rust
  let list_response: JsonValue = list_resp.json().await.expect("Failed to parse list");
  ```

### tests/acceptance/s4_daemon_restart.rs

Total errors: 46

#### anyhow (1 occurrences)

- Line 177: `Daemon failed to start`
  ```rust
  Err(anyhow::anyhow!("Daemon failed to start"))
  ```

#### assert (1 occurrences)

- Line 219: `Worker should have written events`
  ```rust
  assert!(mid_event_count > 0, "Worker should have written events");
  ```

#### assert_eq (4 occurrences)

- Line 207: `First daemon should return beads`
  ```rust
  assert_eq!(resp1.status(), 200, "First daemon should return beads");
  ```

- Line 231: `Second daemon should return beads`
  ```rust
  assert_eq!(resp2.status(), 200, "Second daemon should return beads");
  ```

- Line 339: `Should see all beads`
  ```rust
  assert_eq!(resp.status(), 200, "Should see all beads");
  ```

- Line 387: `Should fetch beads in cycle {}`
  ```rust
  assert_eq!(resp.status(), 200, "Should fetch beads in cycle {}", cycle);
  ```

#### expect (35 occurrences)

- Line 31: `workspace root`
  ```rust
  .expect("workspace root")
  ```

- Line 105: `create temp dir`
  ```rust
  let temp_dir = TempDir::new().expect("create temp dir");
  ```

- Line 107: `create .hoop dir`
  ```rust
  fs::create_dir_all(&hoop_dir).expect("create .hoop dir");
  ```

- Line 122: `write projects.yaml`
  ```rust
  .expect("write projects.yaml");
  ```

- Line 131: `write config.yml`
  ```rust
  .expect("write config.yml");
  ```

- Line 132: `create data dir`
  ```rust
  fs::create_dir_all(hoop_dir.join("data")).expect("create data dir");
  ```

- Line 191: `write claim`
  ```rust
  worker.write_claim("bd-001").expect("write claim");
  ```

- Line 192: `write complete`
  ```rust
  worker.write_complete("bd-001").expect("write complete");
  ```

- Line 193: `write claim`
  ```rust
  worker.write_claim("bd-002").expect("write claim");
  ```

- Line 197: `Failed to spawn first daemon`
  ```rust
  .expect("Failed to spawn first daemon");
  ```

- Line 205: `Failed to fetch beads from first daemon`
  ```rust
  .expect("Failed to fetch beads from first daemon");
  ```

- Line 209: `Failed to parse beads`
  ```rust
  let beads1: serde_json::Value = resp1.json().await.expect("Failed to parse beads");
  ```

- Line 215: `write complete`
  ```rust
  worker.write_complete("bd-002").expect("write complete");
  ```

- Line 216: `write claim`
  ```rust
  worker.write_claim("bd-003").expect("write claim");
  ```

- Line 223: `Failed to spawn second daemon`
  ```rust
  .expect("Failed to spawn second daemon");
  ```

- Line 229: `Failed to fetch beads from second daemon`
  ```rust
  .expect("Failed to fetch beads from second daemon");
  ```

- Line 233: `Failed to parse beads`
  ```rust
  let beads2: serde_json::Value = resp2.json().await.expect("Failed to parse beads");
  ```

- Line 263: `write claim`
  ```rust
  worker.write_claim(&bead_id).expect("write claim");
  ```

- Line 265: `write complete`
  ```rust
  worker.write_complete(&bead_id).expect("write complete");
  ```

- Line 271: `Failed to spawn first daemon`
  ```rust
  .expect("Failed to spawn first daemon");
  ```

- Line 277: `Failed to spawn second daemon`
  ```rust
  .expect("Failed to spawn second daemon");
  ```

- Line 302: `Failed to spawn first daemon`
  ```rust
  .expect("Failed to spawn first daemon");
  ```

- Line 307: `write claim`
  ```rust
  worker.write_claim("bd-restart-1").expect("write claim");
  ```

- Line 308: `write complete`
  ```rust
  worker.write_complete("bd-restart-1").expect("write complete");
  ```

- Line 309: `write claim`
  ```rust
  worker.write_claim("bd-restart-2").expect("write claim");
  ```

- Line 320: `Failed to spawn second daemon`
  ```rust
  .expect("Failed to spawn second daemon");
  ```

- Line 322: `write complete`
  ```rust
  worker.write_complete("bd-restart-2").expect("write complete");
  ```

- Line 323: `write claim`
  ```rust
  worker.write_claim("bd-restart-3").expect("write claim");
  ```

- Line 337: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 358: `write claim`
  ```rust
  worker.write_claim("bd-s4-1").expect("write claim");
  ```

- Line 359: `write complete`
  ```rust
  worker.write_complete("bd-s4-1").expect("write complete");
  ```

- Line 364: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 385: `Failed to fetch beads`
  ```rust
  .expect("Failed to fetch beads");
  ```

- Line 389: `Failed to parse beads`
  ```rust
  let beads: serde_json::Value = resp.json().await.expect("Failed to parse beads");
  ```

- Line 408: `write claim`
  ```rust
  worker.write_claim(&format!("bd-s4-{}", cycle * 10 + 2)).expect("write claim");
  ```

#### unwrap (5 occurrences)

- Line 103: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 182: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 252: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 292: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

- Line 346: `\.unwrap\(\)`
  ```rust
  let _guard = LOCK.lock().unwrap();
  ```

### tests/acceptance/s5_workspace_deleted.rs

Total errors: 38

#### anyhow (1 occurrences)

- Line 130: `Daemon failed to start`
  ```rust
  Err(anyhow::anyhow!("Daemon failed to start"))
  ```

#### assert (1 occurrences)

- Line 265: `Daemon should still be healthy`
  ```rust
  assert!(resp.status().is_success(), "Daemon should still be healthy");
  ```

#### assert_eq (2 occurrences)

- Line 175: `Initial readyz should return 200`
  ```rust
  assert_eq!(status, 200, "Initial readyz should return 200");
  ```

- Line 249: `Projects endpoint should still work`
  ```rust
  assert_eq!(resp.status(), 200, "Projects endpoint should still work");
  ```

#### expect (22 occurrences)

- Line 27: `Failed to create .beads dir`
  ```rust
  fs::create_dir_all(&beads_dir).expect("Failed to create .beads dir");
  ```

- Line 29: `Failed to create issues.jsonl`
  ```rust
  fs::write(&issues_path, b"").expect("Failed to create issues.jsonl");
  ```

- Line 37: `Failed to create temp dir`
  ```rust
  let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
  ```

- Line 39: `Failed to create .hoop dir`
  ```rust
  fs::create_dir_all(&hoop_dir).expect("Failed to create .hoop dir");
  ```

- Line 68: `Failed to write projects.yaml`
  ```rust
  .expect("Failed to write projects.yaml");
  ```

- Line 77: `Failed to write config.yml`
  ```rust
  .expect("Failed to write config.yml");
  ```

- Line 79: `Failed to create data dir`
  ```rust
  .expect("Failed to create data dir");
  ```

- Line 166: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 173: `Failed to get readyz status`
  ```rust
  .expect("Failed to get readyz status");
  ```

- Line 179: `Failed to remove .beads from project A`
  ```rust
  fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads from project A");
  ```

- Line 228: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 237: `Failed to remove .beads from project A`
  ```rust
  fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads from project A");
  ```

- Line 247: `Failed to fetch projects`
  ```rust
  .expect("Failed to fetch projects");
  ```

- Line 251: `Failed to parse projects`
  ```rust
  let projects: serde_json::Value = resp.json().await.expect("Failed to parse projects");
  ```

- Line 263: `Failed to check health`
  ```rust
  .expect("Failed to check health");
  ```

- Line 291: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 299: `Failed to get readyz status`
  ```rust
  .expect("Failed to get readyz status");
  ```

- Line 304: `Failed to remove .beads from project A`
  ```rust
  fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads from project A");
  ```

- Line 311: `Failed to get readyz status after deletion`
  ```rust
  .expect("Failed to get readyz status after deletion");
  ```

- Line 364: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 373: `Failed to remove .beads`
  ```rust
  fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads");
  ```

- Line 382: `Failed to check health`
  ```rust
  .expect("Failed to check health");
  ```

#### unwrap (12 occurrences)

- Line 148: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 152: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 156: `\.unwrap\(\)`
  ```rust
  let project_c_dir = tempfile::tempdir().unwrap();
  ```

- Line 210: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 214: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 218: `\.unwrap\(\)`
  ```rust
  let project_c_dir = tempfile::tempdir().unwrap();
  ```

- Line 273: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 277: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 281: `\.unwrap\(\)`
  ```rust
  let project_c_dir = tempfile::tempdir().unwrap();
  ```

- Line 346: `\.unwrap\(\)`
  ```rust
  let project_a_dir = tempfile::tempdir().unwrap();
  ```

- Line 350: `\.unwrap\(\)`
  ```rust
  let project_b_dir = tempfile::tempdir().unwrap();
  ```

- Line 354: `\.unwrap\(\)`
  ```rust
  let project_c_dir = tempfile::tempdir().unwrap();
  ```

### tests/acceptance/s6_machine_mode.rs

Total errors: 31

#### anyhow (1 occurrences)

- Line 98: `Daemon failed to start within timeout`
  ```rust
  Err(anyhow::anyhow!("Daemon failed to start within timeout"))
  ```

#### assert (4 occurrences)

- Line 122: `Status should be a JSON object`
  ```rust
  assert!(status.is_object(), "Status should be a JSON object");
  ```

- Line 166: `Projects should be an array`
  ```rust
  assert!(projects.is_array(), "Projects should be an array");
  ```

- Line 223: `Should be parseable by jq`
  ```rust
  assert!(projects.is_array(), "Should be parseable by jq");
  ```

- Line 227: `Each project should be an object`
  ```rust
  assert!(project.is_object(), "Each project should be an object");
  ```

#### assert_eq (4 occurrences)

- Line 118: `Status endpoint should return 200`
  ```rust
  assert_eq!(resp.status(), 200, "Status endpoint should return 200");
  ```

- Line 141: `Projects endpoint should return 200`
  ```rust
  assert_eq!(resp.status(), 200, "Projects endpoint should return 200");
  ```

- Line 252: `Healthz endpoint should return 200`
  ```rust
  assert_eq!(resp.status(), 200, "Healthz endpoint should return 200");
  ```

- Line 337: `All concurrent requests should succeed`
  ```rust
  assert_eq!(success_count, 10, "All concurrent requests should succeed");
  ```

#### expect (22 occurrences)

- Line 36: `workspace root`
  ```rust
  .expect("workspace root");
  ```

- Line 107: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 116: `Failed to fetch status`
  ```rust
  .expect("Failed to fetch status");
  ```

- Line 120: `Failed to parse status`
  ```rust
  let status: JsonValue = resp.json().await.expect("Failed to parse status");
  ```

- Line 131: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 139: `Failed to fetch projects`
  ```rust
  .expect("Failed to fetch projects");
  ```

- Line 152: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 160: `Failed to fetch projects`
  ```rust
  .expect("Failed to fetch projects");
  ```

- Line 164: `Failed to parse projects`
  ```rust
  let projects: JsonValue = resp.json().await.expect("Failed to parse projects");
  ```

- Line 175: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 192: `Failed to fetch endpoint`
  ```rust
  .expect("Failed to fetch endpoint");
  ```

- Line 210: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 218: `Failed to fetch projects`
  ```rust
  .expect("Failed to fetch projects");
  ```

- Line 220: `Failed to parse projects`
  ```rust
  let projects: JsonValue = resp.json().await.expect("Failed to parse projects");
  ```

- Line 242: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 250: `Failed to fetch healthz`
  ```rust
  .expect("Failed to fetch healthz");
  ```

- Line 261: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 269: `Failed to fetch readyz`
  ```rust
  .expect("Failed to fetch readyz");
  ```

- Line 284: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 293: `Failed to fetch bead`
  ```rust
  .expect("Failed to fetch bead");
  ```

- Line 310: `Failed to spawn daemon`
  ```rust
  .expect("Failed to spawn daemon");
  ```

- Line 327: `Task panicked`
  ```rust
  let result = handle.await.expect("Task panicked");
  ```

### tests/cli_test_helpers.rs

Total errors: 159

#### assert (29 occurrences)

- Line 338: `Flag must be true after extraction`
  ```rust
  assert!(no_interactive, "Flag must be true after extraction");
  ```

- Line 390: `CLI must parse flag as true`
  ```rust
  assert!(cli.no_interactive, "CLI must parse flag as true");
  ```

- Line 498: `Parent must have flag set`
  ```rust
  assert!(parent_cli.no_interactive, "Parent must have flag set");
  ```

- Line 571: `Top level must have flag`
  ```rust
  assert!(cli.no_interactive, "Top level must have flag");
  ```

- Line 577: `Flag accessible at Projects level`
  ```rust
  assert!(cli.no_interactive, "Flag accessible at Projects level");
  ```

- Line 583: `Confirm flag must be true`
  ```rust
  assert!(confirm, "Confirm flag must be true");
  ```

- Line 585: `Flag accessible at Remove level`
  ```rust
  assert!(cli.no_interactive, "Flag accessible at Remove level");
  ```

- Line 646: `Level 0: Global flag must be true`
  ```rust
  assert!(cli.no_interactive, "Level 0: Global flag must be true");
  ```

- Line 652: `Level 1: Flag accessible in Projects`
  ```rust
  assert!(cli.no_interactive, "Level 1: Flag accessible in Projects");
  ```

- Line 658: `Remove`
  ```rust
  assert!(*confirm, "Remove's --confirm flag must be true");
  ```

- Line 661: `Level 2: Flag accessible in Remove`
  ```rust
  assert!(cli.no_interactive, "Level 2: Flag accessible in Remove");
  ```

- Line 760: `Flag must be parsed as true`
  ```rust
  assert!(cli.no_interactive, "Flag must be parsed as true");
  ```

- Line 790: `Flag must be false when not specified`
  ```rust
  assert!(!cli_no_flag.no_interactive, "Flag must be false when not specified");
  ```

- Line 822: `Flag must be true at top level`
  ```rust
  assert!(cli.no_interactive, "Flag must be true at top level");
  ```

- Line 827: `Flag accessible at Projects level`
  ```rust
  assert!(cli.no_interactive, "Flag accessible at Projects level");
  ```

- Line 836: `Child must receive no_interactive flag`
  ```rust
  assert!(child_has_flag, "Child must receive no_interactive flag");
  ```

- Line 960: `Parent must have flag set`
  ```rust
  assert!(parent_cli.no_interactive, "Parent must have flag set");
  ```

- Line 995: `Top level must have flag`
  ```rust
  assert!(cli.no_interactive, "Top level must have flag");
  ```

- Line 1001: `Flag accessible at Projects level`
  ```rust
  assert!(cli.no_interactive, "Flag accessible at Projects level");
  ```

- Line 1008: `Flag accessible at Remove level`
  ```rust
  assert!(cli.no_interactive, "Flag accessible at Remove level");
  ```

- Line 1069: `Level 0: Global flag must be true`
  ```rust
  assert!(cli.no_interactive, "Level 0: Global flag must be true");
  ```

- Line 1075: `Level 1: Flag accessible in Projects`
  ```rust
  assert!(cli.no_interactive, "Level 1: Flag accessible in Projects");
  ```

- Line 1081: `Remove`
  ```rust
  assert!(*confirm, "Remove's --confirm flag must be true");
  ```

- Line 1084: `Level 2: Flag accessible in Remove`
  ```rust
  assert!(cli.no_interactive, "Level 2: Flag accessible in Remove");
  ```

- Line 1168: `Flag must be parsed as true`
  ```rust
  assert!(cli.no_interactive, "Flag must be parsed as true");
  ```

- Line 1198: `Flag must be false when not specified`
  ```rust
  assert!(!cli_no_flag.no_interactive, "Flag must be false when not specified");
  ```

- Line 1230: `Flag must be true at top level`
  ```rust
  assert!(cli.no_interactive, "Flag must be true at top level");
  ```

- Line 1235: `Flag accessible at Projects level`
  ```rust
  assert!(cli.no_interactive, "Flag accessible at Projects level");
  ```

- Line 1244: `Child must receive no_interactive flag`
  ```rust
  assert!(child_has_flag, "Child must receive no_interactive flag");
  ```

#### assert_eq (27 occurrences)

- Line 180: `Both positions must yield the same value`
  ```rust
  //!     assert_eq!(before, after, "Both positions must yield the same value");
  ```

- Line 181: `no_interactive should be true`
  ```rust
  //!     assert_eq!(before, true, "no_interactive should be true");
  ```

- Line 204: `no_interactive value must be consistent`
  ```rust
  //!     assert_eq!(before, after, "no_interactive value must be consistent");
  ```

- Line 205: `no_interactive should be true`
  ```rust
  //!     assert_eq!(before, true, "no_interactive should be true");
  ```

- Line 394: `Extracted value must match CLI value`
  ```rust
  assert_eq!(no_interactive, true, "Extracted value must match CLI value");
  ```

- Line 520: `
`
  ```rust
  assert_eq!(child_cli.no_interactive, child_cli_flag_at_end.no_interactive,
  ```

- Line 670: `
`
  ```rust
  assert_eq!(cli.no_interactive, true,
  ```

- Line 711: `
`
  ```rust
  assert_eq!(env.get("HOOP_NO_INTERACTIVE"), Some(&"1".to_string()),
  ```

- Line 840: `Environment variable must be `
  ```rust
  assert_eq!(env_value, "1", "Environment variable must be '1'");
  ```

- Line 1093: `
`
  ```rust
  assert_eq!(cli.no_interactive, true,
  ```

- Line 1119: `
`
  ```rust
  assert_eq!(env.get("HOOP_NO_INTERACTIVE"), Some(&"1".to_string()),
  ```

- Line 1248: `Environment variable must be `
  ```rust
  assert_eq!(env_value, "1", "Environment variable must be '1'");
  ```

- Line 1338: `flag value must be position-independent`
  ```rust
  /// assert_eq!(before, after, "flag value must be position-independent");
  ```

- Line 1779: `no_interactive value must be consistent`
  ```rust
  assert_eq!(before, after, "no_interactive value must be consistent");
  ```

- Line 1780: `no_interactive should be true`
  ```rust
  assert_eq!(before, true, "no_interactive should be true");
  ```

- Line 1837: `no_interactive value must be position-independent`
  ```rust
  assert_eq!(before, after, "no_interactive value must be position-independent");
  ```

- Line 1838: `no_interactive should be true`
  ```rust
  assert_eq!(before, true, "no_interactive should be true");
  ```

- Line 1896: `
`
  ```rust
  assert_eq!(cli.no_interactive, true,
  ```

- Line 1906: `
`
  ```rust
  assert_eq!(cli.no_interactive, true,
  ```

- Line 1957: `
`
  ```rust
  assert_eq!(cli.no_interactive, true,
  ```

- Line 1967: `
`
  ```rust
  assert_eq!(cli.no_interactive, true,
  ```

- Line 2032: `
`
  ```rust
  assert_eq!(result, $expected,
  ```

- Line 2051: `
`
  ```rust
  assert_eq!(result, $expected,
  ```

- Line 2077: `
`
  ```rust
  assert_eq!(cli_global.no_interactive, cli_subcommand.no_interactive,
  ```

- Line 2082: `
`
  ```rust
  assert_eq!(cli_global.no_interactive, $consistency,
  ```

- Line 2216: `Values must match`
  ```rust
  assert_eq!(before, after, "Values must match");
  ```

- Line 2357: `
`
  ```rust
  assert_eq!(cli.no_interactive, true,
  ```

#### expect (2 occurrences)

- Line 398: `projects.rs must exist`
  ```rust
  .expect("projects.rs must exist");
  ```

- Line 412: `main.rs must exist`
  ```rust
  .expect("main.rs must exist");
  ```

#### panic (16 occurrences)

- Line 81: `Expected Scan command`
  ```rust
  //!         _ => panic!("Expected Scan command"),
  ```

- Line 132: `Expected Scan command`
  ```rust
  //!         _ => panic!("Expected Scan command"),
  ```

- Line 587: `Expected Remove command`
  ```rust
  _ => panic!("Expected Remove command"),
  ```

- Line 590: `Expected Projects command`
  ```rust
  _ => panic!("Expected Projects command"),
  ```

- Line 663: `Expected Remove command at Level 2`
  ```rust
  _ => panic!("Expected Remove command at Level 2"),
  ```

- Line 666: `Expected Projects command at Level 1`
  ```rust
  _ => panic!("Expected Projects command at Level 1"),
  ```

- Line 830: `Expected Projects command`
  ```rust
  _ => panic!("Expected Projects command"),
  ```

- Line 1010: `Expected Remove command`
  ```rust
  _ => panic!("Expected Remove command"),
  ```

- Line 1013: `Expected Projects command`
  ```rust
  _ => panic!("Expected Projects command"),
  ```

- Line 1086: `Expected Remove command at Level 2`
  ```rust
  _ => panic!("Expected Remove command at Level 2"),
  ```

- Line 1089: `Expected Projects command at Level 1`
  ```rust
  _ => panic!("Expected Projects command at Level 1"),
  ```

- Line 1238: `Expected Projects command`
  ```rust
  _ => panic!("Expected Projects command"),
  ```

- Line 2178: `Expected Scan command`
  ```rust
  _ => panic!("Expected Scan command"),
  ```

- Line 2192: `Expected Remove command`
  ```rust
  _ => panic!("Expected Remove command"),
  ```

- Line 2206: `Expected Projects subcommand`
  ```rust
  _ => panic!("Expected Projects subcommand"),
  ```

- Line 2368: `Expected Projects command`
  ```rust
  _ => panic!("Expected Projects command"),
  ```

#### should_panic (3 occurrences)

- Line 2125: `expected = "no_interactive should be true"`
  ```rust
  #[should_panic(expected = "no_interactive should be true")]
  ```

- Line 2140: `expected = "no_interactive should be false"`
  ```rust
  #[should_panic(expected = "no_interactive should be false")]
  ```

- Line 2156: `expected = "no_interactive value must be consistent"`
  ```rust
  #[should_panic(expected = "no_interactive value must be consistent")]
  ```

#### unwrap (82 occurrences)

- Line 47: `\.unwrap\(\)`
  ```rust
  //!     let cli = parse_cli_args(&args).unwrap();
  ```

- Line 73: `\.unwrap\(\)`
  ```rust
  //!     let cli = parse_cli_args(&args).unwrap();
  ```

- Line 98: `\.unwrap\(\)`
  ```rust
  //!     let cli = parse_cli_args(&args).unwrap();
  ```

- Line 124: `\.unwrap\(\)`
  ```rust
  //!     let cli = parse_cli_args(&args).unwrap();
  ```

- Line 157: `\.unwrap\(\)`
  ```rust
  //!     let cli_before = parse_cli_args(&args_before).unwrap();
  ```

- Line 161: `\.unwrap\(\)`
  ```rust
  //!     let cli_after = parse_cli_args(&args_after).unwrap();
  ```

- Line 192: `\.unwrap\(\)`
  ```rust
  //!     let cli = parse_cli_args(&args).unwrap();
  ```

- Line 214: `\.unwrap\(\)`
  ```rust
  //!     let cli = parse_cli_args(&args).unwrap();
  ```

- Line 332: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 353: `\.unwrap\(\)`
  ```rust
  let code = std::fs::read_to_string("src/projects.rs").unwrap();
  ```

- Line 362: `\.unwrap\(\)`
  ```rust
  let main_code = std::fs::read_to_string("src/main.rs").unwrap();
  ```

- Line 389: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 449: `\.unwrap\(\)`
  ```rust
  let parent_cli = parse_cli_args(&parent_args).unwrap();
  ```

- Line 469: `\.unwrap\(\)`
  ```rust
  let child_cli = parse_cli_args(&child_args).unwrap();
  ```

- Line 497: `\.unwrap\(\)`
  ```rust
  let parent_cli = parse_cli_args(&parent_args).unwrap();
  ```

- Line 509: `\.unwrap\(\)`
  ```rust
  let child_cli = parse_cli_args(&child_args).unwrap();
  ```

- Line 519: `\.unwrap\(\)`
  ```rust
  let child_cli_flag_at_end = parse_cli_args(&child_args_flag_at_end).unwrap();
  ```

- Line 568: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 614: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 643: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 700: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 737: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 759: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 789: `\.unwrap\(\)`
  ```rust
  let cli_no_flag = parse_cli_args(&args_no_flag).unwrap();
  ```

- Line 819: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 843: `\.unwrap\(\)`
  ```rust
  let projects_code = std::fs::read_to_string("src/projects.rs").unwrap();
  ```

- Line 881: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 911: `\.unwrap\(\)`
  ```rust
  let parent_cli = parse_cli_args(&parent_args).unwrap();
  ```

- Line 931: `\.unwrap\(\)`
  ```rust
  let child_cli = parse_cli_args(&child_args).unwrap();
  ```

- Line 959: `\.unwrap\(\)`
  ```rust
  let parent_cli = parse_cli_args(&parent_args).unwrap();
  ```

- Line 970: `\.unwrap\(\)`
  ```rust
  let child_cli = parse_cli_args(&child_args).unwrap();
  ```

- Line 992: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 1037: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 1066: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 1110: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 1145: `\.unwrap\(\)`
  ```rust
  .unwrap();
  ```

- Line 1167: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 1197: `\.unwrap\(\)`
  ```rust
  let cli_no_flag = parse_cli_args(&args_no_flag).unwrap();
  ```

- Line 1227: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 1279: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 1300: `\.unwrap\(\)`
  ```rust
  /// let cli = parse_cli_args(&args).unwrap();
  ```

- Line 1314: `\.unwrap\(\)`
  ```rust
  /// let cli = parse_cmd_string("hoop --no-interactive scan /tmp").unwrap();
  ```

- Line 1348: `\.unwrap\(\)`
  ```rust
  let cli_before = parse_cli_args(&full_args_before).unwrap();
  ```

- Line 1358: `\.unwrap\(\)`
  ```rust
  let cli_after = parse_cli_args(&full_args_after).unwrap();
  ```

- Line 1477: `\.unwrap\(\)`
  ```rust
  let cli_before = parse_cli_args(&args_before).unwrap();
  ```

- Line 1489: `\.unwrap\(\)`
  ```rust
  let cli_after = parse_cli_args(&args_after).unwrap();
  ```

- Line 1513: `\.unwrap\(\)`
  ```rust
  let cli_before = parse_cli_args(&args_before).unwrap();
  ```

- Line 1518: `\.unwrap\(\)`
  ```rust
  let cli_after = parse_cli_args(&args_after).unwrap();
  ```

- Line 1576: `\.unwrap\(\)`
  ```rust
  let parent_cli = parse_cli_args(&$parent_args).unwrap();
  ```

- Line 1671: `\.unwrap\(\)`
  ```rust
  let parent_cli = parse_cli_args(&$parent_args).unwrap();
  ```

- Line 1725: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&$args).unwrap();
  ```

- Line 1760: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args($args).unwrap();
  ```

- Line 1769: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args($args).unwrap();
  ```

- Line 1788: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args($args).unwrap();
  ```

- Line 1807: `\.unwrap\(\)`
  ```rust
  let cli = parse_cmd_string(&args).unwrap();
  ```

- Line 1814: `\.unwrap\(\)`
  ```rust
  let cli = parse_cmd_string(&args).unwrap();
  ```

- Line 1821: `\.unwrap\(\)`
  ```rust
  let cli = parse_cmd_string(&args).unwrap();
  ```

- Line 1828: `\.unwrap\(\)`
  ```rust
  let cli = parse_cmd_string(&args).unwrap();
  ```

- Line 1844: `\.unwrap\(\)`
  ```rust
  let cli = parse_cmd_string(&args).unwrap();
  ```

- Line 1883: `\.unwrap\(\)`
  ```rust
  ///     let cli = parse_cli_args(&args).unwrap();
  ```

- Line 1894: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 1904: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 1944: `\.unwrap\(\)`
  ```rust
  ///     let cli = parse_cli_args(&args).unwrap();
  ```

- Line 1955: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 1965: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 2029: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 2048: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 2066: `\.unwrap\(\)`
  ```rust
  let cli_global = parse_cli_args(&args_global).unwrap();
  ```

- Line 2075: `\.unwrap\(\)`
  ```rust
  let cli_subcommand = parse_cli_args(&args_subcommand).unwrap();
  ```

- Line 2097: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 2103: `\.unwrap\(\)`
  ```rust
  let cli = parse_cmd_string("hoop --no-interactive scan /tmp").unwrap();
  ```

- Line 2120: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 2128: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 2135: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 2143: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 2171: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 2185: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 2199: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&args).unwrap();
  ```

- Line 2225: `\.unwrap\(\)`
  ```rust
  let cli_long = parse_cli_args(&args_long).unwrap();
  ```

- Line 2226: `\.unwrap\(\)`
  ```rust
  let cli_short = parse_cli_args(&args_short).unwrap();
  ```

- Line 2321: `\.unwrap\(\)`
  ```rust
  let parent_cli = parse_cli_args(&parent_args).unwrap();
  ```

- Line 2354: `\.unwrap\(\)`
  ```rust
  let cli = parse_cli_args(&full_args).unwrap();
  ```

