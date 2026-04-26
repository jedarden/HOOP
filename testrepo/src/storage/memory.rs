// testrepo synthetic module: storage/memory
// Auto-generated for integration testing

#![allow(dead_code)]

pub fn memory_demo() -> String {
    format!("Demo from {} module", "storage/memory")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory() {
        assert_eq!(memory_demo(), format!("Demo from {} module", "storage/memory"));
    }
}
