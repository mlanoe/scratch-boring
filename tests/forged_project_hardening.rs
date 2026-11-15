//! Regression tests for hand-forged `.sb3`/`project.json` inputs that used to
//! hang, overflow the stack or exhaust memory (found in an audit):
//!
//! - `substack_self_cycle.json`: a C-block whose `SUBSTACK` names itself --
//!   `resolve_stack` only guarded the `next` chain, so this recursed forever.
//! - `substack_dag_blowup.json`: 40 chained `if/else` whose two branches both
//!   point at the same next block -- 2^40 resolutions without a per-script
//!   "already linked" set.
//! - `direction_huge_turn.json`: `turn right 1e10` / `turn left 1e30` /
//!   `point in direction 1e38` -- `normalize_direction` folded by 360 in a
//!   `while` loop, which never terminates once `x - 360 == x` in float32.
//! - `pen_far_offstage.json`: pen down then `go to x:1e9 y:1e9` -- the line
//!   rasterizer iterated over an unclamped bounding box (~1e9 rows).
//! - `broadcast_wait_self_recursion.json`: `when I receive A` doing
//!   `broadcast A and wait` -- unbounded native recursion (stack overflow).
//!
//! Each test only needs to *return*: before the fixes they hung or aborted.
use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    draw_pen_marks, run_greenflag_and_report_json, run_pending_threads, setup_scratch_project,
    spawn_pen_canvas, sync_keyboard_state, sync_mouse_state, PenCanvas, ProjectPath,
};
use std::sync::Arc;

fn done(path: &str) -> String {
    run_greenflag_and_report_json(Arc::from(path), Arc::from("done")).to_string()
}

#[test]
fn a_self_referencing_substack_loads_and_runs_the_rest_of_the_script() {
    assert_eq!(done("assets/substack_self_cycle.json"), "1");
}

#[test]
fn shared_substack_targets_do_not_blow_up_exponentially() {
    // Only the first statement's effect is asserted; reaching it proves the
    // load finished (it used to take ~2^40 steps).
    let v = done("assets/substack_dag_blowup.json");
    assert!(v == "1" || v == "2", "unexpected `done`: {v}");
}

#[test]
fn huge_and_non_finite_turns_terminate() {
    assert_eq!(done("assets/direction_huge_turn.json"), "1");
}

#[test]
fn a_self_recursive_broadcast_and_wait_does_not_overflow_the_stack() {
    assert_eq!(done("assets/broadcast_wait_self_recursion.json"), "1");
}

#[test]
fn pen_lines_far_outside_the_stage_are_clamped_to_the_canvas() {
    let path: Arc<str> = Arc::from("assets/pen_far_offstage.json");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, (spawn_pen_canvas, setup_scratch_project))
        .add_systems(
            Update,
            (sync_keyboard_state, sync_mouse_state, run_pending_threads, draw_pen_marks).chain(),
        );
    app.update();
    // Reaching here at all is the assertion (it used to loop ~1e9 times).
    assert!(app.world().get_resource::<PenCanvas>().is_some());
}
