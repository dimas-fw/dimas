// Copyright © 2024 Stephan Kunz

//! `dimas-commands` errors

#[doc(hidden)]
extern crate alloc;

// region		--- modules
use alloc::string::String;
use thiserror::Error;
// endregion:	--- modules

// region:		--- Error
/// `dimasctl` error type
#[derive(Error, Debug)]
pub enum Error {
	/// Should not happen
	#[error("this should not have happened in file {0} at line {1}")]
	Unexpected(String, u32),
}
// region:		--- Error

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Error>();
	}
}