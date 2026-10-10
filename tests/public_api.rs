//! Checks of the public API surface, seen from outside the crate.
//!
//! - NFR-05, INV-CORE-3: every public type can be sent and shared between
//!   threads, and is `'static`.
//! - ADR-0007 item 4: the error types implement the standard traits, so any
//!   SabiQuant error converts into a boxed standard error with `?`.
//!
//! The behavior of the error types is tested inside the crate
//! (TS-CORE-01), because only SabiQuant's own operations create errors.

use sabiquant::error::{Error, IdentifierError, Interval, NonFinite, ValueError};
use std::error::Error as StdError;
use std::fmt::Debug;
use std::hash::Hash;

fn assert_value_type<T: Clone + Debug + PartialEq + Send + Sync + 'static>() {}

fn assert_copy_type<T: Copy + Debug + PartialEq + Send + Sync + 'static>() {}

fn assert_error_type<T: StdError + Clone + PartialEq + Send + Sync + 'static>() {}

#[test]
fn public_types_are_thread_safe_values() {
    assert_value_type::<Error>();
    assert_value_type::<ValueError>();
    assert_value_type::<IdentifierError>();
    assert_copy_type::<NonFinite>();
    assert_copy_type::<Interval>();
}

/// No interior mutability is observable (INV-CORE-2): every public type is
/// unwind safe, so a value seen after a caught panic is never half-updated.
#[test]
fn public_types_are_unwind_safe() {
    fn assert_unwind_safe<T: std::panic::UnwindSafe + std::panic::RefUnwindSafe + Unpin>() {}
    assert_unwind_safe::<Error>();
    assert_unwind_safe::<ValueError>();
    assert_unwind_safe::<IdentifierError>();
    assert_unwind_safe::<NonFinite>();
    assert_unwind_safe::<Interval>();
}

#[test]
fn error_types_implement_the_standard_traits() {
    assert_error_type::<Error>();
    assert_error_type::<ValueError>();
    assert_error_type::<IdentifierError>();
}

#[test]
fn non_finite_classes_can_be_map_keys() {
    fn assert_key<T: Eq + Hash>() {}
    assert_key::<NonFinite>();
}

#[test]
fn errors_convert_with_the_question_mark_operator() {
    // Compile-time checks: each conversion that `?` performs exists.
    let _: fn(ValueError) -> Error = Error::from;
    let _: fn(IdentifierError) -> Error = Error::from;
    let _: fn(Error) -> Box<dyn StdError + Send + Sync> = Into::into;
}
