//! End-to-end test for `sensing_setdragmode` and the real click-and-drag
//! mouse interaction it unlocks (`handle_sprite_dragging`, `scratch.br`)
//! -- previously a documented no-op (`sensing_setdragmode` had nothing to
//! toggle, since no drag interaction existed at all).
//!
//! `assets/drag.json`: two sprites, both 40x40 placeholder boxes at
//! (0, 0) (no real `.sb3` costume assets) -- "NotDraggable" (never made
//! draggable) and "Draggable" (its own green-flag script calls
//! `set drag mode to [draggable]`, layered in front of "NotDraggable").
//!
//! No `WindowPlugin` (headless `MinimalPlugins`) -- `MouseState` never
//! gets synced from a real window in that configuration, so this sets it
//! directly between ticks, the same technique `tests/click_hats.rs`
//! already established.
//!
//! Exercises: a non-draggable sprite is never picked even when it
//! overlaps the click point; the draggable sprite IS picked (proving the
//! AABB-hit-test + draggable-gating + z-order pick all work); the sprite
//! stays glued to its own grabbed point as the mouse moves (the "grab
//! offset stays constant" formula, not a snap-to-cursor-center); and
//! releasing the mouse stops the drag (the sprite stays put, not
//! continuing to follow further mouse movement).

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    check_key_press_hats, handle_sprite_dragging, position_value, run_pending_threads,
    setup_scratch_project, sync_keyboard_state, sync_mouse_state, MouseState, Positions,
    ProjectPath,
};
use std::sync::Arc;

fn test_app() -> App {
    let path: Arc<str> = Arc::from("assets/drag.json");
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
                sync_keyboard_state,
                sync_mouse_state,
                check_key_press_hats,
                run_pending_threads,
                handle_sprite_dragging,
            )
                .chain(),
        );
    app
}

fn pos(app: &mut App, target_name: &'static str, axis: &'static str) -> Arc<str> {
    let positions = app.world().resource::<Positions>().clone();
    position_value(positions, Arc::from(target_name), Arc::from(axis))
}

#[test]
fn dragging_moves_only_the_draggable_sprite_and_stops_on_release() {
    let mut app = test_app();

    // Tick 1: green flag runs, "Draggable" calls set drag mode to
    // [draggable]. Mouse starts at the headless-fixed origin (0, 0) --
    // both sprites' own position, well inside either 40x40 box.
    app.update();
    assert_eq!(pos(&mut app, "Draggable", "x").as_ref(), "0");
    assert_eq!(pos(&mut app, "NotDraggable", "x").as_ref(), "0");

    // Press: "Draggable" is the only draggable sprite here, so it (not
    // "NotDraggable", even though both overlap the click point) should be
    // the one that starts following the mouse.
    app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
    app.update();

    // Move the mouse by (50, 30) and let another tick run -- "Draggable"
    // (grabbed at its own origin, offset (0, 0)) should move by that same
    // exact delta; "NotDraggable" must not move at all.
    app.world_mut().resource_mut::<MouseState>().x = 50.0;
    app.world_mut().resource_mut::<MouseState>().y = 30.0;
    app.update();
    assert_eq!(pos(&mut app, "Draggable", "x").as_ref(), "50", "the draggable sprite should follow the mouse by the same delta it was grabbed at");
    assert_eq!(pos(&mut app, "Draggable", "y").as_ref(), "30");
    assert_eq!(pos(&mut app, "NotDraggable", "x").as_ref(), "0", "the non-draggable sprite must never move, even though it overlapped the same click point");
    assert_eq!(pos(&mut app, "NotDraggable", "y").as_ref(), "0");

    // Release, then move the mouse further -- "Draggable" must stay right
    // where it was dropped, not keep following.
    app.world_mut().resource_mut::<ButtonInput<MouseButton>>().release(MouseButton::Left);
    app.world_mut().resource_mut::<MouseState>().x = 999.0;
    app.world_mut().resource_mut::<MouseState>().y = 999.0;
    app.update();
    assert_eq!(pos(&mut app, "Draggable", "x").as_ref(), "50", "releasing the mouse should stop the drag -- the sprite must not keep following");
    assert_eq!(pos(&mut app, "Draggable", "y").as_ref(), "30");
}
