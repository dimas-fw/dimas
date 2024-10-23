// Copyright © 2023 Stephan Kunz

//! The `DiMAS` specific error enum `DimasError` together with a
//! type alias for [`core::result::Result`] to write only `Result<T>`.
//!

#[doc(hidden)]
extern crate alloc;

// region:		--- modules
#[cfg(doc)]
use super::enums::OperationState;
use alloc::{boxed::Box, string::String};
// endregion:	--- modules

// region:		--- types
/// Result type alias.
pub type Result<T> = core::result::Result<T, Box<dyn core::error::Error + Send + Sync + 'static>>;
// endregion:	--- types

// region:		--- Error
/// Core error type.
pub enum Error {
	/// decoding failed
	Decoding {
		/// the original bitcode error
		source: Box<dyn core::error::Error + Send + Sync>,
	},
	/// sending reply failed
	Reply {
		/// the original zenoh error
		source: Box<dyn core::error::Error + Send + Sync>,
	},
	/// empty request
	EmptyQuery,
	/// An unknown [`OperationState`] is given
	UnknownOperationState {
		/// name of the operation state
		state: String,
	},
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
			Self::Decoding { source } => {
				write!(f, "creation of zenoh session failed: reason {source}")
			}
			Self::Reply { source } => write!(f, "publishing a put message failed: reason {source}"),
			Self::EmptyQuery => write!(f, "query was empty"),
			Self::UnknownOperationState { state } => {
				write!(f, "the operation state {state} is unknown")
			}
		}
	}
}

impl core::error::Error for Error {
	fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
		match *self {
			Self::Decoding { ref source } | Self::Reply { ref source } => Some(source.as_ref()),
			Self::EmptyQuery | Self::UnknownOperationState { .. } => None,
		}
	}
}
// endregion:   --- boilerplate

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
