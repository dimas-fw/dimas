// Copyright © 2024 Stephan Kunz

//! Traits for communication capabilities.
//!

// region:      --- modules
use crate::error::Error;
use dimas_core::{
	error::Result,
	message_types::{Message, QueryableMsg},
};
// endregion:   --- modules

// region:		--- CommunicatorMethods
/// the communication methods to be implemented by a single session Communicator implementation
#[allow(clippy::module_name_repetitions)]
pub trait CommunicatorMethods {
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
		_callback: Option<&mut dyn FnMut(QueryableMsg) -> Result<()>>,
	) -> Result<()> {
		Err(Error::NotImplemented.into())
	}

	/// Request an observation for [`Message`] from the given `selector`
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn observe(&self, _selector: &str, _message: Option<Message>) -> Result<()> {
		Err(Error::NotImplemented.into())
	}

	/// Request a stream configured by [`Message`] from the given `selector`
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn watch(&self, _selector: &str, _message: Message) -> Result<()> {
		Err(Error::NotImplemented.into())
	}

	/* not used currently
	/// Send a put message [`Message`] from the given `session` to the given `selector`.
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn put_from(&self, _session_id: &str, _selector: &str, _message: Message) -> Result<()> {
		Err(Error::NotImplemented.into())
	}

	/// Send a delete message from the given `session` to the given `selector`.
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn delete_from(&self, _session_id: &str, _selector: &str) -> Result<()> {
		Err(Error::NotImplemented.into())
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

	/// Request an observation for [`Message`] from the given `session` from the given `selector`
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn observe_from(
		&self,
		_session: &str,
		_selector: &str,
		_message: Option<Message>,
	) -> Result<()> {
		Err(Error::NotImplemented.into())
	}

	/// Request a stream configured by [`Message`] from the given `session` from the given `selector`
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn watch_from(&self, _session: &str, _selector: &str, _message: Message) -> Result<()> {
		Err(Error::NotImplemented.into())
	}
	*/
}
// endregion:	--- CommunicatorMethods

// region:		--- CommunicatorImplementation
/// the communication methods to be implemented by any Communicator implementation
#[allow(clippy::module_name_repetitions)]
pub trait CommunicatorImplementationMethods {
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
		_callback: Option<&mut dyn FnMut(QueryableMsg) -> Result<()>>,
	) -> Result<()> {
		Err(Error::NotImplemented.into())
	}

	/// Request an observation for [`Message`] from the given `selector`
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn observe(&self, _selector: &str, _message: Option<Message>) -> Result<()> {
		Err(Error::NotImplemented.into())
	}

	/// Request a stream configured by [`Message`] from the given `selector`
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn watch(&self, _selector: &str, _message: Message) -> Result<()> {
		Err(Error::NotImplemented.into())
	}
}
// endregion:	---CommunicatorImplementation
