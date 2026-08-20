//! End-to-end test for `motion_glidesecstoxy` -- the largest remaining
//! real opcode gap once `sensing_of` landed (36 occurrences across the
//! tracked real-project pool). Unlike every other Motion stack block
//! (`motion_gotoxy` etc, instant), this is a real *tween*: the sprite
//! visibly moves across several ticks, not a same-tick jump -- same
//! multi-tick, manual-`ScratchTimer.elapsed` technique
//! `tests/say_think_for_secs.rs`/`tests/scheduler.rs`'s own `control_wait`
//! test already use, since `sync_scratch_timer` (real wall-clock-driven)
//! isn't in this test's own `Update` chain at all.
//!
//! `assets/glide_secs_to_xy.json`: sprite A starts at (0, 0); its
//! green-flag script is `glide 2 secs to x: 100 y: 200`.

use bevy::prelude::*;
use scratch_boring::{play_queued_sounds, run_pending_threads, setup_scratch_project, sync_keyboard_state, sync_mouse_state, sync_transforms_from_positions, ProjectPath, ScratchSprite, ScratchTimer};
use std::sync::Arc;

fn test_app() -> App {
    let path: Arc<str> = Arc::from("assets/glide_secs_to_xy.json");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(bevy::input::ButtonInput::<KeyCode>::default())
        .insert_resource(bevy::input::ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        // Deliberately no `sync_scratch_timer` -- this test drives
        // `ScratchTimer.elapsed` by hand for exact, reproducible
        // interpolation checks, same reason `say_think_for_secs.rs` does.
        .add_systems(Update, (sync_keyboard_state, sync_mouse_state, run_pending_threads, play_queued_sounds, sync_transforms_from_positions).chain());
    app
}

fn sprite_a_xy(app: &mut App) -> (f32, f32) {
    let world = app.world_mut();
    let mut query = world.query::<(&ScratchSprite, &Transform)>();
    let (_, transform) = query.iter(world).find(|(sprite, _)| sprite.target_id.as_ref() == "A").expect("sprite A should have a ScratchSprite+Transform entity");
    (transform.translation.x, transform.translation.y)
}

#[test]
fn glide_secs_to_xy_interpolates_position_across_ticks_then_lands_exactly() {
    let mut app = test_app();

    // Tick 1: the green-flag thread runs `glide 2 secs to (100, 200)` --
    // this parks on a `Gliding` frame right away (`ScratchTimer.elapsed`
    // is still 0.0, its own captured start time), no interpolation has
    // happened yet on this same tick.
    app.update();
    assert_eq!(sprite_a_xy(&mut app), (0.0, 0.0), "the glide should have parked on its own frame without moving yet, same tick it started");

    // Tick 2, timer still at 0.0 -- `elapsed - start_time == 0`, so the
    // sprite should still be exactly at its own start position (frac 0).
    app.update();
    assert_eq!(sprite_a_xy(&mut app), (0.0, 0.0), "no time has passed since the glide started, position should be unchanged");

    // Halfway through the 2s duration -- linearly interpolated position.
    app.world_mut().resource_mut::<ScratchTimer>().elapsed = 1.0;
    app.update();
    assert_eq!(sprite_a_xy(&mut app), (50.0, 100.0), "halfway through a 2s glide from (0,0) to (100,200) should be exactly (50, 100)");

    // Past the glide's own duration -- lands exactly on the target, not
    // an interpolated-but-slightly-off value, and the frame is popped
    // (nothing left in the script).
    app.world_mut().resource_mut::<ScratchTimer>().elapsed = 3.0;
    app.update();
    assert_eq!(sprite_a_xy(&mut app), (100.0, 200.0), "past the glide's own duration, the sprite should land exactly on the target position");
}
