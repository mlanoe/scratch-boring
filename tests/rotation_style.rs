//! End-to-end test for real Scratch's three rotation styles ("all
//! around"/"left-right"/"don't rotate"), ported from `RenderedTarget.
//! _getRenderedDirectionAndScale` (`rendered-target.js`) -- see `sync_
//! transforms_from_positions`'s own doc comment in `scratch.br` for the
//! exact formulas and the hand-checked sanity cases.
//!
//! `assets/rotation_style.json`: five sprites, each with a single green-
//! flag script (`point in direction <N>`) and its own `rotationStyle`:
//!
//! ```text
//! AllAroundUp    ("all around", direction 0)    -> rotation +90 deg CCW, no flip
//! AllAroundDown  ("all around", direction 180)  -> rotation -90 deg (90 deg CW), no flip
//! LeftRightLeft  ("left-right", direction -90)  -> no rotation, flip_x = true
//! LeftRightRight ("left-right", direction 90)   -> no rotation, flip_x = false
//! DontRotate     ("don't rotate", direction -90) -> no rotation, flip_x = false
//! ```

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    play_queued_sounds, run_pending_threads, setup_scratch_project, sync_transforms_from_positions,
    ProjectPath, ScratchSprite,
};
use std::sync::Arc;

/// Recovers the original `Quat::from_rotation_z(radians)` angle back out
/// of a quaternion known to only ever rotate around Z -- `2 *
/// atan2(z, w)` is the standard inverse for exactly this shape.
fn z_rotation_radians(q: Quat) -> f32 {
    2.0 * q.z.atan2(q.w)
}

#[test]
fn rotation_styles_match_real_scratchs_own_rendered_direction_and_scale() {
    let path: Arc<str> = Arc::from("assets/rotation_style.json");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(Update, (run_pending_threads, play_queued_sounds, sync_transforms_from_positions).chain());
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&ScratchSprite, &Transform, &Sprite)>();
    let mut found: std::collections::HashMap<String, (f32, bool)> = query
        .iter(world)
        .map(|(sprite, transform, spr)| (sprite.target_id.to_string(), (z_rotation_radians(transform.rotation), spr.flip_x)))
        .collect();

    let eps = 0.001;
    let (rot, flip) = found.remove("AllAroundUp").expect("AllAroundUp sprite should exist");
    assert!((rot - std::f32::consts::FRAC_PI_2).abs() < eps, "\"all around\" facing up (direction 0) should rotate +90 degrees CCW from the costume's own \"facing right\" pose, got {rot} radians");
    assert!(!flip, "\"all around\" never flips, only rotates");

    let (rot, flip) = found.remove("AllAroundDown").expect("AllAroundDown sprite should exist");
    assert!((rot + std::f32::consts::FRAC_PI_2).abs() < eps, "\"all around\" facing down (direction 180) should rotate -90 degrees (90 CW), got {rot} radians");
    assert!(!flip);

    let (rot, flip) = found.remove("LeftRightLeft").expect("LeftRightLeft sprite should exist");
    assert!(rot.abs() < eps, "\"left-right\" never rotates the transform, got {rot} radians");
    assert!(flip, "\"left-right\" facing left (direction -90, negative) should flip horizontally");

    let (rot, flip) = found.remove("LeftRightRight").expect("LeftRightRight sprite should exist");
    assert!(rot.abs() < eps);
    assert!(!flip, "\"left-right\" facing right (direction 90, non-negative) should NOT flip");

    let (rot, flip) = found.remove("DontRotate").expect("DontRotate sprite should exist");
    assert!(rot.abs() < eps, "\"don't rotate\" never rotates, got {rot} radians");
    assert!(!flip, "\"don't rotate\" never flips either, regardless of direction");
}
