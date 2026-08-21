//! Concurrent audit hash-chain test
//!
//! Verifies that concurrent calls to write_audit_row produce a valid hash chain
//! without TOCTOU races or error-to-genesis fallbacks.
//!
//! Test scenario:
//! 1. Spawn N threads, each concurrently calling write_audit_row
//! 2. Each thread writes M audit rows
//! 3. After all threads complete, verify_hash_chain() must return Ok
//!
//! This test would fail with the old implementation due to:
//! - TOCTOU race: concurrent reads of the same hash_prev
//! - Error-to-genesis fallback: SQLITE_BUSY mapped to GENESIS_HASH

use hoop_daemon::fleet;
use serial_test::serial;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

/// Helper: create a fresh test database with a unique name per test
fn setup_test_db(test_name: &str) -> tempfile::TempDir {
    // Clean up any existing test database
    cleanup_test_db();

    let dir = tempfile::tempdir().expect("tempdir");
    // Add a UUID to ensure truly unique database names
    let uuid = uuid::Uuid::new_v4().to_string()[..8].to_string();
    let db_path = dir.path().join(format!("fleet-{}-{}.db", test_name, uuid));

    eprintln!("Test {}: Creating database at {:?}", test_name, db_path);

    // Verify the database file doesn't exist yet
    assert!(!db_path.exists(), "Database file should not exist yet: {:?}", db_path);

    std::env::set_var("_HOOP_FLEET_DB_PATH", &db_path);
    fleet::init_fleet_db_at(db_path.clone()).expect("init fleet db");

    // Verify the database file now exists
    assert!(db_path.exists(), "Database file should exist after init: {:?}", db_path);

    dir
}

/// Helper: cleanup test database
fn cleanup_test_db() {
    // Close any open SQLite connections by removing the path reference
    std::env::remove_var("_HOOP_FLEET_DB_PATH");
    // Give SQLite time to release file handles
    std::thread::sleep(std::time::Duration::from_millis(100));
}

#[test]
#[serial]
fn test_concurrent_audit_row_writes() {
    // Force close any SQLite connections from previous tests
    cleanup_test_db();
    std::thread::sleep(std::time::Duration::from_millis(500));

    let _db_dir = setup_test_db("audit_row_writes");

    // Verify we start with an empty database
    let path = std::env::var("_HOOP_FLEET_DB_PATH").expect("_HOOP_FLEET_DB_PATH should be set");
    eprintln!("Test audit_row_writes: Checking database at {}", path);

    // Open to verify we're reading the actual file
    let conn = rusqlite::Connection::open(&path).expect("open db");

    let initial_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM actions", [], |row| row.get(0))
        .expect("count rows");

    eprintln!("Test audit_row_writes: Initial row count = {}", initial_count);

    assert_eq!(initial_count, 0, "Database should start empty, found {} rows", initial_count);
    drop(conn);

    const NUM_THREADS: usize = 10;
    const ROWS_PER_THREAD: usize = 20;

    let error_count = std::sync::Arc::new(AtomicUsize::new(0));
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(NUM_THREADS));
    let mut handles = vec![];

    // Spawn N threads, each writing M rows concurrently
    for thread_id in 0..NUM_THREADS {
        let error_count = error_count.clone();
        let barrier = barrier.clone();
        let handle = thread::spawn(move || {
            barrier.wait(); // Synchronize thread start
            for row_id in 0..ROWS_PER_THREAD {
                let result = fleet::write_audit_row(
                    &format!("thread-{}-row-{}", thread_id, row_id),
                    fleet::ActionKind::BeadCreated,
                    "test-target",
                    Some("test-project"),
                    None,
                    fleet::ActionResult::Success,
                    None,
                    None,
                    None,
                    None,
                );

                if result.is_err() {
                    error_count.fetch_add(1, Ordering::SeqCst);
                    eprintln!("Thread {} row {} failed: {:?}", thread_id, row_id, result);
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("thread panic");
    }

    // Verify no errors occurred
    let errors = error_count.load(Ordering::SeqCst);
    assert_eq!(errors, 0, "Expected no errors, got {}", errors);

    // Close all SQLite connections by forcing database close
    drop(_db_dir);
    cleanup_test_db();

    // Verify hash chain integrity
    let verify_result = fleet::verify_hash_chain();
    assert!(
        verify_result.is_ok(),
        "Hash chain verification failed: {:?}",
        verify_result
    );

    // Verify we wrote the expected number of rows
    let path = std::env::var("_HOOP_FLEET_DB_PATH")
        .expect("_HOOP_FLEET_DB_PATH should be set");
    let conn = rusqlite::Connection::open(&path).expect("open db");
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM actions", [], |row| row.get(0))
        .expect("count rows");
    assert_eq!(
        count,
        (NUM_THREADS * ROWS_PER_THREAD) as i64,
        "Expected {} rows, got {}",
        NUM_THREADS * ROWS_PER_THREAD,
        count
    );
}

#[test]
#[serial]
fn test_concurrent_with_high_contention() {
    let _db_dir = setup_test_db("high_contention");

    // Verify we start with an empty database
    let path = std::env::var("_HOOP_FLEET_DB_PATH").expect("_HOOP_FLEET_DB_PATH should be set");
    eprintln!("Test high_contention: Checking database at {}", path);

    // Open to verify we're reading the actual file
    let conn = rusqlite::Connection::open(&path).expect("open db");

    let initial_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM actions", [], |row| row.get(0))
        .expect("count rows");

    eprintln!("Test high_contention: Initial row count = {}", initial_count);

    assert_eq!(initial_count, 0, "Database should start empty, found {} rows", initial_count);
    drop(conn);

    const NUM_THREADS: usize = 20;
    const ROWS_PER_THREAD: usize = 10;

    let error_count = std::sync::Arc::new(AtomicUsize::new(0));
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(NUM_THREADS));
    let mut handles = vec![];

    // Spawn N threads, each writing M rows concurrently
    // This tests high contention where SQLITE_BUSY is likely
    for thread_id in 0..NUM_THREADS {
        let error_count = error_count.clone();
        let barrier = barrier.clone();
        let handle = thread::spawn(move || {
            barrier.wait(); // Synchronize thread start
            for row_id in 0..ROWS_PER_THREAD {
                let result = fleet::write_audit_row(
                    &format!("high-contention-{}-{}", thread_id, row_id),
                    fleet::ActionKind::StitchCreated,
                    "test-target-high-contention",
                    Some("test-project-high"),
                    None,
                    fleet::ActionResult::Success,
                    None,
                    None,
                    None,
                    None,
                );

                if result.is_err() {
                    error_count.fetch_add(1, Ordering::SeqCst);
                    eprintln!(
                        "High contention thread {} row {} failed: {:?}",
                        thread_id, row_id, result
                    );
                }

                // Add small delay to increase contention
                thread::sleep(Duration::from_micros(100));
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("thread panic");
    }

    // Verify no errors occurred
    let errors = error_count.load(Ordering::SeqCst);
    assert_eq!(errors, 0, "Expected no errors under high contention, got {}", errors);

    // Verify hash chain integrity even under high contention
    let verify_result = fleet::verify_hash_chain();
    assert!(
        verify_result.is_ok(),
        "Hash chain verification failed under high contention: {:?}",
        verify_result
    );
}
