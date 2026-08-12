# HOOP Test Assertion Error Messages

**Extracted:** 2026-08-12T13:53:14.241582Z
**Files processed:** 220
**Total findings:** 2462

This is a comprehensive inventory of assertion error messages in HOOP test files.

---

## Summary by Pattern Type

- **unwrap**: 1476 occurrences
- **assert**: 499 occurrences
- **assert_eq**: 367 occurrences
- **panic**: 94 occurrences
- **unwrap_err**: 25 occurrences
- **assert_ne**: 1 occurrences

---

## unwrap patterns (1476 occurrences)

### 1. hoop-daemon/tests/projection_file_audit.rs:305

**Pattern:** `unwrap`

**Line:**
```rust
let code = r#"std::fs::write("fleet_status.json", &encoded).unwrap();"#;
```

### 2. hoop-daemon/tests/projection_file_audit.rs:318

**Pattern:** `unwrap`

**Line:**
```rust
let code = r#"std::fs::write("live-workers.json", &bytes).unwrap();"#;
```

### 3. hoop-daemon/tests/projection_file_audit.rs:331

**Pattern:** `unwrap`

**Line:**
```rust
let code = r#"std::fs::File::create("fleet_state.json").unwrap();"#;
```

### 4. hoop-daemon/tests/projection_file_audit.rs:344

**Pattern:** `unwrap`

**Line:**
```rust
let code = r#"std::fs::write("fleet_state.yaml", &serialized).unwrap();"#;
```

### 5. hoop-daemon/tests/projection_file_audit.rs:357

**Pattern:** `unwrap`

**Line:**
```rust
let code = r#"std::fs::write("live-fleet.json", data).unwrap();"#;
```

### 6. hoop-daemon/tests/projection_file_audit.rs:423

**Pattern:** `unwrap`

**Line:**
```rust
let code = r#"std::fs::write("worker_status.json", &data).unwrap();"#;
```

### 7. hoop-daemon/tests/backup_restore_cycle.rs:21

**Pattern:** `unwrap`

**Line:**
```rust
let test_dir = TempDir::new().unwrap();
```

### 8. hoop-daemon/tests/backup_restore_cycle.rs:23

**Pattern:** `unwrap`

**Line:**
```rust
fs::create_dir_all(&hoop_dir).unwrap();
```

### 9. hoop-daemon/tests/backup_restore_cycle.rs:35

**Pattern:** `unwrap`

**Line:**
```rust
fs::create_dir_all(&snapshot_dir).unwrap();
```

### 10. hoop-daemon/tests/backup_restore_cycle.rs:42

**Pattern:** `unwrap`

**Line:**
```rust
.unwrap();
```

### 11. hoop-daemon/tests/backup_restore_cycle.rs:48

**Pattern:** `unwrap`

**Line:**
```rust
copy_dir_recursive(&attachments_src, &attachments_dst).unwrap();
```

### 12. hoop-daemon/tests/backup_restore_cycle.rs:56

**Pattern:** `unwrap`

**Line:**
```rust
.unwrap();
```

### 13. hoop-daemon/tests/backup_restore_cycle.rs:61

**Pattern:** `unwrap`

**Line:**
```rust
.unwrap();
```

### 14. hoop-daemon/tests/backup_restore_cycle.rs:64

**Pattern:** `unwrap`

**Line:**
```rust
fs::remove_dir_all(&hoop_dir).unwrap();
```

### 15. hoop-daemon/tests/backup_restore_cycle.rs:70

**Pattern:** `unwrap`

**Line:**
```rust
fs::create_dir_all(&hoop_dir).unwrap();
```

### 16. hoop-daemon/tests/backup_restore_cycle.rs:71

**Pattern:** `unwrap`

**Line:**
```rust
fs::copy(snapshot_dir.join("fleet.db"), hoop_dir.join("fleet.db")).unwrap();
```

### 17. hoop-daemon/tests/backup_restore_cycle.rs:76

**Pattern:** `unwrap`

**Line:**
```rust
copy_dir_recursive(&attachments_src, &attachments_dst).unwrap();
```

### 18. hoop-daemon/tests/backup_restore_cycle.rs:83

**Pattern:** `unwrap`

**Line:**
```rust
.unwrap();
```

### 19. hoop-daemon/tests/backup_restore_cycle.rs:88

**Pattern:** `unwrap`

**Line:**
```rust
.unwrap();
```

### 20. hoop-daemon/tests/backup_restore_cycle.rs:155

**Pattern:** `unwrap`

**Line:**
```rust
let creds = creds.unwrap();
```

... and 1456 more

---

## assert patterns (499 occurrences)

### 1. hoop-daemon/tests/backup_restore_cycle.rs:67

**Pattern:** `assert`

**Error message:** `State should be deleted`

**Line:**
```rust
assert!(!hoop_dir.exists(), "State should be deleted");
```

### 2. hoop-daemon/tests/backup_restore_cycle.rs:144

**Pattern:** `assert`

**Error message:** `Should return None when credentials missing`

**Line:**
```rust
assert!(creds.is_none(), "Should return None when credentials missing");
```

### 3. hoop-daemon/tests/backup_restore_cycle.rs:153

**Pattern:** `assert`

**Error message:** `Should succeed when encryption disabled`

**Line:**
```rust
assert!(creds.is_some(), "Should succeed when encryption disabled");
```

### 4. hoop-daemon/tests/backup_restore_cycle.rs:158

**Pattern:** `assert`

**Error message:** `age_key should be None when encryption disabled`

**Line:**
```rust
assert!(creds.age_key.is_none(), "age_key should be None when encryption disabled");
```

### 5. hoop-daemon/tests/backup_restore_cycle.rs:166

**Pattern:** `assert`

**Error message:** `Should succeed when age key provided`

**Line:**
```rust
assert!(creds.is_some(), "Should succeed when age key provided");
```

### 6. hoop-daemon/tests/backup_restore_cycle.rs:169

**Pattern:** `assert`

**Error message:** `age_key should be Some when encryption enabled`

**Line:**
```rust
assert!(creds.age_key.is_some(), "age_key should be Some when encryption enabled");
```

### 7. hoop-daemon/tests/backup_restore_cycle.rs:178

**Pattern:** `assert`

**Error message:** `Should return None when age key missing but encryption enabled`

**Line:**
```rust
assert!(creds.is_none(), "Should return None when age key missing but encryption enabled");
```

### 8. hoop-daemon/tests/backup_restore_cycle.rs:229

**Pattern:** `assert`

**Error message:** `Encrypted file should exist`

**Line:**
```rust
assert!(encrypted_file.exists(), "Encrypted file should exist");
```

### 9. hoop-daemon/tests/backup_restore_cycle.rs:311

**Pattern:** `assert`

**Error message:** `Backup should fail when encryption enabled but age key missing`

**Line:**
```rust
assert!(result.is_err(), "Backup should fail when encryption enabled but age key missing");
```

### 10. hoop-daemon/tests/backup_restore_cycle.rs:350

**Pattern:** `assert`

**Error message:** `Config should have encryption enabled`

**Line:**
```rust
assert!(config.encryption, "Config should have encryption enabled");
```

### 11. hoop-daemon/tests/backup_restore_cycle.rs:351

**Pattern:** `assert`

**Error message:** `Credentials should have age key`

**Line:**
```rust
assert!(credentials.age_key.is_some(), "Credentials should have age key");
```

### 12. hoop-daemon/tests/backup_restore_cycle.rs:391

**Pattern:** `assert`

**Error message:** `Config should have encryption disabled`

**Line:**
```rust
assert!(!config.encryption, "Config should have encryption disabled");
```

### 13. hoop-daemon/tests/backup_restore_cycle.rs:392

**Pattern:** `assert`

**Error message:** `Credentials should not have age key`

**Line:**
```rust
assert!(credentials.age_key.is_none(), "Credentials should not have age key");
```

### 14. hoop-daemon/tests/pure_functions.rs:351

**Pattern:** `assert`

**Error message:** `ANSI strip too slow: {:?}`

**Line:**
```rust
assert!(ansi_time.as_millis() < 100, "ANSI strip too slow: {:?}", ansi_time);
```

### 15. hoop-daemon/tests/pure_functions.rs:359

**Pattern:** `assert`

**Error message:** `Cost functions too slow: {:?}`

**Line:**
```rust
assert!(cost_time.as_millis() < 10, "Cost functions too slow: {:?}", cost_time);
```

### 16. hoop-daemon/tests/pure_functions.rs:368

**Pattern:** `assert`

**Error message:** `Embedding too slow: {:?}`

**Line:**
```rust
assert!(embed_time.as_millis() < 500, "Embedding too slow: {:?}", embed_time);
```

### 17. hoop-daemon/tests/pure_functions.rs:376

**Pattern:** `assert`

**Error message:** `Similarity too slow: {:?}`

**Line:**
```rust
assert!(similarity_time.as_millis() < 50, "Similarity too slow: {:?}", similarity_time);
```

### 18. hoop-daemon/tests/pure_functions.rs:398

**Pattern:** `assert`

**Error message:** `Status derivation too slow: {:?}`

**Line:**
```rust
assert!(status_time.as_millis() < 1000, "Status derivation too slow: {:?}", status_time);
```

### 19. hoop-daemon/tests/pure_functions.rs:406

**Pattern:** `assert`

**Error message:** `Tag join too slow: {:?}`

**Line:**
```rust
assert!(tag_time.as_millis() < 100, "Tag join too slow: {:?}", tag_time);
```

### 20. hoop-daemon/tests/pure_functions.rs:415

**Pattern:** `assert`

**Error message:** `Prompt substitute too slow: {:?}`

**Line:**
```rust
assert!(sub_time.as_millis() < 100, "Prompt substitute too slow: {:?}", sub_time);
```

... and 479 more

---

## assert_eq patterns (367 occurrences)

### 1. hoop-daemon/tests/backup_restore_cycle.rs:445

**Pattern:** `assert_eq`

**Error message:** `Cron schedule should have 5 fields`

**Line:**
```rust
assert_eq!(parts.len(), 5, "Cron schedule should have 5 fields");
```

### 2. hoop-daemon/tests/s5_workspace_deleted.rs:168

**Pattern:** `assert_eq`

**Error message:** `Initial readyz should return 200`

**Line:**
```rust
assert_eq!(status, 200, "Initial readyz should return 200");
```

### 3. hoop-daemon/tests/s5_workspace_deleted.rs:169

**Pattern:** `assert_eq`

**Error message:** `Initial readyz status should be ok`

**Line:**
```rust
assert_eq!(readyz.status, "ok", "Initial readyz status should be ok");
```

### 4. hoop-daemon/tests/s5_workspace_deleted.rs:277

**Pattern:** `assert_eq`

**Error message:** `Projects endpoint should still work`

**Line:**
```rust
assert_eq!(resp.status(), 200, "Projects endpoint should still work");
```

### 5. hoop-daemon/tests/pure_functions.rs:193

**Pattern:** `assert_eq`

**Error message:** `world`

**Line:**
```rust
assert_eq!(tokens, vec!["hello", "world", "foo", "bar"]);
```

### 6. hoop-daemon/tests/pure_functions.rs:199

**Pattern:** `assert_eq`

**Error message:** `world`

**Line:**
```rust
assert_eq!(tokens, vec!["hello", "world", "foo", "bar"]);
```

### 7. hoop-daemon/tests/pure_functions.rs:294

**Pattern:** `assert_eq`

**Error message:** `file`

**Line:**
```rust
assert_eq!(vars, vec!["custom", "file", "project"]);
```

### 8. hoop-daemon/tests/config_reload_audit.rs:125

**Pattern:** `assert_eq`

**Error message:** `should find exactly one config_reloaded row`

**Line:**
```rust
assert_eq!(rows.len(), 1, "should find exactly one config_reloaded row");
```

### 9. hoop-daemon/tests/reflection_detector_integration.rs:171

**Pattern:** `assert_eq`

**Error message:** `Should propose 1 pattern from 3 similar negatives`

**Line:**
```rust
assert_eq!(proposed, 1, "Should propose 1 pattern from 3 similar negatives");
```

### 10. hoop-daemon/tests/reflection_detector_integration.rs:186

**Pattern:** `assert_eq`

**Error message:** `Should have 1 reflection ledger entry`

**Line:**
```rust
assert_eq!(entries.len(), 1, "Should have 1 reflection ledger entry");
```

### 11. hoop-daemon/tests/reflection_detector_integration.rs:196

**Pattern:** `assert_eq`

**Error message:** `Should have 3 source stitches`

**Line:**
```rust
assert_eq!(source_stitches.len(), 3, "Should have 3 source stitches");
```

### 12. hoop-daemon/tests/reflection_detector_integration.rs:235

**Pattern:** `assert_eq`

**Error message:** `Should propose 1 preference pattern`

**Line:**
```rust
assert_eq!(proposed, 1, "Should propose 1 preference pattern");
```

### 13. hoop-daemon/tests/reflection_detector_integration.rs:273

**Pattern:** `assert_eq`

**Error message:** `Should propose 1 correction pattern`

**Line:**
```rust
assert_eq!(proposed, 1, "Should propose 1 correction pattern");
```

### 14. hoop-daemon/tests/reflection_detector_integration.rs:326

**Pattern:** `assert_eq`

**Error message:** `Should not propose patterns: worker stitches ignored, operator below threshold`

**Line:**
```rust
assert_eq!(proposed, 0, "Should not propose patterns: worker stitches ignored, operator below threshold");
```

### 15. hoop-daemon/tests/reflection_detector_integration.rs:446

**Pattern:** `assert_eq`

**Error message:** `Should not propose patterns: old stitches outside window`

**Line:**
```rust
assert_eq!(proposed, 0, "Should not propose patterns: old stitches outside window");
```

### 16. hoop-daemon/tests/reflection_detector_integration.rs:572

**Pattern:** `assert_eq`

**Error message:** `Should have 2 audit rows, one per injected rule`

**Line:**
```rust
assert_eq!(audit_rows.len(), 2, "Should have 2 audit rows, one per injected rule");
```

### 17. hoop-daemon/tests/reflection_detector_integration.rs:606

**Pattern:** `assert_eq`

**Error message:** `applied_count should be 1 after injection`

**Line:**
```rust
assert_eq!(applied_count, 1, "applied_count should be 1 after injection");
```

### 18. hoop-daemon/tests/reflection_detector_integration.rs:624

**Pattern:** `assert_eq`

**Error message:** `applied_count should be 2 after second injection`

**Line:**
```rust
assert_eq!(applied_count, 2, "applied_count should be 2 after second injection");
```

### 19. hoop-daemon/tests/reflection_detector_integration.rs:633

**Pattern:** `assert_eq`

**Error message:** `Should have 4 audit rows total (2 per injection)`

**Line:**
```rust
assert_eq!(count, 4, "Should have 4 audit rows total (2 per injection)");
```

### 20. hoop-daemon/tests/quarantine_integration.rs:57

**Pattern:** `assert_eq`

**Error message:** `should parse 3 good lines`

**Line:**
```rust
assert_eq!(good.len(), 3, "should parse 3 good lines");
```

... and 347 more

---

## panic patterns (94 occurrences)

### 1. hoop-daemon/tests/projection_file_audit.rs:229

**Pattern:** `panic`

**Error message:** `failed to read {}: {}`

**Line:**
```rust
Err(e) => panic!("failed to read {}: {}", file.display(), e),
```

### 2. hoop-daemon/tests/projection_file_audit.rs:262

**Pattern:** `panic`

**Error message:** `{}`

**Line:**
```rust
panic!("{}", msg);
```

### 3. hoop-daemon/tests/backup_restore_cycle.rs:641

**Pattern:** `panic`

**Error message:** `age-keygen failed: {}`

**Line:**
```rust
panic!("age-keygen failed: {}", String::from_utf8_lossy(&output.stderr));
```

### 4. hoop-daemon/tests/pure_functions.rs:544

**Pattern:** `panic`

**Error message:** `Expected Quiet with 999 days`

**Line:**
```rust
_ => panic!("Expected Quiet with 999 days"),
```

### 5. hoop-daemon/tests/protocol_contract.rs:28

**Pattern:** `panic`

**Error message:** `fixture file missing: {}`

**Line:**
```rust
.unwrap_or_else(|_| panic!("fixture file missing: {}", path.display()));
```

### 6. hoop-daemon/tests/protocol_contract.rs:30

**Pattern:** `panic`

**Error message:** `invalid JSON in fixture {}: {}`

**Line:**
```rust
.unwrap_or_else(|e| panic!("invalid JSON in fixture {}: {}", path.display(), e))
```

### 7. hoop-daemon/tests/protocol_contract.rs:268

**Pattern:** `panic`

**Error message:** `expected ControlResponse::Status`

**Line:**
```rust
_ => panic!("expected ControlResponse::Status"),
```

### 8. hoop-daemon/tests/protocol_contract.rs:288

**Pattern:** `panic`

**Error message:** `expected ControlResponse::Error`

**Line:**
```rust
_ => panic!("expected ControlResponse::Error"),
```

### 9. hoop-daemon/tests/protocol_contract.rs:654

**Pattern:** `panic`

**Error message:** `fixture {} must deserialize as WsEvent: {}`

**Line:**
```rust
.unwrap_or_else(|e| panic!("fixture {} must deserialize as WsEvent: {}", path, e));
```

### 10. hoop-daemon/tests/integration_harness.rs:815

**Pattern:** `panic`

**Error message:** `Expected text message, got {:?}`

**Line:**
```rust
panic!("Expected text message, got {:?}", init_msg);
```

### 11. hoop-daemon/tests/testrepo_integration.rs:299

**Pattern:** `panic`

**Error message:** `First message must be text, got {:?}`

**Line:**
```rust
panic!("First message must be text, got {:?}", first_msg);
```

### 12. hoop-daemon/tests/per_project_redaction_integration.rs:103

**Pattern:** `panic`

**Error message:** `Expected Variant0 project`

**Line:**
```rust
panic!("Expected Variant0 project");
```

### 13. hoop-daemon/tests/per_project_redaction_integration.rs:118

**Pattern:** `panic`

**Error message:** `Expected Variant0 project`

**Line:**
```rust
panic!("Expected Variant0 project");
```

### 14. hoop-daemon/tests/per_project_redaction_integration.rs:128

**Pattern:** `panic`

**Error message:** `Expected Variant0 project`

**Line:**
```rust
panic!("Expected Variant0 project");
```

### 15. hoop-daemon/tests/property_invariants.rs:278

**Pattern:** `panic`

**Error message:** `Event type mismatch at index {}`

**Line:**
```rust
_ => panic!("Event type mismatch at index {}", i),
```

### 16. hoop-daemon/tests/golden_transcripts_regression.rs:107

**Pattern:** `panic`

**Error message:** `Failed to read scenario directory {scenario_path:?}: {e}`

**Line:**
```rust
panic!("Failed to read scenario directory {scenario_path:?}: {e}")
```

### 17. hoop-daemon/tests/golden_transcripts_regression.rs:170

**Pattern:** `panic`

**Error message:** `Failed to read {:?}: {}`

**Line:**
```rust
.unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
```

### 18. hoop-daemon/tests/golden_transcripts_regression.rs:199

**Pattern:** `panic`

**Error message:** `Failed to read {:?}: {}`

**Line:**
```rust
.unwrap_or_else(|e| panic!("Failed to read {:?}: {}", scenario_dir, e))
```

### 19. hoop-daemon/tests/golden_transcripts_regression.rs:212

**Pattern:** `panic`

**Error message:** `Failed to read {:?}: {}`

**Line:**
```rust
.unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
```

### 20. hoop-daemon/tests/golden_transcripts_regression.rs:235

**Pattern:** `panic`

**Error message:** `Failed to read {:?}: {}`

**Line:**
```rust
.unwrap_or_else(|e| panic!("Failed to read {:?}: {}", simple_dir, e))
```

... and 74 more

---

## unwrap_err patterns (25 occurrences)

### 1. hoop-daemon/tests/backup_restore_cycle.rs:313

**Pattern:** `unwrap_err`

**Line:**
```rust
let error_msg = result.unwrap_err().to_string();
```

### 2. hoop-daemon/tests/claimed_at_parsing.rs:191

**Pattern:** `unwrap_err`

**Line:**
```rust
let err = parse_result.unwrap_err();
```

### 3. hoop-daemon/tests/claimed_at_parsing.rs:713

**Pattern:** `unwrap_err`

**Line:**
```rust
let err = parse_result.unwrap_err();
```

### 4. hoop-daemon/tests/skills_integration.rs:153

**Pattern:** `unwrap_err`

**Line:**
```rust
let errors = result.unwrap_err();
```

### 5. hoop-daemon/tests/skills_integration.rs:200

**Pattern:** `unwrap_err`

**Line:**
```rust
let errors = result.unwrap_err();
```

### 6. hoop-daemon/tests/skills_integration.rs:246

**Pattern:** `unwrap_err`

**Line:**
```rust
let errors = result.unwrap_err();
```

### 7. hoop-daemon/tests/disaster_recovery_runbook.rs:198

**Pattern:** `unwrap_err`

**Line:**
```rust
let err = result.unwrap_err().to_string();
```

### 8. hoop-daemon/tests/disaster_recovery_runbook.rs:556

**Pattern:** `unwrap_err`

**Line:**
```rust
let err = result.unwrap_err().to_string();
```

### 9. hoop-daemon/tests/per_project_redaction_integration.rs:188

**Pattern:** `unwrap_err`

**Line:**
```rust
let err = result.unwrap_err();
```

### 10. hoop-daemon/tests/per_project_redaction_integration.rs:266

**Pattern:** `unwrap_err`

**Line:**
```rust
let err = result.unwrap_err();
```

### 11. hoop-daemon/tests/mutation_handler_test.rs:163

**Pattern:** `unwrap_err`

**Line:**
```rust
let reject = result.unwrap_err();
```

### 12. hoop-daemon/tests/mutation_handler_test.rs:205

**Pattern:** `unwrap_err`

**Line:**
```rust
let reject = result.unwrap_err();
```

### 13. hoop-daemon/tests/mutation_handler_test.rs:238

**Pattern:** `unwrap_err`

**Line:**
```rust
let reject = result.unwrap_err();
```

### 14. hoop-daemon/tests/config_reload_cycle.rs:262

**Pattern:** `unwrap_err`

**Line:**
```rust
let yaml_err = result.unwrap_err();
```

### 15. hoop-daemon/tests/config_reload_cycle.rs:275

**Pattern:** `unwrap_err`

**Line:**
```rust
let yaml_err2 = result2.unwrap_err();
```

### 16. hoop-daemon/tests/config_reload_cycle.rs:293

**Pattern:** `unwrap_err`

**Line:**
```rust
let yaml_err3 = result3.unwrap_err();
```

### 17. hoop-daemon/tests/create_only_stub.rs:265

**Pattern:** `unwrap_err`

**Line:**
```rust
let err = result.unwrap_err();
```

### 18. hoop-cli/tests/cli_test_helpers.rs:2606

**Pattern:** `unwrap_err`

**Line:**
```rust
assert_eq!(result.unwrap_err(), "No arguments provided".to_string());
```

### 19. hoop-cli/tests/cli_test_helpers.rs:2613

**Pattern:** `unwrap_err`

**Line:**
```rust
assert_eq!(result.unwrap_err(), "No arguments provided".to_string());
```

### 20. hoop-cli/tests/cli_test_helpers.rs:2620

**Pattern:** `unwrap_err`

**Line:**
```rust
assert_eq!(result.unwrap_err(), "No arguments provided".to_string());
```

... and 5 more

---

## assert_ne patterns (1 occurrences)

### 1. hoop-daemon/tests/config_reload_cycle.rs:140

**Pattern:** `assert_ne`

**Error message:** `content hash must change on valid edit`

**Line:**
```rust
assert_ne!(hash_v1, hash_v2, "content hash must change on valid edit");
```

---

