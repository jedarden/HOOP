use serde_yaml::Value;

fn main() {
    let yaml = r#"schema_version: "1.0.0"
agent:
  adapter: [invalid, yaml
"#;
    
    match serde_yaml::from_str::<Value>(yaml) {
        Ok(v) => {
            println!("Parsed OK: {:?}", v);
            // Try to access the adapter field
            if let Some(agent) = v.get("agent") {
                if let Some(adapter) = agent.get("adapter") {
                    println!("adapter value: {:?}", adapter);
                }
            }
        }
        Err(e) => {
            println!("Parse error: {}", e);
            if let Some(loc) = e.location() {
                println!("Location: line {}, column {}", loc.line(), loc.column());
            }
        }
    }
}
