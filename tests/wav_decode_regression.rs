//! Regression test for a real bug found by actually running the game
//! (`flamingo_demo.sb3`, a real downloaded project) rather than any
//! headless method: `bevy`'s own default `audio` feature only enables
//! `vorbis` (`bevy`'s own Cargo.toml, `audio = ["bevy_audio", "vorbis"]`)
//! -- NOT any WAV codec at all. Every Scratch project's own sounds are
//! *always* `.wav` (the sb3 format's own fixed convention), so any real
//! project's very first sound would panic the whole app outright:
//! ```text
//! thread 'Compute Task Pool (0)' panicked at bevy_audio-0.19.1/src/audio_source.rs:101:14:
//! called `Result::unwrap()` on an `Err` value: UnrecognizedFormat
//! ```
//! Invisible to every other test in this suite -- none of them decode a
//! real sound byte-for-byte, only queue a `SoundRequest` headlessly
//! (`SoundQueue.play_queue`). Fixed with `bevy`'s own `symphonia-wav`
//! feature (`Cargo.toml`, see that dependency's own doc comment for why
//! not the confusingly-named plain `wav` feature, which maps to a
//! different, non-symphonia decoder).
//!
//! This test decodes a real `.wav` byte buffer through the *exact* same
//! `rodio::Decoder::builder()...build()` call `bevy_audio`'s own
//! `Decodable::decoder` implementation makes (confirmed against real
//! `bevy_audio` source), without needing a live `AudioPlugin`/audio
//! device at all -- catching a regression here doesn't require actually
//! running the rendered game again.

use rodio::Decoder;
use std::io::Cursor;

// A minimal, valid, standard PCM WAV file (16-bit mono, a handful of
// samples) -- same RIFF/WAVE/fmt/data chunk shape `file`(1) confirms for
// every real Scratch sound asset this project has ever extracted.
const MINIMAL_WAV: &[u8] = &[
    b'R', b'I', b'F', b'F', 36, 0, 0, 0, b'W', b'A', b'V', b'E', b'f', b'm', b't', b' ', 16, 0, 0,
    0, // fmt chunk size
    1, 0, // PCM
    1, 0, // mono
    0x44, 0xac, 0, 0, // 44100 Hz
    0x88, 0x58, 0x01, 0, // byte rate
    2, 0, // block align
    16, 0, // bits per sample
    b'd', b'a', b't', b'a', 0, 0, 0, 0, // data chunk, zero samples
];

#[test]
fn a_real_wav_buffer_decodes_without_the_bevy_symphonia_wav_feature_missing() {
    let cursor = Cursor::new(MINIMAL_WAV.to_vec());
    let decoded = Decoder::builder()
        .with_data(cursor)
        .with_byte_len(MINIMAL_WAV.len() as u64)
        .build();

    assert!(
        decoded.is_ok(),
        "decoding a real WAV buffer failed ({:?}) -- this is exactly the crash real Scratch \
         projects hit on their very first sound when bevy's own `symphonia-wav` feature isn't \
         enabled (see Cargo.toml's own `bevy` dependency comment)",
        decoded.err()
    );
}
