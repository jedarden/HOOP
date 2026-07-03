// Test to check if invoke_br_write is available under create-only-write
// This SHOULD fail to compile when create-only-write is active

#[cfg(feature = "create-only-write")]
fn test_invoke_br_write_not_available() {
    // This line should cause a compilation error because invoke_br_write
    // should not exist when create-only-write is active
    use hoop_daemon::br_verbs::invoke_br_write;
    let _ = invoke_br_write(hoop_daemon::br_verbs::WriteVerb::Close, &[]);
}

fn main() {
    #[cfg(feature = "create-only-write")]
    test_invoke_br_write_not_available();
}
