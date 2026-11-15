//! Performance regression guard: a loop over a 3000-item list used to clone the
//! whole list on every `add`/`item N of` (O(N^2)). `assets/perf_list_loop.json`
//! builds a 3000-item list then sums it item by item; sum 0..2999 + ... is
//! checked for correctness, and the run must stay well under the (generous,
//! debug-build) time bound.
use scratch_boring::run_greenflag_and_report_json;
use std::sync::Arc;
use std::time::Instant;

#[test]
fn looping_over_a_3000_item_list_is_not_quadratic() {
    let start = Instant::now();
    let sum = run_greenflag_and_report_json(Arc::from("assets/perf_list_loop.json"), Arc::from("sum")).to_string();
    let elapsed = start.elapsed();
    println!("perf_list_loop: {elapsed:?}");
    // items are 0..2999 and the second loop reads indexes 1..3000 (last read is item 3000 = "").
    assert_eq!(sum, "4498500", "sum of items 1..=3000 (0..2999, then an out-of-range read adds 0)");
    assert!(elapsed.as_secs() < 20, "took {elapsed:?}");
}
