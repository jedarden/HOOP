use hoop_schema::HoopConfig;
use hoop_daemon::config_resolver::ConfigError;

fn main() {
    // Test 1: Wrong type for schema_version
    let yaml1 = r#"
schema_version: 1
"#;
    let result1: Result<HoopConfig, _> = serde_yaml::from_str(yaml1);
    match result1 {
        Ok(v) => println!("Unexpected success"),
        Err(yaml_err) => {
            println!("Test 1 - schema_version: 1");
            println!("Raw serde_yaml error: {}", yaml_err);
            println!("Location: {:?}", yaml_err.location());
            let config_err = ConfigError::from_yaml(&yaml_err);
            println!("ConfigError field: {:?}", config_err.field);
            println!("ConfigError expected: {:?}", config_err.expected);
            println!("ConfigError got: {:?}", config_err.got);
        }
    }

    // Test 2: Wrong type for agent.adapter
    let yaml2 = r#"
schema_version: "1.0.0"
agent:
  adapter: 42
"#;
    let result2: Result<HoopConfig, _> = serde_yaml::from_str(yaml2);
    match result2 {
        Ok(v) => println!("YAML2 success unexpectedly"),
        Err(yaml_err) => {
            println!("\nTest 2 - agent.adapter: 42");
            println!("Raw serde_yaml error: {}", yaml_err);
            println!("Location: {:?}", yaml_err.location());
            let config_err = ConfigError::from_yaml(&yaml_err);
            println!("ConfigError field: {:?}", config_err.field);
            println!("ConfigError expected: {:?}", config_err.expected);
            println!("ConfigError got: {:?}", config_err.got);
        }
    }

    // Test 3: Wrong type for agent.model
    let yaml3 = r#"
schema_version: "1.0.0"
agent:
  adapter: claude
  model: 12345
"#;
    let result3: Result<HoopConfig, _> = serde_yaml::from_str(yaml3);
    match result3 {
        Ok(v) => println!("YAML3 success unexpectedly"),
        Err(yaml_err) => {
            println!("\nTest 3 - agent.model: 12345");
            println!("Raw serde_yaml error: {}", yaml_err);
            println!("Location: {:?}", yaml_err.location());
            let config_err = ConfigError::from_yaml(&yaml_err);
            println!("ConfigError field: {:?}", config_err.field);
            println!("ConfigError expected: {:?}", config_err.expected);
            println!("ConfigError got: {:?}", config_err.got);
        }
    }
}
