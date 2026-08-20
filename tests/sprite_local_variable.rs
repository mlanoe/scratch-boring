//! End-to-end test for sprite-local variable read/write (Phase A of the
//! sprite-local variables & lists plan) -- the first real fix for a long-
//! documented structural gap: `Vars` used to only ever hold Stage/global
//! values, so any sprite-local variable reference silently resolved
//! against `Vars.get`'s own `0` default instead of erroring, a real,
//! silently-wrong-behavior gap (not a missing-opcode crash) affecting
//! even simple multi-sprite projects (each sprite tracking its own
//! score/health being a very common pattern).
//!
//! `assets/sprite_local_variable.json`: sprite A has its own *local*
//! variable "score" (initial `1`); the Stage separately has a *global*
//! variable also named "score" (initial `100`) -- deliberately the same
//! name, to prove local/global ids never collide (they're distinct real
//! sb3 hash ids under the hood, `var_owner` is the only thing that
//! decides which storage a given id resolves through). A's own green-
//! flag script:
//!
//! ```text
//! change [my own local] score by 10   -- 1 + 10 = 11
//! result_a_local = [my own local] score
//! result_global = [the Stage's global] score  -- untouched, still 100
//! ```

use scratch_boring::run_greenflag_and_report_json;
use std::sync::Arc;

#[test]
fn sprite_local_variable_reads_and_writes_independently_of_a_same_named_global() {
    let path: Arc<str> = Arc::from("assets/sprite_local_variable.json");
    let get = |name: &'static str| run_greenflag_and_report_json(path.clone(), Arc::from(name));

    assert_eq!(get("result_a_local").as_ref(), "11", "A's own local \"score\" should have been changed by 10 (1 -> 11), independent of the Stage's own global \"score\"");
    assert_eq!(get("result_global").as_ref(), "100", "the Stage's global \"score\" should be completely untouched by A's own local variable of the same name");
}
