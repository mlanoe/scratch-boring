//! Regression test for unbounded zip-entry decompression ("zip bomb") --
//! both `read_project_json_from_zip` (`boring/sb3_loader.br`, the
//! `project.json` entry itself) and `read_zip_entry_bytes`
//! (`boring/assets.br`, every costume/sound asset) called
//! `ZipFile.readToEnd` straight into an unbounded buffer, with no check on
//! the entry's own declared uncompressed size first. A `.sb3` a few KB on
//! disk with an extreme compression ratio (a real, not lied-about, giant
//! declared size -- easiest to produce here with a long run of a single
//! repeated byte, which `deflate` compresses to almost nothing) could make
//! either loading step allocate hundreds of MB to GB from a tiny download,
//! before a single byte of the entry was ever actually used.
//!
//! Fixed via `MAX_ZIP_ENTRY_BYTES` (declared once in `sb3_loader.br`,
//! reused from `assets.br` -- see its own doc comment): both entry points
//! now check `ZipFile.size()` (the entry's declared uncompressed size, a
//! cheap central-directory read) against a 100 MiB cap *before* calling
//! `ZipFile.readToEnd`, rejecting (returning `None`) an oversized entry
//! outright.

use scratch_boring::{extract_current_costume_to_temp_file, Sb3Project};
use std::io::Write;
use std::sync::Arc;

/// One byte repeated `size` times -- a real (not lied-about) entry this
/// large, but one that `deflate` compresses down to near nothing, so the
/// resulting `.sb3` on disk stays small regardless of `size`.
fn oversized_payload(size: usize) -> Vec<u8> {
    vec![b'a'; size]
}

const ONE_HUNDRED_MIB: usize = 100 * 1024 * 1024;

fn write_zip(entries: &[(&str, &[u8])]) -> std::path::PathBuf {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "scratch_boring_zip_bomb_test_{}_{}.sb3",
        std::process::id(),
        entries.len()
    ));
    let file = std::fs::File::create(&path).expect("create temp .sb3");
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();
    for (name, bytes) in entries {
        zip.start_file(*name, options).expect("start entry");
        zip.write_all(bytes).expect("write entry bytes");
    }
    zip.finish().expect("finish zip");
    path
}

#[test]
fn a_project_json_entry_declaring_an_oversized_uncompressed_size_is_rejected() {
    // Real (uncompressed-size-honest) content, just highly compressible --
    // an empty-but-valid project padded with a giant repeated-byte string
    // in an unmodeled field (serde ignores unknown top-level keys, see
    // `RawProject`'s own doc comment) so the *declared* size genuinely
    // exceeds the cap, not a forged central-directory lie.
    let padding = "a".repeat(ONE_HUNDRED_MIB + 1024);
    let oversized_json = format!(r#"{{"targets":[],"monitors":[],"_padding":"{padding}"}}"#);
    let path = write_zip(&[("project.json", oversized_json.as_bytes())]);

    let path_arc: Arc<str> = Arc::from(path.to_str().unwrap());
    let project = Sb3Project::load_for_boring(path_arc);

    assert!(project.is_none(), "an oversized project.json entry must be rejected, not decompressed");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_small_project_json_entry_still_loads_normally() {
    // Sanity control for the test above: the same zip shape, but under the
    // cap, must still load -- proves the cap itself (not some unrelated
    // regression) is what rejected the oversized case.
    let small_json = r#"{"targets":[],"monitors":[]}"#;
    let path = write_zip(&[("project.json", small_json.as_bytes())]);

    let path_arc: Arc<str> = Arc::from(path.to_str().unwrap());
    let project = Sb3Project::load_for_boring(path_arc);

    assert!(project.is_some(), "a small, well-formed project.json must still load");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_costume_asset_entry_declaring_an_oversized_uncompressed_size_is_rejected() {
    let project_json = r#"{
        "targets": [
            {
                "isStage": false,
                "name": "Sprite1",
                "variables": {},
                "lists": {},
                "broadcasts": {},
                "blocks": {},
                "comments": {},
                "currentCostume": 0,
                "costumes": [
                    {
                        "name": "costume1",
                        "dataFormat": "png",
                        "assetId": "0123456789abcdef0123456789abcdef",
                        "md5ext": "huge.png",
                        "rotationCenterX": 0,
                        "rotationCenterY": 0
                    }
                ],
                "sounds": [],
                "volume": 100,
                "layerOrder": 0
            }
        ],
        "monitors": []
    }"#;
    let huge_asset = oversized_payload(ONE_HUNDRED_MIB + 1024);
    let path = write_zip(&[("project.json", project_json.as_bytes()), ("huge.png", &huge_asset)]);

    let path_arc: Arc<str> = Arc::from(path.to_str().unwrap());
    let out = extract_current_costume_to_temp_file(path_arc, Arc::from("Sprite1"));

    assert!(out.is_none(), "an oversized costume asset entry must be rejected, not decompressed");

    let _ = std::fs::remove_file(&path);
}
