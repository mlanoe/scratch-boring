//! End-to-end test for `data_showlist`/`data_hidelist` -- dynamically
//! toggling a list watcher's visibility at runtime. Needed a real
//! architecture change first (not just a new opcode): `build_list_
//! watchers` used to gate on a monitor's own *initial* `is_visible()`,
//! so a list that started hidden had no watcher entity at all to ever
//! show later -- same upgrade `data_showvariable`/`data_hidevariable`
//! already drove for `Vars`/`VariableWatcherTag` (see `tests/watcher_
//! visibility_toggle.rs`), now mirrored onto `Lists`/`ListWatcherTag`.
//!
//! `assets/list_watcher_visibility.json`: two Stage lists, `a` (monitor
//! starts hidden) and `b` (monitor starts visible); Sprite1's own
//! green-flag script:
//!
//! ```text
//! show list "a"   -> a's watcher entity flips to Visibility::Visible
//! hide list "b"   -> b's watcher entity flips to Visibility::Hidden
//! ```

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{play_queued_sounds, run_pending_threads, setup_scratch_project, sync_keyboard_state, sync_list_watchers, sync_mouse_state, ListWatcherTag, ProjectPath};
use std::sync::Arc;

#[test]
fn show_and_hide_list_flip_the_real_watcher_visibility_component() {
    let path: Arc<str> = Arc::from("assets/list_watcher_visibility.json");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(Update, (sync_keyboard_state, sync_mouse_state, run_pending_threads, play_queued_sounds, sync_list_watchers).chain());

    // Both blocks are leaf statements (no wait involved), so the whole
    // script finishes within this one tick.
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&ListWatcherTag, &Visibility)>();
    let mut found: Vec<(String, bool)> = query.iter(world).map(|(tag, vis)| (tag.label.to_string(), *vis == Visibility::Visible)).collect();
    found.sort();

    assert_eq!(found.len(), 2, "both lists should have their own watcher entity, got: {found:?}");
    assert_eq!(found[0], ("a".to_string(), true), "\"show list a\" should flip a's watcher to visible, even though its monitor started hidden");
    assert_eq!(found[1], ("b".to_string(), false), "\"hide list b\" should flip b's watcher to hidden, even though its monitor started visible");
}
