//! End-to-end test for `"any"` KEY_OPTION semantics, on both `sensing_
//! keypressed` (level-triggered) and `event_whenkeypressed` (edge-
//! triggered) -- previously an OR-chain over only the (space + 4 arrows,
//! later also every letter/digit) *named* keys this project tracks, a
//! documented approximation of real Scratch's own true "any physical key
//! at all" (`this._keysPressed.length > 0` in real scratch-vm, populated
//! from every raw keydown, not scoped to nameable keys). Now reads Bevy's
//! own `ButtonInput::get_pressed()`/`get_just_pressed()` directly instead
//! -- true parity, not just "any of the named ones".
//!
//! Deliberately presses `KeyCode::Comma` -- a real physical key with NO
//! KEY_OPTION dropdown entry at all (confirmed against real
//! `scratch-blocks` source, `key_option_just_pressed`'s own doc comment)
//! -- to prove the fix: the OLD OR-chain implementation would have missed
//! this key entirely (it's not one of the 41 named ones), while the new
//! `get_pressed()`/`get_just_pressed()`-based check correctly reports
//! "any" regardless of which physical key was actually pressed.
//!
//! `assets/any_key.json`: a `when any key pressed` hat and a `key any
//! pressed?` reporter, same shape `tests/input.rs`/`tests/letter_and_
//! digit_keys.rs` already established for named keys.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    check_key_press_hats, run_pending_threads, setup_scratch_project, stage_var_value,
    sync_keyboard_state, sync_mouse_state, ProjectPath, Vars,
};
use std::sync::Arc;

const FIXTURE: &str = "assets/any_key.json";

fn test_app() -> App {
    let path: Arc<str> = Arc::from(FIXTURE);
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(Update, (sync_keyboard_state, sync_mouse_state, check_key_press_hats, run_pending_threads).chain());
    app
}

fn var(app: &mut App, name: &'static str) -> Arc<str> {
    let vars = app.world().resource::<Vars>().clone();
    stage_var_value(Arc::from(FIXTURE), vars, Arc::from(name))
}

#[test]
fn any_key_hat_and_reporter_fire_for_an_unnamed_key_like_comma() {
    let mut app = test_app();
    app.update();
    assert_ne!(var(&mut app, "anyHat").as_ref(), "true", "hat must not fire before any key is ever pressed");
    assert_ne!(var(&mut app, "anyReporter").as_ref(), "true", "reporter must read false before any key is pressed");

    // Comma has no KEY_OPTION dropdown entry at all -- the real
    // discriminator between "any of the 41 named keys" (the old,
    // documented approximation) and true "any physical key".
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Comma);
    app.update();

    assert_eq!(var(&mut app, "anyHat").as_ref(), "true", "\"when any key pressed\" should fire for Comma even though it has no dropdown name of its own");
    assert_eq!(var(&mut app, "anyReporter").as_ref(), "true", "\"key any pressed?\" should read true while Comma is held, same reasoning");
}
