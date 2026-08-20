//! End-to-end test for sprite-local list read/write (Phase D of the
//! sprite-local variables & lists plan) -- the list mirror of Phase A's
//! `tests/sprite_local_variable.rs`.
//!
//! `assets/sprite_local_list.json`: sprite A has its own *local* list
//! "items" (initial `["L1"]`); the Stage separately has a *global* list
//! also named "items" (initial `["Z1", "Z2", "Z3"]`) -- deliberately the
//! same name, to prove local/global ids never collide. A's own green-flag
//! script:
//!
//! ```text
//! add "L2" to [my own local] items      -- ["L1", "L2"], length 2
//! result_len_local   = length of [my own local] items    -- 2
//! result_first_local = item 1 of [my own local] items     -- "L1"
//! result_len_global   = length of [the Stage's global] items   -- untouched, 3
//! result_first_global = item 1 of [the Stage's global] items    -- untouched, "Z1"
//! ```

use scratch_boring::run_greenflag_and_report_json;
use std::sync::Arc;

#[test]
fn sprite_local_list_reads_and_writes_independently_of_a_same_named_global() {
    let path: Arc<str> = Arc::from("assets/sprite_local_list.json");
    let get = |name: &'static str| run_greenflag_and_report_json(path.clone(), Arc::from(name));

    assert_eq!(get("result_len_local").as_ref(), "2", "A's own local \"items\" should have grown to length 2 after adding \"L2\"");
    assert_eq!(get("result_first_local").as_ref(), "L1", "A's own local \"items\" should still start with its own original first item");
    assert_eq!(get("result_len_global").as_ref(), "3", "the Stage's global \"items\" should be completely untouched by A's own local list of the same name");
    assert_eq!(get("result_first_global").as_ref(), "Z1", "the Stage's global \"items\" should still start with its own original first item");
}
