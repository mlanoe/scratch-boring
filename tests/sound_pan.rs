//! End-to-end test for `sound_seteffectto`'s PAN effect -- unlike PITCH
//! (a plain `PlaybackSettings.speed` scalar `bevy_audio` already exposes
//! directly, see `tests/sound_pitch.rs`), `bevy_audio` has no stereo-pan
//! primitive of its own at all. `play_queued_sounds` repurposes its
//! *spatial audio* system instead (its own doc comment: "implemented via
//! simple left-right stereo panning") -- a real, calibrated (not
//! guessed) approximation, verified against real `rodio`/`bevy_audio`
//! source for the exact formula. See `play_queued_sounds`'/`spawn_
//! camera`'s own doc comments for the full derivation.
//!
//! Uses `assets/sound_pan_test.sb3` (a real zip, generated from the same
//! real minimal WAV sound `tests/sound_pitch.rs`'s own fixture uses --
//! see that file's own header comment):
//!
//! ```text
//! Sprite1, when green flag clicked:
//!   set pan effect to 100   (full right)
//!   play sound "beep"
//! ```
//!
//! At full pan (100), the calibrated offset is `(100/100) * 0.9 = 0.9`
//! world units on the spawned sound entity's own `Transform.translation.x`
//! -- exactly verifiable, not approximate, since it's this project's own
//! fixed calibration constant, not real audio output.

use bevy::audio::AudioPlugin;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{play_queued_sounds, run_pending_threads, setup_scratch_project, sync_keyboard_state, sync_mouse_state, ProjectPath};
use std::sync::Arc;

#[test]
fn sound_play_applies_the_current_pan_effect_as_real_spatial_positioning() {
    let path: Arc<str> = Arc::from("assets/sound_pan_test.sb3");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default(), AudioPlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(
            Update,
            (sync_keyboard_state, sync_mouse_state, run_pending_threads, play_queued_sounds).chain(),
        );

    // One tick: the green-flag script sets pan to 100 then plays -- all
    // leaf statements, no wait involved, so the whole script (and play_
    // queued_sounds draining the resulting request into a real entity)
    // finishes within this tick.
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&PlaybackSettings, &Transform)>();
    let (settings, transform) = query
        .iter(world)
        .next()
        .expect("sound_play should have spawned one entity with real PlaybackSettings/Transform");

    assert!(settings.spatial, "a nonzero pan effect should engage Bevy's own spatial-audio mode");
    assert!(
        (transform.translation.x - 0.9).abs() < 0.001,
        "pan effect 100 (full right) should map to a calibrated +0.9 world-unit offset, got {}",
        transform.translation.x
    );
    assert_eq!(transform.translation.y, 0.0, "pan positioning is x-axis only");
}

#[test]
fn sound_play_stays_non_spatial_when_pan_is_left_at_its_zero_default() {
    // Reuses tests/sound_pitch.rs's own fixture -- its script never
    // touches PAN at all, so this confirms the common case (the vast
    // majority of real projects) never pays the spatial-mode cost.
    let path: Arc<str> = Arc::from("assets/sound_pitch_test.sb3");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default(), AudioPlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(
            Update,
            (sync_keyboard_state, sync_mouse_state, run_pending_threads, play_queued_sounds).chain(),
        );

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&PlaybackSettings>();
    let settings = query.iter(world).next().expect("sound_play should have spawned one entity");
    assert!(!settings.spatial, "a project that never sets PAN should never engage spatial audio");
}
