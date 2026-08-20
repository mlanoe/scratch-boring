//! End-to-end test for `sensing_of`'s "volume" property -- a real,
//! previously-documented gap (`SensingOf`'s own doc comment used to say
//! it'd need `SoundQueue` threaded through `eval`) closed in the same
//! round `Touching`'s own clone-costume-size gap and `CostumeNumberName`'s
//! own clone-"name" gap were (`SoundQueue` now threaded through `eval`
//! entirely, not just `exec_simple`).
//!
//! `assets/sensing_of_volume.json`: sprite "A" sets its own volume to 42,
//! then reads back both its own volume (via `sensing_of`, not directly)
//! and the Stage's own volume (untouched, real Scratch's own default 100).
//!
//! ```text
//! set volume to 42
//! set result_a_volume to (A's volume)          -- via sensing_of, not SetVolumeTo's own state
//! set result_stage_volume to (Stage's volume)  -- untouched: 100
//! ```

use scratch_boring::run_greenflag_and_report_json;
use std::sync::Arc;

#[test]
fn sensing_of_volume_reads_the_real_per_instance_volume() {
    let path: Arc<str> = Arc::from("assets/sensing_of_volume.json");
    let get = |name: &'static str| run_greenflag_and_report_json(path.clone(), Arc::from(name));

    assert_eq!(get("result_a_volume").as_ref(), "42", "sensing_of's \"volume\" property should read the real per-instance SoundQueue.volumes value, not fall through to the \"no matching variable\" default");
    assert_eq!(get("result_stage_volume").as_ref(), "100", "the Stage's own volume should still be real Scratch's own default (100), untouched by A's own sound_setvolumeto");
}
