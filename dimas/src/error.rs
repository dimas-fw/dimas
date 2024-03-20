// Copyright © 2023 Stephan Kunz

//! The `DiMAS` specific error enum [`DimasError`] togehter with a type alias for [`std::result::Result`] to write only `Result<T>`.
//!
//! # Examples
//! ```rust,no_run
//! # use dimas::prelude::*;
//! # #[tokio::main(flavor = "multi_thread")]
//! # async fn main() -> Result<()> {
//! # Ok(())
//! # }
//! ```
//!

// region:		--- types
/// Type alias for `std::result::Result` to ease up implementation
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;
// endregion:	--- types

// region:    --- Error
/// `DiMAS` Error type
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum DimasError {
	/// this error should never happen
	#[error("should not happen")]
	ShouldNotHappen,
	/// The `put` of a `Publisher` failed
	#[error("Publisher 'put' failed")]
	PutMessage,
	/// The `delete` of a `Publisher` failed
	#[error("Publisher 'delete' failed")]
	DeleteMessage,
	/// There was no key expression given to the Builder
	#[error("no key expression given")]
	NoKeyExpression,
	/// There was no callback function given to the Builder
	#[error("no callback given")]
	NoCallback,
	/// There was no interval duration given to the `TimerBuilder`
	#[error("no interval given")]
	NoInterval,
	/// There was no name given to the `TimerBuilder`
	#[error("no name given")]
	NoName,
	/// Encoding of message failed
	#[error("message encoding failed")]
	EncodingMessage,
	/// Converting of message failed
	#[error("converting value into 'Vec<u8>' failed")]
	ConvertingValue,
	/// Decoding of message failed
	#[error("message decoding failed")]
	DecodingMessage,
	/// Read access to properties failed
	#[error("read  of properties failed")]
	ReadProperties,
	/// Write access to properties failed
	#[error("write  of properties failed")]
	WriteProperties,
	/// Lock on callback failed
	#[error("could not execute callback")]
	ExecuteCallback,

	/// File not found
	#[error("Could not find file: {0}")]
	FileNotFound(String),

	/// `zenoh` session creation failed
	#[error("Creation of zenoh session failed: {0}")]
	SessionCreation(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

	// should be last line
	/// auto conversion for boxed `std::error::Error`
	#[error(transparent)]
	StdError(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
} // endregion: --- Error

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<DimasError>();
	}
}
