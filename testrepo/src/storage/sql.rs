// testrepo synthetic module: storage/sql
// Auto-generated for integration testing

#![allow(dead_code)]

pub fn sql_demo() -> String {
    format!("Demo from {} module", "storage/sql")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql() {
        assert_eq!(sql_demo(), format!("Demo from {} module", "storage/sql"));
    }
}
