//! End-to-end test for `motion_ifonedgebounce`, ported from real
//! scratch-vm's own `Scratch3MotionBlocks.ifOnEdgeBounce` -- see
//! `interpreter.br`'s own `IfOnEdgeBounce` case for the full algorithm and
//! the unrotated-AABB approximation it makes (same trade `sensing_
//! touchingobject`'s own `Touching` case already makes).
//!
//! `assets/motion_ifonedgebounce.json`: two sprites, neither with a real
//! costume asset (so `TargetCostumeDims` falls back to its own documented
//! 40x40 default -- half-width/half-height 20).
//!
//! ```text
//! Bouncer:   x=235, y=0, direction=90 (facing right)
//!   -- right edge of its box is at 235+20=255, past the stage's own 240
//!   boundary, so it's touching the right edge and nothing else.
//!   if on edge bounce
//!   -> direction reflects to face away from the right wall: -90 (left)
//!   -> x clamped back inside the stage: 235 + (240-255) = 220
//!   -> y untouched: 0
//!
//! Untouched: x=0, y=0, direction=45 -- nowhere near any edge.
//!   if on edge bounce
//!   -> completely unaffected: x=0, y=0, direction=45
//! ```

use scratch_boring::run_greenflag_and_report_json;
use std::sync::Arc;

#[test]
fn if_on_edge_bounce_reflects_direction_and_clamps_position_when_touching_an_edge() {
    let path: Arc<str> = Arc::from("assets/motion_ifonedgebounce.json");
    let get = |name: &'static str| run_greenflag_and_report_json(path.clone(), Arc::from(name));

    assert_eq!(get("result_bouncer_x").as_ref(), "220", "touching the right edge should clamp x back inside the stage (235 + (240 - 255))");
    assert_eq!(get("result_bouncer_y").as_ref(), "0", "y should be untouched -- only the right edge was touched, not top/bottom");
    assert_eq!(get("result_bouncer_dir").as_ref(), "-90", "facing right (90) into the right wall should bounce to facing left (-90)");
}

#[test]
fn if_on_edge_bounce_is_a_complete_no_op_when_not_touching_any_edge() {
    let path: Arc<str> = Arc::from("assets/motion_ifonedgebounce.json");
    let get = |name: &'static str| run_greenflag_and_report_json(path.clone(), Arc::from(name));

    assert_eq!(get("result_untouched_x").as_ref(), "0");
    assert_eq!(get("result_untouched_y").as_ref(), "0");
    assert_eq!(get("result_untouched_dir").as_ref(), "45", "direction should be completely untouched when nowhere near any edge");
}
