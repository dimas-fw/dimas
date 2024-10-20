// Copyright © 2024 Stephan Kunz

//! Trait for communication capabilities.
//!

use super::error::Error;
use dimas_core::{
	error::Result,
	message_types::{Message, QueryableMsg},
};

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
