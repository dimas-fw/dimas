// Copyright © 2024 Stephan Kunz

//! `dimas-time` errors

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::string::String;
use thiserror::Error;

// region:		--- Error
/// `dimas-time` error type.
#[derive(Error, Debug)]
pub enum Error {
	/// ivalid #include directive
	#[error("invalid '#include' at {0} in {0}")]
	InvalidInclude(String),
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
