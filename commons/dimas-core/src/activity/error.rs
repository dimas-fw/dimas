// Copyright © 2024 Stephan Kunz

//! [`Activity`] errors
//!

#[doc(hidden)]
extern crate alloc;

// region:		--- modules
use thiserror::Error;
// endregion:	--- modules

// region:		--- Error
/// `dimas-core::Activity` error type.
#[derive(Error, Debug)]
pub enum Error {}
// endregion:	--- Error