//! Regression test for a second real crash found the same round as
//! `tests/wav_decode_regression.rs` -- `bevy`'s own default `audio`
//! feature has no MP3 codec either (only `vorbis`), and a real sb3
//! sound's own `dataFormat` isn't always `"wav"`: `Coconut clicker`'s
//! own imported "Popsicle" sound is `dataFormat: "mp3"` (a real, valid
//! sb3 possibility, not a one-off quirk of that one demo). Without any
//! MP3 codec, `symphonia`'s own format probe scans up to its default
//! byte limit looking for a recognizable format boundary and gives up:
//! ```text
//! ERROR symphonia_core::probe: reached probe limit of 1048576 bytes.
//! thread 'Compute Task Pool (3)' panicked at bevy_audio-0.19.1/src/audio_source.rs:101:14:
//! called `Result::unwrap()` on an `Err` value: UnrecognizedFormat
//! ```
//! Found by actually running a second real downloaded project right
//! after the WAV fix landed. Fixed with `bevy`'s own plain `mp3` feature
//! (`Cargo.toml`'s own `bevy` dependency comment) -- unlike WAV/vorbis,
//! MP3 has no separate `symphonia-mp3` alias exposed at the top-level
//! `bevy` crate, confirmed against its own Cargo.toml.
//!
//! Same technique as the WAV regression test: decode a real MP3 byte
//! buffer through the *exact* `rodio::Decoder::builder()...build()` call
//! `bevy_audio`'s own `Decodable::decoder` makes, headlessly.
//!
//! The bytes below are a genuinely tiny (0.1s, 32kbps), purely
//! *synthesized* 440Hz sine tone encoded with `ffmpeg`/`libmp3lame`
//! locally -- not extracted from any real project's own third-party
//! audio asset (this project deliberately never commits third-party
//! `.sb3`/sound content, same reasoning `assets/coconut_clicker_demo.sb3`
//! itself was never checked in).

use rodio::Decoder;
use std::io::Cursor;

#[rustfmt::skip]
const TINY_SINE_MP3: &[u8] = &[
    73, 68, 51, 4, 0, 0, 0, 0, 0, 35, 84, 83, 83, 69, 0, 0, 0, 15, 0, 0, 3, 76, 97, 118, 102, 54,
    50, 46, 49, 50, 46, 49, 48, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 243, 112, 192, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 73, 110, 102, 111, 0, 0, 0, 15, 0, 0, 0, 6, 0, 0, 3, 41, 0, 90, 90, 90, 90, 90,
    90, 90, 90, 90, 90, 90, 90, 90, 90, 90, 90, 123, 123, 123, 123, 123, 123, 123, 123, 123, 123,
    123, 123, 123, 123, 123, 123, 123, 156, 156, 156, 156, 156, 156, 156, 156, 156, 156, 156, 156,
    156, 156, 156, 156, 189, 189, 189, 189, 189, 189, 189, 189, 189, 189, 189, 189, 189, 189, 189,
    189, 189, 222, 222, 222, 222, 222, 222, 222, 222, 222, 222, 222, 222, 222, 222, 222, 222, 222,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 76,
    97, 118, 99, 54, 50, 46, 50, 56, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 2, 163, 0, 0, 0, 0, 0,
    0, 3, 41, 96, 124, 38, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 243, 64, 196, 0, 20,
    104, 134, 112, 23, 88, 24, 0, 127, 228, 160, 38, 58, 99, 166, 58, 67, 164, 90, 155, 192, 238,
    130, 164, 45, 153, 132, 230, 216, 156, 234, 117, 153, 181, 32, 161, 169, 187, 142, 238, 70, 33,
    135, 241, 252, 140, 70, 41, 44, 59, 129, 129, 129, 139, 7, 222, 32, 4, 1, 12, 184, 127, 163,
    119, 47, 225, 142, 3, 127, 12, 114, 254, 224, 196, 64, 9, 131, 249, 48, 65, 216, 12, 255, 119,
    229, 195, 224, 128, 99, 74, 108, 68, 80, 194, 6, 3, 2, 1, 0, 255, 243, 66, 196, 9, 22, 201, 114,
    157, 159, 154, 104, 2, 0, 0, 24, 18, 97, 105, 63, 114, 94, 37, 146, 17, 69, 134, 13, 33, 6, 204,
    4, 180, 11, 76, 91, 34, 107, 82, 0, 19, 232, 200, 158, 137, 136, 150, 254, 22, 224, 90, 129, 90,
    252, 145, 30, 163, 212, 203, 252, 115, 12, 49, 52, 122, 143, 95, 252, 200, 188, 94, 49, 46, 151,
    82, 255, 252, 188, 73, 24, 151, 75, 166, 69, 226, 241, 223, 242, 161, 32, 104, 74, 18, 6, 138,
    213, 22, 73, 91, 18, 73, 7, 239, 28, 100, 255, 243, 64, 196, 9, 17, 160, 86, 88, 127, 221, 0, 2,
    138, 4, 23, 13, 140, 5, 9, 0, 66, 113, 130, 226, 241, 146, 99, 193, 246, 234, 137, 147, 67, 121,
    64, 128, 215, 144, 148, 243, 191, 177, 170, 246, 99, 5, 109, 161, 170, 138, 232, 35, 58, 157,
    223, 251, 63, 206, 175, 95, 152, 178, 189, 159, 191, 103, 237, 69, 157, 189, 61, 255, 222, 229,
    13, 69, 17, 173, 17, 187, 78, 195, 88, 86, 3, 0, 144, 29, 48, 14, 18, 179, 1, 208, 207, 49, 35,
    13, 243, 22, 66, 14, 50, 255, 243, 66, 196, 29, 20, 8, 90, 44, 117, 94, 16, 0, 82, 16, 99, 242,
    227, 14, 50, 99, 20, 147, 1, 128, 150, 48, 102, 3, 227, 4, 64, 46, 42, 0, 91, 38, 128, 84, 251,
    76, 176, 230, 43, 217, 122, 117, 125, 31, 250, 159, 252, 170, 152, 191, 255, 95, 255, 103, 246,
    104, 255, 251, 116, 170, 16, 113, 199, 24, 50, 6, 211, 114, 94, 84, 1, 7, 12, 67, 9, 194, 65,
    147, 28, 213, 17, 252, 207, 40, 51, 14, 11, 141, 33, 1, 53, 128, 65, 13, 193, 94, 17, 160, 89,
    176, 255, 243, 64, 196, 40, 30, 113, 190, 116, 83, 156, 144, 0, 132, 227, 68, 46, 50, 104, 147,
    224, 15, 65, 98, 162, 29, 202, 232, 155, 129, 216, 0, 112, 134, 253, 230, 236, 155, 133, 135, 6,
    69, 22, 80, 106, 239, 211, 123, 136, 74, 57, 66, 22, 24, 209, 245, 253, 238, 251, 144, 34, 52,
    176, 77, 22, 74, 5, 47, 255, 252, 166, 76, 22, 9, 178, 96, 225, 124, 184, 97, 255, 240, 112, 19,
    12, 1, 12, 152, 127, 255, 244, 25, 48, 247, 46, 72, 227, 12, 96, 48, 205, 124, 24, 8, 255, 243,
    66, 196, 9, 21, 216, 218, 60, 1, 217, 72, 0, 40, 21, 128, 50, 86, 55, 224, 50, 131, 109, 20, 9,
    34, 150, 52, 157, 104, 130, 1, 48, 129, 102, 202, 82, 128, 85, 115, 44, 127, 167, 218, 74, 128,
    129, 66, 16, 68, 202, 200, 166, 21, 0, 36, 201, 161, 201, 74, 100, 34, 150, 174, 62, 82, 234,
    161, 61, 14, 149, 5, 186, 217, 226, 46, 163, 220, 69, 136, 148, 122, 163, 201, 196, 93, 71, 184,
    139, 207, 114, 213, 76, 65, 77, 69, 52, 46, 48, 85, 85, 85, 85, 85, 85,
];

#[test]
fn a_real_mp3_buffer_decodes_without_the_bevy_mp3_feature_missing() {
    let cursor = Cursor::new(TINY_SINE_MP3.to_vec());
    let decoded = Decoder::builder()
        .with_data(cursor)
        .with_byte_len(TINY_SINE_MP3.len() as u64)
        .build();

    assert!(
        decoded.is_ok(),
        "decoding a real MP3 buffer failed ({:?}) -- this is exactly the crash real Scratch \
         projects with an imported (non-.wav) sound hit when bevy's own `mp3` feature isn't \
         enabled (see Cargo.toml's own `bevy` dependency comment)",
        decoded.err()
    );
}
