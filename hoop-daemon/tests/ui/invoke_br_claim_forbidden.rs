// This file MUST fail to compile under `create-only-write`.
// It proves that `invoke_br_write` with Claim variant does not exist.

use hoop_daemon::br_verbs::{invoke_br_write, WriteVerb};

fn main() {
    let _ = invoke_br_write(WriteVerb::Claim, &["bd-test"]);
}
