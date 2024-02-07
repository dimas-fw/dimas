// Copyright © 2023 Stephan Kunz
#![crate_type = "lib"]
#![crate_name = "dimas"]
//#![warn(missing_docs)]

//! This crate is on [crates.io](https://crates.io/crates/dimas).
//!
//! `DiMAS` is tested on Windows (Version 10) and Linux (Ubuntu/Debian flavours) but should also run on `MacOS`.
//!
//! [DiMAS](https://github.com/dimas-fw/dimas/tree/main/dimas) follows the semantic versioning principle with the enhancement,
//! that until version 1.0.0 each new version may include breaking changes, which will be noticed in the changelog.

#[doc = include_str!("../README.md")]
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
