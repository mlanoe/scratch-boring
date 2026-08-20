//! Regression test for an unbounded frame-stack growth on a directly
//! self-recursive custom block with no loop/wait as a base case:
//! `boring/interpreter.br`'s `step_thread` (`ProcCall` arm) pushed a
//! fresh `ProcCallFrame` on every call with no depth guard at all, unlike
//! `RepeatN`/`RepeatUntilCond`/`Forever` (guarded by `warp_guard`). Since
//! an ordinary (non-warp) `ProcCall` never yields on its own -- pushing
//! the callee frame and continuing the very same `while true:` pass is
//! exactly right for a normal, non-recursive call -- a custom block that
//! calls itself directly with nothing in between would grow `stack`
//! without bound *inside a single `step_thread` call*, never returning to
//! let the next Bevy tick run at all: a real, total freeze of the whole
//! app (not just an abandoned thread), reachable just by opening a
//! project with this shape.
//!
//! `assets/recursive_proc_no_base_case.json` encodes exactly that: a
//! custom block `loopy` whose body increments a Stage variable `runs` by
//! 1 and then calls `loopy` again, directly, with no condition, loop, or
//! wait anywhere in it.
//!
//! Fixed in `boring/interpreter.br` (`proc_call_depth`/
//! `proc_call_depth_cap`, threaded through the `ProcCall` statement arm
//! and the `ProcCallFrame(_)` pop arm): past that many levels of actual
//! nesting, a further call degrades to a no-op (same graceful-degradation
//! precedent as an unresolved procedure name) instead of pushing another
//! frame, so this now loads and finishes (leaving `runs` at some bounded,
//! nonzero count) rather than hanging or growing memory without bound.
use scratch_boring::run_greenflag_and_report_json;
use std::sync::Arc;

#[test]
fn a_directly_self_recursive_custom_block_terminates_instead_of_hanging() {
    let path: Arc<str> = Arc::from("assets/recursive_proc_no_base_case.json");

    // The real assertion is simply that this line above returned at all --
    // before the fix, `loopy` calling itself with no base case would have
    // grown the frame stack without bound inside a single `step_thread`
    // call, hanging (or OOMing) the whole process well before ever
    // reaching here.
    let runs = run_greenflag_and_report_json(path, Arc::from("runs"));

    let runs: f64 = runs.parse().unwrap_or_else(|_| panic!("expected `runs` to be numeric, got {runs:?}"));
    assert!(runs > 0.0, "expected `loopy` to have run at least once, got runs={runs}");
    // Bounded well below what unrestrained recursion would have produced
    // (millions, given `warp_guard_cap`'s own order of magnitude) --
    // proving the depth cap actually kicked in and stopped the recursion,
    // not just that this test process happened to finish in time.
    assert!(runs < 20000.0, "expected the self-recursion to be capped well below 20000, got runs={runs}");
}
