//! End-to-end test for Phase B of the sprite-local variables & lists plan
//! -- `control_create_clone_of`'s copy of a sprite-local variable's
//! *current* value (not its initial one), and that further mutation on
//! either the clone or the original never cross-contaminates. Also covers
//! clone-of-a-clone, using the variable's own value as both the test
//! signal and a self-limiting recursion guard (real Scratch scripts
//! guard their own clone recursion the same way -- there's no separate
//! mechanism this project would need to add just for the test).
//!
//! `assets/clone_local_variable.json`: Sprite1 has its own local variable
//! `n` (initial `1`). Green-flag script: `change n by 9` (n: 1 -> 10,
//! mutating the *original's* own copy), `create clone of myself` (the
//! new clone, call it C1, should inherit n = 10, not the initial 1),
//! `set x to n` (Sprite1's own x becomes 10). Sprite1's own `when I start
//! as a clone` hat: if `n == 10` (true only for a first-generation clone
//! like C1, whose n was just inherited as exactly 10) -- `change n by 90`
//! (n: 10 -> 100), `set x to n` (that clone's own x becomes 100), `create
//! clone of myself` (a second clone, C2, cloned *from C1*, should inherit
//! n = 100, C1's own current value, not the original's 1 or 10); else (n
//! != 10, true for C2, whose own n was inherited as 100) -- `change n by
//! 900` (n: 100 -> 1000), `set x to n` (C2's own x becomes 1000), no
//! further cloning (the guard breaks the recursion after exactly one more
//! generation).
//!
//! One clone's own `start as clone` thread only starts running the tick
//! *after* it's created (`spawn_pending_clones`' own one-tick lag, same
//! shape `tests/start_as_clone.rs` already documents) -- three ticks
//! total: tick 1 creates C1, tick 2 runs C1's own hat (mutates + creates
//! C2), tick 3 runs C2's own hat (mutates, no further cloning).

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{despawn_pending_clones, play_queued_sounds, run_pending_threads, setup_scratch_project, spawn_pending_clones, sync_transforms_from_positions, ProjectPath, ScratchSprite};
use std::sync::Arc;

#[test]
fn cloning_inherits_the_sources_current_local_variable_value_not_its_initial_one() {
    let path: Arc<str> = Arc::from("assets/clone_local_variable.json");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(Update, (run_pending_threads, play_queued_sounds, spawn_pending_clones, despawn_pending_clones, sync_transforms_from_positions).chain());

    app.update(); // tick 1: Sprite1 mutates n to 10, creates C1, sets its own x to 10
    app.update(); // tick 2: C1's own hat runs -- n was inherited as 10, mutates to 100, sets its own x, creates C2
    app.update(); // tick 3: C2's own hat runs -- n was inherited as 100 (not 10), mutates to 1000, sets its own x

    let world = app.world_mut();
    let mut query = world.query::<(&ScratchSprite, &Transform)>();
    let mut found: Vec<(String, f32)> = query.iter(world).map(|(sprite, transform)| (sprite.target_id.to_string(), transform.translation.x)).collect();
    found.sort_by(|a, b| a.0.cmp(&b.0));

    assert_eq!(found.len(), 4, "expected Stage + original + two clones (C1, C2), got: {found:?}");

    let original = found.iter().find(|(id, _)| id == "Sprite1").expect("original Sprite1 should still exist");
    assert_eq!(original.1, 10.0, "the original's own n should stay 10 (1 + 9), untouched by either clone's own further mutation");

    let mut clones: Vec<&(String, f32)> = found.iter().filter(|(id, _)| id.starts_with("Sprite1_clone_")).collect();
    clones.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    assert_eq!(clones.len(), 2, "expected exactly two clones (C1 then C2), got: {found:?}");
    assert_eq!(clones[0].1, 100.0, "C1 should have inherited the original's mutated n (10), then changed it by 90 itself (10 + 90 = 100)");
    assert_eq!(clones[1].1, 1000.0, "C2 (cloned from C1, not from the original) should have inherited C1's own current n (100), then changed it by 900 itself (100 + 900 = 1000)");
}
