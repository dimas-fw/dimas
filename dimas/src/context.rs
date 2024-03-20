// Copyright © 2023 Stephan Kunz

//! [`Context`] is the representation of an [`Agent`]'s internal and user defined properties.
//! Never use it directly but through [`ArcContext`], which provides thread safe access.
//!
//! # Examples
//! ```rust,no_run
//! # use dimas::prelude::*;
//! # #[tokio::main(flavor = "multi_thread")]
//! # async fn main() -> Result<()> {
//! # Ok(())
//! # }
//! ```
//!

// region:		--- modules
use crate::com::communicator::Communicator;
use crate::prelude::*;
use crate::utils::TaskSignal;
#[cfg(any(
	feature = "liveliness",
	feature = "publisher",
	feature = "query",
	feature = "queryable",
	feature = "subscriber",
	feature = "timer",
))]
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::mpsc::Sender;
#[cfg(any(
	feature = "publisher",
	feature = "query",
))]
use tracing::error;
use tracing::{instrument, Level};
use zenoh::publication::Publisher;
use zenoh::query::ConsolidationMode;
// endregion:	--- modules

// region:		--- types
// the initial size of the HashMaps
#[cfg(any(
	feature = "liveliness",
	feature = "publisher",
	feature = "query",
	feature = "queryable",
	feature = "subscriber",
	feature = "timer",
))]
const INITIAL_SIZE: usize = 9;
// endregion:	--- types

// region:		--- ArcContext
/// `ArcContext` is a thread safe atomic reference counted [`Context`]. <br>
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
	/// Internal function for starting all registered tasks
	/// # Errors
	/// Currently none
	#[allow(unused_variables)]
	pub(crate) fn start_registered_tasks(&self, tx: &Sender<TaskSignal>) -> Result<()> {
		// start all registered queryables
		#[cfg(feature = "queryable")]
		self.queryables
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|queryable| {
				queryable.1.start(self.clone(), tx.clone());
			});

		// start all registered subscribers
		#[cfg(feature = "subscriber")]
		self.subscribers
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|subscriber| {
				subscriber.1.start(self.clone(), tx.clone());
			});

		// start liveliness subscriber
		#[cfg(feature = "liveliness")]
		self.liveliness_subscribers
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|subscriber| {
				subscriber.1.start(self.clone(), tx.clone());
			});

		// wait a little bit before starting active part
		//tokio::time::sleep(Duration::from_millis(10)).await;

		// init all registered publishers
		#[cfg(feature = "publisher")]
		self.publishers
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|publisher| {
				if let Err(reason) = publisher.1.init(self) {
					error!(
						"could not initialize publisher for {}, reason: {}",
						publisher.1.key_expr, reason
					);
				};
			});

		// init all registered queries
		#[cfg(feature = "query")]
		self.queries
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|query| {
				if let Err(reason) = query.1.init(self) {
					error!(
						"could not initialize query for {}, reason: {}",
						query.1.key_expr, reason
					);
				};
			});

		// start all registered timers
		#[cfg(feature = "timer")]
		self.timers
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|timer| {
				timer.1.start(self.clone(), tx.clone());
			});

		Ok(())
	}

	/// Internal function for stopping all registered tasks
	/// # Errors
	/// Currently none
	pub fn stop_registered_tasks(&mut self) -> Result<()> {
		// reverse order of start!
		// stop all registered timers
		#[cfg(feature = "timer")]
		self.timers
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|timer| {
				timer.1.stop();
			});

		// de-init all registered queries
		#[cfg(feature = "query")]
		self.queries
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|query| {
				if let Err(reason) = query.1.de_init() {
					error!(
						"could not de-initialize query for {}, reason: {}",
						query.1.key_expr, reason
					);
				};
			});

		// init all registered publishers
		#[cfg(feature = "publisher")]
		self.publishers
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|publisher| {
				if let Err(reason) = publisher.1.de_init() {
					error!(
						"could not de-initialize publisher for {}, reason: {}",
						publisher.1.key_expr, reason
					);
				};
			});

		#[cfg(feature = "liveliness")]
		{
			// stop all registered liveliness subscribers
			#[cfg(feature = "liveliness")]
			self.liveliness_subscribers
				.write()
				.map_err(|_| DimasError::ShouldNotHappen)?
				.iter_mut()
				.for_each(|subscriber| {
					subscriber.1.stop();
				});
		}

		// stop all registered subscribers
		#[cfg(feature = "subscriber")]
		self.subscribers
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|subscriber| {
				subscriber.1.stop();
			});

		// stop all registered queryables
		#[cfg(feature = "queryable")]
		self.queryables
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|queryable| {
				queryable.1.stop();
			});
		Ok(())
	}

	/// Get a builder for a [`LivelinessSubscriber`]
	#[cfg(feature = "liveliness")]
	#[must_use]
	pub fn liveliness_subscriber(
		&self,
	) -> LivelinessSubscriberBuilder<
		P,
		crate::com::liveliness_subscriber::NoPutCallback,
		crate::com::liveliness_subscriber::Storage<P>,
	> {
		LivelinessSubscriberBuilder::new(self.prefix()).storage(self.liveliness_subscribers.clone())
	}
	/// Get a builder for a [`LivelinessSubscriber`]
	#[cfg(not(feature = "liveliness"))]
	#[must_use]
	pub fn liveliness_subscriber(
		&self,
	) -> LivelinessSubscriberBuilder<
		P,
		crate::com::liveliness_subscriber::NoPutCallback,
		crate::com::liveliness_subscriber::NoStorage,
	> {
		LivelinessSubscriberBuilder::new(self.prefix())
	}

	/// Get a builder for a [`Publisher`]
	#[cfg(feature = "publisher")]
	#[must_use]
	pub fn publisher(
		&self,
	) -> PublisherBuilder<crate::com::publisher::NoKeyExpression, crate::com::publisher::Storage> {
		PublisherBuilder::new(self.prefix()).storage(self.publishers.clone())
	}
	/// Get a builder for a [`Publisher`]
	#[cfg(not(feature = "publisher"))]
	#[must_use]
	pub fn publisher(
		&self,
	) -> PublisherBuilder<crate::com::publisher::NoKeyExpression, crate::com::publisher::NoStorage>
	{
		PublisherBuilder::new(self.prefix())
	}

	/// Get a builder for a [`Query`]
	#[cfg(feature = "query")]
	#[must_use]
	pub fn query(
		&self,
	) -> QueryBuilder<
		P,
		crate::com::query::NoKeyExpression,
		crate::com::query::NoResponseCallback,
		crate::com::query::Storage<P>,
	> {
		QueryBuilder::new(self.prefix()).storage(self.queries.clone())
	}
	/// Get a builder for a [`Query`]
	#[cfg(not(feature = "query"))]
	#[must_use]
	pub fn query(
		&self,
	) -> QueryBuilder<
		P,
		crate::com::query::NoKeyExpression,
		crate::com::query::NoResponseCallback,
		crate::com::query::NoStorage,
	> {
		QueryBuilder::new(self.prefix())
	}

	/// Get a builder for a [`Queryable`]
	#[cfg(feature = "queryable")]
	#[must_use]
	pub fn queryable(
		&self,
	) -> QueryableBuilder<
		P,
		crate::com::queryable::NoKeyExpression,
		crate::com::queryable::NoRequestCallback,
		crate::com::queryable::Storage<P>,
	> {
		QueryableBuilder::new(self.prefix()).storage(self.queryables.clone())
	}
	/// Get a builder for a [`Queryable`]
	#[cfg(not(feature = "queryable"))]
	#[must_use]
	pub fn queryable(
		&self,
	) -> QueryableBuilder<
		P,
		crate::com::queryable::NoKeyExpression,
		crate::com::queryable::NoRequestCallback,
		crate::com::queryable::NoStorage,
	> {
		QueryableBuilder::new(self.prefix())
	}

	/// Get a builder for a [`Subscriber`]
	#[cfg(feature = "subscriber")]
	#[must_use]
	pub fn subscriber(
		&self,
	) -> SubscriberBuilder<
		P,
		crate::com::subscriber::NoKeyExpression,
		crate::com::subscriber::NoPutCallback,
		crate::com::subscriber::Storage<P>,
	> {
		SubscriberBuilder::new(self.prefix()).storage(self.subscribers.clone())
	}
	/// Get a builder for a [`Subscriber`]
	#[cfg(not(feature = "subscriber"))]
	#[must_use]
	pub fn subscriber(
		&self,
	) -> SubscriberBuilder<
		P,
		crate::com::subscriber::NoKeyExpression,
		crate::com::subscriber::NoPutCallback,
		crate::com::subscriber::NoStorage,
	> {
		SubscriberBuilder::new(self.prefix())
	}

	/// Get a builder for a [`Timer`]
	#[cfg(feature = "timer")]
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
		TimerBuilder::new(self.prefix()).storage(self.timers.clone())
	}
	/// Get a builder for a [`Timer`]
	#[cfg(not(feature = "timer"))]
	#[must_use]
	pub fn timer(
		&self,
	) -> TimerBuilder<
		P,
		crate::timer::NoKeyExpression,
		crate::timer::NoInterval,
		crate::timer::NoIntervalCallback,
		crate::timer::NoStorage,
	> {
		TimerBuilder::new(self.prefix())
	}
}
// endregion:	--- ArcContext

// region:		--- Context
/// Context makes all relevant data of the agent accessible via accessor methods.
#[derive(Debug, Clone)]
pub struct Context<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// The agents property structure
	pub(crate) props: Arc<RwLock<P>>,
	pub(crate) communicator: Arc<Communicator>,
	// registered liveliness subscribers
	#[cfg(feature = "liveliness")]
	pub(crate) liveliness_subscribers: Arc<RwLock<HashMap<String, LivelinessSubscriber<P>>>>,
	#[cfg(feature = "publisher")]
	pub(crate) publishers: Arc<RwLock<HashMap<String, crate::com::publisher::Publisher>>>,
	#[cfg(feature = "query")]
	pub(crate) queries: Arc<RwLock<HashMap<String, crate::com::query::Query<P>>>>,
	// registered queryables
	#[cfg(feature = "queryable")]
	pub(crate) queryables: Arc<RwLock<HashMap<String, Queryable<P>>>>,
	// registered subscribers
	#[cfg(feature = "subscriber")]
	pub(crate) subscribers: Arc<RwLock<HashMap<String, Subscriber<P>>>>,
	// registered timer
	#[cfg(feature = "timer")]
	pub(crate) timers: Arc<RwLock<HashMap<String, Timer<P>>>>,
}

impl<P> Context<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor for the `Context`
	pub(crate) fn new(config: Config, props: P) -> Result<Self> {
		let communicator = Arc::new(Communicator::new(config)?);
		Ok(Self {
			communicator,
			props: Arc::new(RwLock::new(props)),
			#[cfg(feature = "liveliness")]
			liveliness_subscribers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			#[cfg(feature = "publisher")]
			publishers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			#[cfg(feature = "query")]
			queries: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			#[cfg(feature = "query")]
			queryables: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			#[cfg(feature = "subscriber")]
			subscribers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			#[cfg(feature = "timer")]
			timers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
		})
	}

	/// Constructor for the `Context` with a prefix
	pub(crate) fn new_with_prefix(
		config: Config,
		props: P,
		prefix: impl Into<String>,
	) -> Result<Self> {
		let communicator = Arc::new(Communicator::new_with_prefix(config, prefix)?);
		Ok(Self {
			communicator,
			props: Arc::new(RwLock::new(props)),
			#[cfg(feature = "liveliness")]
			liveliness_subscribers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			#[cfg(feature = "publisher")]
			publishers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			#[cfg(feature = "query")]
			queries: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			#[cfg(feature = "query")]
			queryables: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			#[cfg(feature = "subscriber")]
			subscribers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
			#[cfg(feature = "timer")]
			timers: Arc::new(RwLock::new(HashMap::with_capacity(INITIAL_SIZE))),
		})
	}

	/// Get the agents uuid
	#[must_use]
	pub fn uuid(&self) -> String {
		self.communicator.uuid()
	}

	/// Get the agents prefix
	#[must_use]
	pub fn prefix(&self) -> Option<String> {
		self.communicator.prefix()
	}

	/// Gives read access to the `Agent`s properties
	/// # Errors
	pub fn read(&self) -> Result<std::sync::RwLockReadGuard<'_, P>> {
		self.props
			.read()
			.map_err(|_| DimasError::ReadProperties.into())
	}

	/// Gives write access to the `Agent`s properties
	/// # Errors
	pub fn write(&self) -> Result<std::sync::RwLockWriteGuard<'_, P>> {
		self.props
			.write()
			.map_err(|_| DimasError::WriteProperties.into())
	}

	pub(crate) fn create_publisher<'publisher>(
		&self,
		key_expr: &str,
	) -> Result<Publisher<'publisher>> {
		self.communicator.create_publisher(key_expr)
	}

	/// Method to do an ad hoc publishing
	/// # Errors
	///   Error is propagated from [`Communicator::put`]
	#[instrument(level = Level::ERROR, skip_all)]
	pub fn put<M>(&self, topic: &str, message: M) -> Result<()>
	where
		M: Encode,
	{
		self.communicator.put(topic, message)
	}

	/// Method to pubish data with a stored Publisher
	/// # Errors
	///
	#[cfg(feature = "publisher")]
	#[instrument(level = Level::ERROR, skip_all)]
	pub fn put_with<M>(&self, topic: &str, message: M) -> Result<()>
	where
		M: Debug + Encode,
	{
		let key_expr = self
			.prefix()
			.take()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		if self
			.publishers
			.read()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.get(&key_expr)
			.is_some()
		{
			self.publishers
				.read()
				.map_err(|_| DimasError::ShouldNotHappen)?
				.get(&key_expr)
				.ok_or(DimasError::ShouldNotHappen)?
				.put(message)?;
		};
		Ok(())
	}

	/// Method to do an ad hoc deletion
	/// # Errors
	///   Error is propagated from Communicator
	#[instrument(level = Level::ERROR, skip_all)]
	pub fn delete(&self, topic: &str) -> Result<()> {
		self.communicator.delete(topic)
	}

	/// Method to delete data with a stored Publisher
	/// # Errors
	///
	#[cfg(feature = "publisher")]
	#[instrument(level = Level::ERROR, skip_all)]
	pub fn delete_with(&self, topic: &str) -> Result<()> {
		let key_expr = self
			.prefix()
			.take()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		if self
			.publishers
			.read()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.get(&key_expr)
			.is_some()
		{
			self.publishers
				.read()
				.map_err(|_| DimasError::ShouldNotHappen)?
				.get(&key_expr)
				.ok_or(DimasError::ShouldNotHappen)?
				.delete()?;
		}
		Ok(())
	}

	/// Method to do an ad hoc query without any consolidation of answers.
	/// Multiple answers may be received for the same timestamp.
	#[instrument(level = Level::ERROR, skip_all)]
	pub fn get<F>(&self, ctx: ArcContext<P>, query_name: &str, callback: F) -> Result<()>
	where
		P: Send + Sync + Unpin + 'static,
		F: Fn(&ArcContext<P>, Message) + Send + Sync + Unpin + 'static,
	{
		self.communicator
			.get(ctx, query_name, ConsolidationMode::None, callback)
	}

	/// Method to query data with a stored Query
	/// # Errors
	///
	#[cfg(feature = "query")]
	#[instrument(level = Level::ERROR, skip_all)]
	pub fn get_with(&self, topic: &str) -> Result<()> {
		let key_expr = self
			.prefix()
			.take()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		if self
			.queries
			.read()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.get(&key_expr)
			.is_some()
		{
			self.queries
				.read()
				.map_err(|_| DimasError::ShouldNotHappen)?
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
