// Test to understand validation behavior
use hoop_daemon::config_resolver::{resolve_from_raw, CliOverrides};

fn main() {
    let yaml = r#"
schema_version: "1.0.0"
audit:
  retention_days: "90"
"#;

    println!("Testing YAML with wrong type for audit.retention_days");
    let cli = CliOverrides {
        bind_addr: None,
        allow_br_mismatch: None,
    };
    let result = resolve_from_raw(cli, yaml);
    println!("Result: {:?}", result);
}
