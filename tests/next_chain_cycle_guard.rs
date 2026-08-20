//! Regression test for an unbounded loop on a cyclic `next` chain:
//! `boring/linker.br`'s `resolve_stack` (the entry point every script body
//! in this project links through -- green-flag hats, broadcasts, clones,
//! procedure bodies, every `if`/loop substack) walked a block chain by
//! `next` pointers with no cycle detection at all. A hand-forged
//! `project.json` where a block's `next` -- directly or transitively --
//! points back at a block already walked in the same chain made this
//! `while let` loop run forever: `result.push(...)` growing without bound
//! (OOM) or the whole load simply hanging, reachable just by opening the
//! malicious project.
//!
//! `assets/next_chain_cycle.json` encodes exactly that: `Sprite1`'s
//! green-flag script runs `blockX` (`set cycleDone to 1`) then `blockY`
//! (`set cycleDone2 to 2`), whose own `next` points back to `blockX` --
//! a genuine two-block cycle in the `next` chain itself (not a reporter
//! input, see `reporter_cycle_guard.rs`'s own sibling test for that
//! separate bug).
//!
//! Fixed in `boring/linker.br` (`resolve_stack`'s own `visited` set):
//! once a `next` id reappears, the walk stops (keeping whatever was
//! already resolved) instead of looping forever -- so this now loads (and
//! runs both statements, exactly once each) rather than hanging.
use scratch_boring::run_greenflag_and_report_json;
use std::sync::Arc;

#[test]
fn a_cyclic_next_chain_loads_and_runs_each_statement_exactly_once() {
    let path: Arc<str> = Arc::from("assets/next_chain_cycle.json");
    let get = |name: &'static str| run_greenflag_and_report_json(path.clone(), Arc::from(name));

    // Both assertions below are only reachable at all if `resolve_stack`
    // terminated -- before the fix, this test would simply never return
    // (an infinite loop, not a crash), timing out the whole test binary.
    assert_eq!(get("cycleDone").as_ref(), "1", "blockX should have run exactly once");
    assert_eq!(get("cycleDone2").as_ref(), "2", "blockY should have run exactly once, not looped back into blockX again");
}
