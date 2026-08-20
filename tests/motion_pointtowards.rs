//! End-to-end test for `motion_pointtowards`/`motion_pointtowards_menu` --
//! the last remaining real opcode gap once `sensing_resettimer` landed (2
//! occurrences: this reporter plus its own menu shadow). Same menu-shadow
//! shape as `motion_goto`'s own TO -- "_mouse_"/"_random_"/a sprite name,
//! no "_edge_" case -- but "_random_" means a genuinely random
//! *direction* here, not a random *position* the way `GoTo`'s own
//! "_random_" is.
//!
//! `assets/motion_pointtowards.json`: sprite A at `(0, 0)` points towards
//! sprite B, directly above it at `(0, 100)`. Real Scratch's own direction
//! convention (`0` = up) makes this an exactly verifiable case: `dx = 0`,
//! `dy = 100`, so the resulting direction should be exactly `0`.

use scratch_boring::run_greenflag_and_report_json;
use std::sync::Arc;

#[test]
fn point_towards_a_sprite_directly_above_reports_direction_zero() {
    let path: Arc<str> = Arc::from("assets/motion_pointtowards.json");
    let result = run_greenflag_and_report_json(path, Arc::from("result"));
    assert_eq!(result.as_ref(), "0", "B sits directly above A (dx=0, dy=100), so the resulting direction should be exactly 0 (real Scratch's own \"0 = up\" convention)");
}
