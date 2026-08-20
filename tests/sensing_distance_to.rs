//! End-to-end tests for `sensing_distanceto` -- a near-free follow-up once
//! `sensing_touchingobject` already built the AABB/menu-shadow machinery
//! this reuses (see that opcode's own round in README.md's "Status").
//!
//! Two separate fixtures/tests, not one -- this project only ever runs the
//! *first* `event_whenflagclicked` hat found project-wide (a real,
//! documented limitation, see README.md's roadmap), so a project-wide
//! "Stage itself always reports 10000" check needs its own single-hat
//! fixture, distinct from the sprite-to-sprite/sprite-to-mouse one (a real
//! bug in an earlier draft of this test: putting both hats in one project
//! meant only the Stage's own script ever ran, silently leaving the
//! sprite-side variables at their unset defaults -- caught by an actual
//! assertion failure, not a build error).

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    play_queued_sounds, run_pending_threads, setup_scratch_project, sync_keyboard_state, sync_mouse_state,
    sync_variable_watchers, ProjectPath, VariableWatcherTag,
};
use std::sync::Arc;

fn build_app(fixture: &str) -> App {
    let path: Arc<str> = Arc::from(fixture);
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(
            Update,
            (sync_keyboard_state, sync_mouse_state, run_pending_threads, play_queued_sounds, sync_variable_watchers).chain(),
        );
    app
}

/// `assets/distance_to.json`: sprite A (at the origin) and sprite B (at
/// (30, 40) -- a 3-4-5 triangle scaled by 10, so `distance to B` from A is
/// exactly `50`, an exactly verifiable expected value). A's own green-flag
/// script:
///
/// ```text
/// dist_to_b = distance to B            -- sqrt(30^2 + 40^2) = 50
/// dist_to_mouse = distance to mouse    -- both A and the headless-default mouse are at (0, 0) -> 0
/// ```
#[test]
fn distance_to_a_sprite_and_the_mouse_reports_real_positions() {
    let mut app = build_app("assets/distance_to.json");

    // Both blocks are leaf statements (no wait involved), so the whole
    // script finishes within this one tick.
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&VariableWatcherTag, &Text2d)>();
    let mut found: Vec<(String, String)> = query.iter(world).map(|(tag, text)| (tag.label.to_string(), text.0.clone())).collect();
    found.sort();

    assert_eq!(found.len(), 2, "expected both Stage variables to have their own watcher entity, got: {found:?}");
    assert_eq!(found[0], ("dist_to_b".to_string(), "dist_to_b: 50".to_string()), "sqrt(30^2 + 40^2) should be exactly 50");
    assert_eq!(
        found[1],
        ("dist_to_mouse".to_string(), "dist_to_mouse: 0".to_string()),
        "A and the headless-default mouse position (0, 0) should be exactly 0 apart"
    );
}

/// `assets/distance_to_stage.json`: a single-hat project where the Stage
/// itself is the one running `distance to mouse` -- real Scratch's own
/// special case (the Stage has no position to measure from, so it always
/// reports `10000`, regardless of what it's measuring to).
#[test]
fn distance_to_from_the_stage_itself_always_reports_10000() {
    let mut app = build_app("assets/distance_to_stage.json");
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&VariableWatcherTag, &Text2d)>();
    let (_, text2d) = query.iter(world).next().expect("stage_dist's own watcher entity should exist");
    assert_eq!(text2d.0, "stage_dist: 10000");
}
