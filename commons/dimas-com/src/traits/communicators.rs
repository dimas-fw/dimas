// Copyright © 2024 Stephan Kunz

//! Traits for communication.
//!

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use crate::error::Error;
use alloc::{boxed::Box, string::String, sync::Arc};
use dimas_core::{
	enums::OperationState,
	error::Result,
	traits::Capability,
};
#[cfg(feature = "std")]
use std::{collections::HashMap, sync::RwLock};
use tracing::error;
use zenoh::Session;
#[cfg(feature = "unstable")]
use super::LivelinessSubscriber;
use super::{MultiSessionCommunicatorMethods, Observer, Publisher, Querier, Responder, SingleSessionCommunicatorMethods};
// endregion:	--- modules

// region:		--- Communicator
/// the methodes to be implemented by any communicator
pub trait Communicator {
	/// Get the liveliness subscribers
	#[cfg(feature = "unstable")]
	#[must_use]
	fn liveliness_subscribers(
		&self,
	) -> Arc<RwLock<HashMap<String, Box<dyn LivelinessSubscriber>>>>;

	/// Get the observers
	#[must_use]
	fn observers(&self) -> Arc<RwLock<HashMap<String, Box<dyn Observer>>>>;

	/// Get the publishers
	#[must_use]
	fn publishers(&self) -> Arc<RwLock<HashMap<String, Box<dyn Publisher>>>>;

	/// Get the queriers
	#[must_use]
	fn queriers(&self) -> Arc<RwLock<HashMap<String, Box<dyn Querier>>>>;

	/// Get the responders
	#[must_use]
	fn responders(&self) -> Arc<RwLock<HashMap<String, Box<dyn Responder>>>>;

	/// Method for upgrading [`OperationState`] all registered capabilities.<br>
	/// The capabilities are upgraded in the order
	/// - [`LivelinessSubscriber`]s
	/// - `Responders`s: [`Observable`]s, [`Queryable`]s, [`Subscriber`]s
	/// - [`Publisher`]s the
	/// - [`Observer`]s and the
	/// - [`Querier`]s
	///
	/// # Errors
	/// Currently none
	fn upgrade_capabilities(&self, new_state: &OperationState) -> Result<()> {
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

	/// Method for downgrading [`OperationState`] all registered capabilities.<br>
	/// The capabilities are downgraded in reverse order of their start in [`start_registered_tasks()`]
	///
	/// # Errors
	/// Currently none
	fn downgrade_capabilities(&self, new_state: &OperationState) -> Result<()> {
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

/// single session communicator
#[allow(clippy::module_name_repetitions)]
pub trait SingleSessionCommunicator:
	Communicator + SingleSessionCommunicatorMethods + Capability + Send + Sync
{
	/// get the communicator session
	fn session(&self) -> Arc<Session>;
}

/// multi session communicator
#[allow(clippy::module_name_repetitions)]
pub trait MultiSessionCommunicator:
	Communicator
	+ SingleSessionCommunicatorMethods
	+ MultiSessionCommunicatorMethods
	+ Capability
	+ Send
	+ Sync
{
	/// get a communicator session
	fn session(&self, session_id: &str) -> Option<Arc<Session>>;
}
// endregion:	--- Communicator
