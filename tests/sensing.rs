//! End-to-end test for the two Sensing blocks that have a genuinely correct
//! headless answer right now (`sensing_username`, `sensing_answer`) -- see
//! `dispatch_reporter`'s own doc comment in `boring/scratch.br` for why
//! every other Sensing block is deliberately not implemented yet (touching/
//! distance-to need rendering+sprite geometry, keyboard/mouse need real
//! input, `ask and wait` needs the pausable scheduler -- none faked with a
//! wrong shortcut).
//!
//! Fixture (`assets/sensing_trivial.json`): `set user to (username)`, `set
//! ans to (answer)`. Both variables start as `"unset"` so the assertions
//! prove the blocks actually ran (not just that the variable kept its
//! initial value).

use scratch_boring::run_greenflag_and_report_json;
use std::sync::Arc;

#[test]
fn username_and_answer_report_empty_string() {
    let path: Arc<str> = Arc::from("assets/sensing_trivial.json");
    let get = |name: &'static str| run_greenflag_and_report_json(path.clone(), Arc::from(name));

    assert_eq!(get("user").as_ref(), "", "sensing_username has no real user context headless");
    assert_eq!(get("ans").as_ref(), "", "sensing_answer with no ask-and-wait ever run");
}
