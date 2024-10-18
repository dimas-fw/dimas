// Copyright © 2023 Stephan Kunz
#![crate_type = "lib"]
#![crate_name = "dimas"]
//#![no_panic]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

#[cfg(doctest)]
doc_comment::doctest!("../README.md");

// region:    --- modules
mod error;

pub mod agent;
pub mod builder;
pub mod context;

// Simplified usage of dimas.
pub mod prelude;

#[cfg(doc)]
use crate::agent::Agent;
pub use dimas_macros::main;
// endregion: --- modules
