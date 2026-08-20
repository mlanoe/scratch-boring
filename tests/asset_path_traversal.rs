//! Regression test for a path-traversal-to-arbitrary-file-write bug: both
//! `extract_current_costume_to_temp_file` (costumes) and
//! `extract_sound_to_temp_file` (sounds) used to build their on-disk cache
//! filename directly out of `asset_id`/`data_format` -- plain fields
//! deserialized straight from an attacker-controlled `project.json`, with no
//! validation at all. A forged `.sb3` whose `assetId` contained `../`
//! segments could make the cache-file write land *outside*
//! `assets/.costume_cache`/`assets/.sound_cache` entirely, at a path chosen
//! by the attacker, with attacker-controlled bytes (the asset payload
//! itself) -- reachable just by loading a malicious project, no user
//! interaction beyond that needed.
//!
//! Fixed in `boring/assets.br` (`is_safe_asset_id`/
//! `costume_extension_for_format`/`sound_extension_for_format`): `asset_id`
//! must be all-hex (Scratch's own asset ids are always an MD5 digest, so a
//! `/`, `.`, or any other path separator is never legitimate), and
//! `data_format` must be one of Scratch's own closed set of costume/sound
//! formats -- both extraction functions now reject (return `None`) rather
//! than ever building a filename from unvalidated input.
//!
//! Builds a malicious `.sb3` at test time (real `zip` crate, already a
//! normal dependency of this crate) rather than checking in a fixture file,
//! since the whole point is a *hand-forged* `assetId`.

use scratch_boring::{extract_current_costume_to_temp_file, extract_sound_to_temp_file};
use std::io::Write;
use std::sync::Arc;

/// A minimal one-sprite `project.json`, with the costume/sound sections
/// filled in by the caller -- everything else is the same shape
/// `assets/broadcasts.json` (this repo's own hand-authored raw-JSON
/// fixture) already uses for a sprite target.
fn malicious_project_json(costumes_json: &str, sounds_json: &str) -> String {
    format!(
        r#"{{
  "targets": [
    {{
      "isStage": false,
      "name": "Sprite1",
      "variables": {{}},
      "lists": {{}},
      "broadcasts": {{}},
      "blocks": {{}},
      "comments": {{}},
      "currentCostume": 0,
      "costumes": [{costumes_json}],
      "sounds": [{sounds_json}],
      "volume": 100,
      "layerOrder": 0,
      "visible": true,
      "x": 0,
      "y": 0,
      "size": 100,
      "direction": 90,
      "draggable": false,
      "rotationStyle": "all around"
    }}
  ],
  "monitors": []
}}"#
    )
}

/// Writes a `.sb3` (a real zip) containing `project.json` plus one asset
/// entry (`asset_entry_name` -> `asset_bytes`), to a fresh temp file.
/// Returns the path.
fn write_malicious_sb3(project_json: &str, asset_entry_name: &str, asset_bytes: &[u8]) -> std::path::PathBuf {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "scratch_boring_traversal_test_{}_{}.sb3",
        std::process::id(),
        asset_entry_name.replace(['/', '.'], "_")
    ));
    let file = std::fs::File::create(&path).expect("create temp .sb3");
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();
    zip.start_file("project.json", options).expect("start project.json entry");
    zip.write_all(project_json.as_bytes()).expect("write project.json");
    zip.start_file(asset_entry_name, options).expect("start asset entry");
    zip.write_all(asset_bytes).expect("write asset bytes");
    zip.finish().expect("finish zip");
    path
}

#[test]
fn a_costume_asset_id_containing_path_traversal_is_rejected_and_never_written() {
    // A real assetId is an MD5 hex digest; this one is a forged traversal
    // payload instead, aimed at escaping `assets/.costume_cache/` entirely.
    let malicious_asset_id = "../../../../../../../../tmp/scratch_boring_pwned_costume";
    let costumes_json = format!(
        r#"{{
            "name": "costume1",
            "dataFormat": "svg",
            "assetId": "{malicious_asset_id}",
            "md5ext": "realasset.svg",
            "rotationCenterX": 0,
            "rotationCenterY": 0
        }}"#
    );
    let project_json = malicious_project_json(&costumes_json, "");
    // Plausible (if tiny) SVG payload -- what would have been written to
    // the attacker-chosen path had the bug still been present.
    let svg_payload = b"<svg xmlns='http://www.w3.org/2000/svg'></svg>";
    let sb3_path = write_malicious_sb3(&project_json, "realasset.svg", svg_payload);

    let path_arc: Arc<str> = Arc::from(sb3_path.to_str().unwrap());
    let out = extract_current_costume_to_temp_file(path_arc, Arc::from("Sprite1"));

    assert!(out.is_none(), "a costume with a path-traversal assetId must be rejected, not extracted");

    // The traversal target itself must never have been created.
    let escaped_target = std::path::Path::new("/tmp/scratch_boring_pwned_costume.svg");
    assert!(
        !escaped_target.exists(),
        "path traversal must not have written a file outside .costume_cache"
    );

    let _ = std::fs::remove_file(&sb3_path);
    let _ = std::fs::remove_file(escaped_target);
}

#[test]
fn a_sound_asset_id_containing_path_traversal_is_rejected_and_never_written() {
    let malicious_asset_id = "../../../../../../../../tmp/scratch_boring_pwned_sound";
    let sounds_json = format!(
        r#"{{
            "name": "sound1",
            "assetId": "{malicious_asset_id}",
            "dataFormat": "wav",
            "format": "",
            "rate": 44100,
            "sampleCount": 0,
            "md5ext": "realasset.wav"
        }}"#
    );
    let project_json = malicious_project_json("", &sounds_json);
    let wav_payload = b"RIFF....WAVEfmt ";
    let sb3_path = write_malicious_sb3(&project_json, "realasset.wav", wav_payload);

    let path_arc: Arc<str> = Arc::from(sb3_path.to_str().unwrap());
    let out = extract_sound_to_temp_file(path_arc, Arc::from("Sprite1"), Arc::from("sound1"));

    assert!(out.is_none(), "a sound with a path-traversal assetId must be rejected, not extracted");

    let escaped_target = std::path::Path::new("/tmp/scratch_boring_pwned_sound.wav");
    assert!(
        !escaped_target.exists(),
        "path traversal must not have written a file outside .sound_cache"
    );

    let _ = std::fs::remove_file(&sb3_path);
    let _ = std::fs::remove_file(escaped_target);
}

#[test]
fn a_costume_with_an_unwhitelisted_data_format_is_rejected() {
    // Not a real Scratch costume format at all -- must be rejected even
    // though `assetId` itself is a real-looking hex digest, since the
    // extension is what ends up on disk as the cache filename's suffix.
    let costumes_json = r#"{
        "name": "costume1",
        "dataFormat": "exe",
        "assetId": "0123456789abcdef0123456789abcdef",
        "md5ext": "realasset.exe",
        "rotationCenterX": 0,
        "rotationCenterY": 0
    }"#;
    let project_json = malicious_project_json(costumes_json, "");
    let sb3_path = write_malicious_sb3(&project_json, "realasset.exe", b"not a real costume");

    let path_arc: Arc<str> = Arc::from(sb3_path.to_str().unwrap());
    let out = extract_current_costume_to_temp_file(path_arc, Arc::from("Sprite1"));
    assert!(out.is_none(), "a non-whitelisted costume data_format must be rejected");

    let _ = std::fs::remove_file(&sb3_path);
}
