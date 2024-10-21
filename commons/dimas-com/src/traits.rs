// Copyright © 2024 Stephan Kunz

//! Traits for communication capabilities.
//!

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use super::error::Error;
use dimas_core::{
	error::Result,
	message_types::{Message, QueryableMsg},
	traits::Capability,
};
#[cfg(feature = "std")]
use std::string::String;

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
		callback: Option<&dyn Fn(QueryableMsg) -> Result<()>>,
	) -> Result<()>;
}

/// `Responder` capabilities
pub trait Responder: Capability + Send + Sync {
	/// Get `selector`
	#[must_use]
	fn selector(&self) -> &str;
}

/// communication capabilities
pub trait Communicator {
	/// Send a put message of type [`Message`] to the given `selector`.
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn put(&self, _selector: &str, _message: Message) -> Result<()> {
		Err(Error::NotImplemented.into())
	}

	/// Send a delete message to the given `selector`.
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn delete(&self, _selector: &str) -> Result<()> {
		Err(Error::NotImplemented.into())
	}

	/// Send a query with an optional specification [`Message`] to the given `selector`.
	/// Answers are collected with the callback
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn get<F>(&self, _selector: &str, _message: Option<Message>, _callback: F) -> Result<()>
	where
		F: FnMut(QueryableMsg) -> Result<()>,
	{
		Err(Error::NotImplemented.into())
	}

	/// Request an observation for [`Message`] from the given `selector`
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn observe(&self, _selector: &str, _message: Message) -> Result<()> {
		Err(Error::NotImplemented.into())
	}

	/// Request a stream configured by [`Message`] from the given `selector`
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn watch(&self, _selector: &str, _message: Message) -> Result<()> {
		Err(Error::NotImplemented.into())
	}
}
