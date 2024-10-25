// Copyright © 2023 Stephan Kunz
#![allow(unused_imports)]
//! Implementation of an [`Agent`]'s internal and user defined properties [`ContextImpl`].
//!
//! Never use it directly but through the type [`Context`], which provides thread safe access.
//! A [`Context`] is handed into every callback function.
//!
//! # Examples
//! ```rust,no_run
//! # use dimas::prelude::*;
//! // The [`Agent`]s properties
//! #[derive(Debug)]
//! struct AgentProps {
//!   counter: i32,
//! }
//! // A [`Timer`] callback
//! fn timer_callback(context: Context<AgentProps>) -> Result<()> {
//!   // reading properties
//!   let mut value = context.read()?.counter;
//!   value +=1;
//!   // writing properties
//!   context.write()?.counter = value;
//!   Ok(())
//! }
//! # #[tokio::main(flavor = "multi_thread")]
//! # async fn main() -> Result<()> {
//! # Ok(())
//! # }
//! ```
//!

// region:		--- modules
// only for doc needed
#[cfg(doc)]
use crate::agent::Agent;
use crate::error::Error;
use core::fmt::Debug;
use dimas_com::multi_communicator::MultiCommunicator;
#[cfg(feature = "unstable")]
use dimas_com::traits::LivelinessSubscriber;
use dimas_com::traits::{
	Communicator, MultiSessionCommunicator, MultiSessionCommunicatorMethods, Observer, Publisher, Querier,
	Responder,
};
use dimas_config::Config;
#[cfg(doc)]
use dimas_core::traits::Context;
use dimas_core::{
	enums::{OperationState, TaskSignal},
	message_types::{Message, QueryableMsg},
	traits::{Capability, ContextAbstraction},
	Result,
};
use dimas_time::Timer;
use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
};
use tokio::sync::mpsc::Sender;
use tracing::{info, instrument, Level};
use zenoh::Session;
// endregion:	--- modules

// region:		--- types
// the initial size of the HashMaps
const INITIAL_SIZE: usize = 9;
// endregion:	--- types

// region:		--- ContextImpl
/// [`ContextImpl`] makes all relevant data of the [`Agent`] accessible via accessor methods.
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct ContextImpl<P>
where
	P: Send + Sync + 'static,
{
	/// The [`Agent`]s uuid
	uuid: String,
	/// The [`Agent`]s name
	/// Name must not, but should be unique.
	name: Option<String>,
	/// A prefix to separate communication for different groups
	prefix: Option<String>,
	/// The [`Agent`]s current operational state
	state: Arc<RwLock<OperationState>>,
	/// A sender for sending signals to owner of context
	sender: Sender<TaskSignal>,
	/// The [`Agent`]s property structure
	props: Arc<RwLock<P>>,
	/// The [`Agent`]s [`Communicator`]
	communicator: Arc<MultiCommunicator>,
	/// Registered [`Timer`]
	timers: Arc<RwLock<HashMap<String, Timer<P>>>>,
}

impl<P> ContextAbstraction for ContextImpl<P>
where
	P: Debug + Send + Sync + 'static,
{
	type Props = P;
	/// Get the name
	#[must_use]
	fn name(&self) -> Option<&String> {
		self.name.as_ref()
	}

	#[must_use]
	fn fq_name(&self) -> Option<String> {
		if self.name().is_some() && self.prefix().is_some() {
			Some(format!(
				"{}/{}",
				self.prefix().expect("snh"),
				self.name().expect("snh")
			))
		} else if self.name().is_some() {
			Some(self.name().expect("snh").to_owned())
		} else {
			None
		}
	}

	#[must_use]
	fn state(&self) -> OperationState {
		self.state.read().expect("snh").clone()
	}

	#[must_use]
	fn uuid(&self) -> String {
		self.uuid.clone()
	}

	#[must_use]
	fn prefix(&self) -> Option<&String> {
		self.prefix.as_ref()
	}

	#[must_use]
	fn sender(&self) -> &Sender<TaskSignal> {
		&self.sender
	}

	fn read(&self) -> Result<std::sync::RwLockReadGuard<'_, P>> {
		self.props
			.read()
			.map_err(|_| Error::ReadAccess.into())
	}

	fn write(&self) -> Result<std::sync::RwLockWriteGuard<'_, P>> {
		self.props
			.write()
			.map_err(|_| Error::WriteAccess.into())
	}

	fn set_state(&self, state: OperationState) -> Result<()> {
		info!("changing state to {}", &state);
		let final_state = state;
		let mut next_state;
		// step up?
		while self.state() < final_state {
			match self.state() {
				OperationState::Error => {
					return Err(Error::ManageState.into());
				}
				OperationState::Created => {
					next_state = OperationState::Configured;
				}
				OperationState::Configured => {
					next_state = OperationState::Inactive;
				}
				OperationState::Inactive => {
					next_state = OperationState::Standby;
				}
				OperationState::Standby => {
					next_state = OperationState::Active;
				}
				OperationState::Active => {
					return self.modify_state_property(OperationState::Error);
				}
			}
			self.upgrade_registered_tasks(next_state)?;
		}

		// step down?
		while self.state() > final_state {
			match self.state() {
				OperationState::Active => {
					next_state = OperationState::Standby;
				}
				OperationState::Standby => {
					next_state = OperationState::Inactive;
				}
				OperationState::Inactive => {
					next_state = OperationState::Configured;
				}
				OperationState::Configured => {
					next_state = OperationState::Created;
				}
				OperationState::Created => {
					return self.modify_state_property(OperationState::Error);
				}
				OperationState::Error => {
					return Err(Error::ManageState.into());
				}
			}
			self.downgrade_registered_tasks(next_state)?;
		}

		Ok(())
	}

	#[instrument(level = Level::ERROR, skip_all)]
	fn put_with(&self, selector: &str, message: Message) -> Result<()> {
		if self
			.publishers()
			.read()
			.map_err(|_| Error::ReadContext("publishers".into()))?
			.get(selector)
			.is_some()
		{
			self.publishers()
				.read()
				.map_err(|_| Error::ReadContext("publishers".into()))?
				.get(selector)
				.ok_or_else(|| Error::Get("publishers".into()))?
				.put(message)?;
		} else {
			todo!(); //self.communicator.put(selector, message)?;
		};
		Ok(())
	}

	#[instrument(level = Level::ERROR, skip_all)]
	fn delete_with(&self, selector: &str) -> Result<()> {
		if self
			.publishers()
			.read()
			.map_err(|_| Error::ReadContext("publishers".into()))?
			.get(selector)
			.is_some()
		{
			self.publishers()
				.read()
				.map_err(|_| Error::ReadContext("publishers".into()))?
				.get(selector)
				.ok_or_else(|| Error::Get("publishers".into()))?
				.delete()?;
		} else {
			todo!(); //self.communicator.delete(selector)?;
		}
		Ok(())
	}

	#[instrument(level = Level::ERROR, skip_all)]
	fn get_with(
		&self,
		selector: &str,
		message: Option<Message>,
		callback: Option<&mut dyn FnMut(QueryableMsg) -> Result<()>>,
	) -> Result<()> {
		if self
			.queriers()
			.read()
			.map_err(|_| Error::ReadContext("queries".into()))?
			.get(selector)
			.is_some()
		{
			self.queriers()
				.read()
				.map_err(|_| Error::ReadContext("queries".into()))?
				.get(selector)
				.ok_or_else(|| Error::Get("queries".into()))?
				.get(message, callback)?;
		} else {
			self.communicator
				.get(selector, message, callback)?;
		};
		Ok(())
	}

	#[instrument(level = Level::ERROR, skip_all)]
	fn observe_with(&self, selector: &str, message: Option<Message>) -> Result<()> {
		self.observers()
			.read()
			.map_err(|_| Error::ReadContext("observers".into()))?
			.get(selector)
			.ok_or_else(|| Error::Get("observers".into()))?
			.request(message)?;
		Ok(())
	}

	#[instrument(level = Level::ERROR, skip_all)]
	fn cancel_observe_with(&self, selector: &str) -> Result<()> {
		self.observers()
			.read()
			.map_err(|_| Error::ReadContext("observers".into()))?
			.get(selector)
			.ok_or_else(|| Error::Get("observers".into()))?
			.cancel()?;
		Ok(())
	}

	fn mode(&self) -> &String {
		self.communicator.mode()
	}

	fn session(&self, session_id: &str) -> Option<Arc<Session>> {
		self.communicator.session(session_id)
	}
}

impl<P> ContextImpl<P>
where
	P: Send + Sync + 'static,
{
	/// Constructor for the [`ContextImpl`]
	/// # Errors
	pub fn new(
		config: &Config,
		props: P,
		name: Option<String>,
		sender: Sender<TaskSignal>,
		prefix: Option<String>,
	) -> Result<Self> {
		let communicator = MultiCommunicator::new(config)?;
		let uuid = communicator.uuid();
		Ok(Self {
			uuid,
			name,
			prefix,
			state: Arc::new(RwLock::new(OperationState::Created)),
			sender,
			communicator: Arc::new(communicator),
			props: Arc::new(RwLock::new(props)),
			timers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
		})
	}

	/// Set the [`Context`]s state
	/// # Errors
	fn modify_state_property(&self, state: OperationState) -> Result<()> {
		*(self
			.state
			.write()
			.map_err(|_| Error::ModifyStruct("state".into()))?) = state;
		Ok(())
	}

	/// Get the liveliness subscribers
	#[cfg(feature = "unstable")]
	#[must_use]
	pub fn liveliness_subscribers(
		&self,
	) -> &Arc<RwLock<HashMap<String, Box<dyn LivelinessSubscriber>>>> {
		self.communicator.liveliness_subscribers()
	}

	/// Get the observers
	#[must_use]
	pub fn observers(&self) -> &Arc<RwLock<HashMap<String, Box<dyn Observer>>>> {
		self.communicator.observers()
	}

	/// Get the publishers
	#[must_use]
	pub fn publishers(&self) -> Arc<RwLock<HashMap<String, Box<dyn Publisher>>>> {
		self.communicator.publishers()
	}

	/// Get the queries
	#[must_use]
	pub fn queriers(&self) -> &Arc<RwLock<HashMap<String, Box<dyn Querier>>>> {
		self.communicator.queriers()
	}

	/// Get the responders
	#[must_use]
	pub fn responders(&self) -> &Arc<RwLock<HashMap<String, Box<dyn Responder>>>> {
		self.communicator.responders()
	}

	/// Get the timers
	#[must_use]
	pub const fn timers(&self) -> &Arc<RwLock<HashMap<String, Timer<P>>>> {
		&self.timers
	}

	/// Internal function for starting all registere)d tasks.<br>
	/// The tasks are started in the order
	/// - [`LivelinessSubscriber`]s
	/// - [`Queryable`]s
	/// - [`Observable`]s
	/// - [`Subscriber`]s  and last
	/// - [`Timer`]s
	///
	/// Beforehand of starting the [`Timer`]s there is the initialisation of the
	/// - [`Publisher`]s the
	/// - [`Observer`]s and the
	/// - [`Query`]s
	///
	/// # Errors
	/// Currently none
	fn upgrade_registered_tasks(&self, new_state: OperationState) -> Result<()> {
		// start communication
		self.communicator
			.manage_operation_state(&new_state)?;

		// start all registered timers
		self.timers
			.write()
			.map_err(|_| Error::ModifyStruct("timers".into()))?
			.iter_mut()
			.for_each(|timer| {
				let _ = timer.1.manage_operation_state(&new_state);
			});

		self.modify_state_property(new_state)?;
		Ok(())
	}

	/// Internal function for stopping all registered tasks.<br>
	/// The tasks are stopped in reverse order of their start in [`Context::start_registered_tasks()`]
	///
	/// # Errors
	/// Currently none
	fn downgrade_registered_tasks(&self, new_state: OperationState) -> Result<()> {
		// reverse order of start!
		// stop all registered timers
		self.timers
			.write()
			.map_err(|_| Error::ModifyStruct("timers".into()))?
			.iter_mut()
			.for_each(|timer| {
				let _ = timer.1.manage_operation_state(&new_state);
			});

		// start communication
		self.communicator
			.manage_operation_state(&new_state)?;

		self.modify_state_property(new_state)?;
		Ok(())
	}
}
// endregion:	--- ContextImpl

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[derive(Debug)]
	struct Props {}

	#[test]
	const fn normal_types() {
		is_normal::<ContextImpl<Props>>();
	}
}
