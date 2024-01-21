// Copyright © 2023 Stephan Kunz
#![crate_type = "lib"]
#![crate_name = "dimas"]
#![warn(missing_docs)]

//!
#[doc = include_str!("../README.md")]
struct _Dummy;    // to get no warnings from clippy

/// Public interface of dimas.
/// Typically it is sufficient to include the prelude with:
/// ```
/// use dimas::prelude::*;
/// ```
pub mod prelude;

// region:    --- modules
mod agent;
mod com;
mod config;
mod context;
mod error;
#[cfg(feature = "timer")]
mod timer;
mod utils;

// endregion: --- modules
