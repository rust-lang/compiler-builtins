//! Approximate implementations.
//!
//! These functions may be smaller or faster than those in the main `math` module, but will
//! not be as accurate.

mod acoshf64;
mod cbrtf64;

pub use acoshf64::acoshf64;
pub use cbrtf64::cbrtf64;
