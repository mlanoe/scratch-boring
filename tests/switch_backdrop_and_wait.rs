//! End-to-end test for `looks_switchbackdroptoandwait`, ported from real
//! scratch-vm's own `switchBackdropAndWait` (`Scratch3LooksBlocks`) --
//! switches the backdrop, which itself starts every `event_
//! whenbackdropswitchesto` hat matching the new name, then *waits* for
//! those newly-started threads to finish before moving on to its own next
//! statement. Same real "drive to completion" semantics `event_
//! broadcastandwait` already has (`BroadcastAndWait`'s own `exec_simple`
//! case) -- see `interpreter.br`'s own `SwitchBackdropToAndWait` case.
//!
//! `assets/switch_backdrop_and_wait.json`: "Switcher"'s green-flag script
//! switches the backdrop from "a" to "b" *and waits*, then immediately
//! reads "Listener"'s own `hat_ran` variable into `result`. "Listener"
//! has a `when backdrop switches to "b"` hat that sets `hat_ran` to 1.
//! This project's headless scheduler never polls for backdrop changes at
//! all (`check_backdrop_switch_hats` is a Bevy `Update` system, never
//! exercised by `run_greenflag_and_report_json`) -- so `result` can only
//! read back `1` if `SwitchBackdropToAndWait` itself drove Listener's hat
//! to completion synchronously, proving genuine (not merely `SwitchBackdropTo`-
//! identical/no-op) "and wait" semantics.

use scratch_boring::run_greenflag_and_report_json;
use std::sync::Arc;

#[test]
fn switchbackdroptoandwait_drives_the_matching_hat_to_completion_before_continuing() {
    let path: Arc<str> = Arc::from("assets/switch_backdrop_and_wait.json");
    let get = |name: &'static str| run_greenflag_and_report_json(path.clone(), Arc::from(name));

    assert_eq!(get("hat_ran").as_ref(), "1", "Listener's own \"when backdrop switches to b\" hat should have run");
    assert_eq!(get("result").as_ref(), "1", "Switcher's own read of hat_ran, taken immediately after switch-and-wait returns, should already see Listener's hat having finished -- proving the wait is real, not a no-op");
}
