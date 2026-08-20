//! End-to-end test for Phase C of the sprite-local variables & lists plan
//! -- a sprite-local variable's on-stage watcher, previously always
//! silently skipped (`build_variable_watchers` used to drop any monitor
//! whose id didn't resolve to a *Stage* variable, see `tests/scratch_vm_
//! fixtures.rs`'s own `#[ignore]`d fixture tests, updated alongside this
//! one for the new, correct behavior). No existing checked-in fixture had
//! a visible sprite-local-variable monitor to exercise this against, so
//! this is a fresh hand-authored one.
//!
//! `assets/sprite_local_watcher.json`: sprite A has its own local
//! variable `score` (initial `5`), with a visible monitor. A's own
//! green-flag script changes it by `3` (5 -> 8) -- the watcher should
//! show the *live*, post-mutation value, not the initial one.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{play_queued_sounds, run_pending_threads, setup_scratch_project, sync_keyboard_state, sync_mouse_state, sync_variable_watchers, ProjectPath, VariableWatcherTag};
use std::sync::Arc;

#[test]
fn a_sprite_local_variables_watcher_shows_its_own_live_value() {
    let path: Arc<str> = Arc::from("assets/sprite_local_watcher.json");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(Update, (sync_keyboard_state, sync_mouse_state, run_pending_threads, play_queued_sounds, sync_variable_watchers).chain());

    // A leaf statement (no wait involved), finishes within this one tick.
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&VariableWatcherTag, &Text2d, &Visibility)>();
    let mut found: Vec<(String, String, bool)> = query.iter(world).map(|(tag, text, vis)| (tag.label.to_string(), text.0.clone(), *vis == Visibility::Visible)).collect();

    assert_eq!(found.len(), 1, "A's own local \"score\" should have its own watcher entity now, got: {found:?}");
    let (label, text, visible) = found.remove(0);
    assert_eq!(label, "score");
    assert_eq!(text, "score: 8", "the watcher should show the live value after A's own script changed it by 3 (5 -> 8), not the initial 5");
    assert!(visible, "this monitor was saved visible");
}
