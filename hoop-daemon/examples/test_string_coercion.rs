use serde_yaml;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Test {
    name: String,
}

fn main() {
    let yaml = r#"
name: 42
"#;
    let result: Result<Test, _> = serde_yaml::from_str(yaml);
    match result {
        Ok(v) => println!("Integer coerced to string: {:?}", v),
        Err(e) => println!("Integer rejected: {}", e),
    }

    let yaml2 = r#"
name: "42"
"#;
    let result2: Result<Test, _> = serde_yaml::from_str(yaml2);
    match result2 {
        Ok(v) => println!("String as expected: {:?}", v),
        Err(e) => println!("String rejected: {}", e),
    }
}
