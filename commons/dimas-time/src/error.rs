// Copyright © 2024 Stephan Kunz

//! Errors from com

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::string::String;

// region:		--- Error
/// Com error type.
pub enum Error {
	/// A Mutex is poisoned.
	MutexPoison(String),
}
// region:		--- Error

// region:      --- boilerplate
impl core::fmt::Display for Error {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl core::fmt::Debug for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::MutexPoison(location) => {
				write!(f, "an Mutex poison error happened in {location}")
			}
		}
	}
}

impl core::error::Error for Error {
	fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
		match *self {
			Self::MutexPoison { .. } => None,
		}
	}
}
// endregion:   --- boilerplate
