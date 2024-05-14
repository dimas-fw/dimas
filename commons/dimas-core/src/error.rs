// Copyright © 2023 Stephan Kunz

//! The `DiMAS` specific error enum `DimasError` together with a type alias for [`std::result::Result`] to write only `Result<T>`.
//!

// region:		--- types
/// Type alias for `std::result::Result` to ease up implementation
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;
// endregion:	--- types

// region:    --- DimasError
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
	Put,
	/// The `delete` of a `Publisher` failed
	#[error("Publisher 'delete' failed")]
	Delete,
	/// The `get` of a `Query` failed
	#[error("Query 'get' failed")]
	Get,
	/// Encoding of message failed
	#[error("message encoding failed")]
	Encoding,
	/// Converting of message failed
	#[error("converting value into 'Vec<u8>' failed")]
	ConvertingValue,
	/// Decoding of message failed
	#[error("message decoding failed")]
	Decoding,
	/// Read access to properties failed
	#[error("read of properties failed")]
	ReadProperties,
	/// Write access to properties failed
	#[error("write of properties failed")]
	WriteProperties,
	/// Lock on callback failed
	#[error("could not execute callback")]
	ExecuteCallback,

	/// Invalid OperationState
	#[error("invalid OperationState {0}")]
	OperationState(String),
	/// File not found
	#[error("could not find file: {0}")]
	FileNotFound(String),
	/// Modifying context failed
	#[error("modifying context for {0} failed")]
	ModifyContext(String),
	/// The `set_state` failed
	#[error("setting the 'OperationState' failed")]
	ManageState,
	/// Reading context failed
	#[error("reading context for {0} failed")]
	ReadContext(String),

	/// `zenoh` session creation failed
	#[error("creation of zenoh session failed with {0}")]
	CreateSession(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
	/// `zenoh` activate sending liveliness failed
	#[error("activation of zenoh liveliness failed with {0}")]
	ActivateLiveliness(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
	/// `zenoh` publisher  declaration failed
	#[error("declaration of zenoh publisher failed with {0}")]
	DeclarePublisher(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

	// should be last line
	/// auto conversion for boxed `std::error::Error`
	#[error(transparent)]
	StdError(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
} // endregion: --- DimasError

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
