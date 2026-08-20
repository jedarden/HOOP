// Test to understand how serde_yaml parses type mismatches
use serde_yaml;

fn main() {
    let yaml = r#"
schema_version: "1.0.0"
audit:
  retention_days: "90"
"#;

    let yml: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();

    println!("Full YAML: {:#?}", yml);
    println!("Type of yml: {:?}", std::any::type_name_of_val(&yml));

    // Navigate to audit.retention_days
    let audit = yml.get("audit").unwrap();
    println!("audit: {:#?}", audit);

    let retention_days = audit.get("retention_days").unwrap();
    println!("retention_days: {:#?}", retention_days);
    println!("retention_days is_string: {}", retention_days.is_string());
    println!("retention_days is_number: {}", retention_days.is_number());
    println!(
        "retention_days is_u64: {}",
        retention_days.as_u64().is_some()
    );
    println!("retention_days as_str: {:?}", retention_days.as_str());
    println!("retention_days as_i64: {:?}", retention_days.as_i64());
}
