//! Regression test for unbounded native recursion on a cyclic reporter
//! (block-input) graph: `boring/linker.br`'s `resolve_expr_block` ->
//! `dispatch_reporter` -> `resolve_input` -> `resolve_expr_block` mutual
//! recursion had no cycle detection or depth limit at all. A hand-forged
//! `project.json` where a reporter block's own input (by block id)
//! transitively points back at itself (block A's `NUM1` is block B, block
//! B's own `NUM1` is block A right back) made resolving either one recurse
//! forever -- a real stack overflow crashing the whole load, reachable
//! before any script ever actually runs, just from opening the malicious
//! project.
//!
//! `assets/reporter_input_cycle.json` encodes exactly that: `Sprite1`'s
//! green-flag script sets `cycleResult` to `blockA + 1`, where `blockA`
//! is `blockB + 1` and `blockB` is `blockA + 1` right back (`operator_add`,
//! `NUM1` referencing the other block by id, `NUM2` a literal `1`) -- a
//! genuine two-block cycle, unreachable any other way (neither block sits
//! on any script's own `next` chain, only referenced through an input).
//!
//! Fixed in `boring/linker.br` (`MAX_REPORTER_DEPTH`/`resolve_expr_block`'s
//! own `depth` parameter, threaded through `dispatch_reporter`/
//! `resolve_input`): past 500 levels deep, resolution falls back to a
//! plain `Literal(0)` instead of recursing further, so this now loads (and
//! the test below completes) rather than crashing with a stack overflow.
//! The exact resulting value isn't the point (it's a byproduct of
//! `MAX_REPORTER_DEPTH`'s own value) -- what matters is that loading a
//! project with this shape returns at all instead of overflowing the
//! stack.

use scratch_boring::run_greenflag_and_report_json;
use std::sync::Arc;

#[test]
fn a_cyclic_reporter_input_graph_loads_without_a_stack_overflow() {
    let path: Arc<str> = Arc::from("assets/reporter_input_cycle.json");
    let result = run_greenflag_and_report_json(path, Arc::from("cycleResult"));

    // The real assertion is simply that this line above returned at all --
    // before the fix, this would have crashed the whole test process with
    // a stack overflow well before ever reaching here. A non-empty numeric
    // result confirms the green-flag script actually ran to completion
    // (the cutoff's `Literal(0)` fallback still lets `operator_add` finish
    // evaluating normally) rather than merely failing to load.
    assert!(!result.is_empty(), "expected the cyclic project to load and run, got empty (failed-to-load) result");
    assert!(
        result.parse::<f64>().is_ok(),
        "expected cycleResult to be a real number after evaluating the (depth-cut-off) Add chain, got {result:?}"
    );
}
