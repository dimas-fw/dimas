// Copyright © 2024 Stephan Kunz

//! Traits for communication capabilities.
//!

#[doc(hidden)]
extern crate alloc;

use alloc::{string::String, sync::Arc};
use dimas_core::{
	error::Result,
	message_types::{Message, QueryableMsg},
	traits::Capability,
};
use zenoh::Session;

use crate::error::Error;

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

// region:		--- communication
/// the communication methods to be implemented by a single session Communicator implementation
pub trait SingleSessionCommunicatorMethods {
	/// Send a put message [`Message`] to the given `selector`.
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
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn get(
		&self,
		_selector: &str,
		_message: Option<Message>,
		_callback: &mut dyn FnMut(QueryableMsg) -> Result<()>,
	) -> Result<()> {
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

/// the communication methods to be implemented by a multi session Communicator
pub trait MultiSessionCommunicatorMethods {
	/// Send a put message [`Message`] to the given `selector`.
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn put(&self, selector: &str, message: Message) -> Result<()> {
		self.put_from("default", selector, message)
	}

	/// Send a put message [`Message`] from the given `session` to the given `selector`.
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn put_from(&self, _session_id: &str, _selector: &str, _message: Message) -> Result<()> {
		Err(Error::NotImplemented.into())
	}

	/// Send a delete message to the given `selector`.
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn delete(&self, selector: &str) -> Result<()> {
		self.delete_from("default", selector)
	}

	/// Send a delete message from the given `session` to the given `selector`.
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn delete_from(&self, _session_id: &str, _selector: &str) -> Result<()> {
		Err(Error::NotImplemented.into())
	}

	/// Send a query with an optional specification [`Message`] to the given `selector`.
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn get(
		&self,
		selector: &str,
		message: Option<Message>,
		callback: Option<&mut dyn FnMut(QueryableMsg) -> Result<()>>,
	) -> Result<()> {
		self.get_from("default", selector, message, callback)
	}

	/// Send a query with an optional specification [`Message`] from the given `session` to the given `selector`.
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn get_from(
		&self,
		_session_id: &str,
		_selector: &str,
		_message: Option<Message>,
		_callback: Option<&mut dyn FnMut(QueryableMsg) -> Result<()>>,
	) -> Result<()> {
		Err(Error::NotImplemented.into())
	}

	/// Request an observation for [`Message`] from the given `selector`
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn observe(&self, selector: &str, message: Option<Message>) -> Result<()> {
		self.observe_from("default", selector, message)
	}

	/// Request an observation for [`Message`] from the given `session` from the given `selector`
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn observe_from(&self, _session: &str, _selector: &str, _message: Option<Message>) -> Result<()> {
		Err(Error::NotImplemented.into())
	}

	/// Request a stream configured by [`Message`] from the given `selector`
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn watch(&self, selector: &str, message: Message) -> Result<()> {
		self.watch_from("default", selector, message)
	}

	/// Request a stream configured by [`Message`] from the given `session` from the given `selector`
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn watch_from(&self, _session: &str, _selector: &str, _message: Message) -> Result<()> {
		Err(Error::NotImplemented.into())
	}
}

/// communicator implementation capabilities
pub trait SingleSessionCommunicator:
	SingleSessionCommunicatorMethods + Capability + Send + Sync
{
}

/// communicator capabilities
pub trait MultiSessionCommunicator:
	MultiSessionCommunicatorMethods + Capability + Send + Sync
{
	/// get a communicator session
	fn session(&self, session_id: &str) -> Option<Arc<Session>>;
}
// endregion:	--- communication
