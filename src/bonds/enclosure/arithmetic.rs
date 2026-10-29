//! Private shared arithmetic for the mathematical price producers.
//!
//! Source staging, exact folds, preferred point fitting and directed log/exp
//! borrow the same scalar Frame and Work. No helper returns a large frame.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "Preserved limb arithmetic selects low words explicitly; retained carries and checked small metadata bounds establish every cast"
)]
#![allow(
    missing_debug_implementations,
    reason = "Private numerical storage is intentionally not Debug: diagnostics must not traverse wide working arrays or arena slots"
)]
#![allow(
    dead_code,
    reason = "The shared private engine contains fixed source and kernel routes consumed by successive model adapters; none is public API"
)]

pub(super) mod arena;
pub(super) mod codec;
pub(super) mod fold;
pub(super) mod scalar;
pub(super) mod source;
