//! End-to-end test for `sensing_current` (real local wall-clock date/time),
//! ported from real scratch-vm's own `Scratch3SensingBlocks.current()` --
//! see `interpreter.br`'s own `CurrentDateTime` case for the exact
//! `chrono` calls and the one real conversion (`DAYOFWEEK`).
//!
//! Genuinely non-deterministic (depends on the real wall-clock moment the
//! test runs) -- unlike every other test in this project, this one can't
//! assert fixed expected values. Instead it takes its own independent
//! `chrono::Local::now()` reading, once immediately before running the
//! green-flag script and once immediately after, and checks each reported
//! field falls within that (necessarily very short) window -- robust to
//! the block's own read landing a moment later than either bound, and to
//! a `second`/`minute`/`hour`/(vanishingly rarely) even `date`/`month`/
//! `year` boundary being crossed mid-test.
//!
//! `assets/sensing_current.json`: one green-flag script reading all seven
//! `CURRENTMENU` values (`YEAR`/`MONTH`/`DATE`/`DAYOFWEEK`/`HOUR`/
//! `MINUTE`/`SECOND`) into their own Stage variable, back to back.

use chrono::{DateTime, Datelike, Local, Timelike};
use scratch_boring::run_greenflag_and_report_json;
use std::sync::Arc;

fn parse(s: &str) -> i64 {
    s.parse().unwrap_or_else(|_| panic!("expected a plain integer, got {s:?}"))
}

/// True if `reported` falls within the closed window `[before, after]` --
/// or, if a field wrapped around mid-test (e.g. `SECOND` rolling over
/// 59 -> 0, so `before > after`), within either tail of that wraparound.
fn in_window(reported: i64, before: i64, after: i64) -> bool {
    if before <= after {
        reported >= before && reported <= after
    } else {
        reported >= before || reported <= after
    }
}

#[test]
fn sensing_current_reads_real_local_wall_clock_time() {
    let path: Arc<str> = Arc::from("assets/sensing_current.json");

    let before: DateTime<Local> = Local::now();
    let get = |name: &'static str| run_greenflag_and_report_json(path.clone(), Arc::from(name));
    let year = parse(&get("result_year"));
    let month = parse(&get("result_month"));
    let date = parse(&get("result_date"));
    let dayofweek = parse(&get("result_dayofweek"));
    let hour = parse(&get("result_hour"));
    let minute = parse(&get("result_minute"));
    let second = parse(&get("result_second"));
    let after: DateTime<Local> = Local::now();

    assert!(in_window(year, before.year() as i64, after.year() as i64), "year {year} should fall within [{}, {}]", before.year(), after.year());
    assert!(in_window(month, before.month() as i64, after.month() as i64), "month {month} should fall within [{}, {}]", before.month(), after.month());
    assert!(in_window(date, before.day() as i64, after.day() as i64), "date {date} should fall within [{}, {}]", before.day(), after.day());
    let dow_before = before.weekday().num_days_from_sunday() as i64 + 1;
    let dow_after = after.weekday().num_days_from_sunday() as i64 + 1;
    assert!(in_window(dayofweek, dow_before, dow_after), "dayofweek {dayofweek} (1=Sunday) should fall within [{dow_before}, {dow_after}]");
    assert!(in_window(hour, before.hour() as i64, after.hour() as i64), "hour {hour} should fall within [{}, {}]", before.hour(), after.hour());
    assert!(in_window(minute, before.minute() as i64, after.minute() as i64), "minute {minute} should fall within [{}, {}]", before.minute(), after.minute());
    assert!(in_window(second, before.second() as i64, after.second() as i64), "second {second} should fall within [{}, {}]", before.second(), after.second());
    assert!((1..=7).contains(&dayofweek), "dayofweek must always be in real Scratch's own 1..=7 (Sunday=1) range regardless of the window check above, got {dayofweek}");
}
