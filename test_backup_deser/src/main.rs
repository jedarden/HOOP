use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct BackupFileConfig {
    pub endpoint: String,
    pub bucket: String,
    pub prefix: String,
    #[serde(default = "default_schedule")]
    pub schedule: String,
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
    #[serde(default)]
    pub encryption: bool,
}

fn default_schedule() -> String {
    "0 4 * * *".to_string()
}

fn default_retention_days() -> u32 {
    30
}

fn main() {
    println!("Testing minimal YAML deserialization...\n");

    // Test 1: Minimal config (defaults should be applied)
    let yaml_input = "endpoint: https://s3.example.com\nbucket: my-bucket\nprefix: backups/";
    println!("YAML input:\n{}\n", yaml_input);

    let yaml_value: serde_yaml::Value = match serde_yaml::from_str(yaml_input) {
        Ok(v) => {
            println!("YAML parsed successfully: {:#?}", v);
            v
        }
        Err(e) => {
            eprintln!("YAML parsing failed: {}", e);
            return;
        }
    };

    let json_value: serde_json::Value = match serde_json::to_value(&yaml_value) {
        Ok(v) => {
            println!("Converted to JSON: {}", v);
            v
        }
        Err(e) => {
            eprintln!("YAML→JSON conversion failed: {}", e);
            return;
        }
    };

    let config: BackupFileConfig = match serde_json::from_value(json_value) {
        Ok(c) => {
            println!("Successfully deserialized: {:#?}", c);
            c
        }
        Err(e) => {
            eprintln!("JSON deserialization failed: {}", e);
            return;
        }
    };

    println!("\n--- Minimal Config Test Results ---");
    println!("endpoint: {}", config.endpoint);
    println!("bucket: {}", config.bucket);
    println!("prefix: {}", config.prefix);
    println!("schedule: {} (expected: '0 4 * * *')", config.schedule);
    println!("retention_days: {} (expected: 30)", config.retention_days);
    println!("encryption: {} (expected: false)", config.encryption);

    println!("\n\nTesting full YAML deserialization...\n");

    // Test 2: Full config (all fields specified)
    let yaml_input_full = "endpoint: https://s3.example.com\nbucket: my-bucket\nprefix: backups/\nschedule: '*/30 * * * *'\nretention_days: 14\nencryption: true";
    println!("YAML input:\n{}\n", yaml_input_full);

    let yaml_value_full: serde_yaml::Value = match serde_yaml::from_str(yaml_input_full) {
        Ok(v) => {
            println!("YAML parsed successfully: {:#?}", v);
            v
        }
        Err(e) => {
            eprintln!("YAML parsing failed: {}", e);
            return;
        }
    };

    let json_value_full: serde_json::Value = match serde_json::to_value(&yaml_value_full) {
        Ok(v) => {
            println!("Converted to JSON: {}", v);
            v
        }
        Err(e) => {
            eprintln!("YAML→JSON conversion failed: {}", e);
            return;
        }
    };

    let config_full: BackupFileConfig = match serde_json::from_value(json_value_full) {
        Ok(c) => {
            println!("Successfully deserialized: {:#?}", c);
            c
        }
        Err(e) => {
            eprintln!("JSON deserialization failed: {}", e);
            return;
        }
    };

    println!("\n--- Full Config Test Results ---");
    println!("endpoint: {}", config_full.endpoint);
    println!("bucket: {}", config_full.bucket);
    println!("prefix: {}", config_full.prefix);
    println!(
        "schedule: {} (expected: '*/30 * * * *')",
        config_full.schedule
    );
    println!(
        "retention_days: {} (expected: 14)",
        config_full.retention_days
    );
    println!("encryption: {} (expected: true)", config_full.encryption);
}
