// Copyright © 2024 Stephan Kunz
#![no_std]

//! Library implements time related things.
//!

// region:    --- modules
mod timer;

// flatten
pub use timer::*;
// endregion: --- modules

#[cfg(test)]
mod tests {}
