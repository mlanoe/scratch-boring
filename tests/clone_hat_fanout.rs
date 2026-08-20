//! End-to-end test for phase D of the Type/Instance model: a live clone
//! responds to ordinary hats too, not just its own `when I start as a
//! clone` script. `assets/clone_broadcast_fanout.json`: Sprite1's
//! green-flag script clones itself, then broadcasts "move"; a *separate*
//! `when I receive move` hat (also on Sprite1) moves whichever instance
//! runs it to (33, 44).
//!
//! If broadcast fanout to clones were missing (the pre-phase-D behavior),
//! only the original would ever receive "move" -- the clone would still be
//! sitting wherever it was at the moment it was cloned (the origin's own
//! position then, (0, 0)). Checking that the clone *also* ends up at
//! (33, 44), not (0, 0), is exactly what would fail without fanout.
//!
//! Same two-tick shape as `tests/start_as_clone.rs`: tick 1 creates the
//! clone and (since the clone's own `instance_types` entry is written
//! synchronously by `control_create_clone_of`, before the broadcast that
//! follows it in the same script even runs) queues a receiver thread for
//! it too; tick 2 actually runs both receivers.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    despawn_pending_clones, play_queued_sounds, run_pending_threads, setup_scratch_project,
    spawn_pending_clones, sync_transforms_from_positions, ProjectPath, ScratchSprite,
};
use std::sync::Arc;

#[test]
fn a_live_clone_also_receives_a_broadcast_its_origin_listens_for() {
    let path: Arc<str> = Arc::from("assets/clone_broadcast_fanout.json");
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

    for (id, x, y) in &found {
        if id == "Stage" {
            continue;
        }
        assert_eq!(
            (*x, *y),
            (33.0, 44.0),
            "{id} should have received the broadcast and moved to (33, 44) -- if this is the clone and it's still at (0, 0), broadcast fanout to clones is broken"
        );
    }
}
