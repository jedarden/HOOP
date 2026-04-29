//! Populate testrepo with load test data
//!
//! This binary generates synthetic bead data for performance budget testing.
//! It creates the directory structure in testrepo/load-test-data/ with:
//! - 20 projects (configurable)
//! - 5 workers per project (configurable)
//! - 300 beads per worker (configurable)
//!
//! Usage:
//!   cargo run --example populate-testrepo
//!
//! Environment variables:
//!   HOOP_LOAD_PROJECTS    - number of projects (default: 20)
//!   HOOP_LOAD_WORKERS     - workers per project (default: 5)
//!   HOOP_LOAD_BEADS       - beads per worker (default: 300)
//!   TESTREPO_PATH         - path to testrepo (default: ../../testrepo)

use std::path::PathBuf;

use hoop_daemon::load_test::{populate_testrepo, LoadTestConfig};

fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let config = LoadTestConfig::default();

    // Get testrepo path
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let default_testrepo = manifest_dir
        .parent()
        .expect("workspace root is parent of hoop-daemon/")
        .join("testrepo");

    let testrepo_path: PathBuf = std::env::var("TESTREPO_PATH")
        .map(PathBuf::from)
        .unwrap_or(default_testrepo);

    println!("=== Populate testrepo with Load Test Data ===");
    println!("Target: {}", testrepo_path.display());
    println!();
    println!("Configuration:");
    println!("  Projects: {}", config.num_projects);
    println!("  Workers per project: {}", config.workers_per_project);
    println!("  Beads per worker: {}", config.beads_per_worker);
    println!("  Total beads: {}", config.total_beads());
    println!();

    // Populate testrepo
    populate_testrepo(config, &testrepo_path)?;

    println!("✓ testrepo populated successfully");
    println!();
    println!("Generated data location:");
    println!("  {}/load-test-data/", testrepo_path.display());
    println!();
    println!("Run load test with:");
    println!("  cargo test --package hoop-daemon --test load_test_integration load_test_ci_performance_budgets -- --nocapture");

    Ok(())
}
