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
// Primary module of `DiMAS` containing the Agent
pub mod agent;
// Module handles communication with other Agents.
pub mod com;
// Configuration for an Agent
pub mod config;
// Context of an Agent
pub mod context;
// Error handling
pub mod error;
// Timer
pub mod timer;
// Helper functions
mod utils;

// Public interface of dimas.
pub mod prelude;
// endregion: --- modules
