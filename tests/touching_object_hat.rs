//! End-to-end test for `event_whentouchingobject` -- an edge-triggered hat
//! (fires once on the tick a sprite *starts* touching, not every tick it
//! stays touching), reusing `sensing_touchingobject`'s own AABB machinery
//! wholesale via a `Block(kind = BlockKind.Touching(...))` built once at
//! link time (`build_touching_object_hats`). Unlike `check_greater_than_
//! hats`' single template-level `was_true` (its TIMER condition is
//! target-agnostic), touching-status genuinely depends on which instance
//! is asking, so this tracks one `was_true` per live instance
//! (`TouchingObjectHat.was_true_by_instance`).
//!
//! `assets/touching_object_hat.json`: A (40x40 placeholder box, same
//! convention as `tests/touching.rs`) starts at (200, 0), far from B at
//! (0, 0) -- not touching (|dx|=200 >= 40). A has a `when touching B` hat
//! that changes a Stage variable `touch_count` by 1. The test drives
//! `Positions` directly across ticks (same technique `tests/backdrop_
//! switch_hat.rs` uses for its own edge-triggered hat, but on position
//! instead of `ButtonInput`) to move A in and out of B's box and confirm:
//! the hat fires exactly once per touch transition, not on every tick
//! it's still touching, and fires again after a release/re-touch cycle.

use bevy::prelude::*;
use scratch_boring::{
    check_touching_object_hats, run_pending_threads, set_position_value, setup_scratch_project,
    stage_var_value, sync_keyboard_state, sync_mouse_state, sync_scratch_timer, Positions,
    ProjectPath, Vars,
};
use std::sync::Arc;

const FIXTURE: &str = "assets/touching_object_hat.json";

fn test_app() -> App {
    let path: Arc<str> = Arc::from(FIXTURE);
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(bevy::input::ButtonInput::<KeyCode>::default())
        .insert_resource(bevy::input::ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(
            Update,
            (
                sync_keyboard_state,
                sync_mouse_state,
                sync_scratch_timer,
                check_touching_object_hats,
                run_pending_threads,
            )
                .chain(),
        );
    app
}

fn touch_count(app: &mut App) -> Arc<str> {
    let vars = app.world().resource::<Vars>().clone();
    stage_var_value(Arc::from(FIXTURE), vars, Arc::from("touch_count"))
}

fn move_a_to(app: &mut App, x: f32) {
    let mut positions = app.world_mut().resource_mut::<Positions>();
    set_position_value(&mut positions, Arc::from("A"), x, 0.0);
}

#[test]
fn event_whentouchingobject_fires_once_per_touch_transition() {
    let mut app = test_app();

    // Tick 1: A starts far away (x=200), not touching yet.
    app.update();
    assert_eq!(touch_count(&mut app).as_ref(), "0", "A starts far from B -- no touch yet");

    // Move A onto B (x=0): both 40x40 placeholder boxes, |dx|=0 < 40 -> touching.
    move_a_to(&mut app, 0.0);
    app.update();
    assert_eq!(touch_count(&mut app).as_ref(), "1", "A just started touching B -- the hat should fire exactly once");

    // Still touching, nothing moved: must NOT refire every tick.
    app.update();
    app.update();
    assert_eq!(touch_count(&mut app).as_ref(), "1", "still touching on later ticks -- the hat must not refire while the touch continues");

    // Move A away again (x=200): touch released.
    move_a_to(&mut app, 200.0);
    app.update();
    assert_eq!(touch_count(&mut app).as_ref(), "1", "A moved away -- releasing a touch must not itself fire the hat");

    // Move A back onto B: a fresh transition should fire again.
    move_a_to(&mut app, 0.0);
    app.update();
    assert_eq!(touch_count(&mut app).as_ref(), "2", "A touched B again after releasing -- the hat should fire a second time");
}
