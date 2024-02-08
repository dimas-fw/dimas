// Copyright © 2023 Stephan Kunz
#![crate_type = "lib"]
#![crate_name = "dimas"]
//#![no_panic]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "nightly")]
#[cfg(doctest)]
doc_comment::doctest!("../README.md");

// region:    --- modules
/// Primary module of `DiMAS` containing the Agent
mod agent;
/// Communication
mod com;
/// Configuration
mod config;
/// Context
mod context;
/// Error handling
mod error;
/// Timer
mod timer;
// Helper functions
//mod utils;

/// Public interface of dimas.
/// Typically it is sufficient to include the prelude with
/// `use dimas::prelude::*;`
pub mod prelude;

// endregion: --- modules
