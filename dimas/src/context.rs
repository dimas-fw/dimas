// Copyright © 2023 Stephan Kunz

//! [`Context`] is the representation of an [`Agent`]'s internal and user defined properties.
//! Never use it directly but through the created [`ArcContext`], which provides thread safe access.
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
//! fn timer_callback(context: &ArcContext<AgentProps>) -> Result<()> {
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
		task_signal::TaskSignal,
	},
	timer::{Timer, TimerBuilder},
};
use bitcode::Encode;
use dimas_com::{communicator::Communicator, Message};
use dimas_config::Config;
use dimas_core::{
	error::{DimasError, Result},
	traits::{ManageState, OperationState},
};
use std::{
	collections::HashMap,
	fmt::Debug,
	ops::Deref,
	sync::{mpsc::Sender, Arc, RwLock},
};
use tracing::{error, info, instrument, Level};
use zenoh::{
	prelude::{sync::SyncResolve, SampleKind},
	query::ConsolidationMode,
};
// endregion:	--- modules

// region:		--- types
// the initial size of the HashMaps
const INITIAL_SIZE: usize = 9;
// endregion:	--- types

// region:		--- ArcContext
/// `ArcContext` is a thread safe atomic reference counted [`Context`].<br>
/// It makes all relevant data of the agent accessible in a thread safe way via accessor methods.
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct ArcContext<P>
where
	P: Send + Sync + Unpin + 'static,
{
	inner: Arc<Context<P>>,
}

impl<P> Clone for ArcContext<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn clone(&self) -> Self {
		Self {
			inner: self.inner.clone(),
		}
	}
}

impl<P> Deref for ArcContext<P>
where
	P: Send + Sync + Unpin + 'static,
{
	type Target = Context<P>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<P> From<Context<P>> for ArcContext<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn from(value: Context<P>) -> Self {
		Self {
			inner: Arc::new(value),
		}
	}
}

impl<P> ArcContext<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Get the [`Context`]s state
	/// # Panics
	#[must_use]
	pub fn state(&self) -> OperationState {
		let val = self.state.read().expect("snh").clone();
		val
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

	/// Set the [`OperationState`].<br>
	/// Setting new state is done step by step
	/// # Errors
	pub fn set_state(&self, state: OperationState) -> Result<()> {
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
		self.inner
			.liveliness_subscribers
			.write()
			.map_err(|_| DimasError::ModifyContext("liveliness subscribers".into()))?
			.iter_mut()
			.for_each(|subscriber| {
				let _ = subscriber.1.manage_state(&new_state);
			});

		// start all registered queryables
		self.inner
			.queryables
			.write()
			.map_err(|_| DimasError::ModifyContext("queryables".into()))?
			.iter_mut()
			.for_each(|queryable| {
				let _ = queryable.1.manage_state(&new_state);
			});

		// start all registered subscribers
		self.inner
			.subscribers
			.write()
			.map_err(|_| DimasError::ModifyContext("subscribers".into()))?
			.iter_mut()
			.for_each(|subscriber| {
				let _ = subscriber.1.manage_state(&new_state);
			});

		// init all registered publishers
		self.inner
			.publishers
			.write()
			.map_err(|_| DimasError::ModifyContext("publishers".into()))?
			.iter_mut()
			.for_each(|publisher| {
				if let Err(reason) = publisher.1.manage_state(&new_state) {
					error!(
						"could not initialize publisher for {}, reason: {}",
						publisher.1.key_expr(),
						reason
					);
				};
			});

		// init all registered queries
		self.inner
			.queries
			.write()
			.map_err(|_| DimasError::ModifyContext("queries".into()))?
			.iter_mut()
			.for_each(|query| {
				if let Err(reason) = query.1.manage_state(&new_state) {
					error!(
						"could not initialize query for {}, reason: {}",
						query.1.key_expr(),
						reason
					);
				};
			});

		// start all registered timers
		self.inner
			.timers
			.write()
			.map_err(|_| DimasError::ModifyContext("timers".into()))?
			.iter_mut()
			.for_each(|timer| {
				let _ = timer.1.manage_state(&new_state);
			});

		self.modify_state_property(new_state)?;
		Ok(())
	}

	/// Internal function for stopping all registered tasks.<br>
	/// The tasks are stopped in reverse order of their start in [`ArcContext::start_registered_tasks()`]
	///
	/// # Errors
	/// Currently none
	fn downgrade_registered_tasks(&self, new_state: OperationState) -> Result<()> {
		// reverse order of start!
		// stop all registered timers
		self.inner
			.timers
			.write()
			.map_err(|_| DimasError::ModifyContext("timers".into()))?
			.iter_mut()
			.for_each(|timer| {
				let _ = timer.1.manage_state(&new_state);
			});

		// de-init all registered queries
		self.inner
			.queries
			.write()
			.map_err(|_| DimasError::ModifyContext("queries".into()))?
			.iter_mut()
			.for_each(|query| {
				if let Err(reason) = query.1.manage_state(&new_state) {
					error!(
						"could not de-initialize query for {}, reason: {}",
						query.1.key_expr(),
						reason
					);
				};
			});

		// de-init all registered publishers
		self.inner
			.publishers
			.write()
			.map_err(|_| DimasError::ModifyContext("publishers".into()))?
			.iter_mut()
			.for_each(|publisher| {
				let _ = publisher.1.manage_state(&new_state);
			});

		// stop all registered subscribers
		self.inner
			.subscribers
			.write()
			.map_err(|_| DimasError::ModifyContext("subscribers".into()))?
			.iter_mut()
			.for_each(|subscriber| {
				let _ = subscriber.1.manage_state(&new_state);
			});

		// stop all registered queryables
		self.inner
			.queryables
			.write()
			.map_err(|_| DimasError::ModifyContext("queryables".into()))?
			.iter_mut()
			.for_each(|queryable| {
				let _ = queryable.1.manage_state(&new_state);
			});

		// stop all registered liveliness subscribers
		self.inner
			.liveliness_subscribers
			.write()
			.map_err(|_| DimasError::ModifyContext("liveliness subscribers".into()))?
			.iter_mut()
			.for_each(|subscriber| {
				let _ = subscriber.1.manage_state(&new_state);
			});

		self.modify_state_property(new_state)?;
		Ok(())
	}

	/// Get a [`LivelinessSubscriberBuilder`], the builder for a [`LivelinessSubscriber`].
	#[must_use]
	pub fn liveliness_subscriber(
		&self,
	) -> LivelinessSubscriberBuilder<
		P,
		crate::com::liveliness::NoPutCallback,
		crate::com::liveliness::Storage<P>,
	> {
		LivelinessSubscriberBuilder::new(self.clone()).storage(self.liveliness_subscribers.clone())
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
		PublisherBuilder::new(self.clone()).storage(self.publishers.clone())
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
		QueryBuilder::new(self.clone()).storage(self.queries.clone())
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
		QueryableBuilder::new(self.clone()).storage(self.queryables.clone())
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
		SubscriberBuilder::new(self.clone()).storage(self.subscribers.clone())
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
		TimerBuilder::new(self.clone()).storage(self.timers.clone())
	}
}
// endregion:	--- ArcContext

// region:		--- Context
/// [`Context`] makes all relevant data of the [`Agent`] accessible via accessor methods.
#[derive(Debug, Clone)]
pub struct Context<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// The [`Agent`]s name.
	/// Name must not, but should be unique.
	name: Option<String>,
	/// The [`Agent`]s current operational state.
	state: Arc<RwLock<OperationState>>,
	/// a sender for sending signals to owner of context
	pub(crate) tx: Sender<TaskSignal>,
	/// The [`Agent`]s property structure
	props: Arc<RwLock<P>>,
	/// The [`Agent`]s [`Communicator`]
	pub(crate) communicator: Arc<Communicator>,
	/// Registered [`LivelinessSubscriber`]
	pub(crate) liveliness_subscribers: Arc<RwLock<HashMap<String, LivelinessSubscriber<P>>>>,
	/// Registered [`Publisher`]
	publishers: Arc<RwLock<HashMap<String, Publisher<P>>>>,
	/// Registered [`Query`]s
	queries: Arc<RwLock<HashMap<String, Query<P>>>>,
	/// Registered [`Queryable`]s
	pub(crate) queryables: Arc<RwLock<HashMap<String, Queryable<P>>>>,
	/// Registered [`Subscriber`]
	pub(crate) subscribers: Arc<RwLock<HashMap<String, Subscriber<P>>>>,
	/// Registered [`Timer`]
	pub(crate) timers: Arc<RwLock<HashMap<String, Timer<P>>>>,
}

impl<P> Context<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor for the [`Context`]
	pub(crate) fn new(
		config: &Config,
		props: P,
		name: Option<String>,
		tx: Sender<TaskSignal>,
		prefix: Option<String>,
	) -> Result<Self> {
		let mut communicator = Communicator::new(config)?;
		if let Some(prefix) = prefix {
			communicator.set_prefix(prefix);
		}
		Ok(Self {
			name,
			state: Arc::new(RwLock::new(OperationState::Created)),
			tx,
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

	/// Get the [`Agent`]s uuid
	#[must_use]
	pub fn uuid(&self) -> String {
		self.communicator.uuid()
	}

	/// Get the [`Agent`]s name
	#[must_use]
	pub const fn name(&self) -> &Option<String> {
		&self.name
	}

	/// Get the [`Agent`]s fully qualified name
	#[must_use]
	pub fn fq_name(&self) -> Option<String> {
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

	/// Get the [`Agent`]s prefix
	#[must_use]
	pub fn prefix(&self) -> &Option<String> {
		self.communicator.prefix()
	}

	/// Gives read access to the [`Agent`]s properties
	///
	/// # Errors
	pub fn read(&self) -> Result<std::sync::RwLockReadGuard<'_, P>> {
		self.props
			.read()
			.map_err(|_| DimasError::ReadProperties.into())
	}

	/// Gives write access to the [`Agent`]s properties
	///
	/// # Errors
	pub fn write(&self) -> Result<std::sync::RwLockWriteGuard<'_, P>> {
		self.props
			.write()
			.map_err(|_| DimasError::WriteProperties.into())
	}

	/// Method to do an ad hoc publishing for a `topic`
	///
	/// # Errors
	///   Error is propagated from [`Communicator::put()`]
	#[instrument(level = Level::ERROR, skip_all)]
	pub fn put<M>(&self, topic: &str, message: M) -> Result<()>
	where
		M: Encode,
	{
		self.communicator.put(topic, message)
	}

	/// Method to publish data with a stored [`Publisher`]
	///
	/// # Errors
	///
	#[instrument(level = Level::ERROR, skip_all)]
	pub fn put_with<M>(&self, topic: &str, message: M) -> Result<()>
	where
		M: Debug + Encode,
	{
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

	/// Method to do an ad hoc deletion for the `topic`
	///
	/// # Errors
	///   Error is propagated from [`Communicator::delete()`]
	#[instrument(level = Level::ERROR, skip_all)]
	pub fn delete(&self, topic: &str) -> Result<()> {
		self.communicator.delete(topic)
	}

	/// Method to delete data with a stored [`Publisher`]
	///
	/// # Errors
	///
	#[instrument(level = Level::ERROR, skip_all)]
	pub fn delete_with(&self, topic: &str) -> Result<()> {
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

	/// Send an ad hoc query using the given `topic`.
	/// The `topic` will be enhanced with the group prefix.
	/// Response will be handled by `callback`, a closure or function with
	/// signature Fn(&[`ArcContext`]<AgentProperties>, [`Response`]).
	/// # Errors
	///
	pub fn get<F>(
		&self,
		ctx: ArcContext<P>,
		topic: &str,
		mode: ConsolidationMode,
		callback: F,
	) -> Result<()>
	where
		P: Send + Sync + Unpin + 'static,
		F: Fn(&ArcContext<P>, Message) + Send + Sync + Unpin + 'static,
	{
		let key_expr = self
			.prefix()
			.clone()
			.take()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		let ctx = ctx;
		let session = self.communicator.session();

		let replies = session
			.get(&key_expr)
			.consolidation(mode)
			//.timeout(Duration::from_millis(1000))
			.res_sync()
			.map_err(|_| DimasError::Get)?;

		while let Ok(reply) = replies.recv() {
			match reply.sample {
				Ok(sample) => match sample.kind {
					SampleKind::Put => {
						let msg = Message(sample);
						callback(&ctx, msg);
					}
					SampleKind::Delete => {
						println!("Delete in Query");
					}
				},
				Err(err) => error!(">> query receive error: {err})"),
			}
		}
		Ok(())
	}

	/// Method to query data with a stored Query
	///
	/// # Errors
	///
	#[instrument(level = Level::ERROR, skip_all)]
	pub fn get_with(&self, topic: &str) -> Result<()> {
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
// endregion:	--- Context

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[derive(Debug)]
	struct Props {}

	#[test]
	const fn normal_types() {
		is_normal::<Context<Props>>();
	}
}
