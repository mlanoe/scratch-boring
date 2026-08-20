//! End-to-end test for Procedures ("My Blocks") -- the single biggest gap
//! jscratch left unimplemented entirely (see README.md and CLAUDE.md's
//! `scratch-boring` section). No real `.sb3` fixture with a custom block
//! was available, so `assets/procedures_nested.json` is hand-authored
//! directly against the real sb3 JSON shape (raw `project.json`, loaded via
//! `run_greenflag_and_report_json` -- see that fixture file for the full
//! block-by-block layout).
//!
//! The fixture defines two custom blocks and calls them nested:
//!   outer(x):  inner(x + 1); set result to x
//!   inner(y):  set temp to y
//!   green flag: outer(5)
//!
//! Expected: inner(6) sets temp=6, then control returns to outer's own
//! (still-bound) frame where `x` is still 5, so result=5. This specifically
//! exercises that a nested procedure call's `Frame` doesn't corrupt or
//! leak into the caller's own frame once the nested call returns -- the
//! same call-frame correctness a real interpreter needs for recursion,
//! even though this particular fixture doesn't recurse (building a
//! recursive-with-a-real-base-case fixture needs `control_if`/a comparison
//! operator, neither implemented yet -- see README.md's roadmap).

use scratch_boring::run_greenflag_and_report_json;
use std::sync::Arc;

#[test]
fn nested_procedure_calls_keep_each_call_frame_isolated() {
    let path: Arc<str> = Arc::from("assets/procedures_nested.json");

    let temp = run_greenflag_and_report_json(path.clone(), Arc::from("temp"));
    assert_eq!(temp.as_ref(), "6", "inner(x + 1) = inner(6) should set temp to 6");

    let result = run_greenflag_and_report_json(path, Arc::from("result"));
    assert_eq!(
        result.as_ref(),
        "5",
        "after the nested inner() call returns, outer's own frame (x=5) must still be intact"
    );
}
