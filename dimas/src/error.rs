// Copyright © 2023 Stephan Kunz

//! Module `error` provides the DiMAS specific `Error`s.

// region:    --- Error
/// `DiMAS` Error type
#[derive(thiserror::Error, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum DimasError {
	/// A custom error message
	#[error("{0}")]
	Custom(String),
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

	// should be last line
	/// standard error for boxed `std::error::Error`
	#[error(transparent)]
	StdError(#[from] Box<dyn std::error::Error + 'static>),
}
// endregion: --- Error
