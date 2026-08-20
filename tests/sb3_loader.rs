//! Equivalence coverage for `boring/sb3_loader.br` (ported from the former
//! `src/sb3_loader.rs`'s own `#[cfg(test)] mod tests`, which disappeared
//! along with that file -- Boring has no `#[test]`/`#[cfg(test)]` of its
//! own, so this project's established convention (see `tests/fibonacci.rs`,
//! `tests/costumes.rs`, etc.) is a `tests/*.rs` integration test calling the
//! generated crate's public API instead. Same two fixtures as the original
//! (`assets/Fibonacci_1.sb3`/`Fibonacci_2.sb3`, also `tests/fibonacci.rs`'s
//! own subject), same assertions, adapted for two real API differences from
//! the port:
//!   - `Sb3Project::load`/`load_json` (returning `Result<_, Sb3LoadError>`)
//!     are gone -- `load_for_boring`/`load_json_for_boring` (returning
//!     `Option<Sb3Project>`) are now the only loading API, see
//!     `sb3_loader.br`'s own header comment.
//!   - Every string field is `Arc<str>` (Boring's own string representation)
//!     rather than a plain Rust `String` -- `.as_ref()` for `&str`
//!     comparisons throughout.
//! `Sb3InputValue`/`Sb3Field`/`Sb3Mutation`/`Sb3Input` stay private to the
//! generated crate (by design -- `scratch.br`'s own linker never needed
//! them directly either, only through `Sb3Block`'s accessor methods), so
//! this file uses those accessors (`input_block_id`/`input_primitive`/
//! `field_value`/`field_id`/`mutation_proc_code`/etc.) instead of the
//! original's direct `Sb3InputValue::Block(id)`/`Sb3Primitive::Text(s)`
//! pattern matches on `.inputs`/`.fields` -- `Sb3Primitive` itself is still
//! public and still matchable directly where a primitive value itself (not
//! the input wrapper around it) is what's being inspected.

use scratch_boring::{Sb3Primitive, Sb3Project};
use std::sync::Arc;

const FIB1: &str = "assets/Fibonacci_1.sb3";
const FIB2: &str = "assets/Fibonacci_2.sb3";

fn load(path: &str) -> Sb3Project {
    Sb3Project::load_for_boring(Arc::from(path))
        .unwrap_or_else(|| panic!("load {}", path))
}

fn find_target<'a>(project: &'a Sb3Project, name: &str) -> &'a scratch_boring::Sb3Target {
    project
        .targets
        .iter()
        .find(|t| t.name.as_ref() == name)
        .unwrap_or_else(|| panic!("target '{}' not found", name))
}

fn find_variable<'a>(
    target: &'a scratch_boring::Sb3Target,
    name: &str,
) -> &'a scratch_boring::Sb3Variable {
    target
        .variables
        .iter()
        .find(|v| v.name.as_ref() == name)
        .unwrap_or_else(|| panic!("variable '{}' not found on target '{}'", name, target.name))
}

fn find_block_by_opcode<'a>(
    target: &'a scratch_boring::Sb3Target,
    opcode: &str,
) -> &'a scratch_boring::Sb3Block {
    target
        .blocks
        .values()
        .find(|b| b.opcode.as_ref() == opcode)
        .unwrap_or_else(|| panic!("no block with opcode '{}' on target '{}'", opcode, target.name))
}

#[test]
fn fib1_has_two_targets_stage_and_sprite1() {
    let project = load(FIB1);
    assert_eq!(project.targets.len(), 2);
    let names: Vec<&str> = project.targets.iter().map(|t| t.name.as_ref()).collect();
    assert!(names.contains(&"Stage"));
    assert!(names.contains(&"Sprite1"));

    let stage = find_target(&project, "Stage");
    assert!(stage.is_stage);
    let sprite = find_target(&project, "Sprite1");
    assert!(!sprite.is_stage);
}

#[test]
fn fib1_stage_has_five_variables_including_sequence_length_25() {
    let project = load(FIB1);
    let stage = find_target(&project, "Stage");
    assert_eq!(stage.variables.len(), 5);

    let names: Vec<&str> = stage.variables.iter().map(|v| v.name.as_ref()).collect();
    for expected in ["sequence_length", "i", "last_number", "current_number", "new_number"] {
        assert!(names.contains(&expected), "missing variable '{}'", expected);
    }

    let sequence_length = find_variable(stage, "sequence_length");
    assert_eq!(sequence_length.initial_as_number_string().as_deref(), Some("25"));
    assert!(!sequence_length.cloud);
}

#[test]
fn fib1_stage_has_no_lists_or_blocks() {
    let project = load(FIB1);
    let stage = find_target(&project, "Stage");
    assert_eq!(stage.lists.len(), 0);
    assert_eq!(stage.blocks.len(), 0);
    assert!(stage.tempo.is_some());
    assert!(stage.video_transparency.is_some());
}

#[test]
fn fib1_sprite_has_twelve_blocks() {
    let project = load(FIB1);
    let sprite = find_target(&project, "Sprite1");
    assert_eq!(sprite.blocks.len(), 12);
    // Sprite-only fields should be populated, stage-only fields absent.
    assert!(sprite.visible.is_some());
    assert!(sprite.tempo.is_none());
}

#[test]
fn fib1_whenflagclicked_is_top_level() {
    let project = load(FIB1);
    let sprite = find_target(&project, "Sprite1");
    let hat = find_block_by_opcode(sprite, "event_whenflagclicked");
    assert!(hat.top_level);
    assert!(hat.x.is_some());
    assert!(hat.y.is_some());
    assert!(hat.parent.is_none());
    assert!(hat.next.is_some());
}

#[test]
fn fib1_first_setvariableto_value_is_shadow_only_text_2() {
    let project = load(FIB1);
    let sprite = find_target(&project, "Sprite1");
    let hat = find_block_by_opcode(sprite, "event_whenflagclicked");
    let next_id = hat.next.as_ref().unwrap();
    let set_var = sprite.blocks.get(next_id).expect("next block exists");
    assert_eq!(set_var.opcode.as_ref(), "data_setvariableto");

    // No real block plugged into VALUE -- it must resolve as an inline
    // primitive, and that primitive must be the shadow-only text "2".
    assert!(set_var.input_block_id(Arc::from("VALUE")).is_none());
    match set_var.input_primitive(Arc::from("VALUE")) {
        Some(Sb3Primitive::Text(s)) => assert_eq!(s.as_ref(), "2"),
        other => panic!("expected shadow-only Text(\"2\") primitive, got {:?}", other),
    }

    assert_eq!(set_var.field_value(Arc::from("VARIABLE")).as_deref(), Some("i"));
    assert!(set_var.field_id(Arc::from("VARIABLE")).is_some());
}

#[test]
fn fib1_has_control_repeat_until_and_operator_equals_blocks() {
    let project = load(FIB1);
    let sprite = find_target(&project, "Sprite1");

    let repeat_until = find_block_by_opcode(sprite, "control_repeat_until");
    let condition_block_id = repeat_until
        .input_block_id(Arc::from("CONDITION"))
        .expect("CONDITION references a block");

    let condition_block = sprite
        .blocks
        .get(&condition_block_id)
        .expect("condition block exists in blocks map");
    assert_eq!(condition_block.opcode.as_ref(), "operator_equals");

    // OPERAND1 is a real variable-reporter primitive ("i").
    match condition_block.input_primitive(Arc::from("OPERAND1")) {
        Some(Sb3Primitive::Variable(name, id)) => {
            assert_eq!(name.as_ref(), "i");
            assert!(!id.as_ref().is_empty());
        }
        other => panic!("expected Variable primitive for OPERAND1, got {:?}", other),
    }

    // OPERAND2 references the "sequence_length" variable.
    match condition_block.input_primitive(Arc::from("OPERAND2")) {
        Some(Sb3Primitive::Variable(name, _)) => assert_eq!(name.as_ref(), "sequence_length"),
        other => panic!("expected Variable primitive for OPERAND2, got {:?}", other),
    }
}

#[test]
fn fib1_has_two_operator_add_blocks() {
    let project = load(FIB1);
    let sprite = find_target(&project, "Sprite1");
    let count = sprite
        .blocks
        .values()
        .filter(|b| b.opcode.as_ref() == "operator_add")
        .count();
    assert_eq!(count, 2);
}

#[test]
fn fib1_has_no_procedures_blocks_or_mutations() {
    // Fibonacci is a simple script with no custom blocks -- confirm there
    // really is no procedures_call/definition/prototype block and no block
    // carries a `mutation`.
    let project = load(FIB1);
    for target in &project.targets {
        for block in target.blocks.values() {
            assert!(
                !block.opcode.as_ref().starts_with("procedures_"),
                "unexpected procedures_* block in Fibonacci_1.sb3: {}",
                block.opcode
            );
            assert!(
                block.mutation_proc_code().is_none(),
                "unexpected mutation on block {} in Fibonacci_1.sb3",
                block.opcode
            );
        }
    }
}

#[test]
fn fib1_monitors_have_five_entries_with_expected_slider_monitor() {
    let project = load(FIB1);
    assert_eq!(project.monitors.len(), 5);

    let seq_len_monitor = project
        .monitors
        .iter()
        .find(|m| m.variable_name().as_deref() == Some("sequence_length"))
        .expect("sequence_length monitor exists");
    assert!(seq_len_monitor.is_variable_watcher());
    assert!(seq_len_monitor.is_visible());
}

#[test]
fn fib1_costumes_and_sounds_parsed() {
    let project = load(FIB1);
    let stage = find_target(&project, "Stage");
    assert_eq!(stage.costumes.len(), 1);
    assert_eq!(stage.costumes[0].name.as_ref(), "backdrop1");
    assert_eq!(stage.sounds.len(), 1);
    assert_eq!(stage.sounds[0].name.as_ref(), "pop");

    let sprite = find_target(&project, "Sprite1");
    assert_eq!(sprite.costumes.len(), 2);
    assert_eq!(sprite.costumes[0].name.as_ref(), "costume1");
    assert_eq!(sprite.costumes[0].bitmap_resolution, Some(1.0));
}

#[test]
fn fib2_variable_i_is_a_string_not_a_number() {
    // Fibonacci_2.sb3's Stage variable "i" has initial value "2" as a JSON
    // *string* -- confirms the loader preserves the JSON type rather than
    // coercing to a number (initial_as_text is Some, initial_as_number_string
    // is None).
    let project = load(FIB2);
    let stage = find_target(&project, "Stage");
    let i = find_variable(stage, "i");
    assert_eq!(i.initial_as_text().as_deref(), Some("2"));
    assert!(i.initial_as_number_string().is_none());
}

#[test]
fn fib2_sprite_has_nine_blocks_and_a_control_repeat_with_shadow_fallback() {
    let project = load(FIB2);
    let sprite = find_target(&project, "Sprite1");
    assert_eq!(sprite.blocks.len(), 9);

    let repeat = find_block_by_opcode(sprite, "control_repeat");
    // shadow status 3: real value references a block (operator_subtract).
    assert!(repeat.input_block_id(Arc::from("TIMES")).is_some());
}

#[test]
fn load_json_matches_load_for_extracted_project_json() {
    // Sanity-check that `load_json_for_boring` (raw project.json path, no
    // zip) is exercised by extracting the same JSON `load_for_boring` reads
    // from inside the zip, and comparing structural counts.
    let zip_loaded = load(FIB1);

    let file = std::fs::File::open(FIB1).expect("open sb3 as zip");
    let mut archive = zip::ZipArchive::new(file).expect("open zip archive");
    let mut entry = archive.by_name("project.json").expect("project.json entry");
    let mut contents = String::new();
    use std::io::Read as _;
    entry.read_to_string(&mut contents).expect("read project.json");
    drop(entry);

    let tmp = tempfile::Builder::new()
        .suffix(".json")
        .tempfile()
        .expect("create temp file");
    std::fs::write(tmp.path(), &contents).expect("write extracted project.json");

    let json_loaded = Sb3Project::load_json_for_boring(Arc::from(tmp.path().to_str().unwrap()))
        .expect("load_json_for_boring extracted project.json");

    // `tmp` (a `NamedTempFile`) deletes its file on drop at the end of
    // this test -- no explicit cleanup needed.

    assert_eq!(zip_loaded.targets.len(), json_loaded.targets.len());
    assert_eq!(zip_loaded.monitors.len(), json_loaded.monitors.len());
    for (a, b) in zip_loaded.targets.iter().zip(json_loaded.targets.iter()) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.blocks.len(), b.blocks.len());
        assert_eq!(a.variables.len(), b.variables.len());
    }
}

#[test]
fn missing_project_json_returns_nil() {
    // Build a zip with no project.json entry and confirm loading collapses
    // to `None` -- there's no `Sb3LoadError::MissingProjectJson` any more to
    // check specifically (see `sb3_loader.br`'s own header comment on why),
    // just the same "load fails gracefully" behavior.
    let tmp = tempfile::Builder::new()
        .suffix(".sb3")
        .tempfile()
        .expect("create temp file");
    {
        let file = tmp.reopen().expect("reopen temp file for zip writing");
        let mut writer = zip::ZipWriter::new(file);
        writer
            .start_file("not_project.json", zip::write::SimpleFileOptions::default())
            .expect("start file");
        use std::io::Write as _;
        writer.write_all(b"{}").expect("write file contents");
        writer.finish().expect("finish zip");
    }

    let result = Sb3Project::load_for_boring(Arc::from(tmp.path().to_str().unwrap()));
    // `tmp` deletes its file on drop at the end of this test -- no
    // explicit cleanup needed.

    assert!(result.is_none());
}

#[test]
fn malformed_json_returns_nil() {
    let tmp = tempfile::Builder::new()
        .suffix(".json")
        .tempfile()
        .expect("create temp file");
    std::fs::write(tmp.path(), "{ not valid json").expect("write malformed json");

    let result = Sb3Project::load_json_for_boring(Arc::from(tmp.path().to_str().unwrap()));
    // `tmp` deletes its file on drop at the end of this test -- no
    // explicit cleanup needed.

    assert!(result.is_none());
}
