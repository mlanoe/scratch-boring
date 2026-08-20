//! Regression test for unbounded JSON-deserialization recursion:
//! `boring/sb3_loader.br`'s `JVal` (the `@serde(untagged)` dynamic-JSON
//! enum backing every block's `inputs`/`fields`, a variable/list's initial
//! value, etc. -- see that enum's own header comment) is exactly as
//! recursive as whatever JSON it's asked to parse, with no depth limit at
//! all. A hand-forged `project.json` containing a value nested tens of
//! thousands of levels deep (`[[[[[...]]]]]`) crashed the whole load with a
//! stack overflow *during deserialization itself* -- independent of, and
//! before, any of `linker.br`'s own cycle/depth guards (which only run
//! *after* a project has already finished loading) ever get a chance to
//! matter.
//!
//! Fixed via `json_nesting_within_limit` (`boring/sb3_loader.br`,
//! `MAX_JSON_NESTING_DEPTH = 256`), called on the raw JSON text before
//! `fromJson::<RawProject>` -- rejects (returns `None`) anything nested
//! deeper than that, well before real deserialization ever starts walking
//! it.

use scratch_boring::Sb3Project;
use std::sync::Arc;

fn write_temp_json(name: &str, contents: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(name);
    std::fs::write(&path, contents).expect("write temp json");
    path
}

#[test]
fn a_project_json_nested_tens_of_thousands_of_levels_deep_is_rejected() {
    // An unmodeled top-level field (serde ignores unknown keys, see
    // `RawProject`'s own doc comment) whose *value* is nested 20,000
    // levels deep -- enough to overflow a real recursive-descent
    // deserializer's stack well before this project's own
    // `MAX_JSON_NESTING_DEPTH` (256) would ever be a legitimate project's
    // actual limit.
    let depth = 20_000;
    let deep_value = format!("{}{}", "[".repeat(depth), "]".repeat(depth));
    let json = format!(r#"{{"targets":[],"monitors":[],"_deep":{deep_value}}}"#);
    let path = write_temp_json("scratch_boring_deep_nesting_test.json", &json);

    let path_arc: Arc<str> = Arc::from(path.to_str().unwrap());
    let project = Sb3Project::load_json_for_boring(path_arc);

    assert!(project.is_none(), "a project.json nested tens of thousands of levels deep must be rejected, not crash");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_shallow_project_json_still_loads_normally() {
    // Sanity control: this repo's own existing raw-JSON fixture, well
    // within any reasonable depth cap, must still load fine.
    let path: Arc<str> = Arc::from("assets/broadcasts.json");
    let project = Sb3Project::load_json_for_boring(path);
    assert!(project.is_some(), "a normal, shallow project.json must still load");
}
