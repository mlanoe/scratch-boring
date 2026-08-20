//! End-to-end tests for `control_start_as_clone` -- phase B of the Type/
//! Instance model, following [`tests/cloning.rs`]'s phase A (create/delete
//! mechanics without a clone ever running its own script). Both fixtures'
//! green-flag script is just `create clone of myself`; the interesting
//! behavior lives in each sprite's separate `when I start as a clone` hat.
//!
//! A clone's own thread is only pushed into `PendingThreads` by `spawn_
//! pending_clones` (chained after `run_pending_threads` the tick it's
//! created), so -- same one-tick lag broadcast-spawned threads already
//! have -- it doesn't actually *run* until the following tick. Every test
//! here calls `app.update()` twice for that reason.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    despawn_pending_clones, play_queued_sounds, run_pending_threads, setup_scratch_project,
    spawn_pending_clones, sync_transforms_from_positions, ProjectPath, ScratchSprite,
};
use std::sync::Arc;

fn build_app(fixture: &str) -> App {
    let path: Arc<str> = Arc::from(fixture);
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
                run_pending_threads,
                play_queued_sounds,
                spawn_pending_clones,
                despawn_pending_clones,
                sync_transforms_from_positions,
            )
                .chain(),
        );
    app
}

#[test]
fn a_clone_runs_its_own_start_as_clone_script_scoped_to_itself() {
    let mut app = build_app("assets/start_as_clone.json");

    // Tick 1: the green-flag script creates the clone and queues its
    // start-as-clone thread. Tick 2: that thread actually runs (see the
    // module doc comment for why it can't run on tick 1).
    app.update();
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&ScratchSprite, &Transform)>();
    let mut found: Vec<(String, f32, f32)> = query
        .iter(world)
        .map(|(sprite, transform)| (sprite.target_id.to_string(), transform.translation.x, transform.translation.y))
        .collect();
    found.sort_by(|a, b| a.0.cmp(&b.0));

    assert_eq!(found.len(), 3, "expected Stage + original + one clone, got: {found:?}");

    let clone_entry = found
        .iter()
        .find(|(id, ..)| id.starts_with("Sprite1_clone_"))
        .unwrap_or_else(|| panic!("no Sprite1_clone_* entity found among: {found:?}"));
    assert_eq!(
        (clone_entry.1, clone_entry.2),
        (77.0, 88.0),
        "the clone's own start-as-clone script should have moved *it*, not the original, to (77, 88)"
    );

    let original = found.iter().find(|(id, ..)| id == "Sprite1").expect("original Sprite1 should still exist");
    assert_eq!(
        (original.1, original.2),
        (0.0, 0.0),
        "the original should be untouched by its clone's own start-as-clone script"
    );
}

#[test]
fn a_clone_can_delete_itself_from_its_own_start_as_clone_script() {
    let mut app = build_app("assets/clone_self_delete.json");

    // Same two-tick shape: tick 1 creates the clone, tick 2 runs its own
    // start-as-clone script (which immediately deletes it).
    app.update();
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&ScratchSprite>();
    let mut found: Vec<String> = query.iter(world).map(|sprite| sprite.target_id.to_string()).collect();
    found.sort();

    assert_eq!(
        found,
        vec!["Sprite1".to_string(), "Stage".to_string()],
        "the clone should have deleted itself via its own start-as-clone script, leaving only Stage + the original"
    );
}
