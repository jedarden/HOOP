use hoop_schema::HoopConfig;

fn main() {
    // Test agent.model with wrong type
    let yaml = r#"
schema_version: "1.0.0"
agent:
  adapter: claude
  model: 12345
"#;
    println!("Testing agent.model with integer...");
    let result: Result<HoopConfig, _> = serde_yaml::from_str(yaml);
    match result {
        Ok(v) => println!("  OK: agent={:?}", v.agent),
        Err(e) => {
            println!("  Error: {}", e.to_string());
            if let Some(loc) = e.location() {
                println!("  Line: {}, Column: {}", loc.line(), loc.column());
            }
        }
    }

    // Test agent_extensions.scripts with wrong type (array instead of string)
    let yaml2 = r#"
schema_version: "1.0.0"
agent_extensions:
  scripts:
    - /path/to/scripts
"#;
    println!("\nTesting agent_extensions.scripts with array...");
    let result2: Result<HoopConfig, _> = serde_yaml::from_str(yaml2);
    match result2 {
        Ok(v) => println!("  OK: agent_extensions={:?}", v.agent_extensions),
        Err(e) => {
            println!("  Error2: {}", e.to_string());
            if let Some(loc) = e.location() {
                println!("  Line: {}, Column: {}", loc.line(), loc.column());
            }
        }
    }

    // Test metrics.enabled with wrong type (string instead of boolean)
    let yaml3 = r#"
schema_version: "1.0.0"
metrics:
  enabled: "true"
"#;
    println!("\nTesting metrics.enabled with string...");
    let result3: Result<HoopConfig, _> = serde_yaml::from_str(yaml3);
    match result3 {
        Ok(v) => println!("  OK: metrics={:?}", v.metrics),
        Err(e) => {
            println!("  Error3: {}", e.to_string());
            if let Some(loc) = e.location() {
                println!("  Line: {}, Column: {}", loc.line(), loc.column());
            }
        }
    }

    // Test audit.hash_chain with wrong type (string instead of boolean)
    let yaml4 = r#"
schema_version: "1.0.0"
audit:
  hash_chain: "true"
"#;
    println!("\nTesting audit.hash_chain with string...");
    let result4: Result<HoopConfig, _> = serde_yaml::from_str(yaml4);
    match result4 {
        Ok(v) => println!("  OK: audit={:?}", v.audit),
        Err(e) => {
            println!("  Error4: {}", e.to_string());
            if let Some(loc) = e.location() {
                println!("  Line: {}, Column: {}", loc.line(), loc.column());
            }
        }
    }

    // Test server.bind_addr with wrong type (integer instead of string)
    let yaml5 = r#"
schema_version: "1.0.0"
server:
  bind_addr: 3000
"#;
    println!("\nTesting server.bind_addr with integer...");
    let result5: Result<HoopConfig, _> = serde_yaml::from_str(yaml5);
    match result5 {
        Ok(v) => println!("  OK: server={:?}", v.server),
        Err(e) => {
            println!("  Error5: {}", e.to_string());
            if let Some(loc) = e.location() {
                println!("  Line: {}, Column: {}", loc.line(), loc.column());
            }
        }
    }
}
