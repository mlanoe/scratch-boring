//! Regression test for the real bug found running "flower" (`repeat
//! [pen_stamp, turn right]`) -- the classic real `pen_stamp` pattern --
//! visually: the drawn flower didn't rotate at all, every one of its 24
//! stamps showing the exact same orientation.
//!
//! Root cause: the original `StampRequest` design (a bare target-id
//! list) deferred reading *everything* -- position, direction,
//! rotation style, costume, size -- to drain time (`draw_stamps`, once
//! per tick), rather than snapshotting it at the moment `pen_stamp`
//! itself ran (`Stamp`'s own `exec_simple` case). A script that stamps,
//! then changes state, then stamps again -- all within one tick, since
//! neither `pen_stamp` nor an ordinary motion opcode is a yield point --
//! had every queued stamp read the *same*, final, post-script state at
//! drain time instead of each one's own state at the moment it actually
//! ran. Fixed by moving the snapshot into `Stamp`'s own `exec_simple`
//! case (`StampRequest`'s own doc comment, `runtime.br`).
//!
//! `assets/pen_stamp_snapshot.json`: `pen_stamp` -> `turn right 90` ->
//! `pen_stamp`, no loop needed (both statements run within the very
//! first tick regardless). Before the fix, both requests would carry
//! whatever direction the target ended up at once the whole script
//! finished (180, twice) -- after the fix, the first should carry the
//! sprite's own real direction *at the moment it stamped* (90, real
//! Scratch's own default), the second the post-turn direction (180).

use scratch_boring::run_greenflag_and_report_stamp_directions_json;
use std::sync::Arc;

#[test]
fn two_stamps_around_a_turn_capture_two_different_directions() {
    let path: Arc<str> = Arc::from("assets/pen_stamp_snapshot.json");
    let result = run_greenflag_and_report_stamp_directions_json(path);
    assert_eq!(result.as_ref(), "90,180", "each queued pen_stamp request should snapshot the direction at the moment it ran, not the script's final direction");
}
