//! Tests `pen_stamp`'s own real geometry -- rotation, scale, and
//! nearest-neighbor sampling -- via `render_stamp_for_test` (`boring/
//! pen.br`), a headless entry point that calls `draw_stamp` directly
//! against a hand-built source image, bypassing `Assets<Image>`/the ECS
//! entirely.
//!
//! That bypass is deliberate, not a shortcut: `draw_stamps` (the real
//! Bevy system) can only be exercised through a full `App` with a real
//! costume loaded via `asset_server.load`, which is genuinely
//! asynchronous -- confirmed directly (a standalone diagnostic looping
//! `app.update()` up to 60 times, even with real sleeps between calls,
//! never advanced a local-file image load past `LoadState::Loading`
//! under `MinimalPlugins`, only `DefaultPlugins`' own full render/windowing
//! setup drives that to completion). That's an environment/test-harness
//! limitation, not a bug in this project's own stamp code -- the full
//! pipeline (asset loading -> `TargetCostumeHandles` -> `draw_stamps` ->
//! canvas) is exactly what this project's own established real-game
//! smoke-test discipline (`cargo run` against a real project) verifies
//! instead, the same way every other Pen round already has.
//!
//! `pen_down_then_move_and_a_simple_unrotated_stamp` covers the
//! straightforward case (`direction = 90`, real Scratch's own default --
//! `radians = (90 - 90) * k = 0`, no rotation at all) as a baseline.
//! `a_rotated_stamp_puts_each_source_corner_in_the_real_rotated_position`
//! is the actually interesting case: a hand-authored 2x2 source image
//! with a different solid color in each corner, stamped at
//! `direction = 180` ("facing down", `radians = (90 - 180) * k = -90°`,
//! a real quarter-turn *clockwise*). Independently hand-derived (not by
//! re-deriving `draw_stamp`'s own formula, but by reasoning about
//! physically rotating a photo 90° clockwise: whatever was at 135°
//! -- top-left -- ends up at 45° -- top-right; each other corner follows
//! the same quarter-turn): top-left -> top-right, top-right -> bottom-
//! right, bottom-left -> top-left, bottom-right -> bottom-left of the
//! *displayed*, rotated result.

use scratch_boring::render_stamp_for_test;

fn pixel_at(data: &[u8], px: u32, py: u32) -> [u8; 4] {
    let i = ((py * 480 + px) * 4) as usize;
    [data[i], data[i + 1], data[i + 2], data[i + 3]]
}

#[test]
fn a_simple_unrotated_stamp_lands_exactly_on_its_own_footprint() {
    // A single 4x4 solid-red source, stamped at stage center with
    // direction 90 (real Scratch's own default -- no rotation at all)
    // and a 4x4 world-unit destination footprint.
    let mut pixels = Vec::new();
    for _ in 0..(4 * 4) {
        pixels.extend_from_slice(&[255u8, 0, 0, 255]);
    }
    let data = render_stamp_for_test(0.0, 0.0, 90.0, "all around".into(), 0.0, &pixels, 4.0, 4.0, 4.0, 4.0);

    // Canvas (240, 180) is stage (0, 0) -- dead center of the footprint.
    assert_eq!(pixel_at(&data, 240, 180), [255, 0, 0, 255], "expected solid red at stage center");
    // Canvas (100, 100) is well outside the 4x4-world-unit footprint.
    assert_eq!(pixel_at(&data, 100, 100), [0, 0, 0, 0], "expected untouched canvas far from the stamp");
}

#[test]
fn a_rotated_stamp_puts_each_source_corner_in_the_real_rotated_position() {
    // A 2x2 source: top-left red, top-right green, bottom-left blue,
    // bottom-right yellow (row-major, row 0 = top -- matches `draw_
    // stamp`'s own `v = (half_h - local_y) / dest_height` convention:
    // `local_y = +half_h` (this sprite's own local "up") maps to `v = 0`,
    // the source image's own first/top row).
    let red = [255u8, 0, 0, 255];
    let green = [0u8, 255, 0, 255];
    let blue = [0u8, 0, 255, 255];
    let yellow = [255u8, 255, 0, 255];
    let mut pixels = Vec::new();
    pixels.extend_from_slice(&red);
    pixels.extend_from_slice(&green);
    pixels.extend_from_slice(&blue);
    pixels.extend_from_slice(&yellow);

    // Stage center, facing 180 ("down"): a real 90-degree *clockwise*
    // turn relative to direction 90's own unrotated reference. Dest
    // footprint 2x2 world units, so each quadrant's own center sits at
    // world (+/-0.5, +/-0.5) -- pixel (px, py) samples world `((px+0.5)
    // - 240, 180 - (py+0.5))` (`draw_stamp`'s own exact inverse of
    // `canvas_px`/`canvas_py`), so world (+0.5, +0.5) is canvas (240,
    // 179), (+0.5, -0.5) is (240, 180), (-0.5, +0.5) is (239, 179), and
    // (-0.5, -0.5) is (239, 180) -- each solidly inside one quadrant,
    // nowhere near a seam.
    let data = render_stamp_for_test(0.0, 0.0, 180.0, "all around".into(), 0.0, &pixels, 2.0, 2.0, 2.0, 2.0);

    // Real-world-rotation-of-a-photo derivation (not a re-derivation of
    // the code's own formula): turning the image 90 degrees clockwise
    // moves top-left -> top-right, top-right -> bottom-right,
    // bottom-right -> bottom-left, bottom-left -> top-left.
    assert_eq!(pixel_at(&data, 240, 179), red, "top-left source pixel should land top-right after a 90-degree clockwise turn");
    assert_eq!(pixel_at(&data, 240, 180), green, "top-right source pixel should land bottom-right after a 90-degree clockwise turn");
    assert_eq!(pixel_at(&data, 239, 179), blue, "bottom-left source pixel should land top-left after a 90-degree clockwise turn");
    assert_eq!(pixel_at(&data, 239, 180), yellow, "bottom-right source pixel should land bottom-left after a 90-degree clockwise turn");
}
