//! End-to-end test for a real bug found while investigating rotation-style
//! support: `looks_switchcostumeto`/`looks_nextcostume`, run from a live
//! CLONE's own script, silently no-op'd -- `TargetCostumes` stays keyed by
//! *origin* name only (a clone shares its origin's own costume list, never
//! has its own), but `SwitchCostumeTo`/`NextCostume` looked it up with the
//! clone's own raw instance id, which never matches anything, so `count`
//! was always `0` and every branch short-circuited into doing nothing at
//! all. Fixed by resolving `frame.target_id` through `SoundQueue.
//! instance_type_of` first, the same established pattern `PlaySound`'s own
//! lookup already used (see `SwitchCostumeTo`'s own doc comment in
//! `interpreter.br`).
//!
//! `assets/clone_costume_switching.json`: Sprite1 has three costumes
//! `"a"`/`"b"`/`"c"`, starting on `"a"` (index 0).
//!
//! ```text
//! green flag (Sprite1, the origin):
//!   create clone of myself
//!   set result_origin to (costume #)      -- Sprite1's own, untouched: 1
//!
//! when I start as a clone (runs on the new clone C1, one tick later --
//! same lag every clone-spawned thread has, see start_as_clone.rs):
//!   switch costume to "next costume"      "a" -> "b" (index 1)
//!   set result_c1_next to (costume #)     -- 2 (was always 0 before the fix)
//!   switch costume to "c"                 by NAME -> index 2
//!   set result_c1_by_name to (costume #)  -- 3 (was always 0 before the fix)
//!   set result_c1_name to (costume name)  -- "c" (see below: this used to
//!                                             be a separately-documented
//!                                             gap of its own, closed once
//!                                             `SoundQueue` was threaded
//!                                             through `eval` too)
//! ```
//!
//! `result_c1_name` was a real, separately-documented gap when this test
//! was first written: `looks_costumenumbername`'s "name" variant reads
//! `eval` directly, which had no `SoundQueue`/`instance_type_of` access to
//! resolve a clone's origin the way `SwitchCostumeTo`/`NextCostume`
//! (`exec_simple`) already did. Closed in a later round once `eval` gained
//! that same access (see `CostumeNumberName`'s own doc comment in
//! `runtime.br`) -- this test's own assertion was deliberately written
//! against the honest *broken* value back then specifically so this fix
//! would have to touch it, rather than silently start "passing" for the
//! wrong reason.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    despawn_pending_clones, play_queued_sounds, run_pending_threads, setup_scratch_project,
    spawn_pending_clones, stage_var_value, sync_transforms_from_positions, ProjectPath, Vars,
};
use std::sync::Arc;

const FIXTURE: &str = "assets/clone_costume_switching.json";

fn var(app: &mut App, name: &'static str) -> Arc<str> {
    let vars = app.world().resource::<Vars>().clone();
    stage_var_value(Arc::from(FIXTURE), vars, Arc::from(name))
}

#[test]
fn clone_can_switch_its_own_costume_by_special_string_and_by_name() {
    let path: Arc<str> = Arc::from(FIXTURE);
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

    // Tick 1: the green-flag script creates the clone and reads its own
    // (untouched) costume #. Tick 2: the clone's own start-as-clone
    // thread actually runs (same one-tick lag as start_as_clone.rs).
    app.update();
    app.update();

    assert_eq!(var(&mut app, "result_origin").as_ref(), "1", "the origin sprite's own costume should be completely untouched by its clone's own switching");
    assert_eq!(var(&mut app, "result_c1_next").as_ref(), "2", "a clone's own \"next costume\" must actually advance its costume, not silently no-op");
    assert_eq!(var(&mut app, "result_c1_by_name").as_ref(), "3", "a clone's own \"switch costume to <name>\" must actually resolve the name against its origin's real costume list");
    assert_eq!(var(&mut app, "result_c1_name").as_ref(), "c", "a clone's own looks_costumenumbername \"name\" variant must now also resolve its origin correctly (see this file's own module doc comment for the round this closed in)");
}
