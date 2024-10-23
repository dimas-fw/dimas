// Copyright © 2024 Stephan Kunz
#![no_std]

//! Library implements time related things.
//!

// region:    --- modules
mod error;
mod timer;
#[cfg(feature = "std")]
mod timer_builder;

// flatten
pub use timer::*;
#[cfg(feature = "std")]
pub use timer_builder::*;
// endregion: --- modules

#[cfg(test)]
mod tests {}
