// Copyright © 2024 Stephan Kunz

//! Implementation of a multi session/protocol communicator
//!

// region:		--- modules
#[cfg(feature = "unstable")]
use crate::traits::LivelinessSubscriber;
use anyhow::Result;
use dimas_config::Config;
use dimas_core::{enums::OperationState, message_types::Message, traits::Operational};
use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
};
use zenoh::{config::ZenohId, Session};

use crate::{
	enums::CommunicatorImplementation,
	error::Error,
	traits::{
		Communicator, CommunicatorImplementationMethods, CommunicatorMethods, Observer, Publisher,
		Querier, Responder,
	},
};
// endregion:	--- modules

// region:		--- types
// the initial size of the HashMaps
const INITIAL_SIZE: usize = 9;
// endregion:	--- types

// region:		--- SingleCommunicator
/// a multi session communicator
#[derive(Debug)]
pub struct SingleCommunicator {
	/// a uuid generated by default zenoh session
	uuid: ZenohId,
	/// the mode of default zenoh session
	mode: String,
	/// The [`Communicator`]s current operational state.
	state: OperationState,
	/// Registered Communicator
	communicator: Arc<CommunicatorImplementation>,
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

impl Operational for SingleCommunicator {
	fn manage_operation_state(&self, new_state: OperationState) -> Result<()> {
		if new_state >= self.state {
			self.upgrade_capabilities(new_state)?;
		} else if new_state < self.state {
			self.downgrade_capabilities(new_state)?;
		}
		Ok(())
	}
}

impl Communicator for SingleCommunicator {
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

	fn uuid(&self) -> std::string::String {
		self.uuid.to_string()
	}

	fn mode(&self) -> &std::string::String {
		&self.mode
	}

	fn default_session(&self) -> Arc<Session> {
		self.communicator.session()
	}

	fn session(&self, id: &str) -> Option<Arc<Session>> {
		if id == "default" {
			Some(self.communicator.session())
		} else {
			None
		}
	}

	#[allow(clippy::vec_init_then_push)]
	fn sessions(&self) -> Vec<Arc<Session>> {
		let mut res = Vec::with_capacity(1);
		res.push(self.communicator.session());
		res
	}
}

impl CommunicatorMethods for SingleCommunicator {
	fn put(&self, selector: &str, message: Message) -> Result<()> {
		let publishers = self
			.publishers
			.read()
			.map_err(|_| Error::ReadAccess("publishers".into()))?;

		#[allow(clippy::single_match_else)]
		match publishers.get(selector) {
			Some(publisher) => publisher.put(message),
			None => match self.communicator.as_ref() {
				CommunicatorImplementation::Zenoh(zenoh) => zenoh.put(selector, message),
			},
		}
	}

	fn delete(&self, selector: &str) -> Result<()> {
		let publishers = self
			.publishers
			.read()
			.map_err(|_| Error::ReadAccess("publishers".into()))?;

		#[allow(clippy::option_if_let_else)]
		match publishers.get(selector) {
			Some(publisher) => publisher.delete(),
			None => match self.communicator.as_ref() {
				CommunicatorImplementation::Zenoh(zenoh) => zenoh.delete(selector),
			},
		}
	}

	fn get(
		&self,
		selector: &str,
		message: Option<dimas_core::message_types::Message>,
		callback: Option<&mut dyn FnMut(dimas_core::message_types::QueryableMsg) -> Result<()>>,
	) -> Result<()> {
		let queriers = self
			.queriers
			.read()
			.map_err(|_| Error::ReadAccess("queriers".into()))?;

		#[allow(clippy::single_match_else)]
		match queriers.get(selector) {
			Some(querier) => querier.get(message, callback),
			None =>
			{
				#[allow(clippy::match_wildcard_for_single_variants)]
				match self.communicator.as_ref() {
					CommunicatorImplementation::Zenoh(zenoh) => {
						zenoh.get(selector, message, callback)
					}
				}
			}
		}
	}

	fn observe(
		&self,
		selector: &str,
		message: Option<dimas_core::message_types::Message>,
	) -> Result<()> {
		let observers = self
			.observers
			.read()
			.map_err(|_| Error::ReadAccess("observers".into()))?;

		#[allow(clippy::option_if_let_else)]
		match observers.get(selector) {
			Some(observer) => observer.request(message),
			None => Err(crate::error::Error::NotImplemented.into()),
		}
	}

	fn watch(&self, _selector: &str, _message: dimas_core::message_types::Message) -> Result<()> {
		Err(crate::error::Error::NotImplemented.into())
	}
}

impl SingleCommunicator {
	/// Constructor
	/// # Errors
	pub fn new(config: &Config) -> Result<Self> {
		let zenoh = crate::zenoh::Communicator::new(config.zenoh_config())?;
		let uuid = zenoh.session().zid();
		let mode = zenoh.mode().to_string();
		let com = Self {
			uuid,
			mode,
			communicator: Arc::new(CommunicatorImplementation::Zenoh(zenoh)),
			state: OperationState::Created,
			#[cfg(feature = "unstable")]
			liveliness_subscribers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			observers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			publishers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			queriers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			responders: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
		};
		Ok(com)
	}
}
// endregion:	--- SingleCommunicator

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<SingleCommunicator>();
	}
}
