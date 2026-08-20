//! End-to-end test for `looks_switchbackdropto`/`looks_backdrops` --
//! identical machinery to `looks_switchcostumeto` (`tests/costume_
//! switching.rs`), hardcoded to the Stage's own target id throughout, so
//! *any* sprite's script can switch it (a project-wide effect, unlike
//! every other Looks block, which always acts on the calling sprite).
//!
//! No `looks_backdropnumbername` reporter is implemented yet (a real,
//! documented gap), so this checks the result via a dedicated headless
//! test-support entry point, `run_greenflag_and_report_backdrop_index_json`,
//! reading `LooksStates["Stage"].costume_index` directly rather than
//! through an ordinary script-authored value.
//!
//! ```text
//! assets/backdrop_switch.json:
//!   Sprite1, when green flag clicked: switch backdrop to "c"   -> index 3 (1-based)
//!
//! assets/backdrop_switch_nonexistent.json:
//!   Sprite1, when green flag clicked:
//!     switch backdrop to "b"             -> index 2
//!     switch backdrop to "nonexistent"   -> coerces to 0 (real Cast.toNumber
//!                                            semantics, see costume_switching.rs's
//!                                            own doc comment), 0-1=-1, wrapped to
//!                                            the *last* backdrop -> index 3
//! ```

use scratch_boring::run_greenflag_and_report_backdrop_index_json;
use std::sync::Arc;

#[test]
fn switching_the_backdrop_from_a_sprites_own_script_affects_the_stage() {
    let path: Arc<str> = Arc::from("assets/backdrop_switch.json");
    let result = run_greenflag_and_report_backdrop_index_json(path);
    assert_eq!(result.as_ref(), "3", "switch backdrop to \"c\" (index 2, 1-based 3) from Sprite1's own script should affect the Stage");
}

#[test]
fn switching_to_a_nonexistent_backdrop_name_wraps_the_same_way_costumes_do() {
    let path: Arc<str> = Arc::from("assets/backdrop_switch_nonexistent.json");
    let result = run_greenflag_and_report_backdrop_index_json(path);
    assert_eq!(result.as_ref(), "3", "\"nonexistent\" coerces to 0, 0-1=-1, wrapped to the last backdrop (index 2, 1-based 3) -- not a no-op, same real Cast.toNumber semantics costume switching already established");
}
