// testrepo synthetic module: network/tcp
// Auto-generated for integration testing

#![allow(dead_code)]

pub fn tcp_demo() -> String {
    format!("Demo from {} module", "network/tcp")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp() {
        assert_eq!(tcp_demo(), format!("Demo from {} module", "network/tcp"));
    }
}
