// testrepo synthetic module: async/task
// Auto-generated for integration testing

#![allow(dead_code)]

pub fn task_demo() -> String {
    format!("Demo from {} module", "async/task")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task() {
        assert_eq!(task_demo(), format!("Demo from {} module", "async/task"));
    }
}
