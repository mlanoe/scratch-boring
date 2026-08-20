//! End-to-end test for the letter/digit key extension to `sensing_
//! keypressed`/`event_whenkeypressed` -- previously scoped to space + the
//! four arrow keys only (a real, documented gap); now also tracks every
//! letter ("a".."z") and digit ("0".."9"), matching real scratch-vm's own
//! KEY_OPTION dropdown values (verified against real `scratch-blocks`
//! source, `sensing_keyoptions`'s `field_dropdown` options -- lowercase
//! single letters/plain digit strings, not guessed). Punctuation keys
//! remain a documented gap.
//!
//! `assets/letter_and_digit_keys.json`: a `when a key pressed` hat and a
//! `when 5 key pressed` hat, each setting its own Stage variable and
//! reading the matching `sensing_keypressed` reporter at hat-fire time --
//! same shape `tests/input.rs` already established for "space".
//!
//! Only tests one representative letter and one representative digit
//! (not all 36) -- the implementation is 36 mechanically-identical
//! `KeyCode::KeyX`/`KeyCode::DigitN` branches (like the existing space/
//! arrow ones), so a bug would almost certainly be a copy-paste slip
//! affecting one or two keys, not a systemic one this narrower check
//! would miss; the whole match arm is visible for review directly either
//! way.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    check_key_press_hats, run_pending_threads, setup_scratch_project, stage_var_value,
    sync_keyboard_state, sync_mouse_state, ProjectPath, Vars,
};
use std::sync::Arc;

const FIXTURE: &str = "assets/letter_and_digit_keys.json";

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
fn letter_key_hat_and_reporter_both_fire_the_same_tick() {
    let mut app = test_app();
    app.update();
    assert_ne!(var(&mut app, "letterHat").as_ref(), "true", "hat must not fire before \"a\" is ever pressed");

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyA);
    app.update();

    assert_eq!(var(&mut app, "letterHat").as_ref(), "true", "event_whenkeypressed \"a\" should have fired this tick");
    assert_eq!(var(&mut app, "letterReporter").as_ref(), "true", "sensing_keypressed \"a\" should already read true within the same tick");
}

#[test]
fn digit_key_hat_and_reporter_both_fire_the_same_tick() {
    let mut app = test_app();
    app.update();
    assert_ne!(var(&mut app, "digitHat").as_ref(), "true", "hat must not fire before \"5\" is ever pressed");

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Digit5);
    app.update();

    assert_eq!(var(&mut app, "digitHat").as_ref(), "true", "event_whenkeypressed \"5\" should have fired this tick");
    assert_eq!(var(&mut app, "digitReporter").as_ref(), "true", "sensing_keypressed \"5\" should already read true within the same tick");
}
