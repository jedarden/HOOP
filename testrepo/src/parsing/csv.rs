// testrepo synthetic module: parsing/csv
// Auto-generated for integration testing

#![allow(dead_code)]

pub fn csv_demo() -> String {
    format!("Demo from {} module", "parsing/csv")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv() {
        assert_eq!(csv_demo(), format!("Demo from {} module", "parsing/csv"));
    }
}
