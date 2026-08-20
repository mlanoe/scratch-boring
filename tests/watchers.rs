//! End-to-end test for variable watchers (on-stage display of a Stage
//! variable's live value). Uses the real `Fibonacci_1.sb3` fixture (also
//! `tests/fibonacci.rs`'s subject) since its `monitors` array has exactly
//! two *visible* variable watchers ("current_number", "sequence_length")
//! and three invisible ones ("i", "last_number", "new_number") -- a good
//! real-world check that an invisible watcher's own entity still exists
//! (needed once `data_showvariable`/`data_hidevariable` could toggle one
//! at runtime -- there'd be nothing to ever show if it didn't) but starts
//! with a real `Visibility::Hidden`, not skipped outright the way it used
//! to be before that round landed.
//!
//! Expected final values (`current_number` = 75025, `sequence_length` =
//! 25) aren't guessed -- they're the exact same values
//! `tests/fibonacci.rs`'s own `fibonacci_1_repeat_until` test already
//! derived and asserts on directly against `Vars`.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    play_queued_sounds, run_pending_threads, setup_scratch_project, sync_keyboard_state,
    sync_mouse_state, sync_speech_bubbles, sync_variable_watchers, ProjectPath, VariableWatcherTag,
};
use std::sync::Arc;

fn test_app(path: Arc<str>) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(
            Update,
            (
                sync_keyboard_state,
                sync_mouse_state,
                run_pending_threads,
                play_queued_sounds,
                sync_speech_bubbles,
                sync_variable_watchers,
            )
                .chain(),
        );
    app
}

#[test]
fn only_visible_watchers_are_spawned_and_show_the_correct_final_value() {
    let mut app = test_app(Arc::from("assets/Fibonacci_1.sb3"));
    // One `app.update()` used to be enough to see the fully-final value,
    // back when every queued script ran to completion within the single
    // tick it was picked up on. Now that the real pausable scheduler
    // exists (`step_thread`/`Thread`), a `repeat until` genuinely yields
    // once per iteration -- exactly matching real Scratch's own control
    // flow blocks, which always yield at a loop-iteration boundary
    // regardless of how little work the iteration did (scratch-vm's own
    // `repeat`/`forever`/`repeat until` block implementations call
    // `util.yield()` unconditionally every iteration; the real per-tick
    // work budget only bounds how many *different threads* get a turn in
    // one tick, never how many iterations a single thread's own loop gets
    // before yielding). Fibonacci_1's `repeat until` needs 25 iterations
    // to reach its final value, so this drives 30 ticks (a small margin
    // over the known-exact 25) rather than just one.
    for _ in 0..30 {
        app.update();
    }

    let world = app.world_mut();
    let mut query = world.query::<(&VariableWatcherTag, &Text2d, &Visibility)>();
    let mut found: Vec<(String, String, bool)> = query
        .iter(world)
        .map(|(tag, text, vis)| (tag.label.to_string(), text.0.clone(), *vis == Visibility::Visible))
        .collect();
    found.sort();

    assert_eq!(
        found.len(),
        5,
        "every one of the fixture's five variable monitors should have its own watcher entity now, visible or not, got: {found:?}"
    );
    assert_eq!(found[0], ("current_number".to_string(), "current_number: 75025".to_string(), true));
    assert_eq!(found[1], ("i".to_string(), "i: 25".to_string(), false), "an invisible monitor still gets a real, correctly-computed entity -- just hidden");
    assert_eq!(found[2], ("last_number".to_string(), "last_number: 46368".to_string(), false));
    assert_eq!(found[3], ("new_number".to_string(), "new_number: 75025".to_string(), false));
    assert_eq!(found[4], ("sequence_length".to_string(), "sequence_length: 25".to_string(), true));
}

/// A watcher's label must reflect the variable's *current* name, not a
/// monitor's own (possibly stale) `params.VARIABLE` -- discovered via a
/// real scratch-vm test fixture, `list-monitor-rename.sb3` (an analogous
/// list-monitor case; no variable-monitor fixture demonstrating this
/// specific staleness was found, so this is a hand-authored fixture,
/// `assets/monitor_rename.json`: a variable saved as `newName` with a
/// monitor whose own `params.VARIABLE` still says `oldName`, simulating a
/// user renaming the variable after placing its watcher).
#[test]
fn watcher_label_uses_the_variables_current_name_not_a_stale_monitor_param() {
    let mut app = test_app(Arc::from("assets/monitor_rename.json"));
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&VariableWatcherTag, &Text2d)>();
    let (tag, text) = query.iter(world).next().expect("one watcher entity should exist");
    assert_eq!(tag.label.as_ref(), "newName", "the label should be the variable's current name");
    assert_eq!(text.0, "newName: 42", "not \"oldName: 42\" (the monitor's own stale params.VARIABLE)");
}
