//! End-to-end test for `looks_backdropnumbername` (`"number"` variant --
//! `"name"` stays a documented gap, same as `looks_costumenumbername`'s
//! own "name" branch, confirmed low-priority here specifically: sampled
//! against real projects using this opcode 62 times total, 0 used
//! "name"). `assets/backdrop_number_name.json` (extends the existing
//! `backdrop_switch.json` fixture): Stage has three backdrops (`a`/`b`/
//! `c`); Sprite1's green-flag script switches to `c` (index 2, so
//! `backdrop #` should read `3`, 1-based) then reads `backdrop #` into a
//! Stage variable.

use scratch_boring::run_greenflag_and_report_json;
use std::sync::Arc;

#[test]
fn backdrop_number_reports_the_real_1_based_index_after_switching() {
    let path: Arc<str> = Arc::from("assets/backdrop_number_name.json");
    let get = |name: &'static str| run_greenflag_and_report_json(path.clone(), Arc::from(name));

    assert_eq!(get("backdropNum").as_ref(), "3", "switched to \"c\" (0-based index 2), so backdrop # should read 3");
}
