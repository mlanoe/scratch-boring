//! End-to-end test for a live CLONE's own speech bubble -- `spawn_pending_
//! clones` spawns a `SpeechBubbleAnchor` child for a new clone the same
//! way `setup_scratch_project` does for an original sprite (see that
//! struct's own doc comment in `scratch.br`).
//!
//! `assets/clone_speech_bubble.json`: Sprite1's green-flag script creates
//! a clone of itself; the clone's own "when I start as a clone" script
//! says "hi from clone". Sprite1 itself never says anything -- neither
//! does the Stage, which gets its own bubble anchor too (every target
//! does, Stage included).
//!
//! ```text
//! green flag (Sprite1):      create clone of myself
//! when I start as a clone:   say "hi from clone"    (runs on the clone,
//!                                                     one tick later)
//! ```

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    despawn_pending_clones, play_queued_sounds, run_pending_threads, setup_scratch_project,
    spawn_pending_clones, sync_speech_bubbles, sync_transforms_from_positions, ProjectPath,
    ScratchSprite, SpeechBubbleAnchor,
};
use std::sync::Arc;

#[test]
fn a_clones_own_say_lands_on_its_own_real_child_bubble_entity() {
    let path: Arc<str> = Arc::from("assets/clone_speech_bubble.json");
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
                sync_speech_bubbles,
            )
                .chain(),
        );

    // Tick 1: the green-flag script creates the clone. Tick 2: the
    // clone's own start-as-clone script (its own separate thread) runs
    // and says its own line (same one-tick lag as start_as_clone.rs).
    app.update();
    app.update();

    let world = app.world_mut();

    // Find the clone's own ScratchSprite entity (its target_id starts
    // with "Sprite1_clone_", same convention CreateCloneOf's own new_id
    // construction always uses).
    let mut sprite_query = world.query::<(Entity, &ScratchSprite)>();
    let (clone_entity, _) = sprite_query
        .iter(world)
        .find(|(_, sprite)| sprite.target_id.starts_with("Sprite1_clone_"))
        .expect("a Sprite1_clone_* entity should exist");

    let mut anchor_query = world.query::<(&SpeechBubbleAnchor, &Text2d, &ChildOf)>();
    let mut found: Vec<(String, String, Entity)> = anchor_query
        .iter(world)
        .map(|(anchor, text, child_of)| (anchor.target_id.to_string(), text.0.clone(), child_of.0))
        .collect();
    found.sort_by(|a, b| a.0.cmp(&b.0));

    assert_eq!(found.len(), 3, "Stage, Sprite1, and its clone should each have exactly one SpeechBubbleAnchor child, got: {found:?}");

    let (origin_target, origin_text, _) = found.iter().find(|(id, ..)| id == "Sprite1").expect("Sprite1's own bubble anchor should exist");
    assert_eq!(origin_target, "Sprite1");
    assert_eq!(origin_text, "", "Sprite1 itself never says anything -- its own bubble must stay empty");

    let (_, clone_text, clone_parent) = found.iter().find(|(id, ..)| id.starts_with("Sprite1_clone_")).expect("the clone's own bubble anchor should exist");
    assert_eq!(clone_text, "hi from clone", "the clone's own say should have landed on its own bubble anchor, not the origin's");
    assert_eq!(*clone_parent, clone_entity, "the clone's bubble anchor should be a real ChildOf child of the clone's own entity, not the origin's");
}
