//! End-to-end coverage for `ScratchNumber` (BigUint/BigInt/BigFraction-backed
//! exact arithmetic), now 100% Boring-authored in `boring/scratch.br` --
//! replacing the old hand-written `src/value_rt.rs` (and its ~15 internal
//! `#[cfg(test)]` cases) the same way `tests/fibonacci.rs` covers the block
//! interpreter end to end.
//!
//! `ScratchNumber`'s internals are no longer directly reachable from Rust
//! (they're generated from Boring source into `src/boring_gen.rs`, not
//! hand-written), so these tests go through a small set of `pub def
//! scratch_number_test_*` functions in `boring/scratch.br` -- each exercises
//! one scenario and returns `ScratchNumber.debug_kind()`'s tagged-string
//! representation (`"Int:5"`, `"BigIntV:..."`, `"Frac:1/3"`, `"Float:2.5"`,
//! ...), which plain `as string`/`Display` can't distinguish (Frac/BigFracV
//! deliberately display as their float approximation, not their tier).
//!
//! Covers the most significant cases from `value_rt.rs`'s old test module:
//! overflow promotion to `BigInt`, demotion back to `Int`, exact vs. inexact
//! division (collapsing to `Int` vs. producing a `Frac`), mixing in a
//! `Float`, floored (Python-style) remainder, division by zero, and
//! cross-tier equality (a value that detours through `BigInt` and back must
//! still equal the plain `Int` it started as).

use scratch_boring::{
    scratch_number_test_cross_tier_equality, scratch_number_test_demotes_back_to_int,
    scratch_number_test_division_by_zero, scratch_number_test_exact_division,
    scratch_number_test_float_mix, scratch_number_test_floored_remainder,
    scratch_number_test_inexact_division, scratch_number_test_overflow_add,
};

/// i64::MAX + 1 overflows the plain `Int(i64)` tier and must promote to
/// `BigIntV` -- mirrors `value_rt.rs`'s old `overflow_promotes_to_big_int`.
#[test]
fn overflow_add_promotes_to_big_int() {
    assert_eq!(
        scratch_number_test_overflow_add().as_ref(),
        "BigIntV:9223372036854775808"
    );
}

/// (i64::MAX + 1) - 1 must demote back down to a plain `Int(i64::MAX)`
/// rather than staying in the `BigIntV` tier -- mirrors the second half of
/// `value_rt.rs`'s `overflow_promotes_to_big_int`.
#[test]
fn demotes_back_to_int_after_overflow_round_trip() {
    assert_eq!(
        scratch_number_test_demotes_back_to_int().as_ref(),
        "Int:9223372036854775807"
    );
}

/// 6 / 2 is an exact division and must collapse to `Int(3)`, not a `Frac` --
/// mirrors `value_rt.rs`'s `exact_division_collapses_to_int`.
#[test]
fn exact_division_collapses_to_int() {
    assert_eq!(scratch_number_test_exact_division().as_ref(), "Int:3");
}

/// 1 / 3 is not exact and must produce a reduced `Frac(1, 3)` -- mirrors
/// `value_rt.rs`'s `inexact_division_produces_frac`.
#[test]
fn inexact_division_produces_frac() {
    assert_eq!(scratch_number_test_inexact_division().as_ref(), "Frac:1/3");
}

/// 2 + 0.5 mixes in a `Float` and must produce `Float(2.5)`, never staying
/// in an exact tier -- mirrors `value_rt.rs`'s
/// `mixing_float_with_int_produces_float`.
#[test]
fn mixing_float_with_int_produces_float() {
    assert_eq!(scratch_number_test_float_mix().as_ref(), "Float:2.5");
}

/// -7 rem 3 == 2 under Scratch/Python-style floored remainder (not the -1
/// a truncating remainder would give) -- mirrors `value_rt.rs`'s
/// `remainder_is_floored_like_python`.
#[test]
fn remainder_is_floored_like_python() {
    assert_eq!(scratch_number_test_floored_remainder().as_ref(), "Int:2");
}

/// 1 / 0 matches Scratch's own division-by-zero semantics: falls back to
/// plain `f64` division, giving `Float(+Infinity)` -- mirrors
/// `value_rt.rs`'s `division_by_zero_matches_scratch_semantics`.
#[test]
fn division_by_zero_produces_float_infinity() {
    assert_eq!(
        scratch_number_test_division_by_zero().as_ref(),
        "Float:inf"
    );
}

/// (i64::MAX + 1) - 1 must compare *equal* to the plain `Int(i64::MAX)` it
/// started as, even though it detoured through the `BigInt` tier -- mirrors
/// `value_rt.rs`'s `partial_eq_treats_equal_values_across_tiers_as_equal`.
#[test]
fn cross_tier_equality_holds_after_round_trip() {
    assert_eq!(scratch_number_test_cross_tier_equality().as_ref(), "true");
}
