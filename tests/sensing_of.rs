//! End-to-end test for `sensing_of`/`sensing_of_object_menu` -- the largest
//! remaining real opcode gap once the Type/Instance model landed (62
//! combined occurrences across this session's widened 5-project real-data
//! pool). Real sampled project data showed PROPERTY holding values like
//! "Parallax"/"SOUND" -- project-specific global variable names, not just
//! Scratch's fixed builtin property set -- so this fixture exercises both
//! a builtin sprite property (`x position`), a Stage global-variable-by-
//! name lookup, and a Stage builtin (`backdrop #`) in one script, using
//! real (non-inlined) `sensing_of_object_menu` shadow blocks throughout,
//! matching the real-block-not-always-inlined shape confirmed for every
//! other menu shadow this session.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{play_queued_sounds, run_pending_threads, setup_scratch_project, sync_keyboard_state, sync_mouse_state, sync_variable_watchers, ProjectPath, VariableWatcherTag};
use std::sync::Arc;

/// `assets/sensing_of.json`: sprite B sits at (30, 40); the Stage has a
/// global variable `score` seeded to `42`. Sprite A's green-flag script:
///
/// ```text
/// result_x        = [of B] x position     -- 30
/// result_var       = [of Stage] score      -- 42 (name lookup, not a builtin)
/// result_backdrop = [of Stage] backdrop #  -- 1 (single costume, 1-indexed)
/// ```
#[test]
fn sensing_of_reads_builtin_properties_and_global_variables_by_name() {
    let path: Arc<str> = Arc::from("assets/sensing_of.json");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(Update, (sync_keyboard_state, sync_mouse_state, run_pending_threads, play_queued_sounds, sync_variable_watchers).chain());

    // Three leaf statements in a row, no wait involved -- one tick is enough.
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&VariableWatcherTag, &Text2d)>();
    let mut found: Vec<(String, String)> = query.iter(world).map(|(tag, text)| (tag.label.to_string(), text.0.clone())).collect();
    found.sort();

    assert_eq!(found.len(), 3, "expected all three Stage variables to have their own watcher entity, got: {found:?}");
    assert_eq!(found[0], ("result_backdrop".to_string(), "result_backdrop: 1".to_string()), "Stage has a single costume, so backdrop # is 1");
    assert_eq!(found[1], ("result_var".to_string(), "result_var: 42".to_string()), "PROPERTY \"score\" isn't a builtin -- should resolve to the Stage's own global variable by name");
    assert_eq!(found[2], ("result_x".to_string(), "result_x: 30".to_string()), "B's own x position is 30");
}
