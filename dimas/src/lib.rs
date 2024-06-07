// Copyright © 2023 Stephan Kunz
#![crate_type = "lib"]
#![crate_name = "dimas"]
//#![no_panic]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

#[cfg(doctest)]
doc_comment::doctest!("../README.md");

// region:    --- modules
// Primary module of `DiMAS` containing the Agent
mod agent;
// Module provides builder.
mod builder;
// Module handles communication with other Agents.
mod com;
// Context of an Agent
mod context;
// Timer
mod timer;

// Public interface of dimas.
pub mod prelude;
// endregion: --- modules
