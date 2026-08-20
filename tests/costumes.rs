//! End-to-end test for Phase 2's real costume loading
//! (`extract_current_costume_to_temp_file`, in `boring/scratch.br` --
//! ported from the former hand-written `src/costume_rt.rs` to pure Boring
//! source once `boring.toml`'s `[external_fns]` section could express the
//! `&`/`&mut` argument borrows zip/resvg's real signatures need; see that
//! file's own comment above the function). Uses `assets/Fibonacci_1.sb3`
//! (jscratch's own real `.sb3` fixture, already the basis of
//! `tests/fibonacci.rs`) for the SVG-rasterization path -- it's the only
//! fixture checked into this repo with real embedded assets, and every one
//! of its costumes is SVG. The bitmap-passthrough path, the nested-zip
//! shape, and the "asset genuinely missing" vs. "asset present but corrupt"
//! distinction are all covered separately against real scratch-vm fixtures
//! in `tests/scratch_vm_fixtures.rs` (`missing_png_sb3_extraction_returns_
//! none_rather_than_panicking`, `corrupt_png_sb3_extraction_passes_through_
//! corrupted_bytes_unvalidated`, `missing_svg_sb3_loads_despite_a_nested_
//! zip_and_extraction_returns_none`, `corrupt_svg_sb3_extraction_returns_
//! none_for_unparseable_xml`) -- not duplicated here.

use scratch_boring::extract_current_costume_to_temp_file;
use std::sync::Arc;

#[test]
fn extracts_and_rasterizes_a_real_svg_costume_to_a_readable_png_temp_file() {
    let path: Arc<str> = Arc::from("assets/Fibonacci_1.sb3");
    let out = extract_current_costume_to_temp_file(path, Arc::from("Sprite1"))
        .expect("Sprite1's current costume (an SVG) should extract and rasterize");

    assert!(out.ends_with(".png"), "SVG costumes should be rasterized to PNG, got: {out}");
    // `out` is relative to the *assets* directory (what `AssetServer::load`
    // expects -- see the function's own doc comment on why it's not an
    // absolute path), so reading it back directly needs the "assets/"
    // prefix restored.
    let full_path = format!("assets/{out}");
    let bytes = std::fs::read(&full_path).unwrap_or_else(|e| panic!("cache file {full_path} unreadable: {e}"));
    // PNG magic bytes -- proves this is a real, decodable image, not just
    // an empty or garbage file.
    assert_eq!(&bytes[0..8], b"\x89PNG\r\n\x1a\n", "expected a valid PNG signature");
}

#[test]
fn extracts_the_stage_backdrop_too() {
    let path: Arc<str> = Arc::from("assets/Fibonacci_1.sb3");
    let out = extract_current_costume_to_temp_file(path, Arc::from("Stage"))
        .expect("Stage's current costume (backdrop1, an SVG) should extract and rasterize");
    assert!(std::path::Path::new(&format!("assets/{out}")).exists());
}

#[test]
fn a_raw_json_fixture_has_no_real_asset_bytes_and_returns_none() {
    // The whole point of the fallback in `setup_scratch_project`: a
    // hand-authored fixture isn't a zip at all, so this must fail cleanly
    // (not panic) and let the caller fall back to a placeholder.
    let path: Arc<str> = Arc::from("assets/broadcasts.json");
    let out = extract_current_costume_to_temp_file(path, Arc::from("Sprite1"));
    assert!(out.is_none());
}

#[test]
fn an_unknown_target_name_returns_none_rather_than_panicking() {
    let path: Arc<str> = Arc::from("assets/Fibonacci_1.sb3");
    let out = extract_current_costume_to_temp_file(path, Arc::from("NoSuchSprite"));
    assert!(out.is_none());
}
