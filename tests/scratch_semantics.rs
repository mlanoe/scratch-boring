//! Regression tests for Scratch-semantics divergences found in an audit
//! (each asserted value is what real scratch-vm produces):
//!
//! - `if "hello"` is true (`Cast.toBoolean`: only "", "0", any-case "false" are false)
//! - `"banana" > "apple"` is true (`Cast.compare` falls back to string comparison)
//! - `repeat 2.5` runs 3 times (`Math.round`)
//! - `sin of 30` is exactly 0.5, `cos of 90` exactly 0, `tan of 180` exactly 0
//! - `letter 1.5 of "abc"` is "a" (`charAt` truncation)
//! - `insert "x" at last` appends (`toListIndex(index, length + 1)`)
//! - `stop all` ends every thread; `stop other scripts in sprite` ends the others
use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    run_greenflag_and_report_json, run_greenflag_and_report_list_json, run_pending_threads,
    setup_scratch_project, sync_keyboard_state, sync_mouse_state, PendingThreads, ProjectPath,
};
use std::sync::Arc;

fn var(name: &str) -> String {
    run_greenflag_and_report_json(Arc::from("assets/scratch_semantics.json"), Arc::from(name)).to_string()
}

#[test]
fn string_and_numeric_semantics_match_scratch_vm() {
    assert_eq!(var("t_truthy"), "1", "a non-empty, non-'0', non-'false' string is truthy");
    assert_eq!(var("t_gt"), "true", "\"banana\" > \"apple\" compares as strings");
    assert_eq!(var("t_repeat"), "3", "repeat 2.5 rounds to 3 iterations");
    assert_eq!(var("t_sin"), "0.5", "sin of 30");
    assert_eq!(var("t_cos"), "0", "cos of 90");
    assert_eq!(var("t_tan"), "0", "tan of 180");
    assert_eq!(var("t_letter"), "a", "letter 1.5 of abc");
}

#[test]
fn insert_at_last_appends() {
    let list = run_greenflag_and_report_list_json(Arc::from("assets/scratch_semantics.json"), Arc::from("L")).to_string();
    assert_eq!(list, "a,b,x", "insert at last must append after the last item");
}

fn live_threads_after_ticks(path: &str, ticks: usize) -> usize {
    let path: Arc<str> = Arc::from(path);
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(Update, (sync_keyboard_state, sync_mouse_state, run_pending_threads).chain());
    for _ in 0..ticks {
        app.update();
    }
    app.world().resource::<PendingThreads>().count() as usize
}

#[test]
fn stop_all_ends_every_thread() {
    assert_eq!(live_threads_after_ticks("assets/stop_all_threads.json", 3), 0);
}

#[test]
fn stop_other_scripts_ends_only_the_other_threads() {
    // The `forever` script is ended; the stopper itself finished on its own.
    assert_eq!(live_threads_after_ticks("assets/stop_other_scripts.json", 3), 0);
}
