// Copyright © 2024 Stephan Kunz

//! Implementation of a multi session/protocol communicator
//!

// region:		--- modules
#[cfg(feature = "unstable")]
use crate::traits::LivelinessSubscriber;
use anyhow::Result;
use dimas_config::Config;
use dimas_core::{
	enums::OperationState,
	message_types::{Message, QueryableMsg},
	traits::Operational,
};
use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
};
use zenoh::{config::ZenohId, Session};

use super::error::Error;
use crate::{
	enums::CommunicatorImplementation,
	traits::{
		Communicator, CommunicatorImplementationMethods, CommunicatorMethods, Observer, Publisher,
		Querier, Responder,
	},
};
// endregion:   --- modules

// region:		--- types
/// the initial size of the `HashMaps`
const INITIAL_SIZE: usize = 9;
/// id for default communication session
const DEFAULT: &str = "default";
// endregion:	--- types

// region:      --- MultiCommunicator
/// a multi session communicator
#[derive(Debug)]
pub struct MultiCommunicator {
	/// a uuid generated by default zenoh session
	uuid: ZenohId,
	/// the mode of default zenoh session
	mode: String,
	/// The [`Communicator`]s current operational state.
	state: OperationState,
	/// Registered Communicators
	communicators: Arc<RwLock<HashMap<String, Arc<CommunicatorImplementation>>>>,
	/// Registered [`LivelinessSubscriber`]
	#[cfg(feature = "unstable")]
	liveliness_subscribers: Arc<RwLock<HashMap<String, Box<dyn LivelinessSubscriber>>>>,
	/// Registered [`Observer`]
	observers: Arc<RwLock<HashMap<String, Box<dyn Observer>>>>,
	/// Registered [`Publisher`]
	publishers: Arc<RwLock<HashMap<String, Box<dyn Publisher>>>>,
	/// Registered [`Query`]s
	queriers: Arc<RwLock<HashMap<String, Box<dyn Querier>>>>,
	/// Registered [`Observable`]s, [`Queryable`]s and [`Subscriber`]s
	responders: Arc<RwLock<HashMap<String, Box<dyn Responder>>>>,
}

impl Operational for MultiCommunicator {
	fn manage_operation_state(&self, new_state: OperationState) -> Result<()> {
		if new_state >= self.state {
			self.upgrade_capabilities(new_state)?;
		} else if new_state < self.state {
			self.downgrade_capabilities(new_state)?;
		}
		Ok(())
	}
}

impl CommunicatorMethods for MultiCommunicator {
	/// Send a put message [`Message`] to the given `selector`.
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn put(&self, selector: &str, message: Message) -> Result<()> {
		let publishers = self
			.publishers
			.read()
			.map_err(|_| Error::ReadAccess("publishers".into()))?;

		#[allow(clippy::single_match_else)]
		match publishers.get(selector) {
			Some(publisher) => publisher.put(message),
			None => {
				let comm = self
					.communicators
					.read()
					.map_err(|_| Error::ReadAccess("publishers".into()))?
					.get(DEFAULT)
					.ok_or_else(|| Error::NoCommunicator(DEFAULT.into()))
					.cloned()?;

				comm.put(selector, message)
			}
		}
	}

	/// Send a delete message to the given `selector`.
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn delete(&self, selector: &str) -> Result<()> {
		let publishers = self
			.publishers
			.read()
			.map_err(|_| Error::ReadAccess("publishers".into()))?;

		#[allow(clippy::single_match_else)]
		match publishers.get(selector) {
			Some(publisher) => publisher.delete(),
			None => {
				let comm = self
					.communicators
					.read()
					.map_err(|_| Error::ReadAccess("publishers".into()))?
					.get(DEFAULT)
					.ok_or_else(|| Error::NoCommunicator(DEFAULT.into()))
					.cloned()?;

				#[allow(clippy::match_wildcard_for_single_variants)]
				match comm.as_ref() {
					CommunicatorImplementation::Zenoh(zenoh) => zenoh.delete(selector),
				}
			}
		}
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
		let queriers = self
			.queriers
			.read()
			.map_err(|_| Error::ReadAccess("queriers".into()))?;

		#[allow(clippy::single_match_else)]
		match queriers.get(selector) {
			Some(querier) => querier.get(message, callback),
			None => {
				let comm = self
					.communicators
					.read()
					.map_err(|_| Error::ReadAccess("queriers".into()))?
					.get(DEFAULT)
					.ok_or_else(|| Error::NoCommunicator(DEFAULT.into()))
					.cloned()?;

				match comm.as_ref() {
					CommunicatorImplementation::Zenoh(zenoh) => {
						zenoh.get(selector, message, callback)
					}
				}
			}
		}
	}

	/// Request an observation for [`Message`] from the given `selector`
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn observe(&self, selector: &str, message: Option<Message>) -> Result<()> {
		let observers = self
			.observers
			.read()
			.map_err(|_| Error::ReadAccess("observers".into()))?;

		#[allow(clippy::single_match_else)]
		match observers.get(selector) {
			Some(observer) => observer.request(message),
			None => {
				let comm = self
					.communicators
					.read()
					.map_err(|_| Error::ReadAccess("observers".into()))?
					.get(DEFAULT)
					.ok_or_else(|| Error::NoCommunicator(DEFAULT.into()))
					.cloned()?;

				#[allow(clippy::match_wildcard_for_single_variants)]
				match comm.as_ref() {
					CommunicatorImplementation::Zenoh(_zenoh) => Err(Error::NotImplemented.into()),
				}
			}
		}
	}

	/// Request a stream configured by [`Message`] from the given `selector`
	/// # Errors
	/// - `NotImplemented`: there is no implementation within this communicator
	fn watch(&self, _selector: &str, _message: Message) -> Result<()> {
		Err(Error::NotImplemented.into())
	}
}

impl Communicator for MultiCommunicator {
	/// the uuid of the default zenoh session
	fn uuid(&self) -> String {
		self.uuid.to_string()
	}

	/// the mode of the default zenoh session
	fn mode(&self) -> &String {
		&self.mode
	}

	/// Get the liveliness subscribers
	#[cfg(feature = "unstable")]
	fn liveliness_subscribers(
		&self,
	) -> Arc<RwLock<HashMap<String, Box<dyn LivelinessSubscriber>>>> {
		self.liveliness_subscribers.clone()
	}

	/// Get the observers
	fn observers(&self) -> Arc<RwLock<HashMap<String, Box<dyn Observer>>>> {
		self.observers.clone()
	}

	/// Get the publishers
	fn publishers(&self) -> Arc<RwLock<HashMap<String, Box<dyn Publisher>>>> {
		self.publishers.clone()
	}

	/// Get the queries
	fn queriers(&self) -> Arc<RwLock<HashMap<String, Box<dyn Querier>>>> {
		self.queriers.clone()
	}

	/// Get the responders
	fn responders(&self) -> Arc<RwLock<HashMap<String, Box<dyn Responder>>>> {
		self.responders.clone()
	}

	fn default_session(&self) -> Arc<Session> {
		let com = self
			.communicators
			.read()
			.expect("snh")
			.get(DEFAULT)
			.cloned()
			.expect("snh");
		match com.as_ref() {
			CommunicatorImplementation::Zenoh(communicator) => communicator.session(),
		}
	}

	fn session(&self, id: &str) -> Option<Arc<zenoh::Session>> {
		let com = self
			.communicators
			.read()
			.expect("snh")
			.get(id)
			.cloned()
			.expect("snh");
		match com.as_ref() {
			CommunicatorImplementation::Zenoh(communicator) => {
				let com = communicator.session();
				Some(com)
			}
		}
	}

	fn sessions(&self) -> Vec<Arc<Session>> {
		let com: Vec<Arc<Session>> = self
			.communicators
			.read()
			.expect("snh")
			.iter()
			.map(|(_id, com)| match com.as_ref() {
				CommunicatorImplementation::Zenoh(communicator) => communicator.session(),
			})
			.collect();
		com
	}
}

impl MultiCommunicator {
	/// Constructor
	/// # Errors
	pub fn new(config: &Config) -> Result<Self> {
		let zenoh = crate::zenoh::Communicator::new(config.zenoh_config())?;
		let uuid = zenoh.session().zid();
		let mode = zenoh.mode().to_string();
		let com = Self {
			uuid,
			mode,
			state: OperationState::Created,
			communicators: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			#[cfg(feature = "unstable")]
			liveliness_subscribers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			observers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			publishers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			queriers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			responders: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
		};
		// add the default communicator
		com.communicators
			.write()
			.map_err(|_| Error::ModifyStruct("commmunicators".into()))?
			.insert(
				"default".to_string(),
				Arc::new(CommunicatorImplementation::Zenoh(zenoh)),
			);
		// create the additional sessions
		if let Some(sessions) = config.sessions() {
			for session in sessions {
				match session.protocol.as_str() {
					"zenoh" => {
						let zenoh = crate::zenoh::Communicator::new(config.zenoh_config())?;
						com.communicators
							.write()
							.map_err(|_| Error::ModifyStruct("commmunicators".into()))?
							.insert(
								session.name.clone(),
								Arc::new(CommunicatorImplementation::Zenoh(zenoh)),
							);
					}
					_ => {
						return Err(Error::UnknownProtocol {
							protocol: session.protocol.clone(),
						}
						.into());
					}
				}
			}
		}

		Ok(com)
	}
}
// endregion:   --- MultiCommunicator

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<MultiCommunicator>();
	}
}
