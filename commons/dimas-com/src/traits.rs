// Copyright © 2024 Stephan Kunz

//! Traits for communication capabilities.
//!

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use crate::error::Error;
use alloc::{boxed::Box, string::String, sync::Arc};
use dimas_core::{
	enums::OperationState,
	error::Result,
	message_types::{Message, QueryableMsg},
	traits::Capability,
};
#[cfg(feature = "std")]
use std::{collections::HashMap, sync::RwLock};
use tracing::error;
use zenoh::Session;

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
/// the methodes to be implemented by any communicator
pub trait Communicator {
	/// Get the liveliness subscribers
	#[cfg(feature = "unstable")]
	#[must_use]
	fn liveliness_subscribers(
		&self,
	) -> &Arc<RwLock<HashMap<String, Box<dyn LivelinessSubscriber>>>>;

	/// Get the observers
	#[must_use]
	fn observers(&self) -> &Arc<RwLock<HashMap<String, Box<dyn Observer>>>>;

	/// Get the publishers
	#[must_use]
	fn publishers(&self) -> Arc<RwLock<HashMap<String, Box<dyn Publisher>>>>;

	/// Get the queriers
	#[must_use]
	fn queriers(&self) -> &Arc<RwLock<HashMap<String, Box<dyn Querier>>>>;

	/// Get the responders
	#[must_use]
	fn responders(&self) -> &Arc<RwLock<HashMap<String, Box<dyn Responder>>>>;

	/// Method for upgrading [`OperationState`] of all registered tasks.<br>
	/// The tasks are upgraded in the order
	/// - [`LivelinessSubscriber`]s
	/// - `Responders`s: [`Observable`]s, [`Queryable`]s, [`Subscriber`]s
	/// - [`Publisher`]s the
	/// - [`Observer`]s and the
	/// - [`Querier`]s
	///
	/// # Errors
	/// Currently none
	fn upgrade_registered_tasks(&self, new_state: &OperationState) -> Result<()> {
		// start liveliness subscriber
		#[cfg(feature = "unstable")]
		self.liveliness_subscribers()
			.write()
			.map_err(|_| Error::ModifyStruct("liveliness subscribers".into()))?
			.iter_mut()
			.for_each(|subscriber| {
				let _ = subscriber.1.manage_operation_state(new_state);
			});

		// start all registered responders
		self.responders()
			.write()
			.map_err(|_| Error::ModifyStruct("subscribers".into()))?
			.iter_mut()
			.for_each(|subscriber| {
				let _ = subscriber.1.manage_operation_state(new_state);
			});

		// init all registered publishers
		self.publishers()
			.write()
			.map_err(|_| Error::ModifyStruct("publishers".into()))?
			.iter_mut()
			.for_each(|publisher| {
				if let Err(reason) = publisher.1.manage_operation_state(new_state) {
					error!(
						"could not initialize publisher for {}, reason: {}",
						publisher.1.selector(),
						reason
					);
				};
			});

		// init all registered observers
		self.observers()
			.write()
			.map_err(|_| Error::ModifyStruct("observers".into()))?
			.iter_mut()
			.for_each(|observer| {
				if let Err(reason) = observer.1.manage_operation_state(new_state) {
					error!(
						"could not initialize observer for {}, reason: {}",
						observer.1.selector(),
						reason
					);
				};
			});

		// init all registered queries
		self.queriers()
			.write()
			.map_err(|_| Error::ModifyStruct("queries".into()))?
			.iter_mut()
			.for_each(|query| {
				if let Err(reason) = query.1.manage_operation_state(new_state) {
					error!(
						"could not initialize query for {}, reason: {}",
						query.1.selector(),
						reason
					);
				};
			});

		Ok(())
	}

	/// Method for downgrading [`OperationState`] of all registered tasks.<br>
	/// The tasks are downgraded in reverse order of their start in [`start_registered_tasks()`]
	///
	/// # Errors
	/// Currently none
	fn downgrade_registered_tasks(&self, new_state: &OperationState) -> Result<()> {
		// reverse order of start!
		// de-init all registered queries
		self.queriers()
			.write()
			.map_err(|_| Error::ModifyStruct("queries".into()))?
			.iter_mut()
			.for_each(|query| {
				if let Err(reason) = query.1.manage_operation_state(new_state) {
					error!(
						"could not de-initialize query for {}, reason: {}",
						query.1.selector(),
						reason
					);
				};
			});

		// de-init all registered observers
		self.observers()
			.write()
			.map_err(|_| Error::ModifyStruct("observers".into()))?
			.iter_mut()
			.for_each(|observer| {
				if let Err(reason) = observer.1.manage_operation_state(new_state) {
					error!(
						"could not de-initialize observer for {}, reason: {}",
						observer.1.selector(),
						reason
					);
				};
			});

		// de-init all registered publishers
		self.publishers()
			.write()
			.map_err(|_| Error::ModifyStruct("publishers".into()))?
			.iter_mut()
			.for_each(|publisher| {
				let _ = publisher.1.manage_operation_state(new_state);
			});

		// stop all registered responders
		self.responders()
			.write()
			.map_err(|_| Error::ModifyStruct("subscribers".into()))?
			.iter_mut()
			.for_each(|subscriber| {
				let _ = subscriber.1.manage_operation_state(new_state);
			});

		// stop all registered liveliness subscribers
		#[cfg(feature = "unstable")]
		self.liveliness_subscribers()
			.write()
			.map_err(|_| Error::ModifyStruct("liveliness subscribers".into()))?
			.iter_mut()
			.for_each(|subscriber| {
				let _ = subscriber.1.manage_operation_state(new_state);
			});

		Ok(())
	}

	/// the uuid of the communicator
	#[must_use]
	fn uuid(&self) -> String;

	/// the mode of the communicator
	#[must_use]
	fn mode(&self) -> &String;
}

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
	fn observe_from(
		&self,
		_session: &str,
		_selector: &str,
		_message: Option<Message>,
	) -> Result<()> {
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
	Communicator + SingleSessionCommunicatorMethods + Capability + Send + Sync
{
}

/// communicator capabilities
pub trait MultiSessionCommunicator:
	Communicator + MultiSessionCommunicatorMethods + Capability + Send + Sync
{
	/// get a communicator session
	fn session(&self, session_id: &str) -> Option<Arc<Session>>;
}
// endregion:	--- communication
