//! Load test runner binary
//!
//! Usage:
//!   cargo run --example load-test-runner -- --url http://localhost:3000
//!
//! Environment variables:
//!   HOOP_LOAD_PROJECTS     - number of projects (default: 20)
//!   HOOP_LOAD_WORKERS      - workers per project (default: 5)
//!   HOOP_LOAD_BEADS        - beads per worker (default: 200)
//!   HOOP_LOAD_CADENCE_MS   - delay between events (default: 10)
//!
//! Exit codes:
//!   0 - All performance budgets satisfied
//!   1 - Performance budget violations
//!   2 - Error running test

use std::time::Duration;

use hoop_daemon::load_test::{run_load_test, LoadTestConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    let mut url = "http://localhost:3000".to_string();
    let mut verbose = false;

    // Parse args
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--url" | "-u" => {
                if i + 1 < args.len() {
                    url = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("--url requires an argument");
                    std::process::exit(2);
                }
            }
            "--verbose" | "-v" => {
                verbose = true;
                i += 1;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                std::process::exit(2);
            }
        }
    }

    let config = LoadTestConfig::default();

    println!("=== HOOP Load Test Runner ===");
    println!("Target: {}", url);
    println!("Configuration:");
    println!("  Projects: {}", config.num_projects);
    println!("  Workers per project: {}", config.workers_per_project);
    println!("  Beads per worker: {}", config.beads_per_worker);
    println!("  Total beads: {}", config.total_beads());
    println!("  Event cadence: {}ms", config.event_cadence_ms);
    println!();
    println!("Performance budgets:");
    println!("  API latency: <{}ms", config.api_latency_budget_ms);
    println!("  WS fan-out lag: <{}ms", config.ws_fanout_lag_budget_ms);
    println!("  Memory ceiling: <{}MB", config.memory_ceiling_bytes / 1024 / 1024);
    println!();

    // Run the load test
    let start = std::time::Instant::now();
    let report = match tokio::time::timeout(
        Duration::from_secs(300), // 5 minute timeout
        run_load_test(&url, config),
    )
    .await
    {
        Ok(Ok(report)) => report,
        Ok(Err(e)) => {
            eprintln!("Error running load test: {}", e);
            std::process::exit(2);
        }
        Err(_) => {
            eprintln!("Load test timed out after 5 minutes");
            std::process::exit(2);
        }
    };
    let elapsed = start.elapsed();

    println!("{}", report.summary());
    println!();
    println!("Total time: {:?}", elapsed);

    if verbose {
        println!();
        println!("Detailed metrics:");
        println!("  API latencies (ms): {:?}", report.api_latencies);
        println!("  WS fan-out lags (ms): {:?}", report.ws_fanout_lags);
        println!(
            "  Memory samples (MB): {:?}",
            report
                .memory_samples
                .iter()
                .map(|m| m / 1024 / 1024)
                .collect::<Vec<_>>()
        );
    }

    // Exit with appropriate code
    if report.passed {
        println!();
        println!("✓ All performance budgets satisfied");
        std::process::exit(0);
    } else {
        println!();
        println!("✗ Performance budget violations detected");
        std::process::exit(1);
    }
}

fn print_help() {
    println!("HOOP Load Test Runner");
    println!();
    println!("Usage:");
    println!("  cargo run --example load-test-runner -- [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -u, --url <URL>       Daemon URL (default: http://localhost:3000)");
    println!("  -v, --verbose         Show detailed metrics");
    println!("  -h, --help            Show this help");
    println!();
    println!("Environment variables:");
    println!("  HOOP_LOAD_PROJECTS    Number of projects (default: 20)");
    println!("  HOOP_LOAD_WORKERS     Workers per project (default: 5)");
    println!("  HOOP_LOAD_BEADS       Beads per worker (default: 200)");
    println!("  HOOP_LOAD_CADENCE_MS  Delay between events in ms (default: 10)");
}
