// Copyright © 2023 Stephan Kunz

// region:    --- types
/// Enables simplified usage of Result with dimas Error type
pub type Result<T> = core::result::Result<T, Error>;
// endregion: --- types

// region:    --- Error
/// `DiMAS` Error type
#[derive(thiserror::Error, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum Error {
	/// A generic error
	#[error("Generic {0}")]
	Generic(String),
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
}

impl Default for Error {
	fn default() -> Self {
		Self::Generic("error".to_string())
	}
}
// endregion: --- Error
