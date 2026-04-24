// This file MUST fail to compile under `create-only-write`.
// It proves that `invoke_br_write` with Update variant does not exist.

use hoop_mcp::br_verbs::{invoke_br_write, WriteVerb};

fn main() {
    let _ = invoke_br_write(WriteVerb::Update, &["bd-test", "--body", "x"]);
}
