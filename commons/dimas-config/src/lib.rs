// Copyright © 2024 Stephan Kunz
#![no_std]

//! Library for configuration
//!

mod config;
mod error;
mod utils;

// flatten
pub use config::Config;
pub use error::Error;

// region:		--- types
#[doc(hidden)]
extern crate alloc;

/// copy of Result type alias from `dimas_core` to avoid dependency
pub(crate) type Result<T> =
	core::result::Result<T, alloc::boxed::Box<dyn core::error::Error + Send + Sync + 'static>>;
// endregion:	--- types
