// testrepo synthetic module: crypto/aes
// Auto-generated for integration testing

#![allow(dead_code)]

pub fn aes_demo() -> String {
    format!("Demo from {} module", "crypto/aes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes() {
        assert_eq!(aes_demo(), format!("Demo from {} module", "crypto/aes"));
    }
}
