// testrepo synthetic module: parsing/json
// Auto-generated for integration testing

#![allow(dead_code)]

pub fn json_demo() -> String {
    format!("Demo from {} module", "parsing/json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json() {
        assert_eq!(json_demo(), format!("Demo from {} module", "parsing/json"));
    }
}
