//! End-to-end test for `sensing_resettimer`. `exec_simple` has no `var
//! ScratchTimer` access to write `elapsed` directly (a real, universal
//! `boring` convention found while spiking a more direct fix: a `var`
//! field always transpiles to a *private* Rust field, which would have
//! broken every existing test driving `ScratchTimer.elapsed` by hand --
//! see `SoundQueue.reset_timer_requested`'s own doc comment) -- so this
//! is a deferred request, applied by `sync_scratch_timer` itself on the
//! *next* tick after the one that queued it, same one-tick-lag trade-off
//! `check_greater_than_hats`/`check_backdrop_switch_hats` already make
//! for a different reason.
//!
//! `assets/sensing_resettimer.json`: Sprite1's green-flag script is just
//! `reset timer`.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{run_pending_threads, setup_scratch_project, sync_keyboard_state, sync_mouse_state, sync_scratch_timer, ProjectPath, ScratchTimer};
use std::sync::Arc;

#[test]
fn reset_timer_sets_elapsed_back_to_zero_the_following_tick() {
    let path: Arc<str> = Arc::from("assets/sensing_resettimer.json");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(Update, (sync_keyboard_state, sync_mouse_state, sync_scratch_timer, run_pending_threads).chain());

    // Tick 1 (also runs `Startup`, which is what inserts `ScratchTimer`
    // in the first place -- it doesn't exist as a resource before this
    // call): `sync_scratch_timer` runs before the green-flag thread does
    // (no reset requested yet), then `run_pending_threads` runs `reset
    // timer`, which only *queues* the request -- doesn't apply this tick.
    app.update();

    // Simulate "more time has passed" since -- proves the next tick's
    // reset genuinely overrides whatever value is there by then, not
    // just that it coincidentally happened to already be near 0.
    app.world_mut().resource_mut::<ScratchTimer>().elapsed = 42.0;

    // Tick 2: `sync_scratch_timer` now sees the still-pending request
    // (queued back in tick 1, never applied yet) and resets `elapsed`
    // straight to 0.0, overriding the 42.0 just set above.
    app.update();
    assert_eq!(app.world().resource::<ScratchTimer>().elapsed, 0.0, "the timer should read back exactly 0.0 the tick after reset timer ran");
}
