//! End-to-end test for the four legacy (Scratch 2.0-compat) pen-color
//! blocks -- `pen_setPenHueToNumber`/`pen_setPenShadeToNumber` here
//! (`pen_changePenHueBy`/`pen_changePenShadeBy` share the exact same
//! `legacy_update_pen_color` machinery, `interpreter.br`). These are
//! `hideFromPalette: true` in real scratch-vm (no modern editor can
//! generate them) but stay fully *functional* there -- a hand-edited/
//! imported Scratch 2.0 project can still contain and run them -- so
//! this project implements them too, not just the modern color-
//! parameter blocks.
//!
//! `assets/pen_legacy_shade_hue.json`: `pen_setPenHueToNumber(HUE=0)`
//! sets the legacy hue to pure red (`HUE/2 = 0` maps onto the modern
//! `color` param's own 0..100 hue scale), `pen_setPenShadeToNumber
//! (SHADE=0)` then darkens it via real scratch-vm's own `_legacy
//! UpdatePenColor` black-mix formula (`shade < 50` mixes towards black
//! by `(10 + shade) / 60`), `pen_setPenSizeTo(10)` sets a large enough
//! dot to sample reliably, `pen_penDown` draws the dot.
//!
//! Hand-derived expected color (verified against real `scratch3_pen`/
//! `util/color.js` source, not guessed): starting from full-saturation,
//! full-brightness red (`hue100_to_full_rgb(0) = (1, 0, 0)`), shade 0
//! mixes towards black by a fraction of `(10 + 0) / 60 = 1/6` --
//! `mix_rgb01(black, red, 1/6) = (1/6, 0, 0)`. Converting that back to
//! HSV (`rgb_to_hsv100`) gives hue 0 (unchanged -- mixing with black
//! never shifts hue), saturation 100% (still fully saturated), and
//! brightness `100/6 ≈ 16.667%`. Rendered back to RGB (`pen_color_of`),
//! that's exactly `(1/6, 0, 0, 1.0)` -- as a `u8` channel, `(1/6) * 255
//! ≈ 42.5`, so the real drawn pixel should land within a couple of
//! `u8` steps of `(42 or 43, 0, 0, 255)`.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    draw_pen_marks, run_pending_threads, setup_scratch_project, spawn_pen_canvas, sync_keyboard_state,
    sync_mouse_state, PenCanvas, ProjectPath,
};
use std::sync::Arc;

#[test]
fn legacy_set_pen_hue_and_shade_produce_the_real_scratch_vm_dark_red() {
    let path: Arc<str> = Arc::from("assets/pen_legacy_shade_hue.json");
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

    app.update();

    let handle = app.world().resource::<PenCanvas>().get_handle();
    let images = app.world().resource::<Assets<Image>>();
    let image = images.get(&handle).expect("the pen canvas image should exist in Assets<Image>");
    let data = image.data.as_ref().expect("the pen canvas should have real pixel data");

    // Sprite1 stays at its saved (0, 0) -- stage center, canvas (240, 180).
    let i = ((180u32 * 480 + 240) * 4) as usize;
    let pixel = [data[i], data[i + 1], data[i + 2], data[i + 3]];

    let close = |actual: u8, expected: u8| (actual as i32 - expected as i32).abs() <= 2;
    assert!(
        close(pixel[0], 43) && pixel[1] == 0 && pixel[2] == 0 && pixel[3] == 255,
        "expected a dark, fully-saturated red dot (~43, 0, 0, 255) from HUE=0/SHADE=0, got {:?}",
        pixel
    );
}
