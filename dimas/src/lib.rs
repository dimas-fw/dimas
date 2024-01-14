//! Copyright © 2023 Stephan Kunz
#![crate_type = "lib"]
#![crate_name = "dimas"]

//! [DiMAS](https://github.com/dimas-fw) /dimas/ is a framework for developping distributed multi agent systems

// region:    --- modules
mod agent;
mod com;
mod config;
mod context;
mod error;
mod message;
#[cfg(feature="timer")]
mod timer;
mod utils;

// the public interface
pub mod prelude;
// endregion: --- modules
