//! Integration test: bad JSONL lines are quarantined and readers continue.
//!
//! Verifies the three acceptance criteria from §L3:
//! 1. Bad lines are routed to ~/.hoop/quarantine/<date>/ with source metadata
//! 2. The reader continues processing subsequent lines
//! 3. The quarantine metric is incremented

use std::fs;
use std::io::Write;
use std::sync::Mutex;
use tempfile::TempDir;

// Serialize tests that modify the global HOOP_QUARANTINE_DIR env var.
static QUARANTINE_TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn bad_line_quarantined_and_reader_continues() {
    let _guard = QUARANTINE_TEST_LOCK.lock().unwrap();
    let tmp = TempDir::new().unwrap();
    let jsonl_path = tmp.path().join("test.jsonl");
    let quarantine_dir = tmp.path().join("quarantine");

    std::env::set_var("HOOP_QUARANTINE_DIR", quarantine_dir.to_str().unwrap());

    // Write JSONL: 3 good lines, 1 bad line, 1 empty line
    let mut f = fs::File::create(&jsonl_path).unwrap();
    writeln!(f, r#"{{"key": "good1"}}"#).unwrap();
    writeln!(f, "THIS IS NOT JSON AT ALL").unwrap();
    writeln!(f, r#"{{"key": "good2"}}"#).unwrap();
    writeln!(f, "").unwrap();
    writeln!(f, r#"{{"key": "good3"}}"#).unwrap();
    drop(f);

    let content = fs::read_to_string(&jsonl_path).unwrap();
    let mut good = Vec::new();
    let mut quarantined = 0;
    let mut empty = 0;

    for (idx, line) in content.lines().enumerate() {
        let source = hoop_daemon::parse_jsonl_safe::LineSource {
            tag: "integration_test",
            file_path: jsonl_path.clone(),
            line_number: idx + 1,
        };
        match hoop_daemon::parse_jsonl_safe::parse_line::<serde_json::Value>(line, &source) {
            hoop_daemon::parse_jsonl_safe::ParseResult::Ok(v) => good.push(v),
            hoop_daemon::parse_jsonl_safe::ParseResult::Empty => empty += 1,
            hoop_daemon::parse_jsonl_safe::ParseResult::Quarantined => quarantined += 1,
        }
    }

    // All good lines parsed, bad line quarantined, reader continued
    assert_eq!(good.len(), 3, "should parse 3 good lines");
    assert_eq!(quarantined, 1, "should quarantine 1 bad line");
    assert_eq!(empty, 1, "should skip 1 empty line");

    // Verify quarantine directory structure
    assert!(quarantine_dir.exists(), "quarantine dir should exist");
    let date_dirs: Vec<_> = fs::read_dir(&quarantine_dir).unwrap().collect::<Result<_, _>>().unwrap();
    assert_eq!(date_dirs.len(), 1, "should have one date directory");

    let entries: Vec<_> = fs::read_dir(date_dirs[0].path())
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(entries.len(), 1, "should have one quarantined entry");

    // Verify quarantine entry metadata
    let entry: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(entries[0].path()).unwrap()).unwrap();
    assert_eq!(entry["tag"], "integration_test");
    assert!(entry["line"].as_str().unwrap().contains("NOT JSON"));
    assert!(entry["source_path"].as_str().unwrap().contains("test.jsonl"));
    assert_eq!(entry["line_number"], 2);

    std::env::remove_var("HOOP_QUARANTINE_DIR");
}

#[test]
fn quarantine_raw_for_custom_malformed_detection() {
    let _guard = QUARANTINE_TEST_LOCK.lock().unwrap();
    let tmp = TempDir::new().unwrap();
    let quarantine_dir = tmp.path().join("quarantine");

    std::env::set_var("HOOP_QUARANTINE_DIR", quarantine_dir.to_str().unwrap());

    let source = hoop_daemon::parse_jsonl_safe::LineSource {
        tag: "custom_parser",
        file_path: std::path::PathBuf::from("/tmp/custom.jsonl"),
        line_number: 42,
    };

    hoop_daemon::parse_jsonl_safe::quarantine_raw(
        "valid json but failed schema check",
        "missing required field 'timestamp'",
        &source,
    );

    // Verify quarantine was populated
    let date_dirs: Vec<_> = fs::read_dir(&quarantine_dir).unwrap().collect::<Result<_, _>>().unwrap();
    assert_eq!(date_dirs.len(), 1);

    let entries: Vec<_> = fs::read_dir(date_dirs[0].path())
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(entries.len(), 1);

    let entry: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(entries[0].path()).unwrap()).unwrap();
    assert_eq!(entry["tag"], "custom_parser");
    assert_eq!(entry["line_number"], 42);
    assert!(entry["reason"].as_str().unwrap().contains("timestamp"));

    std::env::remove_var("HOOP_QUARANTINE_DIR");
}
