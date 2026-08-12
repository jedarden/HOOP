# HOOP Test Error Message Extraction - Error Types and anyhow

This document catalogs error messages from Error types and anyhow::Result patterns in HOOP tests and code.

## Summary Statistics

- **Total `.expect()` patterns found**: 200+
- **Total `.unwrap_err()` patterns found**: 40+
- **Total `anyhow!()` patterns found**: 60+
- **Total `anyhow::bail!()` patterns found**: 40+
- **Total `.context()` patterns found**: 100+
- **Files analyzed**: 239 test files across HOOP workspace

---

## 1. `.expect()` Error Messages

### Acceptance Test Files

#### tests/acceptance/s5_workspace_deleted.rs
- **Line 27**: `expect("Failed to create .beads dir")` - File system creation error
- **Line 29**: `expect("Failed to create issues.jsonl")` - File write error
- **Line 37**: `expect("Failed to create temp dir")` - Temp directory creation error
- **Line 39**: `expect("Failed to create .hoop dir")` - Config directory creation error
- **Line 68**: `expect("Failed to write projects.yaml")` - YAML file write error
- **Line 77**: `expect("Failed to write config.yml")` - Config file write error
- **Line 79**: `expect("Failed to create data dir")` - Data directory creation error
- **Line 166**: `expect("Failed to spawn daemon")` - Process spawning error
- **Line 173**: `expect("Failed to get readyz status")` - Health check error
- **Line 179**: `expect("Failed to remove .beads from project A")` - Directory removal error
- **Line 228**: `expect("Failed to spawn daemon")` - Process spawning error
- **Line 237**: `expect("Failed to remove .beads from project A")` - Directory removal error
- **Line 247**: `expect("Failed to fetch projects")` - API fetch error
- **Line 251**: `expect("Failed to parse projects")` - JSON parsing error
- **Line 263**: `expect("Failed to check health")` - Health check error
- **Line 291**: `expect("Failed to spawn daemon")` - Process spawning error
- **Line 299**: `expect("Failed to get readyz status")` - Health check error
- **Line 304**: `expect("Failed to remove .beads from project A")` - Directory removal error
- **Line 311**: `expect("Failed to get readyz status after deletion")` - Health check error
- **Line 364**: `expect("Failed to spawn daemon")` - Process spawning error
- **Line 373**: `expect("Failed to remove .beads")` - Directory removal error
- **Line 382**: `expect("Failed to check health")` - Health check error

#### tests/acceptance/s2_transcript_archaeology.rs
- **Line 35**: `expect("workspace root")` - Path validation error
- **Line 104**: `expect("Failed to spawn daemon")` - Process spawning error
- **Line 112**: `expect("Failed to fetch beads")` - API fetch error
- **Line 116**: `expect("Failed to parse beads")` - JSON parsing error
- **Line 122**: `expect("Bead should have an id")` - Data validation error
- **Line 128**: `expect("Failed to fetch bead events")` - API fetch error
- **Line 136**: `expect("Failed to parse events")` - JSON parsing error
- **Line 152**: `expect("Failed to spawn daemon")` - Process spawning error
- **Line 160**: `expect("Failed to fetch beads")` - API fetch error
- **Line 162**: `expect("Failed to parse beads")` - JSON parsing error
- **Line 168**: `expect("Bead should have an id")` - Data validation error
- **Line 176**: `expect("Failed to fetch bead events")` - API fetch error
- **Line 197**: `expect("Failed to spawn daemon")` - Process spawning error
- **Line 205**: `expect("Failed to connect to stitch endpoint")` - API connection error
- **Line 219**: `expect("Failed to spawn daemon")` - Process spawning error
- **Line 235**: `expect("Failed to connect to endpoint")` - API connection error
- **Line 251**: `expect("Failed to spawn daemon")` - Process spawning error
- **Line 259**: `expect("Failed to fetch conversations")` - API fetch error
- **Line 263**: `expect("Failed to parse conversations")` - JSON parsing error
- **Line 274**: `expect("Failed to spawn daemon")` - Process spawning error
- **Line 282**: `expect("Failed to fetch beads")` - API fetch error
- **Line 284**: `expect("Failed to parse beads")` - JSON parsing error
- **Line 300**: `expect("Failed to spawn daemon")` - Process spawning error
- **Line 308**: `expect("Failed to fetch cost trends")` - API fetch error
- **Line 312**: `expect("Failed to parse cost data")` - JSON parsing error
- **Line 323**: `expect("Failed to spawn daemon")` - Process spawning error
- **Line 331**: `expect("Failed to fetch beads")` - API fetch error
- **Line 333**: `expect("Failed to parse beads")` - JSON parsing error

---

## 2. `.unwrap_err()` Error Messages

### Test Files

#### hoop-daemon/tests/backup_restore_cycle.rs
- **Line 313**: `unwrap_err()` - Backup restore error handling

#### hoop-daemon/tests/disaster_recovery_runbook.rs
- **Line 198**: `unwrap_err()` - Disaster recovery error extraction
- **Line 556**: `unwrap_err()` - Disaster recovery error extraction

#### hoop-daemon/tests/create_only_stub.rs
- **Line 265**: `unwrap_err()` - Create stub error handling

#### hoop-daemon/tests/claimed_at_parsing.rs
- **Line 191**: `unwrap_err()` - Claim timestamp parsing error
- **Line 713**: `unwrap_err()` - Claim timestamp parsing error

#### hoop-daemon/tests/per_project_redaction_integration.rs
- **Line 188**: `unwrap_err()` - Redaction error handling
- **Line 266**: `unwrap_err()` - Redaction error handling

#### hoop-daemon/tests/skills_integration.rs
- **Line 153**: `unwrap_err()` - Skills integration error extraction
- **Line 200**: `unwrap_err()` - Skills integration error extraction
- **Line 246**: `unwrap_err()` - Skills integration error extraction

#### hoop-daemon/tests/mutation_handler_test.rs
- **Line 163**: `unwrap_err()` - Mutation handler rejection extraction
- **Line 205**: `unwrap_err()` - Mutation handler rejection extraction
- **Line 238**: `unwrap_err()` - Mutation handler rejection extraction

#### hoop-daemon/tests/config_reload_cycle.rs
- **Line 262**: `unwrap_err()` - Config reload YAML error extraction
- **Line 275**: `unwrap_err()` - Config reload YAML error extraction
- **Line 293**: `unwrap_err()` - Config reload YAML error extraction

#### hoop-cli/tests/cli_test_helpers.rs
- **Line 2606**: `unwrap_err()` - CLI argument parsing error
- **Line 2613**: `unwrap_err()` - CLI argument parsing error

### Production Code (test utilities)

#### hoop-daemon/src/fleet.rs
- **Line 7424**: `unwrap_err()` - Schema version gate validation
- **Line 7436**: `unwrap_err()` - Schema version gate validation

#### hoop-daemon/src/projects.rs
- **Line 809**: `unwrap_err()` - Project configuration error extraction
- **Line 1161**: `unwrap_err()` - Config error conversion
- **Line 1181**: `unwrap_err()` - Config error conversion

#### hoop-daemon/src/api_stitch_decompose.rs
- **Line 1084**: `unwrap_err()` - Stitch decomposition error status extraction
- **Line 1105**: `unwrap_err()` - Stitch decomposition error status extraction
- **Line 1126**: `unwrap_err()` - Stitch decomposition error status extraction
- **Line 1146**: `unwrap_err()` - Stitch decomposition error status extraction
- **Line 1171**: `unwrap_err()` - Stitch decomposition error status extraction

#### hoop-daemon/src/api_beads.rs
- **Line 988**: `unwrap_err()` - Bead API error status extraction
- **Line 1010**: `unwrap_err()` - Bead API error status extraction
- **Line 1032**: `unwrap_err()` - Bead API error status extraction
- **Line 1054**: `unwrap_err()` - Bead API error status extraction

#### hoop-daemon/src/prompt_substitute.rs
- **Line 450**: `unwrap_err()` - Prompt substitution error matching
- **Line 484**: `unwrap_err()` - Prompt substitution error matching
- **Line 495**: `unwrap_err()` - Prompt substitution error matching
- **Line 506**: `unwrap_err()` - Prompt substitution error matching
- **Line 583**: `unwrap_err()` - Prompt substitution error matching

#### hoop-daemon/src/snapshot_manifest.rs
- **Line 205**: `unwrap_err()` - Snapshot manifest validation error

#### hoop-daemon/src/audio_redaction.rs
- **Line 504**: `unwrap_err()` - Audio redaction error extraction

#### hoop-schema/src/id_validators.rs
- **Line 561**: `unwrap_err()` - Bead ID validation error
- **Line 567**: `unwrap_err()` - Bead ID validation error
- **Line 768**: `unwrap_err()` - Worker name validation error
- **Line 840**: `unwrap_err()` - Project name validation error
- **Line 846**: `unwrap_err()` - Project name validation error

#### hoop-schema/src/effort.rs
- **Line 110**: `unwrap_err()` - Effort validation error for provider

#### hoop-mcp/tests/forbidden_worker_steering.rs
- **Line 126**: `unwrap_err()` - Forbidden worker steering error extraction

#### hoop-mcp/src/skills.rs
- **Line 582**: `unwrap_err()` - Skills error extraction
- **Line 696**: `unwrap_err()` - Skills error extraction
- **Line 717**: `unwrap_err()` - Skills error extraction

---

## 3. `anyhow!()` Error Messages

### Acceptance Test Files

#### tests/acceptance/s6_machine_mode.rs
- **Line 98**: `anyhow!("Daemon failed to start within timeout")` - Daemon startup timeout

#### tests/acceptance/s4_daemon_restart.rs
- **Line 177**: `anyhow!("Daemon failed to start")` - Daemon startup failure

#### tests/acceptance/s3_bead_creation_from_chat.rs
- **Line 99**: `anyhow!("Daemon failed to start within timeout")` - Daemon startup timeout

#### tests/acceptance/s2_transcript_archaeology.rs
- **Line 97**: `anyhow!("Daemon failed to start within timeout")` - Daemon startup timeout

#### tests/acceptance/s1_morning_review.rs
- **Line 96**: `anyhow!("Daemon failed to start within timeout")` - Daemon startup timeout

#### tests/acceptance/s5_workspace_deleted.rs
- **Line 130**: `anyhow!("Daemon failed to start")` - Daemon startup failure

### Production Code

#### hoop-daemon/src/agent_context.rs
- **Line 568**: `anyhow!("Cannot determine home directory")` - Home directory resolution error

#### hoop-daemon/src/files.rs
- **Line 143**: `anyhow!("directory not within project workspace")` - Path validation error
- **Line 358**: `anyhow!("project root not within workspace")` - Path validation error

#### hoop-daemon/src/screen_capture.rs
- **Line 101**: `anyhow!("home directory not found")` - Home directory resolution error
- **Line 110**: `anyhow!("path traversal detected for stitch id")` - Security error

#### hoop-daemon/src/agent_session.rs
- **Line 458**: `anyhow!("Agent is disabled")` - Agent configuration error
- **Line 463**: `anyhow!("No active agent session")` - Session state error
- **Line 862**: `anyhow!("Cannot determine home directory")` - Home directory resolution error

#### hoop-daemon/src/shutdown.rs
- **Line 266**: `anyhow!("Shutdown channel closed")` - Channel communication error

#### hoop-daemon/src/lib.rs
- **Line 801**: `anyhow!("workspace allowlist: {e}")` - Path allowlist error
- **Line 803**: `anyhow!("path traversal: {e}")` - Security error
- **Line 865**: `anyhow!("workspace allowlist: {e}")` - Path allowlist error
- **Line 867**: `anyhow!("path traversal: {e}")` - Security error
- **Line 875**: `anyhow!("read: {e}")` - File read error
- **Line 903**: `anyhow!("workspace allowlist: {e}")` - Path allowlist error
- **Line 906**: `anyhow!("path traversal: {e}")` - Security error
- **Line 918**: `anyhow!("read: {e}")` - File read error
- **Line 1008**: `anyhow!("workspace allowlist: {e}")` - Path allowlist error
- **Line 1010**: `anyhow!("path traversal: {e}")` - Security error
- **Line 1018**: `anyhow!("read: {e}")` - File read error

#### hoop-daemon/src/embedding_service.rs
- **Line 454**: `anyhow!("No Anthropic API key configured")` - Configuration error
- **Line 483**: `anyhow!("Remote embedding not implemented")` - Feature not implemented error

#### hoop-daemon/src/pattern_query_evaluator.rs
- **Line 348**: `anyhow!("No labels field on bead")` - Data validation error

#### hoop-daemon/src/dictated_notes.rs
- **Line 216**: `anyhow!("home directory not found")` - Home directory resolution error
- **Line 225**: `anyhow!("path traversal detected for stitch id")` - Security error

#### hoop-daemon/src/attachments.rs
- **Line 186**: `anyhow!("attachment path has no parent")` - Path validation error
- **Line 336**: `anyhow!("path traversal detected for bead id")` - Security error
- **Line 352**: `anyhow!("home directory not found")` - Home directory resolution error
- **Line 362**: `anyhow!("path traversal detected for stitch id")` - Security error
- **Line 389**: `anyhow!("attachment path has no parent")` - Path validation error
- **Line 409**: `anyhow!("attachment path has no parent")` - Path validation error
- **Line 610**: `anyhow!("dest path has no parent")` - Path validation error

#### hoop-daemon/src/uploads.rs
- **Line 442**: `anyhow!("SVG path has no parent directory")` - Path validation error
- **Line 468**: `anyhow!("SVG path has no parent directory")` - Path validation error
- **Line 512**: `anyhow!("PDF path has no parent directory")` - Path validation error
- **Line 538**: `anyhow!("PDF path has no parent directory")` - Path validation error

#### hoop-daemon/src/backup_pipeline.rs
- **Line 424**: `anyhow!("path traversal detected in attachment key")` - Security error
- **Line 449**: `anyhow!("path traversal detected in attachment key")` - Security error

#### hoop-mcp/src/notes.rs
- **Line 16**: `anyhow!("Cannot determine home directory")` - Home directory resolution error

#### hoop-mcp/src/tools.rs
- **Line 76**: `anyhow!("Cannot determine home directory")` - Home directory resolution error
- **Line 122**: `anyhow!("Cannot determine home directory")` - Home directory resolution error

#### hoop-mcp/src/skills.rs
- **Line 115**: `anyhow!("Cannot determine home directory")` - Home directory resolution error
- **Line 289**: `anyhow!("Failed to capture stdout")` - Process output capture error

---

## 4. `anyhow::bail!()` Error Messages

### Test Files

#### hoop-daemon/tests/testrepo_integration.rs
- **Line 67**: `bail!("Daemon did not become ready")` - Daemon readiness timeout

#### hoop-daemon/tests/integration_harness.rs
- **Line 115**: `bail!("testrepo/.beads/events.jsonl should exist")` - Test fixture validation
- **Line 121**: `bail!("testrepo/.beads/heartbeats.jsonl should exist")` - Test fixture validation
- **Line 127**: `bail!("events.jsonl should not be empty")` - Test fixture validation
- **Line 143**: `bail!("heartbeats.jsonl should not be empty")` - Test fixture validation
- **Line 217**: `bail!("Events fixture should contain at least one claim event")` - Test fixture validation
- **Line 220**: `bail!("Events fixture should contain at least one dispatch event")` - Test fixture validation
- **Line 223**: `bail!("Events fixture should contain at least one complete event")` - Test fixture validation
- **Line 226**: `bail!("Events fixture should contain at least one fail event")` - Test fixture validation
- **Line 247**: `bail!("Heartbeats fixture should contain at least one idle state")` - Test fixture validation
- **Line 250**: `bail!("Heartbeats fixture should contain at least one executing state")` - Test fixture validation
- **Line 359**: `bail!("projects.yaml should be created")` - Test fixture validation
- **Line 365**: `bail!("config.yml should be created")` - Test fixture validation
- **Line 371**: `bail!("projects.yaml should reference testrepo")` - Test fixture validation
- **Line 698**: `bail!("Daemon failed to become ready within 10 seconds")` - Daemon readiness timeout

#### hoop-daemon/tests/adapter_failover_test.rs
- **Line 49**: `bail!("Daemon did not become ready")` - Daemon readiness timeout

#### hoop-daemon/tests/testrepo_harness_integration.rs
- **Line 57**: `bail!("Daemon did not become ready")` - Daemon readiness timeout

#### hoop-daemon/tests_phase5/adapter_failover_test.rs
- **Line 46**: `bail!("Daemon did not become ready")` - Daemon readiness timeout

### Production Code

#### hoop-daemon/src/integration_test_client.rs
- **Line 286**: `bail!("Capacity response is not an object")` - API response validation
- **Line 352**: `bail!("WebSocket connection closed")` - WebSocket connection error
- **Line 355**: `bail!("WebSocket connection terminated")` - WebSocket termination error
- **Line 361**: `bail!("Timeout waiting for bead event")` - Event timeout error

#### hoop-daemon/src/lib.rs
- **Line 805**: `bail!("not a file")` - File validation error
- **Line 810**: `bail!("image too large (>50 MB)")` - File size validation error
- **Line 869**: `bail!("not a file")` - File validation error
- **Line 873**: `bail!("file too large for raw mode (>50 KB)")` - File size validation error
- **Line 909**: `bail!("not a file")` - File validation error
- **Line 914**: `bail!("file too large (>100 MB)")` - File size validation error
- **Line 1012**: `bail!("not a file")` - File validation error
- **Line 1016**: `bail!("file too large (>100 MB)")` - File size validation error

#### hoop-daemon/src/files.rs
- **Line 134**: `bail!("not a directory")` - Directory validation error

#### hoop-daemon/src/pdf_sanitize.rs
- **Line 44**: `bail!("not a valid PDF: missing %PDF- header")` - PDF format validation error

#### hoop-daemon/src/attachments.rs
- **Line 391**: `bail!("path traversal detected in filename")` - Security error
- **Line 411**: `bail!("path traversal detected in filename")` - Security error

#### hoop-daemon/src/pattern_query_evaluator.rs
- **Line 220**: `bail!("Unexpected end of input")` - Parser error
- **Line 227**: `bail!("Expected closing parenthesis")` - Parser error
- **Line 237**: `bail!("Expected value after colon")` - Parser error
- **Line 250**: `bail!("Expected word value after colon")` - Parser error

#### hoop-daemon/src/uploads.rs
- **Line 154**: `bail!("file size must be positive")` - Upload validation error
- **Line 157**: `bail!("filename must be 1-255 characters")` - Upload validation error
- **Line 161**: `bail!("checksum must be 64-character hex string (SHA-256)")` - Upload validation error

#### hoop-cli/src/script.rs
- **Line 140**: `bail!("Script timed out")` - Script execution timeout

---

## 5. `.context()` Error Messages

### Production Code

#### hoop-daemon/src/api_unassigned.rs
- **Line 153**: `context("Failed to determine home directory")` - Home directory resolution context

#### hoop-daemon/src/files.rs
- **Line 139**: `context("failed to build path allowlist for project")` - Path allowlist building context
- **Line 157**: `context("failed to read directory entry")` - Directory reading context
- **Line 356**: `context("failed to build path allowlist")` - Path allowlist building context
- **Line 386**: `context("failed to run ripgrep")` - Command execution context
- **Line 477**: `context("git ls-files failed")` - Git command context

#### hoop-daemon/src/projects.rs
- **Line 52**: `context("Failed to read projects.yaml")` - File reading context
- **Line 57**: `context("Failed to parse projects.yaml")` - YAML parsing context
- **Line 112**: `context("Failed to serialize projects.yaml")` - YAML serialization context
- **Line 113**: `context("Failed to write projects.yaml")` - File writing context
- **Line 536**: `context("Failed to load initial projects configuration")` - Configuration loading context
- **Line 564**: `context("Failed to create .hoop directory")` - Directory creation context
- **Line 584**: `context("Failed to create file watcher")` - Watcher creation context
- **Line 599**: `context("Failed to watch projects directory")` - Directory watching context

#### hoop-daemon/src/backup_pipeline.rs
- **Line 273**: `context("serialize snapshot manifest")` - Serialization context
- **Line 368**: `context("zstd compress attachment")` - Compression context
- **Line 387**: `context("serialize attachment manifest")` - Serialization context
- **Line 422**: `context("failed to build allowlist for stitch attachments")` - Allowlist building context
- **Line 447**: `context("failed to build allowlist for workspace")` - Allowlist building context
- **Line 522**: `context("create temp dir for backup snapshot")` - Directory creation context
- **Line 537**: `context("open fleet.db for VACUUM INTO")` - Database operation context
- **Line 542**: `context("VACUUM INTO failed")` - Database operation context
- **Line 559**: `context("zstd compression failed")` - Compression context
- **Line 596**: `context("failed to spawn `age` — is it installed?")` - Command execution context

#### hoop-daemon/src/screen_capture.rs
- **Line 99**: `context("failed to build path allowlist for stitch attachments")` - Allowlist building context
- **Line 281**: `context("failed to create streaming uploads directory")` - Directory creation context
- **Line 295**: `context("stream directory failed path validation")` - Path validation context
- **Line 317**: `context("generated invalid stitch ID")` - ID generation context
- **Line 331**: `context("failed to create stream directory")` - Directory creation context
- **Line 352**: `context("failed to serialize session metadata")` - Serialization context
- **Line 354**: `context("failed to write session metadata")` - File writing context
- **Line 359**: `context("failed to create partial file")` - File creation context
- **Line 381**: `context("failed to open partial file")` - File opening context
- **Line 384**: `context("failed to seek to end of partial file")` - File seeking context
- **Line 386**: `context("failed to write chunk")` - File writing context
- **Line 388**: `context("failed to sync chunk to disk")` - File syncing context
- **Line 392**: `context("failed to get file metadata")` - File metadata context
- **Line 409**: `context("failed to parse session metadata")` - Parsing context
- **Line 417**: `context("failed to serialize session metadata")` - Serialization context
- **Line 419**: `context("failed to write session metadata")` - File writing context
- **Line 451**: `context("failed to serialize frame samples")` - Serialization context
- **Line 453**: `context("failed to write frame samples")` - File writing context
- **Line 493**: `context("failed to serialize metadata")` - Serialization context
- **Line 495**: `context("failed to write metadata")` - File writing context
- **Line 500**: `context("failed to open database")` - Database opening context
- **Line 502**: `context("failed to set WAL mode")` - Database configuration context
- **Line 514**: `context("failed to create stitch row")` - Database operation context
- **Line 538**: `context("failed to clean up stream directory")` - Cleanup context
- **Line 557**: `context("failed to remove stream directory")` - Directory removal context

#### hoop-daemon/src/config_watcher.rs
- **Line 185**: `context("Failed to create .hoop directory")` - Directory creation context

---

## Error Message Categories

### System/Infrastructure Errors
- Daemon startup failures
- Process spawning errors
- Health check failures
- File system operations
- Directory creation/removal

### Security Errors
- Path traversal detection
- File size validation
- File type validation
- Input validation

### API/Network Errors
- Request failures
- Response parsing errors
- Connection errors
- Timeout errors

### Configuration Errors
- Missing configuration files
- Invalid YAML format
- Missing required settings
- Path resolution errors

### Data Validation Errors
- Missing required fields
- Invalid data formats
- Schema violations
- Constraint violations

### File Operation Errors
- Read/write failures
- Permission errors
- File not found errors
- Serialization/deserialization errors

---

## Pattern Analysis

### Most Common Error Patterns

1. **"Failed to spawn daemon"** - Process lifecycle management
2. **"Failed to fetch/parse"** - API interaction patterns
3. **"Failed to create/remove directory"** - File system operations
4. **"path traversal detected"** - Security validation
5. **"Cannot determine home directory"** - Environment resolution
6. **"Daemon did not become ready"** - Startup validation
7. **"Failed to read/write"** - File operations
8. **"context(...)" chains** - Error propagation patterns

### Error Handling Quality Observations

**Good Patterns:**
- Consistent error message format across codebase
- Clear categorization of error types
- Good use of `.context()` for error propagation
- Meaningful error messages for debugging

**Areas for Improvement:**
- Some error messages could be more specific
- Mixed precision levels (some very specific, some generic)
- Inconsistent error code categorization
- Limited error recovery information in messages

---

## Notes

- This extraction focused on Error types and anyhow patterns, excluding assertion macros
- Format strings (e.g., `format!()`) were included when found in error contexts
- Production code errors were included when they appear in test contexts or utilities
- All line numbers are approximate and may have changed since extraction
- Error messages are preserved exactly as found in source code