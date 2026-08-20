//! Cross-checks against real fixtures from `scratchfoundation/scratch-vm`'s
//! own official (archived, still public) test suite --
//! `test/fixtures/*.sb3` in that repo. AGPLv3-licensed; this project is
//! GPL-3.0-or-later, so these fixtures are deliberately NOT vendored into
//! this repo (see `.gitignore` and `scripts/fetch_scratch_vm_fixtures.sh`).
//! Every test here is `#[ignore]`d for that reason -- they don't run under
//! a plain `cargo test`. To run them:
//!
//! ```text
//! ./scripts/fetch_scratch_vm_fixtures.sh
//! cargo test -- --ignored
//! ```
//!
//! Expected values are hand-derived from each fixture's own real
//! `project.json` (same methodology `tests/fibonacci.rs` uses for its own
//! real `.sb3` fixtures), not copied from scratch-vm's own test
//! assertions -- those check scratch-vm's own internal data structures
//! (block field-reference counts, monitor thread bookkeeping) that don't
//! map onto this project's architecture; what matters here is "does this
//! real project execute correctly", the same question every other test in
//! this repo asks.

use scratch_boring::{run_greenflag_and_report, run_greenflag_and_report_list};
use std::sync::Arc;

const FIXTURES_DIR: &str = "assets/scratch_vm_fixtures";

fn fixture_path(name: &str) -> Option<Arc<str>> {
    let full = format!("{FIXTURES_DIR}/{name}");
    if std::path::Path::new(&full).exists() {
        Some(Arc::from(full))
    } else {
        eprintln!(
            "skipping: {full} not found -- run ./scripts/fetch_scratch_vm_fixtures.sh first"
        );
        None
    }
}

/// `variable_characters.sb3`: real project with a variable named `"foo`
/// (a literal leading double-quote) and a list named `a&b` -- a real
/// stress test that name-based lookup (`find_stage_var_id`/
/// `find_stage_list_id`) handles special characters in Scratch's own
/// names correctly, not just our own hand-authored ASCII-only fixture
/// names.
///
/// Sprite1's green-flag script (read directly from the fixture's own
/// `project.json`, not scratch-vm's own test assertions):
/// `turn right 15` (unrelated to this test's focus, included as-is),
/// `set "foo to "foo"`, `add "thing" to list "a&b"` (which starts with
/// `['thing', "thing'1"]` already saved in the project).
#[test]
#[ignore]
fn variable_characters_sb3_handles_special_character_names() {
    let Some(path) = fixture_path("variable_characters.sb3") else { return };

    let foo_value = run_greenflag_and_report(path.clone(), Arc::from("\"foo"));
    assert_eq!(foo_value.as_ref(), "foo", "the variable named \\\"foo should be set to \"foo\"");

    let list_contents = run_greenflag_and_report_list(path, Arc::from("a&b"));
    assert_eq!(
        list_contents.as_ref(),
        "thing,thing'1,thing",
        "the list named a&b should have \"thing\" appended to its saved [thing, thing'1]"
    );
}

/// `broadcast_special_chars.sb3`: two top-level `event_broadcast` blocks
/// with special characters in their message names ("< perfect", "a&b"),
/// no receiver scripts at all -- there's nothing observable to assert on
/// (scratch-vm's own test only checks its *importer's* internal
/// bookkeeping, not runtime behavior), so this is a load-and-run-without-
/// panicking robustness check: special characters in a broadcast message
/// name must not crash the loader or the linker.
#[test]
#[ignore]
fn broadcast_special_chars_sb3_loads_and_runs_without_panicking() {
    let Some(path) = fixture_path("broadcast_special_chars.sb3") else { return };
    // Reports an arbitrary, deliberately-nonexistent variable -- the point
    // is only that loading and running the green-flag script(s) doesn't
    // panic; `Vars.get` on an unresolved id already falls back to "0"
    // (see tests/fibonacci.rs's own `unknown_variable_name_reports_zero`).
    let result = run_greenflag_and_report(path, Arc::from("nonexistent"));
    assert_eq!(result.as_ref(), "0");
}

/// `top-level-reporters.sb3`: two orphaned reporter blocks
/// (`motion_xposition`, `looks_size`) sitting on Sprite1's canvas with no
/// hat above them at all -- real Scratch's "click a lone reporter to see
/// its value" editor feature. No `event_whenflagclicked` hat exists
/// anywhere in this project, so `build_greenflag_script` should resolve
/// to an empty script and running it should be a complete no-op, not a
/// crash.
#[test]
#[ignore]
fn top_level_reporters_sb3_with_no_greenflag_hat_is_a_no_op() {
    let Some(path) = fixture_path("top-level-reporters.sb3") else { return };
    let result = run_greenflag_and_report(path, Arc::from("nonexistent"));
    assert_eq!(result.as_ref(), "0");
}

/// `list-monitor-rename.sb3`: two `data_listcontents` monitors (list
/// watchers) with stale `params.LIST` names -- this is the fixture that
/// originally revealed the watcher-label staleness bug (see
/// `tests/watchers.rs`'s own fix, verified there against a hand-authored
/// variable-monitor fixture instead, since list watchers weren't
/// implemented at all yet at the time). `ListWatcherTag`/`build_list_
/// watchers`/`sync_list_watchers` are what makes both survive now: the
/// visible Stage list's monitor caches "old global" in `params.LIST`
/// while the list itself was renamed to "renamed global"; the second
/// visible monitor watches a list that's sprite-local ("renamed local" on
/// "Sprite1", used to be skipped entirely before sprite-local lists were
/// real -- `Lists.list_owner`/`.local_table`, sprite-local variables &
/// lists Phase D). Expect exactly *two* list watcher entities, each
/// labeled with its own list's *current* name, never the monitor's stale
/// one.
#[test]
#[ignore]
fn list_monitor_rename_sb3_loads_and_runs_without_panicking() {
    let Some(path) = fixture_path("list-monitor-rename.sb3") else { return };
    let result = run_greenflag_and_report(path, Arc::from("nonexistent"));
    assert_eq!(result.as_ref(), "0");
}

/// See `list_monitor_rename_sb3_loads_and_runs_without_panicking`'s own
/// doc comment for the fixture's exact shape -- this is the actual
/// watcher-entity check, the list-watcher counterpart of `tests/
/// watchers.rs`'s `watcher_label_uses_the_variables_current_name_not_a_
/// stale_monitor_param`.
#[test]
#[ignore]
fn list_monitor_rename_sb3_watches_only_the_stage_list_with_its_current_name() {
    use bevy::input::ButtonInput;
    use bevy::prelude::*;
    use scratch_boring::{setup_scratch_project, sync_list_watchers, ListWatcherTag, ProjectPath};

    let Some(path) = fixture_path("list-monitor-rename.sb3") else { return };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(Update, sync_list_watchers);
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&ListWatcherTag, &Text2d)>();
    let mut found: Vec<(String, String)> = query
        .iter(world)
        .map(|(tag, text)| (tag.label.to_string(), text.0.clone()))
        .collect();
    found.sort();

    assert_eq!(
        found,
        vec![
            ("renamed global".to_string(), "renamed global\n".to_string()),
            ("renamed local".to_string(), "renamed local\n".to_string()),
        ],
        "both the Stage list and Sprite1's own local list should now have their own watcher entity, each labeled with its own list's current name, never the monitor's stale params.LIST (\"old global\"/\"old local\"); contents are empty (both lists start and stay empty in this fixture), got: {found:?}"
    );
}

/// `missing_png.sb3`: the "Green Guy" target's own costume PNG is
/// referenced in `project.json` but genuinely absent from the zip (only
/// `project.json` and an unrelated `.wav`/`.svg` are present) --
/// `extract_current_costume_to_temp_file`'s zip
/// lookup (`archive.by_name(...)`) must fail gracefully (`None`), not
/// panic, so `setup_scratch_project` falls back to the flat-color
/// placeholder for this target instead of crashing the whole project.
#[test]
#[ignore]
fn missing_png_sb3_extraction_returns_none_rather_than_panicking() {
    use scratch_boring::extract_current_costume_to_temp_file;
    let Some(path) = fixture_path("missing_png.sb3") else { return };
    let result = extract_current_costume_to_temp_file(path, Arc::from("Green Guy"));
    assert!(result.is_none(), "the referenced PNG doesn't exist in the zip at all");
}

/// `corrupt_png.sb3`: the "Green Guy" target's costume PNG *is* present
/// in the zip, but its bytes are deliberately corrupted (a valid PNG
/// magic header followed by garbage text instead of a real image, not a
/// truncated/empty file). This project's extraction step never validates
/// image content -- only zip presence -- so this documents that behavior
/// rather than fixing anything: extraction succeeds (`Some`, the corrupt
/// bytes get written to the cache file as-is); the real decode failure
/// only ever surfaces later, asynchronously, inside Bevy's own
/// `AssetServer` (which logs an error and simply doesn't render that
/// sprite -- already proven non-fatal, same as this project's earlier
/// "unapproved path" `AssetServer` error before that was fixed).
#[test]
#[ignore]
fn corrupt_png_sb3_extraction_passes_through_corrupted_bytes_unvalidated() {
    use scratch_boring::extract_current_costume_to_temp_file;
    let Some(path) = fixture_path("corrupt_png.sb3") else { return };
    let result = extract_current_costume_to_temp_file(path, Arc::from("Green Guy"));
    assert!(result.is_some(), "bytes exist in the zip, so extraction succeeds even though their content is corrupt");
}

/// `missing_svg.sb3`: a *structurally different* zip from every other
/// fixture here -- `project.json` and every asset live one directory
/// level deep (`"Missing Blue Guy (broken)/project.json"`, not
/// `"project.json"` at the archive root; this is what a file manager's
/// plain "Compress" on a folder produces, not what the Scratch editor
/// itself exports, but real Scratch/`scratch-parser` accepts it anyway --
/// see `find_entry_at_root_or_one_level_deep`'s own doc comment). Before
/// that function existed, `Sb3Project::load`'s exact-name `archive.
/// by_name("project.json")` couldn't find it at all, so even `Sb3Project::
/// load` itself failed on this real fixture, not just costume extraction.
/// On top of the nested-zip shape, "Blue Square Guy"'s one costume's
/// asset (`a267f8b97ee9cf8aa9832aa0b4cfd9eb.svg`) is *also* genuinely
/// absent from the zip (confirmed directly against the zip's own file
/// listing) -- same "extraction returns `None`, no panic" contract as
/// `missing_png.sb3`, just with the nested path added on top.
#[test]
#[ignore]
fn missing_svg_sb3_loads_despite_a_nested_zip_and_extraction_returns_none() {
    use scratch_boring::{extract_current_costume_to_temp_file, Sb3Project};
    let Some(path) = fixture_path("missing_svg.sb3") else { return };

    // `load_for_boring` (not the removed `Sb3Project::load`) is now the only
    // loading API -- see `boring/sb3_loader.br`'s own header comment. Cloned
    // since `path` is also moved into `extract_current_costume_to_temp_file`
    // below.
    let project = Sb3Project::load_for_boring(path.clone()).expect(
        "project.json lives one directory level deep in this zip -- load must still succeed",
    );
    assert_eq!(project.targets.len(), 2);
    // `t.name` is `Arc<str>` (Boring's own string representation, now that
    // `sb3_loader.rs` is ported to Boring) -- `.as_ref()` for the `&str`
    // comparison, `Arc<str>` has no direct `PartialEq<&str>` the way the
    // old hand-written `String` field did.
    assert!(project.targets.iter().any(|t| t.name.as_ref() == "Blue Square Guy"));

    let result =
        extract_current_costume_to_temp_file(path, Arc::from("Blue Square Guy"));
    assert!(
        result.is_none(),
        "the costume asset is genuinely absent from the zip, nested path notwithstanding"
    );
}

/// `corrupt_svg.sb3`: same nested-zip shape as `missing_svg.sb3` (see its
/// own doc comment), but "Blue Square Guy"'s costume asset *is* present
/// in the zip -- its SVG content is deliberately invalid XML (a
/// hand-inserted `<here is some nonsense that will make this costume not
/// valid svg>` tag, confirmed directly against the asset's own bytes).
/// Unlike a corrupt PNG/sound (passed through unvalidated, see
/// `corrupt_png_sb3_extraction_passes_through_corrupted_bytes_
/// unvalidated`), an SVG *is* validated at extraction time --
/// rasterization to PNG happens synchronously right here, not lazily
/// inside Bevy's `AssetServer` -- so invalid XML makes `Tree.fromData`
/// (`resvg::usvg::Tree::from_data`) fail and this correctly returns `None`
/// too, for a
/// different reason than `missing_svg.sb3`'s (asset present, but
/// unparseable, vs. genuinely absent).
#[test]
#[ignore]
fn corrupt_svg_sb3_extraction_returns_none_for_unparseable_xml() {
    use scratch_boring::extract_current_costume_to_temp_file;
    let Some(path) = fixture_path("corrupt_svg.sb3") else { return };
    let result =
        extract_current_costume_to_temp_file(path, Arc::from("Blue Square Guy"));
    assert!(
        result.is_none(),
        "the asset exists but its XML is invalid, so SVG parsing fails and extraction returns None"
    );
}

/// `missing_sound.sb3`/`corrupt_sound.sb3`: sound counterparts of
/// `missing_png.sb3`/`corrupt_png.sb3`, and (unlike `missing_svg.sb3`/
/// `corrupt_svg.sb3`) both flat zips -- no nested-directory fix needed
/// here, this only confirms `extract_sound_to_temp_file` has the same
/// "genuinely absent -> None, corrupt-but-present -> Some, unvalidated"
/// contract as the costume/PNG extractors, for the sound path specifically.
#[test]
#[ignore]
fn missing_and_corrupt_sound_sb3_have_the_same_contract_as_png() {
    use scratch_boring::extract_sound_to_temp_file;
    if let Some(path) = fixture_path("missing_sound.sb3") {
        let result = extract_sound_to_temp_file(
            path,
            Arc::from("Sprite1"),
            Arc::from("Boop Sound Recording"),
        );
        assert!(result.is_none(), "the sound asset doesn't exist in the zip at all");
    }
    if let Some(path) = fixture_path("corrupt_sound.sb3") {
        let result = extract_sound_to_temp_file(
            path,
            Arc::from("Stage"),
            Arc::from("pop"),
        );
        assert!(
            result.is_some(),
            "bytes exist in the zip, so extraction succeeds even though the sound is truncated/corrupt"
        );
    }
}

/// `cloud_variables_simple.sb3`: a Stage variable named `"☁ firstCloud"`
/// (the cloud-variable prefix character Scratch's own UI uses) stored in
/// the 3-element `[name, value, true]` JSON form (`Sb3Variable::cloud`,
/// already parsed since this project first handled variables at all --
/// this fixture is the first *real* one confirming that 3-element shape
/// against an actual scratch-vm test asset, not just the hand-derived
/// reasoning in that struct's own doc comment). This project doesn't
/// model cloud variables specially at runtime (no networked persistence,
/// no scratch-vm-style variable-count limit enforcement -- both
/// server/UI-level concerns, not interpreter ones) -- so this is a load-
/// and-report robustness check: the ☁ character in the name must not
/// break anything, and the variable's saved initial value ("100") must
/// come through correctly despite no green-flag script existing in this
/// fixture at all to explicitly set it.
#[test]
#[ignore]
fn cloud_variables_simple_sb3_reports_the_saved_initial_value() {
    let Some(path) = fixture_path("cloud_variables_simple.sb3") else { return };
    let result = run_greenflag_and_report(path, Arc::from("☁ firstCloud"));
    assert_eq!(result.as_ref(), "100");
}

/// `origin.sb3`: a *third*, independently-found real fixture with the
/// same nested-zip shape as `missing_svg.sb3`/`corrupt_svg.sb3` (see
/// `find_entry_at_root_or_one_level_deep`'s own doc comment) --
/// `project.json` and every asset live under a top-level `"origin/"`
/// directory. Its actual point (an optional `meta.origin` string field
/// some Scratch derivatives stamp onto a project) is irrelevant here --
/// `meta` is parsed and discarded entirely (see `Sb3Project`'s own doc
/// comment) -- so this is purely a second, independent confirmation that
/// the nested-zip fix generalizes, not a fixture chosen for its
/// documented purpose.
#[test]
#[ignore]
fn origin_sb3_loads_despite_its_own_independent_nested_zip() {
    use scratch_boring::Sb3Project;
    let Some(path) = fixture_path("origin.sb3") else { return };
    // `load_for_boring` (not the removed `Sb3Project::load`) is now the only
    // loading API -- see `boring/sb3_loader.br`'s own header comment.
    let project =
        Sb3Project::load_for_boring(path).expect("project.json lives under \"origin/\" in this zip too");
    assert_eq!(project.targets.len(), 2);
}

/// `cloud_variables_limit.sb3`: ten simultaneous visible `data_variable`
/// monitors, all Stage-scoped cloud variables ("☁ 1".."☁ 10") -- a real
/// scratch-vm fixture for its own cloud-variable-count-limit test (a
/// server/UI-level concern this project doesn't model, see
/// `cloud_variables_simple_sb3_reports_the_saved_initial_value`'s own
/// comment), repurposed here as a breadth stress test of the variable-
/// watcher pipeline: no green-flag script exists in this fixture, so all
/// ten must survive (every one is Stage-scoped, none sprite-local) and
/// show their saved initial value (`0`) with their ☁-prefixed name
/// intact.
#[test]
#[ignore]
fn cloud_variables_limit_sb3_shows_all_ten_stage_cloud_watchers() {
    use bevy::input::ButtonInput;
    use bevy::prelude::*;
    use scratch_boring::{setup_scratch_project, sync_variable_watchers, ProjectPath, VariableWatcherTag};

    let Some(path) = fixture_path("cloud_variables_limit.sb3") else { return };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(Update, sync_variable_watchers);
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&VariableWatcherTag, &Text2d)>();
    let mut found: Vec<(String, String)> = query
        .iter(world)
        .map(|(tag, text)| (tag.label.to_string(), text.0.clone()))
        .collect();
    found.sort();

    let mut expected: Vec<(String, String)> = (1..=10)
        .map(|n| (format!("☁ {n}"), format!("☁ {n}: 0")))
        .collect();
    // Lexicographic, not numeric, sort -- matches `found`'s own `.sort()`
    // above (a plain string sort puts "☁ 10" before "☁ 2").
    expected.sort();
    assert_eq!(found, expected);
}

/// `monitored_variables.sb3`: both of its visible `data_variable`
/// monitors ("jamalvar" on target "Jamal", "refereevar1" on target
/// "Referee") are sprite-local, with no Stage-scoped variable monitor at
/// all (unlike `monitors.sb3`, which mixes sprite-local and Stage
/// monitors). Used to be the *zero-surviving-watchers* edge case back
/// when sprite-local variable watchers were always skipped -- now that
/// they're real (`Vars.var_owner`/`.local_table`, sprite-local variables
/// & lists Phase C), both should show their own real, saved initial
/// value (confirmed directly against the real fixture's own
/// `target.variables` -- neither target has a green-flag hat, so neither
/// value is ever mutated before this test's own single tick reads it).
#[test]
#[ignore]
fn monitored_variables_sb3_shows_both_sprite_local_watchers_with_their_real_values() {
    use bevy::input::ButtonInput;
    use bevy::prelude::*;
    use scratch_boring::{setup_scratch_project, sync_variable_watchers, ProjectPath, VariableWatcherTag};

    let Some(path) = fixture_path("monitored_variables.sb3") else { return };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(Update, sync_variable_watchers);
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&VariableWatcherTag, &Text2d)>();
    let mut found: Vec<(String, String)> = query
        .iter(world)
        .map(|(tag, text)| (tag.label.to_string(), text.0.clone()))
        .collect();
    found.sort();

    assert_eq!(
        found,
        vec![("jamalvar".to_string(), "jamalvar: 12".to_string()), ("refereevar1".to_string(), "refereevar1: cat".to_string())],
        "both sprite-local variable monitors should now have their own watcher entity, showing their real saved values, got: {found:?}"
    );
}

/// `edge-triggered-hat.sb3`: `when timer > -1: move 10 steps` on Sprite1
/// -- the real official fixture that directly motivated this project's
/// `sensing_timer`/`event_whengreaterthan` feature (see `tests/timer.rs`
/// for the hand-authored fixture that actually exercises the edge-
/// triggering logic in a controlled, deterministic way). `-1` is always
/// less than a timer that starts at `0.0` and only ever increases, so the
/// hat fires exactly once, at the very first tick, moving the sprite from
/// its saved starting position (x=50, y=0, direction=90 -- confirmed
/// directly from the fixture's own `project.json`, not assumed to be the
/// default 0/0/90) to x=60 (direction 90 = facing right, so `move 10
/// steps` is a pure +x move) and never again.
#[test]
#[ignore]
fn edge_triggered_hat_sb3_fires_once_at_the_first_tick() {
    use bevy::input::ButtonInput;
    use bevy::prelude::*;
    use scratch_boring::{
        check_greater_than_hats, play_queued_sounds, run_pending_threads, setup_scratch_project,
        sync_keyboard_state, sync_mouse_state, sync_scratch_timer, Positions, ProjectPath,
    };

    let Some(path) = fixture_path("edge-triggered-hat.sb3") else { return };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(
            Update,
            (
                sync_keyboard_state,
                sync_mouse_state,
                sync_scratch_timer,
                check_greater_than_hats,
                run_pending_threads,
                play_queued_sounds,
            )
                .chain(),
        );

    // First tick: timer starts at 0.0, already > -1, so the hat fires
    // immediately (an edge from "before the project ever ran" into
    // "true").
    app.update();
    let positions = app.world().resource::<Positions>().clone();
    assert_eq!(
        format!("{}", scratch_boring::position_value(positions.clone(), Arc::from("Sprite1"), Arc::from("x"))),
        "60",
        "the hat should have fired once at tick 1 and moved the sprite from x=50 to x=60"
    );

    // Second tick: timer is now > 0, still > -1 -- must not re-fire.
    app.update();
    let positions = app.world().resource::<Positions>().clone();
    assert_eq!(
        format!("{}", scratch_boring::position_value(positions, Arc::from("Sprite1"), Arc::from("x"))),
        "60",
        "must not re-fire on a later tick (edge-triggered, not level-triggered)"
    );
}

/// `monitors.sb3`: every monitor opcode scratch-vm itself tests in one
/// project -- extension monitors (which this project doesn't implement
/// watchers for, a real documented gap) alongside four `data_variable`
/// monitors -- one visible Stage variable ("my variable" = "my"), one
/// *invisible* Stage variable ("secret_slide" = 81), and two visible
/// *sprite-local* variables ("tee" = 60 on target "Shirt-T", "hearty" =
/// "happy" on target "Heart") -- and eleven visible reporter monitors:
/// three Motion (`motion_xposition`/`motion_yposition`/`motion_direction`),
/// one `looks_size`, two `looks_costumenumbername` (NUMBER_NAME="number"
/// and "name") on "Shirt-T", two `looks_backdropnumbername` (same two
/// NUMBER_NAME variants, target-agnostic -- the Stage's own backdrop, not
/// scoped to any sprite), plus three target-agnostic `sensing_current`
/// (year/month/date) -- all eleven now supported (`ReporterWatcherTag`/
/// `sync_reporter_watchers`; `sensing_current` and `looks_
/// backdropnumbername` were the last two reporter opcodes this fixture
/// exercises that didn't have a matching watcher yet). This fixture
/// originally drove `build_variable_watchers`' sprite-local-skip
/// *workaround*, back when `Vars` only ever held Stage-scoped values; now
/// that sprite-local variables are real (`Vars.var_owner`/`.local_table`,
/// sprite-local variables & lists Phase C), all four survive. Expect
/// exactly *four* variable watcher entities: "my variable" (visible),
/// "secret_slide" (hidden -- Shirt-T's own green-flag script explicitly
/// hides it, same value it already started with), "tee" (visible,
/// untouched by any script), and "hearty" (visible, untouched). Also
/// exactly eleven reporter watcher entities: eight deterministic ones
/// (Shirt-T's saved x/y/direction/size/costume, plus the Stage's own
/// saved backdrop # and name -- no script in this fixture ever moves
/// Shirt-T, switches its costume, or switches the Stage's backdrop)
/// checked against fixed expected strings, plus three genuinely
/// non-deterministic `sensing_current` ones (real local wall-clock date)
/// checked against this test's own independent `chrono::Local::now()`
/// reading instead.
#[test]
#[ignore]
fn monitors_sb3_watches_all_four_variables_stage_and_sprite_local_alike() {
    use bevy::input::ButtonInput;
    use bevy::prelude::*;
    use scratch_boring::{
        play_queued_sounds, run_pending_threads, setup_scratch_project, sync_keyboard_state,
        sync_mouse_state, sync_reporter_watchers, sync_speech_bubbles, sync_variable_watchers,
        ProjectPath, ReporterWatcherTag, VariableWatcherTag,
    };

    let Some(path) = fixture_path("monitors.sb3") else { return };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(
            Update,
            (
                sync_keyboard_state,
                sync_mouse_state,
                run_pending_threads,
                play_queued_sounds,
                sync_speech_bubbles,
                sync_variable_watchers,
                sync_reporter_watchers,
            )
                .chain(),
        );
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&VariableWatcherTag, &Text2d, &Visibility)>();
    let mut found: Vec<(String, String, bool)> = query
        .iter(world)
        .map(|(tag, text, vis)| (tag.label.to_string(), text.0.clone(), *vis == Visibility::Visible))
        .collect();
    found.sort();

    assert_eq!(
        found.len(),
        4,
        "all four variable monitors should now have their own watcher entity, Stage and sprite-local alike, got: {found:?}"
    );
    assert_eq!(found[0], ("hearty".to_string(), "hearty: happy".to_string(), true), "Heart's own local \"hearty\", untouched by any script");
    assert_eq!(found[1], ("my variable".to_string(), "my variable: my".to_string(), true));
    assert_eq!(found[2].0, "secret_slide", "the invisible Stage variable should still get a real watcher entity");
    assert!(!found[2].2, "secret_slide's monitor is invisible -- its entity should start Visibility::Hidden");
    assert_eq!(found[3], ("tee".to_string(), "tee: 60".to_string(), true), "Shirt-T's own local \"tee\", untouched by its green-flag script (which only hides secret_slide, never touches tee)");

    let now = chrono::Local::now();
    let world = app.world_mut();
    let mut reporter_query = world.query::<(&ReporterWatcherTag, &Text2d)>();
    let mut reporters: Vec<(String, String)> = reporter_query
        .iter(world)
        .map(|(tag, text)| (tag.label.to_string(), text.0.clone()))
        .collect();
    reporters.sort();

    // `monitors.sb3` also has three `sensing_current` monitors (year/
    // month/date, all target-agnostic -- `sensing_current` reads real
    // wall-clock time, so unlike everything else this fixture asserts,
    // these three can't be fixed expected strings. Pulled out and
    // checked loosely (label + plausible current value) before the
    // exact-equality check below covers the remaining six, deterministic
    // ones.
    use chrono::Datelike;
    let mut current_watchers: Vec<(String, String)> = vec![];
    reporters.retain(|(label, text)| {
        if label == "year" || label == "month" || label == "date" {
            current_watchers.push((label.clone(), text.clone()));
            false
        } else {
            true
        }
    });
    current_watchers.sort();
    assert_eq!(current_watchers.len(), 3, "expected exactly the year/month/date sensing_current monitors, got: {current_watchers:?}");
    assert_eq!(current_watchers[0], ("date".to_string(), format!("date: {}", now.day())));
    assert_eq!(current_watchers[1], ("month".to_string(), format!("month: {}", now.month())));
    assert_eq!(current_watchers[2], ("year".to_string(), format!("year: {}", now.year())));

    assert_eq!(
        reporters,
        vec![
            (
                "Shirt-T costume #".to_string(),
                "Shirt-T costume #: 1".to_string()
            ),
            (
                "Shirt-T costume name".to_string(),
                "Shirt-T costume name: shirt-t".to_string()
            ),
            (
                "Shirt-T direction".to_string(),
                "Shirt-T direction: 40".to_string()
            ),
            (
                "Shirt-T size".to_string(),
                "Shirt-T size: 30".to_string()
            ),
            (
                "Shirt-T x position".to_string(),
                "Shirt-T x position: -220.2282257080078".to_string()
            ),
            (
                "Shirt-T y position".to_string(),
                "Shirt-T y position: -161.13259887695313".to_string()
            ),
            (
                "backdrop #".to_string(),
                "backdrop #: 2".to_string()
            ),
            (
                "backdrop name".to_string(),
                "backdrop name: School".to_string()
            ),
        ],
        // x/y are Shirt-T's saved position round-tripped through f32 (its
        // one green-flag script only hides a variable, never moves it) --
        // the extra trailing digits are `ScratchNumber::from_f32`'s own
        // f32->exact-fraction promotion, not a bug introduced here. Size
        // (30) and costume (index 0, "shirt-t", its own only costume) are
        // likewise Shirt-T's untouched saved values. The Stage's own
        // backdrop (index 1, "School") is likewise untouched -- no script
        // in this fixture ever switches it.
    );
}

/// `timer-monitor.sb3`: a lone visible `sensing_timer` monitor
/// (`spriteName: None` -- confirmed target-agnostic for real, not just an
/// assumption) and one orphaned top-level `sensing_timer` reporter block
/// with no green-flag hat above it (so `PendingThreads` stays empty,
/// nothing ever runs). Drove adding a fourth opcode to
/// `build_reporter_watchers`'s Motion-only trio: `Sb3Monitor::is_sensing_
/// timer_watcher` + `BlockKind::Timer` (already implemented, unmodified)
/// via a `target_id = ""` `ReporterWatcher`. Expect exactly one reporter
/// watcher entity, labeled "timer", reading the headless `ScratchTimer`'s
/// own starting value -- `0` (same reasoning as `tests/timer.rs`'s
/// `sensing_timer_headless_always_reports_the_starting_zero`: a single
/// `app.update()` with no `Time::advance_by` never advances it), not
/// scratch-vm's own saved snapshot value (`235.639`, a number this
/// headless run has no way to reproduce and isn't trying to).
#[test]
#[ignore]
fn timer_monitor_sb3_watches_the_target_agnostic_timer_reporter() {
    use bevy::input::ButtonInput;
    use bevy::prelude::*;
    use scratch_boring::{
        setup_scratch_project, sync_reporter_watchers, ProjectPath, ReporterWatcherTag,
    };

    let Some(path) = fixture_path("timer-monitor.sb3") else { return };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(Update, sync_reporter_watchers);
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&ReporterWatcherTag, &Text2d)>();
    let found: Vec<(String, String)> = query
        .iter(world)
        .map(|(tag, text)| (tag.label.to_string(), text.0.clone()))
        .collect();

    assert_eq!(found, vec![("timer".to_string(), "timer: 0".to_string())]);
}

/// `monitors.sb3`'s own Stage has a real bitmap backdrop, "School"
/// (index 1), with a saved `bitmapResolution: 2` -- a real, previously
/// untested case (every hand-authored fixture and `Fibonacci_1.sb3`'s own
/// costumes are SVG, whose `bitmapResolution` is always absent/1). The
/// real PNG asset itself is 960x720 pixels; real Scratch's own semantics
/// (a retina-authored bitmap costume renders/collides at HALF its raw
/// pixel size) means `TargetCostumeDims` should report the *logical*
/// 480x360 -- not 960x720 -- confirming `build_target_costume_dims`'s own
/// `bitmap_resolution` divisor (`assets.br`) against a real asset, not
/// just a hand-verified formula.
#[test]
#[ignore]
fn monitors_sb3s_retina_backdrop_reports_its_bitmap_resolution_halved_logical_size() {
    use scratch_boring::run_report_costume_dims;

    let Some(path) = fixture_path("monitors.sb3") else { return };

    let w = run_report_costume_dims(path.clone(), Arc::from("Stage"), 1, Arc::from("width"));
    let h = run_report_costume_dims(path, Arc::from("Stage"), 1, Arc::from("height"));

    assert_eq!(w.as_ref(), "480", "a bitmapResolution:2 costume's real 960px-wide PNG should report a logical width of 480 (960 / 2), not its raw pixel width");
    assert_eq!(h.as_ref(), "360", "same for height: 720 / 2 = 360");
}
