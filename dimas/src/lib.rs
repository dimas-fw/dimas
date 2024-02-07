// Copyright © 2023 Stephan Kunz
#![crate_type = "lib"]
#![crate_name = "dimas"]
#![warn(missing_docs)]

#[doc = include_str!("../README.md")]
#[cfg(feature = "nightly")]
#[cfg(doctest)]
doc_comment::doctest!("../README.md");

// region:    --- modules
/// Primary module of `DiMAS` containing the Agent
pub mod agent;
/// Communication
pub(crate) mod com;
/// Configuration
pub mod config;
/// Context
pub mod context;
/// Error handling
pub mod error;
/// Public interface of dimas.
/// Typically it is sufficient to include the prelude with
/// `use dimas::prelude::*;`
pub mod prelude;
/// Timer
#[cfg(feature = "timer")]
pub(crate) mod timer;
// Helper functions
//mod utils;
// endregion: --- modules
