//! End-to-end test for three real gaps found by auditing every remaining
//! scratch-vm block category against this project's own supported-opcode
//! list (the same technique that fully closed out Motion), batched into
//! one round since each is small and self-contained:
//!
//! - `sound_volume` -- the Sound category's own dedicated reporter for
//!   the calling instance's current volume (`interpreter.br`'s own
//!   `Volume` case).
//! - `looks_nextbackdrop` -- the Stage's own equivalent of `looks_
//!   nextcostume` (`NextBackdrop`'s own case).
//! - `event_whenstageclicked` -- a genuinely separate hat opcode from
//!   `event_whenthisspriteclicked` real Scratch's own editor saves when
//!   this exact block is dragged onto the Stage (confirmed against real
//!   source) -- previously silently never fired at all, since this
//!   project only ever matched the sprite variant (`check_click_hats`'s
//!   own doc comment in `scratch.br`).
//!
//! `assets/scratch_vm_category_audit.json`:
//!
//! ```text
//! Sprite1 (green flag): set volume to 55
//!                        set result_volume to (volume)       -- 55
//! Stage    (green flag): next backdrop                       "a" -> "b"
//!                        set result_backdrop to (backdrop #) -- 2
//! Stage    (when stage clicked): set result_stage_clicked to 1
//! ```

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    check_click_hats, run_greenflag_and_report_json, run_pending_threads, setup_scratch_project,
    stage_var_value, sync_keyboard_state, sync_mouse_state, ProjectPath, Vars,
};
use std::sync::Arc;

const FIXTURE: &str = "assets/scratch_vm_category_audit.json";

#[test]
fn sound_volume_reads_the_real_per_instance_volume() {
    let path: Arc<str> = Arc::from(FIXTURE);
    let get = |name: &'static str| run_greenflag_and_report_json(path.clone(), Arc::from(name));

    assert_eq!(get("result_volume").as_ref(), "55", "sound_volume should read the real SoundQueue.volumes value sound_setvolumeto just wrote, via a dedicated reporter block");
}

#[test]
fn looks_nextbackdrop_advances_the_stages_own_backdrop() {
    let path: Arc<str> = Arc::from(FIXTURE);
    let get = |name: &'static str| run_greenflag_and_report_json(path.clone(), Arc::from(name));

    assert_eq!(get("result_backdrop").as_ref(), "2", "looks_nextbackdrop should advance the Stage from \"a\" (index 0) to \"b\" (index 1), reported as costume # 2");
}

#[test]
fn event_whenstageclicked_fires_a_real_click_on_the_stage() {
    let path: Arc<str> = Arc::from(FIXTURE);
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(Update, (sync_keyboard_state, sync_mouse_state, check_click_hats, run_pending_threads).chain());

    let var = |app: &mut App, name: &'static str| {
        let vars = app.world().resource::<Vars>().clone();
        stage_var_value(Arc::from(FIXTURE), vars, Arc::from(name))
    };

    // Tick 1: nothing pressed yet, and the green-flag scripts run too
    // (harmless -- they don't touch result_stage_clicked).
    app.update();
    assert_ne!(var(&mut app, "result_stage_clicked").as_ref(), "1", "no click yet");

    // Mouse starts at the headless-fixed origin (0, 0) -- well within
    // the Stage's own fixed 480x360 bounds. Press: the Stage's own click
    // hat should fire (there's no other click hat in this fixture to
    // confuse it with -- Sprite1 has none).
    app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
    app.update();
    assert_eq!(var(&mut app, "result_stage_clicked").as_ref(), "1", "clicking anywhere on the Stage should fire its own \"when stage clicked\" hat");
}
