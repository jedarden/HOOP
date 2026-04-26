//! Integration test integration_5

#[test]
fn test_integration_5_basic() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_integration_5_advanced() {
    let result = vec![1, 2, 3].iter().sum::<i32>();
    assert_eq!(result, 6);
}

#[tokio::test]
async fn test_integration_5_async() {
    let _ = tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    assert!(true);
}
