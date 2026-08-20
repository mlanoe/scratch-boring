//! End-to-end test for `sensing_loudness`/`sensing_loud` -- this project
//! has no real microphone-capture subsystem (a genuinely new OS-level
//! dependency, unlike `sensing_setdragmode`'s own reclassification from
//! "blocked" to "just needed an interaction system" -- see `runtime.br`'s
//! own `current_loudness` doc comment). Rather than fake a plausible
//! number, both reporters return exactly what real scratch-vm's own
//! `getLoudness()`/`isLoud()` return under the identical condition this
//! project is *always* in (no audio engine available, confirmed against
//! real source): `-1` and `false`, real defined behavior, not a guess.
//!
//! `assets/loudness.json`: one green-flag script reading both reporters
//! into their own Stage variables.

use scratch_boring::run_greenflag_and_report_json;
use std::sync::Arc;

#[test]
fn loudness_and_loud_report_real_scratchs_own_no_microphone_defaults() {
    let path: Arc<str> = Arc::from("assets/loudness.json");
    let get = |name: &'static str| run_greenflag_and_report_json(path.clone(), Arc::from(name));

    assert_eq!(get("loudness_result").as_ref(), "-1", "sensing_loudness should report real Scratch's own -1 sentinel for \"no audio engine available\"");
    assert_eq!(get("loud_result").as_ref(), "false", "sensing_loud (loudness > 10) should be false, since -1 is never greater than 10");
}
