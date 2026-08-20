//! Regression test for a real, previously-silent bug: `sensing_keypressed`'s
//! own KEY_OPTION input is a menu-shadow block (`sensing_keyoptions`) --
//! `resolve_input`'s generic inline-primitive path only covers the
//! *compacted* form (no real shadow block, a bare `[1, [10, "value"]]`
//! literal), which every hand-authored fixture in this project used until
//! now. A real editor-exported project references the shadow as a genuine
//! block instead (confirmed via the opcode-histogram check against five
//! fresh real projects -- 44 occurrences total, not covered by any of the
//! three originally tracked projects) -- `sensing_keyoptions` needed the
//! same dedicated dispatch case `sensing_touchingobjectmenu`/`sound_
//! sounds_menu`/`motion_goto_menu`/`control_create_clone_of_menu` each
//! already got, and didn't have it, so KEY_OPTION silently resolved to the
//! wrong (or an empty) value for any such project.
//!
//! `assets/sensing_keyoptions_shadow.json`: Sprite1's green-flag script
//! loops forever, reading `key [up arrow] pressed?` (via the real shadow-
//! block encoding, not the inline form) into a Stage variable.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    check_key_press_hats, run_pending_threads, setup_scratch_project, stage_var_value, sync_keyboard_state,
    sync_mouse_state, ProjectPath, Vars,
};
use std::sync::Arc;

fn var(app: &mut App, name: &'static str) -> Arc<str> {
    let vars = app.world().resource::<Vars>().clone();
    stage_var_value(Arc::from("assets/sensing_keyoptions_shadow.json"), vars, Arc::from(name))
}

#[test]
fn sensing_keypressed_resolves_a_real_shadow_block_key_option_not_just_the_inlined_form() {
    let path: Arc<str> = Arc::from("assets/sensing_keyoptions_shadow.json");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(Update, (sync_keyboard_state, sync_mouse_state, check_key_press_hats, run_pending_threads).chain());

    app.update();
    assert_eq!(
        var(&mut app, "keyResult").as_ref(),
        "false",
        "up arrow isn't pressed yet -- if KEY_OPTION had failed to resolve to \"up arrow\" at all, this would still coincidentally read false, so this alone doesn't prove the fix; the real proof is the next assertion, after actually pressing it"
    );

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ArrowUp);
    app.update();
    assert_eq!(
        var(&mut app, "keyResult").as_ref(),
        "true",
        "sensing_keypressed should read true once up arrow is actually pressed -- only possible if KEY_OPTION resolved to the real shadow block's own \"up arrow\" field value, not an empty/wrong default"
    );
}
