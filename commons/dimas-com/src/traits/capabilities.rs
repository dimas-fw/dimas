// Copyright © 2024 Stephan Kunz

//! Traits for communication capabilities.
//!

#[doc(hidden)]
extern crate alloc;

use alloc::string::String;
use dimas_core::{
	error::Result,
	message_types::{Message, QueryableMsg},
	traits::Capability,
};

// region:		--- capabilities
/// `LivelinessSubscriber` capabilities
pub trait LivelinessSubscriber: Capability + Send + Sync {
	/// get token
	fn token(&self) -> &String;
}

/// `Observer` capabilities
pub trait Observer: Capability + Send + Sync {
	/// Get `selector`
	#[must_use]
	fn selector(&self) -> &str;

	/// Cancel a running observation
	/// # Errors
	fn cancel(&self) -> Result<()>;

	/// Request an observation with an optional [`Message`].
	/// # Errors
	fn request(&self, message: Option<Message>) -> Result<()>;
}

/// `Publisher` capabilities
pub trait Publisher: Capability + Send + Sync {
	/// Get `selector`
	#[must_use]
	fn selector(&self) -> &str;

	/// Send a "put" message
	/// # Errors
	fn put(&self, message: Message) -> Result<()>;

	/// Send a "delete" message
	/// # Errors
	fn delete(&self) -> Result<()>;
}

/// `Querier` capabilities
pub trait Querier: Capability + Send + Sync {
	/// Get `selector`
	#[must_use]
	fn selector(&self) -> &str;

	/// Run a Querier with an optional [`Message`].
	/// # Errors
	fn get(
		&self,
		message: Option<Message>,
		callback: Option<&mut dyn FnMut(QueryableMsg) -> Result<()>>,
	) -> Result<()>;
}

/// `Responder` capabilities
pub trait Responder: Capability + Send + Sync {
	/// Get `selector`
	#[must_use]
	fn selector(&self) -> &str;
}
// endregion:	--- capabilities
