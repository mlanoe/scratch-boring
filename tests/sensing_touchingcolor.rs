//! Tests `sensing_touchingcolor`'s own real geometry/color-matching
//! logic (`scan_scene_for_color`, `runtime.br`) via `run_report_
//! touching_color_test` (`boring/scheduler.br`), a headless entry point
//! that hands it a hand-built 480x360 scene buffer directly, bypassing
//! `compose_scene_for_touching`/`Assets<Image>`/the ECS entirely.
//!
//! That bypass is deliberate, not a shortcut -- the same reasoning
//! `tests/pen_stamp.rs` already established for `render_stamp_for_test`:
//! exercising the *real* pipeline end to end needs real costume assets
//! to actually finish loading via `asset_server.load`, which a headless
//! `MinimalPlugins` test can't drive to completion at all (confirmed
//! empirically, `render_stamp_for_test`'s own doc comment). The real,
//! full pipeline (`compose_scene_for_touching` compositing a real
//! backdrop/sprites, `TouchingColor`'s own `eval` case reading it) is
//! exactly what this project's own established real-game smoke-test
//! discipline (`cargo run` against a real project) verifies instead.
//!
//! Scene buffer: opaque white everywhere (real Scratch's own stage
//! background default) except a solid red square, canvas x in
//! [200, 280) / y in [140, 220) -- stage coordinates roughly
//! (-40..40, -40..40) around stage center, using the exact same
//! `canvas_px`/`canvas_py` conversion `scan_scene_for_color`'s own
//! caller uses (`stage_x + 240`/`180 - stage_y`).
//!
//! Real scratch-render's own pixel-comparison tolerance (`pen_colors_
//! touch`) is exercised too: querying a color a few steps off from pure
//! red/white (still within the real 5-bit R/G, 4-bit B tolerance) still
//! reports touching, not just an exact-bytes match.

use scratch_boring::run_report_touching_color_test;

fn build_scene_with_red_square() -> Vec<u8> {
    let mut data = vec![255u8; 480 * 360 * 4];
    for py in 140..220 {
        for px in 200..280 {
            let i = ((py * 480 + px) * 4) as usize;
            data[i] = 255;
            data[i + 1] = 0;
            data[i + 2] = 0;
            data[i + 3] = 255;
        }
    }
    data
}

#[test]
fn a_sprite_over_the_red_square_reports_touching_red() {
    let scene = build_scene_with_red_square();
    // Stage center, half-width/height 10 -- squarely inside the square.
    assert!(run_report_touching_color_test(&scene, 0.0, 0.0, 10.0, 10.0, "#ff0000".into()));
}

#[test]
fn a_sprite_far_from_the_red_square_does_not_touch_red_but_touches_the_white_background() {
    let scene = build_scene_with_red_square();
    assert!(!run_report_touching_color_test(&scene, 200.0, 150.0, 5.0, 5.0, "#ff0000".into()));
    assert!(run_report_touching_color_test(&scene, 200.0, 150.0, 5.0, 5.0, "#ffffff".into()));
}

#[test]
fn a_sprite_over_the_red_square_does_not_touch_an_unrelated_color() {
    let scene = build_scene_with_red_square();
    assert!(!run_report_touching_color_test(&scene, 0.0, 0.0, 10.0, 10.0, "#00ff00".into()));
}

#[test]
fn real_scratch_renders_own_pixel_tolerance_still_matches_a_slightly_off_color() {
    let scene = build_scene_with_red_square();
    // #f80000 differs from #ff0000 only in the red channel's own low 3
    // bits (0xff & 0b11111000 == 0xf8 & 0b11111000) -- still "touching"
    // under real scratch-render's own tolerance, not an exact-bytes miss.
    assert!(run_report_touching_color_test(&scene, 0.0, 0.0, 10.0, 10.0, "#f80000".into()));
}
