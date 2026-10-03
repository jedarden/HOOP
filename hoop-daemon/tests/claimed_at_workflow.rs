//! End-to-end coverage for malformed claim timestamps.
//!
//! These tests exercise the same path used by the daemon at runtime:
//! `events.jsonl` → `EventTailer` → `ProjectSupervisor` → `fleet.db`.
//! The parser-only tests in `claimed_at_parsing.rs` intentionally do not cover
//! the database projection or the worker-event projection.

use chrono::{DateTime, Utc};
use hoop_daemon::cost::CostAggregator;
use hoop_daemon::fleet;
use hoop_daemon::shutdown::ShutdownCoordinator;
use hoop_daemon::stuck_detector::StuckDetector;
use hoop_daemon::supervisor::{ProjectSupervisor, SupervisorDeps};
use hoop_daemon::vector_index::VectorIndex;
use hoop_daemon::ws::WorkerRegistry;
use hoop_daemon::{Bead, BeadStatus, BeadType};
use rusqlite::{Connection, OptionalExtension};
use serial_test::serial;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

const PROJECT: &str = "timestamp-edge-project";

/// The daemon resolves both its event log and default database from process
/// environment. Keep these tests serialized so their hermetic overrides do
/// not race with one another.
struct EnvironmentGuard {
    home: Option<std::ffi::OsString>,
    db_path: Option<std::ffi::OsString>,
}

impl EnvironmentGuard {
    fn set(home: &Path, db_path: &Path) -> Self {
        let guard = Self {
            home: std::env::var_os("HOME"),
            db_path: std::env::var_os("_HOOP_FLEET_DB_PATH"),
        };
        std::env::set_var("HOME", home);
        std::env::set_var("_HOOP_FLEET_DB_PATH", db_path);
        guard
    }
}

impl Drop for EnvironmentGuard {
    fn drop(&mut self) {
        match self.home.take() {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
        match self.db_path.take() {
            Some(value) => std::env::set_var("_HOOP_FLEET_DB_PATH", value),
            None => std::env::remove_var("_HOOP_FLEET_DB_PATH"),
        }
    }
}

struct Workflow {
    // Field order is intentional: ProjectSupervisor is dropped before the
    // environment guard, so its event task cannot observe a restored HOME.
    _temp_dir: TempDir,
    _environment: EnvironmentGuard,
    supervisor: ProjectSupervisor,
    registry: Arc<WorkerRegistry>,
    events_path: PathBuf,
    db_path: PathBuf,
}

impl Workflow {
    async fn new(bead_ids: &[&str]) -> Self {
        let temp_dir = tempfile::tempdir().expect("create isolated workflow directory");
        let hoop_dir = temp_dir.path().join(".hoop");
        fs::create_dir_all(hoop_dir.join("scripts")).expect("create HOOP directories");

        let events_path = hoop_dir.join("events.jsonl");
        hoop_daemon::atomic_write::atomic_write_file(&events_path, b"")
            .expect("create empty events log");
        let db_path = hoop_dir.join("fleet.db");
        let environment = EnvironmentGuard::set(temp_dir.path(), &db_path);
        fleet::init_fleet_db().expect("initialize isolated fleet database");

        let (bead_tx, _) = tokio::sync::broadcast::channel(64);
        let (session_tx, _) = tokio::sync::broadcast::channel(64);
        let (monitor_tx, _) = tokio::sync::broadcast::channel(64);
        let registry = Arc::new(WorkerRegistry::new(monitor_tx, session_tx.clone()));
        let beads = Arc::new(std::sync::RwLock::new(
            bead_ids.iter().map(|id| test_bead(id)).collect(),
        ));
        let shutdown = Arc::new(ShutdownCoordinator::new());
        let cost_aggregator = Arc::new(std::sync::RwLock::new(
            CostAggregator::new(hoop_dir.join("pricing.yml")).expect("create cost aggregator"),
        ));
        let vector_index = Arc::new(std::sync::RwLock::new(VectorIndex::new()));
        let stuck_detector = Arc::new(std::sync::Mutex::new(StuckDetector::new()));

        let supervisor = ProjectSupervisor::new(
            SupervisorDeps {
                bead_tx,
                session_tx,
                worker_registry: registry.clone(),
                beads,
                shutdown,
                cost_aggregator,
                vector_index,
                stuck_detector,
            },
            hoop_dir.join("scripts"),
        );
        supervisor
            .start_event_tailer()
            .await
            .expect("start event tailer");

        Self {
            _temp_dir: temp_dir,
            _environment: environment,
            supervisor,
            registry,
            events_path,
            db_path,
        }
    }

    async fn stop(&self) {
        self.supervisor.stop_event_tailer().await;
        // Let the event consumer observe the tailer's sender being dropped
        // before the environment and temporary database are torn down.
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

fn test_bead(id: &str) -> Bead {
    let now = Utc::now();
    Bead {
        id: id.to_string(),
        title: format!("Timestamp edge case {id}"),
        description: None,
        status: BeadStatus::Open,
        priority: 2,
        issue_type: BeadType::Test,
        created_at: now,
        updated_at: now,
        created_by: "integration-test".to_string(),
        dependencies: Vec::new(),
        project: PROJECT.to_string(),
        workspace: "/tmp/timestamp-edge-workspace".to_string(),
    }
}

fn append_events(path: &Path, events: &[&str]) {
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .expect("open events log for append");
    for event in events {
        writeln!(file, "{event}").expect("append event");
    }
    file.sync_all().expect("flush events log");
}

fn collision_row(db_path: &Path, bead_id: &str) -> Option<(Option<String>, Option<String>)> {
    let conn = Connection::open(db_path).expect("open fleet database");
    conn.query_row(
        "SELECT claimed_at, worker FROM collision_index WHERE bead_id = ?1",
        [bead_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .optional()
    .expect("read collision index")
}

async fn wait_for_collision_row(db_path: &Path, bead_id: &str) -> (Option<String>, Option<String>) {
    for _ in 0..100 {
        if let Some(row) = collision_row(db_path, bead_id) {
            return row;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("timed out waiting for collision row for {bead_id}");
}

async fn wait_for_collision_row_to_disappear(db_path: &Path, bead_id: &str) {
    for _ in 0..100 {
        if collision_row(db_path, bead_id).is_none() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("timed out waiting for collision row removal for {bead_id}");
}

fn assert_database_is_healthy(db_path: &Path) {
    let conn = Connection::open(db_path).expect("open fleet database for integrity check");
    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .expect("run SQLite integrity check");
    assert_eq!(integrity, "ok");

    let duplicate_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) - COUNT(DISTINCT bead_id) FROM collision_index",
            [],
            |row| row.get(0),
        )
        .expect("check collision index uniqueness");
    assert_eq!(
        duplicate_count, 0,
        "collision index must not contain duplicates"
    );

    let last_event_at: Option<String> = conn
        .query_row(
            "SELECT last_event_at FROM project_status WHERE project = ?1",
            [PROJECT],
            |row| row.get(0),
        )
        .expect("read project activity timestamp");
    assert!(
        last_event_at
            .as_deref()
            .is_some_and(|timestamp| DateTime::parse_from_rfc3339(timestamp).is_ok()),
        "project activity timestamps must remain valid RFC3339 values: {last_event_at:?}"
    );
}

#[tokio::test]
#[serial]
async fn empty_claimed_at_survives_claim_and_terminal_event_workflow() {
    let workflow = Workflow::new(&["bd-empty-claim"]).await;

    append_events(
        &workflow.events_path,
        &[r#"{"event":"claim","ts":"","worker":"worker-empty","bead":"bd-empty-claim"}"#],
    );
    let (claimed_at, worker) = wait_for_collision_row(&workflow.db_path, "bd-empty-claim").await;

    assert_eq!(worker.as_deref(), Some("worker-empty"));
    let fallback = claimed_at.expect("empty claimed_at should receive a safe fallback");
    assert!(
        DateTime::parse_from_rfc3339(&fallback).is_ok(),
        "fallback claimed_at must remain parseable: {fallback}"
    );
    assert_eq!(
        workflow
            .registry
            .get_bead_events("bd-empty-claim")
            .await
            .len(),
        1
    );

    append_events(
        &workflow.events_path,
        &[
            r#"{"event":"close","ts":"2026-04-21T18:42:11Z","worker":"worker-empty","bead":"bd-empty-claim"}"#,
        ],
    );
    wait_for_collision_row_to_disappear(&workflow.db_path, "bd-empty-claim").await;
    assert_eq!(
        workflow
            .registry
            .get_bead_events("bd-empty-claim")
            .await
            .len(),
        2
    );
    assert_database_is_healthy(&workflow.db_path);
    workflow.stop().await;
}

#[tokio::test]
#[serial]
async fn missing_claimed_at_is_ignored_without_blocking_following_events() {
    let workflow = Workflow::new(&["bd-missing-claim", "bd-after-missing"]).await;

    // `ts` is the source field that becomes collision_index.claimed_at. A
    // missing source field is an invalid event, but must not kill the tailer.
    append_events(
        &workflow.events_path,
        &[
            r#"{"event":"claim","worker":"worker-missing","bead":"bd-missing-claim"}"#,
            r#"{"event":"claim","ts":"2026-04-21T18:42:10Z","worker":"worker-after-missing","bead":"bd-after-missing"}"#,
        ],
    );

    let (claimed_at, worker) = wait_for_collision_row(&workflow.db_path, "bd-after-missing").await;
    assert_eq!(worker.as_deref(), Some("worker-after-missing"));
    assert!(DateTime::parse_from_rfc3339(claimed_at.as_deref().unwrap()).is_ok());
    assert!(
        collision_row(&workflow.db_path, "bd-missing-claim").is_none(),
        "missing claimed_at must not create a partial collision row"
    );
    assert_eq!(
        workflow
            .registry
            .get_bead_events("bd-missing-claim")
            .await
            .len(),
        0,
        "invalid event must not be projected as a claim"
    );
    assert_eq!(
        workflow
            .registry
            .get_bead_events("bd-after-missing")
            .await
            .len(),
        1,
        "the next valid event must still be projected"
    );

    append_events(
        &workflow.events_path,
        &[
            r#"{"event":"release","ts":"2026-04-21T18:42:12Z","worker":"worker-after-missing","bead":"bd-after-missing"}"#,
        ],
    );
    wait_for_collision_row_to_disappear(&workflow.db_path, "bd-after-missing").await;
    assert_database_is_healthy(&workflow.db_path);
    workflow.stop().await;
}

#[tokio::test]
#[serial]
async fn malformed_timestamps_and_terminal_events_work_together() {
    let workflow = Workflow::new(&[
        "bd-valid-combination",
        "bd-empty-combination",
        "bd-malformed-combination",
        "bd-missing-combination",
    ])
    .await;

    append_events(
        &workflow.events_path,
        &[
            r#"{"event":"claim","ts":"2026-04-21T18:42:10Z","worker":"worker-valid","bead":"bd-valid-combination"}"#,
            r#"{"event":"claim","ts":"","worker":"worker-empty","bead":"bd-empty-combination"}"#,
            r#"{"event":"claim","ts":"April 21, 2026","worker":"worker-malformed","bead":"bd-malformed-combination"}"#,
            r#"{"event":"claim","worker":"worker-missing","bead":"bd-missing-combination"}"#,
        ],
    );

    let (valid_claimed_at, _) =
        wait_for_collision_row(&workflow.db_path, "bd-valid-combination").await;
    let (empty_claimed_at, _) =
        wait_for_collision_row(&workflow.db_path, "bd-empty-combination").await;
    let (malformed_claimed_at, _) =
        wait_for_collision_row(&workflow.db_path, "bd-malformed-combination").await;

    assert!(DateTime::parse_from_rfc3339(valid_claimed_at.as_deref().unwrap()).is_ok());
    assert!(DateTime::parse_from_rfc3339(empty_claimed_at.as_deref().unwrap()).is_ok());
    assert!(DateTime::parse_from_rfc3339(malformed_claimed_at.as_deref().unwrap()).is_ok());
    assert!(collision_row(&workflow.db_path, "bd-missing-combination").is_none());

    // A malformed terminal timestamp must not prevent cleanup of the claim.
    append_events(
        &workflow.events_path,
        &[
            r#"{"event":"release","ts":"not-a-timestamp","worker":"worker-malformed","bead":"bd-malformed-combination"}"#,
            r#"{"event":"close","ts":"","worker":"worker-empty","bead":"bd-empty-combination"}"#,
        ],
    );
    wait_for_collision_row_to_disappear(&workflow.db_path, "bd-malformed-combination").await;
    wait_for_collision_row_to_disappear(&workflow.db_path, "bd-empty-combination").await;

    assert!(collision_row(&workflow.db_path, "bd-valid-combination").is_some());
    assert_eq!(
        workflow
            .registry
            .get_bead_events("bd-malformed-combination")
            .await
            .len(),
        2
    );
    assert_database_is_healthy(&workflow.db_path);
    workflow.stop().await;
}

#[tokio::test]
#[serial]
async fn long_multibyte_malformed_claim_timestamp_does_not_crash_projection() {
    let workflow = Workflow::new(&["bd-multibyte-claim", "bd-after-multibyte"]).await;
    let malformed = format!("{}🔥not-a-timestamp", "é".repeat(49));
    let event = format!(
        r#"{{"event":"claim","ts":"{}","worker":"worker-multibyte","bead":"bd-multibyte-claim"}}"#,
        malformed
    );
    let following = r#"{"event":"claim","ts":"2026-04-21T18:42:10Z","worker":"worker-after-multibyte","bead":"bd-after-multibyte"}"#;

    append_events(&workflow.events_path, &[&event, following]);

    let (claimed_at, _) = wait_for_collision_row(&workflow.db_path, "bd-multibyte-claim").await;
    assert!(DateTime::parse_from_rfc3339(claimed_at.as_deref().unwrap()).is_ok());

    let (following_claimed_at, _) =
        wait_for_collision_row(&workflow.db_path, "bd-after-multibyte").await;
    assert!(DateTime::parse_from_rfc3339(following_claimed_at.as_deref().unwrap()).is_ok());
    assert_database_is_healthy(&workflow.db_path);
    workflow.stop().await;
}
