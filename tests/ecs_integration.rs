//! Headless proof that Phase 2's Bevy wiring actually runs the interpreter
//! end to end through a real `Update` system, not just compiles --
//! `MinimalPlugins` (no window/render/audio) plus `AssetPlugin` (same
//! technique as `breakout-boring/tests/ecs_integration.rs`, `AssetPlugin`
//! added on top since `setup_scratch_project` now needs a real
//! `Res<AssetServer>` for costume loading -- without it, `AssetServer`
//! doesn't exist as a resource under bare `MinimalPlugins` and the system
//! panics with "Resource does not exist"). Reuses `assets/broadcasts.json`
//! (see `tests/broadcasts.rs` for the full script listing/expected values)
//! since it's the richest existing fixture (multiple targets, both
//! `broadcast` and `broadcast and wait`) -- it's a raw-JSON fixture with no
//! real costume bytes, so every sprite still falls back to the flat-color
//! placeholder here; the costume-extraction path itself is covered by
//! `tests/costumes.rs` against a real `.sb3`.
//!
//! What this proves that `tests/broadcasts.rs` alone doesn't: that
//! `setup_scratch_project` (a Startup system) correctly loads the project,
//! inserts `Vars`/`Lists`/`ProcTable`/`Broadcasts`/`PendingThreads` as real
//! Bevy resources, spawns one `ScratchSprite` entity per target, and that
//! `run_pending_threads` (a real `Update` system) drains the queued
//! green-flag script and runs it via the *same* `run_script`/`exec`/`eval`
//! Phase 1 already tests directly -- i.e. that script execution now
//! genuinely happens on Bevy's own tick, not a plain function call from the
//! test itself.

use bevy::prelude::*;
use scratch_boring::{
    run_pending_threads, setup_scratch_project, stage_var_value, ProjectPath, ScratchSprite, Vars,
};
use std::sync::Arc;

fn test_app() -> App {
    let path: Arc<str> = Arc::from("assets/broadcasts.json");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        // AssetPlugin: gives `setup_scratch_project` a real `Res<AssetServer>`.
        // ImagePlugin: registers the `Image` asset type `Sprite::from_color`/
        // `from_image` both need -- same combo the Phase 0 spike
        // (`sprite_texture.br`) already confirmed works headless.
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(Update, run_pending_threads);
    app
}

#[test]
fn startup_spawns_one_sprite_per_target_and_inserts_runtime_resources() {
    let mut app = test_app();
    // Bevy runs the Startup schedule once, on the first `update()` call,
    // before Update -- so setup_scratch_project's inserted resources and
    // spawned entities are already present by the time this first
    // `update()` returns.
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&ScratchSprite>();
    let sprite_count = query.iter(world).count();
    // assets/broadcasts.json has 3 targets (Stage + Sprite1 + Sprite2, per
    // tests/broadcasts.rs's own doc comment).
    assert_eq!(sprite_count, 3, "expected one ScratchSprite entity per target");

    assert!(
        world.get_resource::<Vars>().is_some(),
        "Vars should have been inserted as a real Bevy resource"
    );
}

#[test]
fn update_tick_runs_the_queued_greenflag_script_via_run_pending_threads() {
    let mut app = test_app();
    // Startup (queues the green-flag script into PendingThreads) and the
    // first Update tick (steps it via run_pending_threads) both happen
    // within this single `update()` call. Since the real pausable
    // scheduler landed (`step_thread`/`Thread`, see runtime.br's/
    // interpreter.br's own doc comments), a *plain* `broadcast` (unlike
    // `broadcast and wait`) is genuine fire-and-forget: it spawns its
    // receiver(s) as brand-new threads but does *not* block the
    // broadcasting thread on them, so those receivers don't get their own
    // first step until the *next* tick -- exactly matching real Scratch's
    // own scheduler, where a plain broadcast's receivers never run
    // instantly inline either. `broadcast and wait` ("go2" in this
    // fixture) is different: it drives its receiver to completion
    // synchronously (`drive_thread_to_completion`), so `receivedC`/`done`
    // are already correct after just one tick.
    app.update();

    let vars = app.world().resource::<Vars>().clone();
    let path: Arc<str> = Arc::from("assets/broadcasts.json");
    let get = |name: &'static str| stage_var_value(path.clone(), vars.clone(), Arc::from(name));

    assert_eq!(
        get("receivedA").as_ref(),
        "false",
        "\"go\" (plain broadcast) correctly hasn't reached its receiver yet -- it was only just spawned this tick"
    );
    assert_eq!(get("receivedB").as_ref(), "false", "same reasoning as receivedA, on the other sprite");
    assert_eq!(get("receivedC").as_ref(), "true", "\"go2\" (broadcast and wait) receiver should already have run, synchronously, within this same tick");
    assert_eq!(get("done").as_ref(), "true", "the broadcasting script itself should continue and finish afterward, not block on \"go\"'s receivers");

    // One more tick steps the "go" receivers spawned above for the first
    // time -- same fixture, same final values tests/broadcasts.rs asserts
    // headlessly, just reached one tick later than the (now outdated)
    // "everything resolves in a single tick" assumption this test used to
    // document.
    app.update();
    let vars = app.world().resource::<Vars>().clone();
    let get = |name: &'static str| stage_var_value(path.clone(), vars.clone(), Arc::from(name));
    assert_eq!(get("receivedA").as_ref(), "true", "same-sprite \"go\" receiver should have run by the second tick");
    assert_eq!(get("receivedB").as_ref(), "true", "other-sprite \"go\" receiver should have run by the second tick (project-wide fan-out)");
}
