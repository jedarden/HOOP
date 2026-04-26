//! Example 1 - Demonstrates TestRepo functionality

fn main() {
    println!("Example 1: Basic usage");
    let data = vec![1, 2, 3, 4, 5];
    let sum: i32 = data.iter().sum();
    println!("Sum: {}", sum);
}
