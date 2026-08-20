//! Regression test for an unguarded arithmetic underflow on `isize::MIN`
//! as a list-index literal: `boring/runtime.br`'s `resolve_list_index`
//! computed `let zero_based = n - 1` where `n` comes straight from
//! `to_isize_checked()` -- which happily returns `isize::MIN` itself for
//! the literal `-9223372036854775808` (a valid Scratch number, and not
//! the overflow case that helper already guards against, since it fits
//! `int64` exactly). `n - 1` then underflows: a panic in a debug build,
//! a silent wraparound to `isize::MAX` in release -- reachable just by
//! running `item # of list` (or `insert at`/`replace item of`) with that
//! literal as the index.
//!
//! `assets/list_index_isize_min.json` encodes exactly that: `myList` is
//! `["a", "b", "c"]`, and the green-flag script sets `result` to `item
//! (-9223372036854775808) of myList`.
//!
//! Fixed in `boring/runtime.br` (`resolve_list_index`'s own `int_min`
//! check before subtracting): that exact value now short-circuits to the
//! same `-1` ("not a valid position") this function already returns for
//! every other out-of-range index, which `Lists.item_at` in turn reports
//! as an empty string -- matching real Scratch's own "out of range ->
//! empty string" behavior for `item # of list`, rather than panicking or
//! silently producing a bogus in-range index.
use scratch_boring::run_greenflag_and_report_json;
use std::sync::Arc;

#[test]
fn an_isize_min_list_index_reports_empty_instead_of_underflowing() {
    let path: Arc<str> = Arc::from("assets/list_index_isize_min.json");

    // The real assertion is simply that this line above returned at all,
    // without panicking on the underflow -- and that it resolved to the
    // same "not a valid position" result every other out-of-range index
    // already produces, not a wrapped-around bogus in-range index.
    let result = run_greenflag_and_report_json(path, Arc::from("result"));
    assert_eq!(
        result.as_ref(),
        "",
        "an isize::MIN index is never valid; `item # of list` should report empty string, same as any other out-of-range index"
    );
}
