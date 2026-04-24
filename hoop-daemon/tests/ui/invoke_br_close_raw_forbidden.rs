// This file MUST fail to compile under `create-only-write`.
// It proves that even a raw call to the Close variant is impossible.

use hoop_daemon::br_verbs::invoke_br_write;

fn main() {
    let _ = invoke_br_write(hoop_daemon::br_verbs::WriteVerb::Close, &[]);
}
