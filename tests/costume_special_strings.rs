//! End-to-end test for the real Scratch special strings this project's own
//! `looks_switchcostumeto`/`looks_switchbackdropto` used to silently miss
//! (see `costume_switching.rs`'s own header comment for the base feature;
//! this covers exactly the "one genuine gap here, not implemented" this
//! project's own doc comments used to flag), plus the `looks_
//! costumenumbername`/`looks_backdropnumbername` "name" variant and
//! `sensing_of`'s "costume name"/"backdrop name" properties -- all four
//! needed the same fix (`TargetCostumes` threaded through `eval`).
//!
//! `assets/costume_special_strings.json`: Sprite1 has three costumes
//! `"a"`/`"b"`/`"c"`, starting on `"b"` (index 1); the Stage has two
//! backdrops `"x"`/`"y"`, starting on `"x"` (index 0) -- deliberately only
//! two, so "random backdrop" (which real Scratch guarantees never repeats
//! the current backdrop) is fully deterministic to assert on: with only
//! one other backdrop to land on, there's no actual randomness involved.
//!
//! ```text
//! switch costume to "next costume"      "b" -> "c" (index 2)
//! set result1 to (costume number)       -> 3
//! switch costume to "next costume"      "c" -> wraps to "a" (index 0)
//! set result2 to (costume number)       -> 1
//! switch costume to "previous costume"  "a" -> wraps to "c" (index 2)
//! set result3 to (costume number)       -> 3
//! set result_name to (costume name)     -> "c"
//! switch backdrop to "next backdrop"    "x" -> "y" (index 1)
//! set result_bd_next to (backdrop #)    -> 2
//! switch backdrop to "previous backdrop" "y" -> wraps to "x" (index 0)
//! set result_bd_prev to (backdrop #)    -> 1
//! switch backdrop to "random backdrop"  only other option is "y" (index 1)
//! set result_bd_rand to (backdrop #)    -> 2
//! set result_bd_name to (backdrop name) -> "y"
//! set result_sensing_costume_name to (sensing_of Sprite1's "costume name")  -> "c"
//! set result_sensing_backdrop_name to (sensing_of Stage's "backdrop name")  -> "y"
//! ```

use scratch_boring::run_greenflag_and_report_json;
use std::sync::Arc;

#[test]
fn switch_costume_and_backdrop_special_strings_and_name_variants() {
    let path: Arc<str> = Arc::from("assets/costume_special_strings.json");
    let get = |name: &'static str| run_greenflag_and_report_json(path.clone(), Arc::from(name));

    assert_eq!(get("result1").as_ref(), "3", "\"next costume\" from b (index 1) should land on c (index 2)");
    assert_eq!(get("result2").as_ref(), "1", "\"next costume\" from c (the last costume) should wrap back to a (index 0)");
    assert_eq!(get("result3").as_ref(), "3", "\"previous costume\" from a (the first costume) should wrap to c (the last costume)");
    assert_eq!(get("result_name").as_ref(), "c", "looks_costumenumbername's \"name\" variant should report the real costume name, not just its 1-based index");

    assert_eq!(get("result_bd_next").as_ref(), "2", "\"next backdrop\" from x (index 0) should land on y (index 1)");
    assert_eq!(get("result_bd_prev").as_ref(), "1", "\"previous backdrop\" from y (index 1) should wrap back to x (index 0)");
    assert_eq!(get("result_bd_rand").as_ref(), "2", "\"random backdrop\" with only 2 backdrops and the current one excluded is fully deterministic: it must land on the only other backdrop");
    assert_eq!(get("result_bd_name").as_ref(), "y", "looks_backdropnumbername's \"name\" variant should report the real backdrop name");

    assert_eq!(get("result_sensing_costume_name").as_ref(), "c", "sensing_of's \"costume name\" property should report the target sprite's real current costume name");
    assert_eq!(get("result_sensing_backdrop_name").as_ref(), "y", "sensing_of's \"backdrop name\" property should report the Stage's real current backdrop name");
}
