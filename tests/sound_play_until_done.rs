//! End-to-end test for `sound_playuntildone` -- the largest remaining real
//! opcode gap once `looks_goforwardbackwardlayers` landed (24
//! occurrences). Unlike plain `sound_play` (fire-and-forget,
//! `exec_simple`), the calling script actually *waits* for playback to
//! finish -- needs the pausable scheduler, so lives in `step_thread`.
//! No real audio-completion callback exists in this project's scheduler,
//! so the wait duration is precomputed once at link time from the
//! sound's own saved sb3 metadata (`SoundQueue.sound_durations`, `rate`/
//! `sampleCount`), then resolved via the same manual-`ScratchTimer.
//! elapsed` technique `tests/say_think_for_secs.rs`/`tests/motion_glide.rs`
//! already established.
//!
//! `assets/sound_play_until_done.json`: sprite A's own sound "beep" has
//! `rate: 10`, `sampleCount: 20` -- an exactly verifiable `20 / 10 = 2.0`
//! second duration. Green-flag script:
//!
//! ```text
//! play sound "beep" until done      -- should park for exactly 2 seconds
//! set result to "done"
//! ```

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{play_queued_sounds, run_pending_threads, setup_scratch_project, sync_keyboard_state, sync_mouse_state, sync_variable_watchers, ProjectPath, ScratchTimer, VariableWatcherTag};
use std::sync::Arc;

fn test_app() -> App {
    let path: Arc<str> = Arc::from("assets/sound_play_until_done.json");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        // Deliberately no `sync_scratch_timer` -- this test drives
        // `ScratchTimer.elapsed` by hand for exact, reproducible timing,
        // same reason `say_think_for_secs.rs`/`motion_glide.rs` do.
        .add_systems(Update, (sync_keyboard_state, sync_mouse_state, run_pending_threads, play_queued_sounds, sync_variable_watchers).chain());
    app
}

fn result_text(app: &mut App) -> String {
    let world = app.world_mut();
    let mut query = world.query::<(&VariableWatcherTag, &Text2d)>();
    let (_, text) = query.iter(world).find(|(tag, _)| tag.label.as_ref() == "result").expect("result's own watcher entity should exist");
    text.0.to_string()
}

#[test]
fn play_until_done_waits_for_the_sounds_real_duration_before_continuing() {
    let mut app = test_app();

    // Tick 1: the green-flag thread queues the sound and parks on the
    // 2-second wait right away -- `set result to "done"` hasn't run yet.
    app.update();
    assert_eq!(result_text(&mut app), "result: waiting", "the wait should have parked before reaching the set-variable statement");

    // A second tick with the timer still near 0 must NOT resolve the wait.
    app.update();
    assert_eq!(result_text(&mut app), "result: waiting", "the sound's own 2s duration hasn't elapsed yet");

    // Push the timer past the 2s threshold: the wait resolves and the
    // very next statement runs in the same tick (no yield in between).
    app.world_mut().resource_mut::<ScratchTimer>().elapsed = 2.5;
    app.update();
    assert_eq!(result_text(&mut app), "result: done", "the sound's own duration should have elapsed, letting the script continue");
}
