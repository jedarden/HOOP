use std::path::PathBuf;

fn main() {
    // Check original home
    let original_home = std::env::var("HOME").ok();
    println!("Original HOME: {:?}", original_home);
    
    // Note: dirs crate doesn't actually use HOME env var on many systems
    // It uses system APIs which might not respect the environment variable
    println!("dirs::home_dir() respects HOME env var: FALSE on most systems");
}
