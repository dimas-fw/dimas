// Copyright © 2024 Stephan Kunz

//! Module implements time related things.
//!

// region:    --- modules
mod timer;
mod timer_builder;

// flatten
pub use timer::*;
pub use timer_builder::*;
// endregion: --- modules

#[cfg(test)]
mod tests {}
