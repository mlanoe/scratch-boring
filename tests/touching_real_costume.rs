//! Verifies `TargetCostumeDims` reports a real `.sb3` costume's *actual*
//! decoded pixel dimensions, not just the 40x40 placeholder fallback
//! `tests/touching.rs`'s raw-JSON fixtures exercise. Cross-checked against
//! an independent Rust-side PNG header parse of the very same cached file
//! (real width/height bytes read directly, not reusing `boring/assets.br`'s
//! own `png_dimensions` logic) -- proves the pure-Boring PNG-header parser
//! (`assets.br`) agrees with reality, not just with itself.

use scratch_boring::{extract_current_costume_to_temp_file, run_report_costume_dims};
use std::sync::Arc;

/// Reads a PNG's width/height straight out of its IHDR chunk -- the same
/// four big-endian bytes at the same fixed offsets `png_dimensions`
/// (`boring/assets.br`) parses, but written independently in Rust so this
/// test isn't just checking Boring's parser against itself.
fn real_png_dimensions(bytes: &[u8]) -> (u32, u32) {
    assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47], "expected a real PNG signature");
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    (width, height)
}

#[test]
fn sprite1s_current_costume_reports_its_real_rasterized_pixel_size() {
    let path: Arc<str> = Arc::from("assets/Fibonacci_1.sb3");

    // Extract (and rasterize) the same costume `TargetCostumeDims` itself
    // extracts, then read its real dimensions directly off disk.
    let cache_relative = extract_current_costume_to_temp_file(path.clone(), Arc::from("Sprite1"))
        .expect("Sprite1's current costume should extract and rasterize to a real PNG");
    let full_path = format!("assets/{cache_relative}");
    let bytes = std::fs::read(&full_path).unwrap_or_else(|e| panic!("cache file {full_path} unreadable: {e}"));
    let (expected_w, expected_h) = real_png_dimensions(&bytes);
    assert!(expected_w > 0 && expected_h > 0, "a real rasterized costume should have a nonzero size");

    let reported_w: f32 = run_report_costume_dims(path.clone(), Arc::from("Sprite1"), 0, Arc::from("width"))
        .parse()
        .expect("width should be a real number");
    let reported_h: f32 = run_report_costume_dims(path, Arc::from("Sprite1"), 0, Arc::from("height"))
        .parse()
        .expect("height should be a real number");

    assert_eq!(reported_w as u32, expected_w, "TargetCostumeDims' reported width should match the real PNG's own IHDR width");
    assert_eq!(reported_h as u32, expected_h, "TargetCostumeDims' reported height should match the real PNG's own IHDR height");
}

#[test]
fn an_out_of_range_costume_index_falls_back_to_the_placeholder_box() {
    let path: Arc<str> = Arc::from("assets/Fibonacci_1.sb3");
    let w = run_report_costume_dims(path.clone(), Arc::from("Sprite1"), 9999, Arc::from("width"));
    let h = run_report_costume_dims(path, Arc::from("Sprite1"), 9999, Arc::from("height"));
    assert_eq!(w.as_ref(), "40", "an out-of-range costume index should fall back to TargetCostumeDims' own documented 40x40 default");
    assert_eq!(h.as_ref(), "40");
}
