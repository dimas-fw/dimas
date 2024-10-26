// Copyright © 2024 Stephan Kunz
#![allow(unused_imports)]
//! Context traits
//!

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use crate::{
	enums::{OperationState, TaskSignal},
	error::Result,
	message_types::{Message, QueryableMsg},
	utils::selector_from,
};
use alloc::{string::String, sync::Arc};
use core::fmt::Debug;
#[cfg(feature = "std")]
use tokio::sync::mpsc::Sender;
use zenoh::Session;
// endregion:	--- modules

// region:		--- Context
/// Typedef for simplified usage
pub type Context<P> = Arc<dyn ContextAbstraction<Props = P>>;

/// Commonalities for the context
#[allow(clippy::module_name_repetitions)]
pub trait ContextAbstraction: Debug + Send + Sync {
	/// The properties structure
	type Props;

	/// Get the name
	#[must_use]
	fn name(&self) -> Option<&String>;

	/// Get the fully qualified name
	#[must_use]
	fn fq_name(&self) -> Option<String>;

	/// Get the [`Context`]s state
	/// # Panics
	#[must_use]
	fn state(&self) -> OperationState;

	/// Set the [`OperationState`].<br>
	/// Setting new state is done step by step
	/// # Errors
	fn set_state(&self, state: OperationState) -> Result<()>;

	/// Get the uuid
	#[must_use]
	fn uuid(&self) -> String;

	/// Get prefix
	#[must_use]
	fn prefix(&self) -> Option<&String>;

	/// Get session mode
	#[must_use]
	fn mode(&self) -> &String;

	/// Get default session reference
	#[must_use]
	fn default_session(&self) -> Arc<Session>;

	/// Get session reference
	#[must_use]
	fn session(&self, session_id: &str) -> Option<Arc<Session>>;

	/// Get sender reference
	#[must_use]
	fn sender(&self) -> &Sender<TaskSignal>;

	/// Gives read access to the properties
	///
	/// # Errors
	fn read(&self) -> Result<std::sync::RwLockReadGuard<'_, Self::Props>>;

	/// Gives write access to the properties
	///
	/// # Errors
	fn write(&self) -> Result<std::sync::RwLockWriteGuard<'_, Self::Props>>;

	/// Method to do a publishing for a `topic`
	/// The `topic` will be enhanced with the prefix.
	/// If there is a publisher stored, it will be used
	/// otherwise an ad-hoc publishing will be done
	///
	/// # Errors
	fn put(&self, topic: &str, message: Message) -> Result<()> {
		let selector = selector_from(topic, self.prefix());
		self.put_with(&selector, message)
	}

	/// Method to do a publishing for a `selector`
	/// If there is a publisher stored, it will be used
	/// otherwise an ad-hoc publishing will be done
	///
	/// # Errors
	fn put_with(&self, selector: &str, message: Message) -> Result<()>;

	/// Method to do a deletion for a `topic`
	/// The `topic` will be enhanced with the prefix.
	/// If there is a publisher stored, it will be used
	/// otherwise an ad-hoc deletion will be done
	///
	/// # Errors
	fn delete(&self, topic: &str) -> Result<()> {
		let selector = selector_from(topic, self.prefix());
		self.delete_with(&selector)
	}

	/// Method to do a deletion for a `selector`
	/// If there is a publisher stored, it will be used
	/// otherwise an ad-hoc deletion will be done
	///
	/// # Errors
	fn delete_with(&self, selector: &str) -> Result<()>;

	/// Send a query for a `topic` with an optional [`Message`].
	/// The `topic` will be enhanced with the prefix.
	/// If there is a query stored, it will be used
	/// otherwise an ad-hoc query will be done
	/// If a callback is given for a stored query,
	/// it will be called instead of the stored callback
	///
	/// # Errors
	fn get(
		&self,
		topic: &str,
		message: Option<Message>,
		callback: Option<&mut dyn FnMut(QueryableMsg) -> Result<()>>,
	) -> Result<()> {
		let selector = selector_from(topic, self.prefix());
		self.get_with(&selector, message, callback)
	}

	/// Send a query for a `selector` with an optional [`Message`].
	/// The `topic` will be enhanced with the prefix.
	/// If there is a query stored, it will be used
	/// otherwise an ad-hoc query will be done
	/// If a callback is given for a stored query,
	/// it will be called instead of the stored callback
	///
	/// # Errors
	fn get_with(
		&self,
		selector: &str,
		message: Option<Message>,
		callback: Option<&mut dyn FnMut(QueryableMsg) -> Result<()>>,
	) -> Result<()>;

	/// Send an observation request for a `topic` with a [`Message`].
	/// The `topic` will be enhanced with the prefix.
	///
	/// # Errors
	fn observe(&self, topic: &str, message: Option<Message>) -> Result<()> {
		let selector = selector_from(topic, self.prefix());
		self.observe_with(&selector, message)
	}

	/// Send an observation request for a `selector` with a [`Message`].
	///
	/// # Errors
	fn observe_with(&self, selector: &str, message: Option<Message>) -> Result<()>;

	/// Cancel an observation request for a `topic`.
	/// The `topic` will be enhanced with the prefix.
	///
	/// # Errors
	fn cancel_observe(&self, topic: &str) -> Result<()> {
		let selector = selector_from(topic, self.prefix());
		self.cancel_observe_with(&selector)
	}

	/// Cancel an observation request for a `selector`.
	///
	/// # Errors
	fn cancel_observe_with(&self, selector: &str) -> Result<()>;
}
// endregion:	--- Context
