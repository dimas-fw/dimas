// Copyright © 2023 Stephan Kunz

//! Implementation of an [`Agent`]'s internal and user defined properties [`Context`].
//! Never use it directly but through the created [`Context`], which provides thread safe access.
//! A reference to this wrapper is handed into every callback function.
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
//! fn timer_callback(context: &Context<AgentProps>) -> Result<()> {
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
use crate::{
	com::{
		liveliness::{LivelinessSubscriber, LivelinessSubscriberBuilder},
		publisher::{Publisher, PublisherBuilder},
		query::{Query, QueryBuilder},
		queryable::{Queryable, QueryableBuilder},
		subscriber::{Subscriber, SubscriberBuilder},
	},
	timer::{Timer, TimerBuilder},
};
use dimas_com::communicator::Communicator;
use dimas_config::Config;
use dimas_core::{
	error::{DimasError, Result},
	message_types::{Message, Response},
	task_signal::TaskSignal,
	traits::{Capability, ContextAbstraction, OperationState},
};
use std::{
	collections::HashMap,
	fmt::Debug,
	ops::Deref,
	sync::{mpsc::Sender, Arc, RwLock},
};
use tracing::{error, info, instrument, Level};
// endregion:	--- modules

// region:		--- types
// the initial size of the HashMaps
const INITIAL_SIZE: usize = 9;
// endregion:	--- types

// region:		--- ArcContextImpl
/// `ArcContextImpl` is a thread safe atomic reference counted implementation of [`Context`].<br>
/// It makes all relevant data of the agent accessible in a thread safe way via accessor methods.
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct ArcContextImpl<P>
where
	P: Send + Sync + Unpin + 'static,
{
	inner: Arc<ContextImpl<P>>,
}

impl<P> Clone for ArcContextImpl<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn clone(&self) -> Self {
		Self {
			inner: self.inner.clone(),
		}
	}
}

impl<P> Deref for ArcContextImpl<P>
where
	P: Send + Sync + Unpin + 'static,
{
	type Target = ContextImpl<P>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<P> From<ContextImpl<P>> for ArcContextImpl<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn from(value: ContextImpl<P>) -> Self {
		Self {
			inner: Arc::new(value),
		}
	}
}

impl<P> ArcContextImpl<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Get a [`LivelinessSubscriberBuilder`], the builder for a [`LivelinessSubscriber`].
	#[must_use]
	pub fn liveliness_subscriber(
		&self,
	) -> LivelinessSubscriberBuilder<
		P,
		crate::com::liveliness::NoPutCallback,
		crate::com::liveliness::Storage<P>,
	> {
		LivelinessSubscriberBuilder::new(self.inner.clone())
			.storage(self.inner.liveliness_subscribers.clone())
	}

	/// Get a [`PublisherBuilder`], the builder for a [`Publisher`].
	#[must_use]
	pub fn publisher(
		&self,
	) -> PublisherBuilder<
		P,
		crate::com::publisher::NoKeyExpression,
		crate::com::publisher::Storage<P>,
	> {
		PublisherBuilder::new(self.inner.clone()).storage(self.inner.publishers.clone())
	}

	/// Get a [`QueryBuilder`], the builder for a [`Query`].
	#[must_use]
	pub fn query(
		&self,
	) -> QueryBuilder<
		P,
		crate::com::query::NoKeyExpression,
		crate::com::query::NoResponseCallback,
		crate::com::query::Storage<P>,
	> {
		QueryBuilder::new(self.inner.clone()).storage(self.inner.queries.clone())
	}

	/// Get a [`QueryableBuilder`], the builder for a [`Queryable`].
	#[must_use]
	pub fn queryable(
		&self,
	) -> QueryableBuilder<
		P,
		crate::com::queryable::NoKeyExpression,
		crate::com::queryable::NoRequestCallback,
		crate::com::queryable::Storage<P>,
	> {
		QueryableBuilder::new(self.inner.clone()).storage(self.inner.queryables.clone())
	}

	/// Get a [`SubscriberBuilder`], the builder for a [`Subscriber`].
	#[must_use]
	pub fn subscriber(
		&self,
	) -> SubscriberBuilder<
		P,
		crate::com::subscriber::NoKeyExpression,
		crate::com::subscriber::NoPutCallback,
		crate::com::subscriber::Storage<P>,
	> {
		SubscriberBuilder::new(self.inner.clone()).storage(self.inner.subscribers.clone())
	}

	/// Get a [`TimerBuilder`], the builder for a [`Timer`].
	#[must_use]
	pub fn timer(
		&self,
	) -> TimerBuilder<
		P,
		crate::timer::NoKeyExpression,
		crate::timer::NoInterval,
		crate::timer::NoIntervalCallback,
		crate::timer::Storage<P>,
	> {
		TimerBuilder::new(self.inner.clone()).storage(self.inner.timers.clone())
	}
}
// endregion:	--- ArcContextImpl

// region:		--- ContextImpl
/// [`ContextImpl`] makes all relevant data of the [`Agent`] accessible via accessor methods.
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct ContextImpl<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// The [`Agent`]s name.
	/// Name must not, but should be unique.
	name: Option<String>,
	/// The [`Agent`]s current operational state.
	state: Arc<RwLock<OperationState>>,
	/// A sender for sending signals to owner of context
	sender: Sender<TaskSignal>,
	/// The [`Agent`]s property structure
	props: Arc<RwLock<P>>,
	/// The [`Agent`]s [`Communicator`]
	communicator: Arc<Communicator>,
	/// Registered [`LivelinessSubscriber`]
	liveliness_subscribers: Arc<RwLock<HashMap<String, LivelinessSubscriber<P>>>>,
	/// Registered [`Publisher`]
	publishers: Arc<RwLock<HashMap<String, Publisher<P>>>>,
	/// Registered [`Query`]s
	queries: Arc<RwLock<HashMap<String, Query<P>>>>,
	/// Registered [`Queryable`]s
	queryables: Arc<RwLock<HashMap<String, Queryable<P>>>>,
	/// Registered [`Subscriber`]
	subscribers: Arc<RwLock<HashMap<String, Subscriber<P>>>>,
	/// Registered [`Timer`]
	timers: Arc<RwLock<HashMap<String, Timer<P>>>>,
}

impl<P> ContextAbstraction<P> for ContextImpl<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Get the name
	#[must_use]
	fn name(&self) -> &Option<String> {
		&self.name
	}

	#[must_use]
	fn fq_name(&self) -> Option<String> {
		if self.name().is_some() && self.prefix().is_some() {
			Some(format!(
				"{}/{}",
				self.prefix().clone().expect("snh"),
				self.name().clone().expect("snh")
			))
		} else {
			self.name().clone()
		}
	}

	#[must_use]
	fn state(&self) -> OperationState {
		let val = self.state.read().expect("snh").clone();
		val
	}

	#[must_use]
	fn uuid(&self) -> String {
		self.communicator.uuid()
	}

	#[must_use]
	fn prefix(&self) -> &Option<String> {
		self.communicator.prefix()
	}

	#[must_use]
	fn sender(&self) -> &Sender<TaskSignal> {
		&self.sender
	}

	#[must_use]
	fn mode(&self) -> &String {
		self.communicator.mode()
	}

	fn session(&self) -> Arc<zenoh::prelude::Session> {
		self.communicator.session()
	}

	fn read(&self) -> Result<std::sync::RwLockReadGuard<'_, P>> {
		self.props
			.read()
			.map_err(|_| DimasError::ReadProperties.into())
	}

	fn write(&self) -> Result<std::sync::RwLockWriteGuard<'_, P>> {
		self.props
			.write()
			.map_err(|_| DimasError::WriteProperties.into())
	}

	fn set_state(&self, state: OperationState) -> Result<()> {
		info!("changing state to {}", &state);
		let final_state = state;
		let mut next_state;
		// step up?
		while self.state() < final_state {
			match self.state() {
				OperationState::Error => {
					return Err(DimasError::ManageState.into());
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
					return Err(DimasError::ManageState.into());
				}
			}
			self.downgrade_registered_tasks(next_state)?;
		}

		Ok(())
	}

	#[instrument(level = Level::ERROR, skip_all)]
	fn put(&self, topic: &str, message: Message) -> Result<()> {
		self.communicator.put(topic, message)
	}

	#[instrument(level = Level::ERROR, skip_all)]
	#[instrument(level = Level::ERROR, skip_all)]
	fn put_with(&self, topic: &str, message: Message) -> Result<()> {
		let key_expr = self
			.prefix()
			.clone()
			.take()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		if self
			.publishers
			.read()
			.map_err(|_| DimasError::ReadContext("publishers".into()))?
			.get(&key_expr)
			.is_some()
		{
			self.publishers
				.read()
				.map_err(|_| DimasError::ReadContext("publishers".into()))?
				.get(&key_expr)
				.ok_or(DimasError::ShouldNotHappen)?
				.put(message)?;
		};
		Ok(())
	}

	#[instrument(level = Level::ERROR, skip_all)]
	fn delete(&self, topic: &str) -> Result<()> {
		self.communicator.delete(topic)
	}

	#[instrument(level = Level::ERROR, skip_all)]
	fn delete_with(&self, topic: &str) -> Result<()> {
		let key_expr = self
			.prefix()
			.clone()
			.take()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		if self
			.publishers
			.read()
			.map_err(|_| DimasError::ReadContext("publishers".into()))?
			.get(&key_expr)
			.is_some()
		{
			self.publishers
				.read()
				.map_err(|_| DimasError::ReadContext("publishers".into()))?
				.get(&key_expr)
				.ok_or(DimasError::ShouldNotHappen)?
				.delete()?;
		}
		Ok(())
	}

	fn get(&self, topic: &str, callback: Box<dyn FnMut(Response)>) -> Result<()> {
		let selector = self
			.prefix()
			.clone()
			.take()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		self.communicator.get(&selector, callback)
	}

	/// Method to query data with a stored Query
	///
	/// # Errors
	///
	#[instrument(level = Level::ERROR, skip_all)]
	fn get_with(&self, topic: &str) -> Result<()> {
		let key_expr = self
			.prefix()
			.clone()
			.take()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		if self
			.queries
			.read()
			.map_err(|_| DimasError::ReadContext("queries".into()))?
			.get(&key_expr)
			.is_some()
		{
			self.queries
				.read()
				.map_err(|_| DimasError::ReadContext("queries".into()))?
				.get(&key_expr)
				.ok_or(DimasError::ShouldNotHappen)?
				.get()?;
		};
		Ok(())
	}
}

impl<P> ContextImpl<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor for the [`ContextInner`]
	pub fn new(
		config: &Config,
		props: P,
		name: Option<String>,
		sender: Sender<TaskSignal>,
		prefix: Option<String>,
	) -> Result<Self> {
		let mut communicator = Communicator::new(config)?;
		if let Some(prefix) = prefix {
			communicator.set_prefix(prefix);
		}
		Ok(Self {
			name,
			state: Arc::new(RwLock::new(OperationState::Created)),
			sender,
			communicator: Arc::new(communicator),
			props: Arc::new(RwLock::new(props)),
			liveliness_subscribers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			publishers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			queries: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			queryables: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			subscribers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			timers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
		})
	}

	/// Set the [`Context`]s state
	/// # Errors
	fn modify_state_property(&self, state: OperationState) -> Result<()> {
		*(self
			.state
			.write()
			.map_err(|_| DimasError::ModifyContext("state".into()))?) = state;
		Ok(())
	}

	/// Get the liveliness subscribers
	#[must_use]
	pub const fn liveliness_subscribers(
		&self,
	) -> &Arc<RwLock<HashMap<String, LivelinessSubscriber<P>>>> {
		&self.liveliness_subscribers
	}

	/// Get the queryables
	#[must_use]
	pub const fn queryables(&self) -> &Arc<RwLock<HashMap<String, Queryable<P>>>> {
		&self.queryables
	}

	/// Get the subscribers
	#[must_use]
	pub const fn subscribers(&self) -> &Arc<RwLock<HashMap<String, Subscriber<P>>>> {
		&self.subscribers
	}

	/// Get the timers
	#[must_use]
	pub const fn timers(&self) -> &Arc<RwLock<HashMap<String, Timer<P>>>> {
		&self.timers
	}

	/// Internal function for starting all registered tasks.<br>
	/// The tasks are started in the order
	/// - [`Queryable`]s
	/// - [`Subscriber`]s
	/// - [`LivelinessSubscriber`]s and last
	/// - [`Timer`]s
	/// Beforehand of starting the [`Timer`]s ther is the initialisation of the
	/// - [`Publisher`]s and the
	/// - [`Query`]s
	///
	/// # Errors
	/// Currently none
	#[allow(unused_variables)]
	fn upgrade_registered_tasks(&self, new_state: OperationState) -> Result<()> {
		// start liveliness subscriber
		self.liveliness_subscribers
			.write()
			.map_err(|_| DimasError::ModifyContext("liveliness subscribers".into()))?
			.iter_mut()
			.for_each(|subscriber| {
				let _ = subscriber.1.manage_operation_state(&new_state);
			});

		// start all registered queryables
		self.queryables
			.write()
			.map_err(|_| DimasError::ModifyContext("queryables".into()))?
			.iter_mut()
			.for_each(|queryable| {
				let _ = queryable.1.manage_operation_state(&new_state);
			});

		// start all registered subscribers
		self.subscribers
			.write()
			.map_err(|_| DimasError::ModifyContext("subscribers".into()))?
			.iter_mut()
			.for_each(|subscriber| {
				let _ = subscriber.1.manage_operation_state(&new_state);
			});

		// init all registered publishers
		self.publishers
			.write()
			.map_err(|_| DimasError::ModifyContext("publishers".into()))?
			.iter_mut()
			.for_each(|publisher| {
				if let Err(reason) = publisher.1.manage_operation_state(&new_state) {
					error!(
						"could not initialize publisher for {}, reason: {}",
						publisher.1.key_expr(),
						reason
					);
				};
			});

		// init all registered queries
		self.queries
			.write()
			.map_err(|_| DimasError::ModifyContext("queries".into()))?
			.iter_mut()
			.for_each(|query| {
				if let Err(reason) = query.1.manage_operation_state(&new_state) {
					error!(
						"could not initialize query for {}, reason: {}",
						query.1.key_expr(),
						reason
					);
				};
			});

		// start all registered timers
		self.timers
			.write()
			.map_err(|_| DimasError::ModifyContext("timers".into()))?
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
			.map_err(|_| DimasError::ModifyContext("timers".into()))?
			.iter_mut()
			.for_each(|timer| {
				let _ = timer.1.manage_operation_state(&new_state);
			});

		// de-init all registered queries
		self.queries
			.write()
			.map_err(|_| DimasError::ModifyContext("queries".into()))?
			.iter_mut()
			.for_each(|query| {
				if let Err(reason) = query.1.manage_operation_state(&new_state) {
					error!(
						"could not de-initialize query for {}, reason: {}",
						query.1.key_expr(),
						reason
					);
				};
			});

		// de-init all registered publishers
		self.publishers
			.write()
			.map_err(|_| DimasError::ModifyContext("publishers".into()))?
			.iter_mut()
			.for_each(|publisher| {
				let _ = publisher.1.manage_operation_state(&new_state);
			});

		// stop all registered subscribers
		self.subscribers
			.write()
			.map_err(|_| DimasError::ModifyContext("subscribers".into()))?
			.iter_mut()
			.for_each(|subscriber| {
				let _ = subscriber.1.manage_operation_state(&new_state);
			});

		// stop all registered queryables
		self.queryables
			.write()
			.map_err(|_| DimasError::ModifyContext("queryables".into()))?
			.iter_mut()
			.for_each(|queryable| {
				let _ = queryable.1.manage_operation_state(&new_state);
			});

		// stop all registered liveliness subscribers
		self.liveliness_subscribers
			.write()
			.map_err(|_| DimasError::ModifyContext("liveliness subscribers".into()))?
			.iter_mut()
			.for_each(|subscriber| {
				let _ = subscriber.1.manage_operation_state(&new_state);
			});

		self.modify_state_property(new_state)?;
		Ok(())
	}
}
// endregion:	--- ContextImpl

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[derive(Debug)]
	struct Props {}

	#[test]
	const fn normal_types() {
		is_normal::<ArcContextImpl<Props>>();
		is_normal::<ContextImpl<Props>>();
	}
}
