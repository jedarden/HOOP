// This file MUST fail to compile under `create-only-write`.
// It proves that `invoke_br_write` does not exist when the feature is active.

use hoop_mcp::br_verbs::{invoke_br_write, WriteVerb};

fn main() {
    let _ = invoke_br_write(WriteVerb::Close, &["bd-test"]);
}
