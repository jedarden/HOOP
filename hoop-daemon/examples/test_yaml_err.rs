fn main() {
    let yaml = r#"
projects:
  - name: 42
    path: /tmp/exists
"#;
    let result: Result<hoop_schema::ProjectsRegistry, _> = serde_yaml::from_str(yaml);
    match result {
        Ok(v) => println!("OK: {:?}", v),
        Err(e) => {
            println!("Error: {:?}", e.to_string());
            if let Some(loc) = e.location() {
                println!("Line: {}, Column: {}", loc.line(), loc.column());
            }
        }
    }

    // Also test missing field
    let yaml2 = r#"
projects:
  - path: /tmp/exists
"#;
    let result2: Result<hoop_schema::ProjectsRegistry, _> = serde_yaml::from_str(yaml2);
    match result2 {
        Ok(v) => println!("OK2: {:?}", v),
        Err(e) => {
            println!("Error2: {:?}", e.to_string());
        }
    }
}
