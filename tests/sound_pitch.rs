//! End-to-end test for `sound_seteffectto`/`sound_changeeffectby`'s PITCH
//! effect -- applied once, at spawn time, onto the real `PlaybackSettings.
//! speed` of the `AudioPlayer` entity `sound_play` triggers (see
//! `play_queued_sounds`' own doc comment for the verified real-Scratch
//! formula: `2^(pitch/120)`). PAN stays a documented gap -- see
//! `SoundQueue.pitches`' own doc comment.
//!
//! Uses `assets/sound_pitch_test.sb3` (a real zip with the same genuine
//! minimal WAV sound `tests/sound_volume.rs`'s own fixture uses, generated
//! by a small script not checked in):
//!
//! ```text
//! Sprite1, when green flag clicked:
//!   set pitch effect to 60
//!   change pitch effect by 60      -> 60 + 60 = 120 (= 12 semitones = 1 octave)
//!   play sound "beep"
//! ```
//!
//! A pitch effect value of 120 (Scratch's own tenths-of-a-semitone units)
//! is exactly 12 semitones -- one octave -- giving a clean, exactly
//! verifiable expected playback rate of `2^(120/120) = 2^1 = 2.0`.

use bevy::audio::AudioPlugin;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{play_queued_sounds, run_pending_threads, setup_scratch_project, sync_keyboard_state, sync_mouse_state, ProjectPath};
use std::sync::Arc;

#[test]
fn sound_play_applies_the_current_pitch_effect_as_a_real_playback_rate() {
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

    // One tick: the green-flag script sets pitch to 60, changes it by 60
    // (-> 120 total), then plays -- all leaf statements, no wait involved,
    // so the whole script (and play_queued_sounds draining the resulting
    // request into a real entity) finishes within this tick.
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&PlaybackSettings>();
    let settings = query
        .iter(world)
        .next()
        .expect("sound_play should have spawned one entity with real PlaybackSettings");
    assert!(
        (settings.speed - 2.0).abs() < 0.001,
        "pitch effect 120 (one octave) should map to playback speed 2.0, got {}",
        settings.speed
    );
}
