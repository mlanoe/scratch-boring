//! End-to-end test for `sensing_dayssince2000`, ported from real
//! scratch-vm's own `Scratch3SensingBlocks.daysSince2000()` -- see
//! `interpreter.br`'s own `DaysSince2000` case for the formula and the
//! hand-worked algebra showing it needs no local-timezone read at all,
//! despite the JS source it's ported from constructing a *local*-midnight
//! `Date` and applying a DST correction: that correction exactly cancels
//! the local UTC offset of the fixed start date, leaving a plain
//! "milliseconds elapsed since the UTC instant 2000-01-01T00:00:00Z,
//! divided by a day" -- the same value for every viewer worldwide at the
//! same real instant.
//!
//! Genuinely non-deterministic (depends on the real moment the test
//! runs), same technique `tests/sensing_current.rs` already established:
//! an independent `Utc::now()` reading taken immediately before and after
//! running the green-flag script, asserting the reported value falls
//! within that (necessarily tiny) window.
//!
//! `assets/sensing_dayssince2000.json`: one green-flag script setting a
//! Stage variable `result` to the reporter's own value.

use chrono::Utc;
use scratch_boring::run_greenflag_and_report_json;
use std::sync::Arc;

const MS_PER_DAY: f64 = 86_400_000.0;
const EPOCH_2000_UTC_MS: f64 = 946_684_800_000.0;

fn days_since_2000_at(now_ms: i64) -> f64 {
    (now_ms as f64 - EPOCH_2000_UTC_MS) / MS_PER_DAY
}

#[test]
fn sensing_dayssince2000_reports_real_elapsed_days_since_a_fixed_utc_instant() {
    let path: Arc<str> = Arc::from("assets/sensing_dayssince2000.json");

    let before = days_since_2000_at(Utc::now().timestamp_millis());
    let reported: f64 = run_greenflag_and_report_json(path, Arc::from("result"))
        .parse()
        .unwrap_or_else(|_| panic!("expected a plain float"));
    let after = days_since_2000_at(Utc::now().timestamp_millis());

    assert!(
        reported >= before && reported <= after,
        "reported {reported} should fall within [{before}, {after}] -- the real elapsed-days window the test itself bracketed the block's own read with"
    );
    // Sanity bound unrelated to timing precision: today is real-world well
    // past 2000, and nowhere near 100 years out yet either.
    assert!(reported > 9000.0 && reported < 36525.0, "reported {reported} should be a plausible days-since-2000 value (roughly year 2024 through 2099)");
}
