//! End-to-end Phase 1 milestone: load a real `.sb3`, link its green-flag
//! script, run it to completion (headless, no Bevy), and check the final
//! variable values -- exercising the whole pipeline built this phase
//! (`sb3_loader` -> linker -> `Block`/`BlockKind` AST -> `eval`/`exec` over
//! `Vars`) against jscratch's own Fibonacci fixtures
//! (`../jscratch/example/*.sb3`, copied into `assets/`).
//!
//! Expected values below aren't guessed -- derived by hand from each
//! fixture's actual `project.json` (unzip -p assets/Fibonacci_N.sb3
//! project.json | python3 -m json.tool) and cross-checked against the
//! literal values Scratch itself saved in each project's `variables` dict
//! (a project's saved variable values are whatever they were at last save,
//! which for both these fixtures happens to be a fresh post-run state --
//! see the inline derivation in each test below).
//!
//! Uses `run_greenflag_and_report`, `boring/scratch.br`'s one `pub def`
//! entry point for this phase -- see that file's own doc comments for the
//! full load -> link -> run pipeline.

use scratch_boring::run_greenflag_and_report;
use std::sync::Arc;

/// Fibonacci_1.sb3's script (see project.json): `set i to 2`, `set
/// last_number to 1`, `set current_number to 1`, then `repeat until (i =
/// sequence_length): set new_number to (last_number + current_number); set
/// last_number to current_number; set current_number to new_number; set i
/// to (i + 1)`. `sequence_length` is a Stage variable never written by any
/// block -- its saved initial value (25) is the loop bound as-is.
///
/// Recurrence: last=1, current=1 before the loop (i.e. fib(1)=fib(2)=1 in
/// 1-indexed terms); each iteration computes new=last+current, shifts
/// last<-current, current<-new. The loop runs while i != sequence_length,
/// starting at i=2 and incrementing by 1 each iteration -- 23 iterations
/// (i = 2..24) before i reaches 25. After k iterations, current = fib(k+2)
/// (1-indexed) -- k=23 -> current = fib(25) = 75025, last = fib(24) =
/// 46368. This matches exactly what Scratch itself had saved in the
/// project's `variables` dict (i=25, last_number=46368, current_number=
/// 75025, new_number=75025) -- i.e. this fixture was saved right after a
/// full run, so a correct interpreter reproduces the saved state exactly.
#[test]
fn fibonacci_1_repeat_until() {
    let path: Arc<str> = Arc::from("assets/Fibonacci_1.sb3");

    let i = run_greenflag_and_report(path.clone(), Arc::from("i"));
    assert_eq!(i.as_ref(), "25", "i should reach sequence_length and stop");

    let last_number = run_greenflag_and_report(path.clone(), Arc::from("last_number"));
    assert_eq!(last_number.as_ref(), "46368", "last_number should be fib(24)");

    let current_number = run_greenflag_and_report(path.clone(), Arc::from("current_number"));
    assert_eq!(current_number.as_ref(), "75025", "current_number should be fib(25)");

    let new_number = run_greenflag_and_report(path.clone(), Arc::from("new_number"));
    assert_eq!(new_number.as_ref(), "75025", "new_number mirrors current_number after the loop");

    let sequence_length = run_greenflag_and_report(path, Arc::from("sequence_length"));
    assert_eq!(sequence_length.as_ref(), "25", "sequence_length is never written by any block");
}

/// Fibonacci_2.sb3's script (see project.json): `set last_number to 1`,
/// `set current_number to 1`, then `repeat (sequence_length - 2): set
/// new_number to (last_number + current_number); set last_number to
/// current_number; set current_number to new_number`. Note this fixture
/// uses `control_repeat` (a fixed count) and `operator_subtract` (for the
/// count expression) rather than Fibonacci_1's `control_repeat_until` +
/// `i` counter -- deliberately exercises a different combination of the
/// same block categories. `i` exists as a Stage variable but is never
/// referenced by any block in this script; it should come through
/// unchanged from its saved initial value (the JSON string `"2"`, not the
/// number 2 -- confirmed directly in the fixture's `variables` dict).
///
/// `sequence_length` = 20 (saved, never written) -> TIMES = 20 - 2 = 18
/// iterations. Same recurrence as Fibonacci_1: after k=18 iterations,
/// current = fib(20) = 6765, last = fib(19) = 4181 -- matching this
/// fixture's own saved state exactly, the same "saved right after a full
/// run" property as Fibonacci_1.
#[test]
fn fibonacci_2_repeat_fixed_count() {
    let path: Arc<str> = Arc::from("assets/Fibonacci_2.sb3");

    let last_number = run_greenflag_and_report(path.clone(), Arc::from("last_number"));
    assert_eq!(last_number.as_ref(), "4181", "last_number should be fib(19)");

    let current_number = run_greenflag_and_report(path.clone(), Arc::from("current_number"));
    assert_eq!(current_number.as_ref(), "6765", "current_number should be fib(20)");

    let new_number = run_greenflag_and_report(path.clone(), Arc::from("new_number"));
    assert_eq!(new_number.as_ref(), "6765", "new_number mirrors current_number after the loop");

    let i = run_greenflag_and_report(path.clone(), Arc::from("i"));
    assert_eq!(i.as_ref(), "2", "i is never referenced by any block in this script");

    let sequence_length = run_greenflag_and_report(path, Arc::from("sequence_length"));
    assert_eq!(sequence_length.as_ref(), "20", "sequence_length is never written by any block");
}

/// A variable name that doesn't exist should report the Phase 1 default
/// (`Vars.get` on an unresolved/empty id falls back to `Num(0)`) rather
/// than panicking -- `find_stage_var_id` returns `""` for a name it can't
/// find, and `Vars.get("")` (no such key in the table) hits the same
/// `else` fallback as every other missing lookup.
#[test]
fn unknown_variable_name_reports_zero_rather_than_panicking() {
    let path: Arc<str> = Arc::from("assets/Fibonacci_1.sb3");
    let result = run_greenflag_and_report(path, Arc::from("this_variable_does_not_exist"));
    assert_eq!(result.as_ref(), "0");
}
