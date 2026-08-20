//! End-to-end test for the Type/Instance model's first phase --
//! `control_create_clone_of` (entity spawn + copied per-instance state) and
//! `control_delete_this_clone`'s real no-op-on-a-non-clone behavior.
//! `assets/cloning.json`: one sprite, `Sprite1`, whose green-flag script
//! moves to (10, 20), sets its size to 150%, clones itself ("_myself_"),
//! then calls `delete this clone` on *itself* -- exercising the "delete
//! this clone is a silent no-op when the calling instance isn't actually a
//! clone" real-Scratch semantics (see `step_thread`'s own `DeleteThisClone`
//! arm doc comment), since `Sprite1` itself never yields, so its own script
//! runs to completion within this test's single `app.update()`.
//!
//! Full end-to-end clone *self*-deletion (a clone's own thread calling
//! `delete this clone` on itself) isn't testable yet -- nothing can make a
//! clone run any script of its own until `control_start_as_clone`/hat
//! fanout (a later phase of the same Type/Instance model work) lands; this
//! test only covers what Create+Delete's own semantics guarantee today.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    despawn_pending_clones, play_queued_sounds, run_pending_threads, setup_scratch_project,
    spawn_pending_clones, sync_transforms_from_positions, sync_visibility_and_size_from_looks_states,
    ProjectPath, ScratchSprite,
};
use std::sync::Arc;

#[test]
fn create_clone_of_myself_spawns_a_second_entity_with_copied_state() {
    let path: Arc<str> = Arc::from("assets/cloning.json");
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
                sync_visibility_and_size_from_looks_states,
            )
                .chain(),
        );

    // Every block in Sprite1's own script is a leaf statement (no `wait`
    // involved) -- the whole script, including the clone-creation and the
    // no-op self-delete, finishes within this one tick. `spawn_pending_
    // clones` (chained right after `run_pending_threads`) picks up the
    // queued clone request the same tick, so the new entity already has
    // its own correct `Transform`/scale by the time this `app.update()`
    // returns -- no second tick needed.
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&ScratchSprite, &Transform)>();
    let mut found: Vec<(String, f32, f32)> = query
        .iter(world)
        .map(|(sprite, transform)| (sprite.target_id.to_string(), transform.translation.x, transform.translation.y))
        .collect();
    found.sort_by(|a, b| a.0.cmp(&b.0));

    // Stage + the original Sprite1 (never deleted -- `delete this clone`
    // on a non-clone is a no-op) + exactly one new clone entity.
    assert_eq!(found.len(), 3, "expected Stage + original + one clone, got: {found:?}");

    let clone_entry = found
        .iter()
        .find(|(id, ..)| id.starts_with("Sprite1_clone_"))
        .unwrap_or_else(|| panic!("no Sprite1_clone_* entity found among: {found:?}"));
    assert_eq!(
        (clone_entry.1, clone_entry.2),
        (10.0, 20.0),
        "the clone should start at its source's own position at the moment of cloning"
    );

    let original = found.iter().find(|(id, ..)| id == "Sprite1").expect("original Sprite1 should still exist");
    assert_eq!(
        (original.1, original.2),
        (10.0, 20.0),
        "the original itself should be untouched by cloning or by the no-op self-delete"
    );
}
