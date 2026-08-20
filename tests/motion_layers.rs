//! End-to-end test for `looks_goforwardbackwardlayers` -- the largest
//! remaining real opcode gap once `motion_glidesecstoxy` landed (28
//! occurrences). Unlike `looks_gotofrontback` (an absolute move to the
//! extreme), this moves *relative* to the current front-to-back ranking:
//! `N` layers, shifting whoever's in between by exactly one slot each --
//! not "add/subtract N from my own order value", which would be wrong
//! whenever orders have gaps. Same "check the real `Transform.translation.
//! z` `sync_z_from_layer_orders` writes" technique `tests/layering.rs`
//! already established, since no reporter exists for this (real Scratch
//! has none either).
//!
//! `assets/go_forward_layers.json`/`assets/go_backward_layers.json`
//! (based directly on `tests/layering.rs`'s own three-sprite A/B/C setup,
//! `layerOrder` 1/2/3 -- A behind, B middle, C in front):
//!
//! ```text
//! go_forward_layers.json:  A's own green-flag script does
//!   "go forward 2 layers" -> A (starting furthest back) should end up
//!   above both B and C.
//! go_backward_layers.json: C's own green-flag script does
//!   "go backward 2 layers" -> C (starting furthest front) should end up
//!   below both A and B.
//! ```

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{run_pending_threads, setup_scratch_project, sync_keyboard_state, sync_mouse_state, sync_z_from_layer_orders, ProjectPath, ScratchSprite};
use std::collections::HashMap;
use std::sync::Arc;

fn z_values(fixture: &str) -> HashMap<String, f32> {
    let path: Arc<str> = Arc::from(fixture);
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(Update, (sync_keyboard_state, sync_mouse_state, run_pending_threads, sync_z_from_layer_orders).chain());
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&ScratchSprite, &Transform)>();
    query.iter(world).map(|(s, t)| (s.target_id.to_string(), t.translation.z)).collect()
}

#[test]
fn go_forward_two_layers_moves_past_both_sprites_in_front() {
    let z = z_values("assets/go_forward_layers.json");
    assert!(z["A"] > z["B"], "A went forward 2 layers from furthest back, should end up above B: {z:?}");
    assert!(z["A"] > z["C"], "A went forward 2 layers from furthest back, should end up above C too: {z:?}");
}

#[test]
fn go_backward_two_layers_moves_past_both_sprites_behind() {
    let z = z_values("assets/go_backward_layers.json");
    assert!(z["C"] < z["A"], "C went backward 2 layers from furthest front, should end up below A: {z:?}");
    assert!(z["C"] < z["B"], "C went backward 2 layers from furthest front, should end up below B too: {z:?}");
}
