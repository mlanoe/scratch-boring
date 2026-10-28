//! Pure-function tests for `effects.br`'s `apply_graphic_effects` -- the six
//! non-`ghost` graphic effects (`color`/`fisheye`/`whirl`/`pixelate`/
//! `mosaic`/`brightness`), checked directly on small RGBA buffers (no ECS,
//! no `Assets<Image>`, same shape as `pen_stamp.rs`'s use of
//! `render_stamp_for_test`).

use scratch_boring::{apply_graphic_effects, effects_are_identity};

const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];

fn solid(w: usize, h: usize, px: [u8; 4]) -> Vec<u8> {
    (0..w * h).flat_map(|_| px).collect()
}

fn px(buf: &[u8], w: usize, x: usize, y: usize) -> [u8; 4] {
    let i = (y * w + x) * 4;
    [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
}

fn apply(src: &[u8], w: usize, h: usize, c: f32, f: f32, wh: f32, p: f32, m: f32, b: f32) -> Vec<u8> {
    apply_graphic_effects(&src.to_vec(),w as f32, h as f32, w as f32, h as f32, c, f, wh, p, m, b)
}

#[test]
fn identity_detected_and_pass_is_a_copy() {
    assert!(effects_are_identity(0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
    assert!(!effects_are_identity(0.0, 0.0, 0.0, 0.0, 0.0, 1.0));
    let src = solid(4, 4, RED);
    let out = apply(&src, 4, 4, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(out, src);
}

#[test]
fn brightness_adds_to_rgb_and_keeps_alpha() {
    let src = solid(2, 2, [100, 100, 100, 200]);
    let out = apply(&src, 2, 2, 0.0, 0.0, 0.0, 0.0, 0.0, 50.0);
    let p = px(&out, 2, 0, 0);
    assert!((p[0] as i32 - 227).abs() <= 1, "100/255 + 0.5 -> ~227, got {}", p[0]);
    assert_eq!(p[3], 200);
    let dark = apply(&src, 2, 2, 0.0, 0.0, 0.0, 0.0, 0.0, -100.0);
    assert_eq!(px(&dark, 2, 0, 0)[0], 0);
}

#[test]
fn color_effect_rotates_hue_100_is_half_turn() {
    // color=100 -> hue shift of 0.5 turns: red -> cyan.
    let out = apply(&solid(2, 2, RED), 2, 2, 100.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let p = px(&out, 2, 0, 0);
    assert!(p[0] <= 2 && p[1] >= 253 && p[2] >= 253, "red + 100 -> cyan, got {:?}", p);
    // color=200 is a full turn: back to red.
    let full = apply(&solid(2, 2, RED), 2, 2, 200.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let p = px(&full, 2, 0, 0);
    assert!(p[0] >= 253 && p[1] <= 2 && p[2] <= 2, "full turn -> red, got {:?}", p);
}

#[test]
fn transparent_pixels_stay_transparent_under_color_and_brightness() {
    let out = apply(&solid(2, 2, [0, 0, 0, 0]), 2, 2, 50.0, 0.0, 0.0, 0.0, 0.0, 50.0);
    assert_eq!(px(&out, 2, 1, 1)[3], 0);
}

#[test]
fn mosaic_tiles_the_image() {
    // 4x1: R R B B. mosaic=10 -> n=2 tiles, each showing the whole image
    // squashed to half width: R B R B.
    let mut src = Vec::new();
    for c in [RED, RED, BLUE, BLUE] {
        src.extend_from_slice(&c);
    }
    let out = apply(&src, 4, 1, 0.0, 0.0, 0.0, 0.0, 10.0, 0.0);
    assert_eq!(px(&out, 4, 0, 0), RED);
    assert_eq!(px(&out, 4, 1, 0), BLUE);
    assert_eq!(px(&out, 4, 2, 0), RED);
    assert_eq!(px(&out, 4, 3, 0), BLUE);
}

#[test]
fn pixelate_flattens_blocks() {
    // 8x1 gradient; pixelate=40 -> 4px blocks (dest_w / 4 = 2 blocks):
    // every pixel in a block equals one sample.
    let mut src = Vec::new();
    for i in 0..8u8 {
        src.extend_from_slice(&[i * 30, 0, 0, 255]);
    }
    let out = apply(&src, 8, 1, 0.0, 0.0, 0.0, 40.0, 0.0, 0.0);
    assert_eq!(px(&out, 8, 0, 0), px(&out, 8, 3, 0));
    assert_eq!(px(&out, 8, 4, 0), px(&out, 8, 7, 0));
    assert_ne!(px(&out, 8, 0, 0), px(&out, 8, 4, 0));
}

#[test]
fn fisheye_and_whirl_keep_center_and_dimensions() {
    let src = solid(9, 9, RED);
    for (f, w) in [(60.0, 0.0), (0.0, 180.0), (-50.0, 90.0)] {
        let out = apply(&src, 9, 9, 0.0, f, w, 0.0, 0.0, 0.0);
        assert_eq!(out.len(), src.len());
        assert_eq!(px(&out, 9, 4, 4), RED, "center unchanged for fisheye={f} whirl={w}");
    }
}

#[test]
fn whirl_rotates_off_center_content() {
    // Marker pixel off center moves under a 180 degree whirl.
    let mut src = solid(9, 9, [0, 0, 0, 0]);
    let i = (4 * 9 + 6) * 4; // x=6,y=4, right of center
    src[i..i + 4].copy_from_slice(&RED);
    let out = apply(&src, 9, 9, 0.0, 0.0, 180.0, 0.0, 0.0, 0.0);
    assert_ne!(out, src);
    assert_eq!(out.len(), src.len());
}
