//! End-to-end test for `motion_setrotationstyle` -- a real block this
//! project's own rotation-style *rendering* support (an earlier round)
//! didn't yet let a script actually invoke: rotation style used to be a
//! read-only Bevy Component, baked in once at Startup from each target's
//! own saved `rotationStyle` and never touched again. See `SoundQueue.
//! rotation_styles`' own doc comment in `runtime.br` for why this needed
//! moving into a real per-instance resource instead.
//!
//! `assets/motion_setrotationstyle.json`: Sprite1's saved `rotationStyle`
//! is "all around", starting direction 180 (facing down). Its green-flag
//! script:
//!
//! ```text
//! set rotation style to "left-right"   -- must override the saved default
//! point in direction -90               -- facing left
//! create clone of myself               -- must inherit the CHANGED style
//! ```
//!
//! Both Sprite1 and its clone should end up with no `Transform` rotation
//! at all (`"left-right"` never rotates the transform) and `Sprite.
//! flip_x = true` (facing left, direction < 0) -- despite Sprite1's own
//! *saved* style being "all around", which would have shown a real
//! +90-degree rotation instead had `motion_setrotationstyle` not actually
//! taken effect.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    play_queued_sounds, run_pending_threads, setup_scratch_project, spawn_pending_clones,
    sync_transforms_from_positions, ProjectPath, ScratchSprite,
};
use std::sync::Arc;

fn z_rotation_radians(q: Quat) -> f32 {
    2.0 * q.z.atan2(q.w)
}

#[test]
fn a_script_can_change_its_own_rotation_style_and_a_clone_inherits_the_change() {
    let path: Arc<str> = Arc::from("assets/motion_setrotationstyle.json");
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
                sync_transforms_from_positions,
            )
                .chain(),
        );
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&ScratchSprite, &Transform, &Sprite)>();
    let found: std::collections::HashMap<String, (f32, bool)> = query
        .iter(world)
        .map(|(sprite, transform, spr)| (sprite.target_id.to_string(), (z_rotation_radians(transform.rotation), spr.flip_x)))
        .collect();

    let eps = 0.001;
    let (rot, flip) = found.get("Sprite1").expect("Sprite1 should exist");
    assert!(rot.abs() < eps, "Sprite1 switched to \"left-right\" at runtime -- it must not rotate its Transform at all, got {rot} radians (would be nonzero if the saved \"all around\" default were used instead)");
    assert!(*flip, "Sprite1 is now facing direction -90 (left) under \"left-right\" -- it must be flipped");

    let (clone_id, (clone_rot, clone_flip)) = found
        .iter()
        .find(|(id, _)| id.starts_with("Sprite1_clone_"))
        .unwrap_or_else(|| panic!("a Sprite1_clone_* entity should exist, got: {found:?}"));
    let _ = clone_id;
    assert!(clone_rot.abs() < eps, "the clone must inherit Sprite1's CHANGED rotation style (\"left-right\"), not its origin's original saved \"all around\", got {clone_rot} radians");
    assert!(*clone_flip, "the clone should also be flipped, inheriting both the changed style and the changed direction");
}
