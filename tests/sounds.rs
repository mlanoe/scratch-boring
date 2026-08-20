//! End-to-end test for Phase 2's real sound loading
//! (`extract_sound_to_temp_file`, in `boring/scratch.br` -- ported from the
//! former hand-written `src/sound_rt.rs` to pure Boring source, the same
//! way `tests/costumes.rs` documents for costumes; see that function's own
//! comment above it). Uses `assets/Fibonacci_1.sb3` (the same real fixture
//! `tests/costumes.rs` uses) since it's the only fixture with real embedded
//! assets: Sprite1 has a sound named "Meow", the Stage has one named "pop",
//! both `.wav`.

use scratch_boring::extract_sound_to_temp_file;
use std::sync::Arc;

#[test]
fn extracts_a_real_wav_sound_and_writes_a_readable_cache_file() {
    let path: Arc<str> = Arc::from("assets/Fibonacci_1.sb3");
    let out = extract_sound_to_temp_file(
        path,
        Arc::from("Sprite1"),
        Arc::from("Meow"),
    )
    .expect("Sprite1's \"Meow\" sound should extract");

    assert!(out.ends_with(".wav"), "expected a .wav cache file, got: {out}");
    let full_path = format!("assets/{out}");
    let bytes = std::fs::read(&full_path).unwrap_or_else(|e| panic!("cache file {full_path} unreadable: {e}"));
    // RIFF/WAVE magic bytes -- proves this is a real, decodable WAV file,
    // not just an empty or garbage file.
    assert_eq!(&bytes[0..4], b"RIFF", "expected a RIFF header");
    assert_eq!(&bytes[8..12], b"WAVE", "expected a WAVE header");
}

#[test]
fn extracts_the_stage_sound_too() {
    let path: Arc<str> = Arc::from("assets/Fibonacci_1.sb3");
    let out = extract_sound_to_temp_file(path, Arc::from("Stage"), Arc::from("pop"))
        .expect("Stage's \"pop\" sound should extract");
    assert!(std::path::Path::new(&format!("assets/{out}")).exists());
}

#[test]
fn a_raw_json_fixture_has_no_real_asset_bytes_and_returns_none() {
    let path: Arc<str> = Arc::from("assets/broadcasts.json");
    let out = extract_sound_to_temp_file(
        path,
        Arc::from("Sprite1"),
        Arc::from("Meow"),
    );
    assert!(out.is_none());
}

#[test]
fn an_unknown_sound_name_returns_none_rather_than_panicking() {
    let path: Arc<str> = Arc::from("assets/Fibonacci_1.sb3");
    let out = extract_sound_to_temp_file(
        path,
        Arc::from("Sprite1"),
        Arc::from("NoSuchSound"),
    );
    assert!(out.is_none());
}
