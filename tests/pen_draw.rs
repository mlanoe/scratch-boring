//! End-to-end test for the Pen extension (`pen_penDown`/`pen_
//! setPenColorToColor`/`pen_setPenSizeTo`, plus the real motion-trail
//! drawing `move_position` does) -- this project's first Phase-4
//! extension, chosen over `text2speech` specifically because it's
//! entirely local (no live network dependency on a remote Scratch-Team-
//! operated API, see `BlockKind`'s own Pen-variant doc comment,
//! `runtime.br`).
//!
//! `assets/pen_draw.json`: Sprite1's green-flag script goes to (-100, 0),
//! sets pen color to pure red (`#ff0000`), sets pen size to 10, puts the
//! pen down (`pen_penDown`'s own real behavior draws an immediate dot at
//! the current position, confirmed against real source), then -- in that
//! *same* script, the *same* tick -- goes to (100, 0).
//!
//! That last `motion_gotoxy` is the real point of this test: pen trail
//! drawing is queued by `move_position` (`interpreter.br`) at the exact
//! moment each individual motion statement runs, not by a Bevy system
//! diffing `Positions` once per tick after the whole script has already
//! finished -- so a script that puts the pen down and then moves again
//! *in the same tick* (no `wait` in between, exactly what this fixture
//! does) must still draw the connecting line. A real downloaded Scratch
//! project ("Simple Pen Drawing Tool", scratch.mit.edu/projects/504719559)
//! does this dozens of times per frame (replaying a whole recorded point
//! history to support undo) and produced visibly wrong line positions
//! under the old per-tick-diff design -- this fixture is the minimal
//! reproduction of that same shape (pen down, then move, same tick).
//!
//! Verifies against the real canvas's own pixel buffer (`Assets<Image>`,
//! via `PenCanvas::get_handle`) -- not just "did it panic": the stage-
//! center pixel (canvas (240, 180), directly on the drawn line's own
//! path between (-100, 0) and (100, 0)) should be pure red with full
//! alpha, while a pixel far outside the line/dot's own reach (canvas
//! (10, 10), corresponding to the stage's own top-left corner) should
//! stay exactly the canvas's initial fully-transparent default -- proof
//! the pen painted *only* where expected, not the whole canvas.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    draw_pen_marks, run_pending_threads, setup_scratch_project, spawn_pen_canvas, sync_keyboard_state,
    sync_mouse_state, PenCanvas, ProjectPath,
};
use std::sync::Arc;

#[test]
fn pen_down_draws_a_dot_then_a_motion_trail_in_the_real_pen_color_and_size() {
    let path: Arc<str> = Arc::from("assets/pen_draw.json");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, (spawn_pen_canvas, setup_scratch_project))
        .add_systems(
            Update,
            (sync_keyboard_state, sync_mouse_state, run_pending_threads, draw_pen_marks).chain(),
        );

    // One tick: the whole green-flag script runs to completion -- goto
    // (-100, 0), set color, set size, pen down (queues an immediate dot
    // at (-100, 0)), then goto (100, 0) in that same synchronous run
    // (queues the connecting line, via `move_position`'s own pen-down
    // check at the moment that second `goto` actually moves the sprite).
    app.update();

    let handle = app.world().resource::<PenCanvas>().get_handle();
    let images = app.world().resource::<Assets<Image>>();
    let image = images.get(&handle).expect("the pen canvas image should exist in Assets<Image>");
    let data = image.data.as_ref().expect("the pen canvas should have real pixel data");

    let pixel_at = |px: u32, py: u32| -> [u8; 4] {
        let i = ((py * 480 + px) * 4) as usize;
        [data[i], data[i + 1], data[i + 2], data[i + 3]]
    };

    // Canvas (240, 180) is stage (0, 0) -- squarely on the drawn line
    // between (-100, 0) and (100, 0).
    let center = pixel_at(240, 180);
    assert!(center[0] > 200 && center[1] < 50 && center[2] < 50 && center[3] > 200, "expected a strongly red, opaque pixel at stage center (on the drawn line), got {:?}", center);

    // Canvas (10, 10) is nowhere near either the dot (at canvas x=140) or
    // the line -- should remain exactly the initial fully-transparent
    // default.
    let far = pixel_at(10, 10);
    assert_eq!(far, [0, 0, 0, 0], "a point far from the drawn line/dot should stay untouched (fully transparent), got {:?}", far);
}
