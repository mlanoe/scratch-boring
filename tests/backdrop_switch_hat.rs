//! End-to-end test for `event_whenbackdropswitchesto` -- unlike real
//! scratch-vm (which fires this hat straight from inside `switchBackdrop`
//! itself, a direct `Runtime.startHats` call), this project has no
//! `Commands`/thread-spawn access from `looks_switchbackdropto`'s own
//! `exec_simple` case (same reason `SoundRequest`/`CloneRequest` exist),
//! so `check_backdrop_switch_hats` instead edge-detects the switch by
//! comparing the Stage's current backdrop name against the previous
//! tick's own -- one tick of lag versus real Scratch's own synchronous
//! fire, same trade-off `check_greater_than_hats` already makes for a
//! different hat.
//!
//! `assets/backdrop_switch_hat.json`: Stage starts on backdrop "a".
//! "Switcher"'s green-flag script switches to "b"; "Listener" has its own
//! `when backdrop switches to "b"` hat, which sets a Stage variable
//! `result` to `"fired"`.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{check_backdrop_switch_hats, run_pending_threads, setup_scratch_project, sync_keyboard_state, sync_mouse_state, sync_variable_watchers, ProjectPath, VariableWatcherTag};
use std::sync::Arc;

fn test_app() -> App {
    let path: Arc<str> = Arc::from("assets/backdrop_switch_hat.json");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(Update, (sync_keyboard_state, sync_mouse_state, check_backdrop_switch_hats, run_pending_threads, sync_variable_watchers).chain());
    app
}

fn result_text(app: &mut App) -> String {
    let world = app.world_mut();
    let mut query = world.query::<(&VariableWatcherTag, &Text2d)>();
    let (_, text) = query.iter(world).find(|(tag, _)| tag.label.as_ref() == "result").expect("result's own watcher entity should exist");
    text.0.to_string()
}

#[test]
fn switching_backdrop_fires_the_matching_hat_the_following_tick() {
    let mut app = test_app();

    // Tick 1: `check_backdrop_switch_hats` sees the Stage still on its
    // own real initial backdrop ("a") -- no fire yet. `run_pending_
    // threads` then runs Switcher's green-flag script, which switches
    // the Stage to "b" within this same tick.
    app.update();
    assert_eq!(result_text(&mut app), "result: not fired", "the hat shouldn't have fired before the switch was even detected");

    // Tick 2: `check_backdrop_switch_hats` now sees "b" != last tick's
    // "a" -- fires Listener's hat, which runs in this same tick (queued
    // before `run_pending_threads`, drained by it in the same pass).
    app.update();
    assert_eq!(result_text(&mut app), "result: fired", "the backdrop switch should have been detected and Listener's hat should have run");
}
