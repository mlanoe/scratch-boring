//! End-to-end test for `motion_glideto` -- the same real tween `motion_
//! glidesecstoxy` already exercises (`tests/motion_glide.rs` covers the
//! interpolation math itself), but with the target position resolved
//! from a TO menu-shadow (a sprite name, "_mouse_", or "_random_") --
//! the exact same dispatch `motion_goto`'s own `exec_simple` case
//! already established -- rather than raw X/Y exprs. See `interpreter.br`'s
//! own `GlideTo` case for why the target is resolved *once*, before the
//! glide starts, not re-sampled every tick.
//!
//! `assets/motion_glideto.json`: sprite B sits stationary at (150, -80)
//! the whole time; sprite A starts at (0, 0) and its green-flag script is
//! `glide 2 secs to [B]`.

use bevy::prelude::*;
use scratch_boring::{
    play_queued_sounds, run_pending_threads, setup_scratch_project, sync_keyboard_state,
    sync_mouse_state, sync_transforms_from_positions, ProjectPath, ScratchSprite, ScratchTimer,
};
use std::sync::Arc;

fn test_app() -> App {
    let path: Arc<str> = Arc::from("assets/motion_glideto.json");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(bevy::input::ButtonInput::<KeyCode>::default())
        .insert_resource(bevy::input::ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        // Deliberately no sync_scratch_timer -- manual ScratchTimer.elapsed,
        // same reason tests/motion_glide.rs drives it by hand.
        .add_systems(Update, (sync_keyboard_state, sync_mouse_state, run_pending_threads, play_queued_sounds, sync_transforms_from_positions).chain());
    app
}

fn sprite_xy(app: &mut App, target_id: &str) -> (f32, f32) {
    let world = app.world_mut();
    let mut query = world.query::<(&ScratchSprite, &Transform)>();
    let (_, transform) = query.iter(world).find(|(sprite, _)| sprite.target_id.as_ref() == target_id).unwrap_or_else(|| panic!("sprite {target_id} should have a ScratchSprite+Transform entity"));
    (transform.translation.x, transform.translation.y)
}

#[test]
fn glide_to_a_sprite_resolves_its_position_once_and_interpolates_to_it() {
    let mut app = test_app();

    // Tick 1: parks on the Gliding frame right away, no movement yet.
    app.update();
    assert_eq!(sprite_xy(&mut app, "A"), (0.0, 0.0), "the glide should have parked on its own frame without moving yet, same tick it started");
    assert_eq!(sprite_xy(&mut app, "B"), (150.0, -80.0), "B is stationary, never touched by any script");

    // Halfway through the 2s duration.
    app.world_mut().resource_mut::<ScratchTimer>().elapsed = 1.0;
    app.update();
    assert_eq!(sprite_xy(&mut app, "A"), (75.0, -40.0), "halfway through a 2s glide from (0,0) to B's (150,-80) should be exactly (75, -40)");

    // Past the glide's own duration -- lands exactly on B's real position.
    app.world_mut().resource_mut::<ScratchTimer>().elapsed = 3.0;
    app.update();
    assert_eq!(sprite_xy(&mut app, "A"), (150.0, -80.0), "past the glide's own duration, A should land exactly on B's real position");
}
