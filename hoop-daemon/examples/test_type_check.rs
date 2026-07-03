// Test to understand validation behavior
use hoop_daemon::config_resolver::validate_config_strict;

fn main() {
    let yaml = r#"
schema_version: "1.0.0"
audit:
  retention_days: "90"
"#;

    println!("Testing YAML with wrong type for audit.retention_days");
    let result = validate_config_strict(yaml);
    println!("Result: {:?}", result);
}
