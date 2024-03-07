// Copyright © 2023 Stephan Kunz

//! Module `error` provides the DiMAS specific `Error`s.

// region:		--- modules
// endregion:	--- modules

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
	PutFailed,
	/// The `delete` of a `Publisher` failed
	#[error("Publisher 'delete' failed")]
	DeleteFailed,
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
	EncodingFailed,
	/// Decoding of message failed
	#[error("message decoding failed")]
	DecodingFailed,
	/// Read access to properties failed
	#[error("read  of properties failed")]
	ReadPropertiesFailed,
	/// Write access to properties failed
	#[error("write  of properties failed")]
	WritePropertiesFailed,
	/// Lock on callback failed
	#[error("could not execute callback")]
	CallbackFailed,

	/// File not found
	#[error("Could not find file: {0}")]
	FileNotFound(String),

	/// No `zenoh` configuration
	#[error("No zenoh configuration")]
	NoZenohConfig,
	/// Error in `zenoh` configuration
	#[error("Cannot parse zenoh configuration: {0}")]
	ParseConfig(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
	/// `zenoh` session creation failed
	#[error("Creation of zenoh session failed: {0}")]
	SessionCreation(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}
// endregion: --- Error
