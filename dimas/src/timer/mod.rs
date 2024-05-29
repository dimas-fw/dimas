// Copyright © 2024 Stephan Kunz

//! Module implements timer.
//!

/// `Timer`
#[allow(clippy::module_inception)]
pub mod timer;
/// `TimerBulider`
#[allow(clippy::module_name_repetitions)]
pub mod timer_builder;

pub use timer::Timer;
pub use timer_builder::*;

#[cfg(test)]
mod tests {}
