// Copyright © 2023 Stephan Kunz

// region:		--- modules
#[cfg(feature = "liveliness")]
use crate::com::liveliness_subscriber::{LivelinessSubscriber, LivelinessSubscriberBuilder};
#[cfg(feature = "publisher")]
use crate::com::publisher::{Publisher, PublisherBuilder};
#[cfg(feature = "query")]
use crate::com::query::{Query, QueryBuilder};
#[cfg(feature = "queryable")]
use crate::com::queryable::{Queryable, QueryableBuilder};
#[cfg(feature = "subscriber")]
use crate::com::subscriber::{Subscriber, SubscriberBuilder};
#[cfg(feature = "timer")]
use crate::timer::{Timer, TimerBuilder};
use crate::{com::communicator::Communicator, context::Context};
#[cfg(any(
	feature = "publisher",
	feature = "query",
	feature = "queryable",
	feature = "subscriber",
	feature = "timer"
))]
use std::collections::HashMap;
use std::{
	fmt::Debug,
	marker::PhantomData,
	ops::Deref,
	sync::{Arc, RwLock},
	time::Duration,
};
use tokio::signal;
#[cfg(feature = "liveliness")]
use zenoh::liveliness::LivelinessToken;
// endregion:	--- modules

// region:		--- Agent
/// Agent
pub struct Agent<'a, P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	com: Arc<Communicator>,
	pd: PhantomData<&'a P>,
	// The agents property structure
	props: Arc<RwLock<P>>,
	#[cfg(feature = "liveliness")]
	liveliness: bool,
	// an optional liveliness token
	#[cfg(feature = "liveliness")]
	liveliness_token: RwLock<Option<LivelinessToken<'a>>>,
	// an optional liveliness subscriber
	#[cfg(feature = "liveliness")]
	liveliness_subscriber: Arc<RwLock<Option<LivelinessSubscriber<P>>>>,
	// registered subscribers
	#[cfg(feature = "subscriber")]
	subscribers: Arc<RwLock<HashMap<String, Subscriber<P>>>>,
	// registered queryables
	#[cfg(feature = "queryable")]
	queryables: Arc<RwLock<HashMap<String, Queryable<P>>>>,
	// registered publisher
	#[cfg(feature = "publisher")]
	publishers: Arc<RwLock<HashMap<String, Publisher>>>,
	// registered queries
	#[cfg(feature = "query")]
	queries: Arc<RwLock<HashMap<String, Query<P>>>>,
	// registered timer
	#[cfg(feature = "timer")]
	timers: Arc<RwLock<HashMap<String, Timer<P>>>>,
}

impl<'a, P> Debug for Agent<'a, P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Agent")
			.field("id", &self.uuid())
			.field("prefix", &self.com.prefix)
			//.field("com", &self.com)
			//.field("pd", &self.pd)
			//.field("props", &self.props)
			//.field("liveliness", &self.liveliness)
			//.field("liveliness_token", &self.liveliness_token)
			//.field("liveliness_subscriber", &self.liveliness_subscriber)
			//.field("subscribers", &self.subscribers)
			//.field("queryables", &self.queryables)
			//.field("publishers", &self.publishers)
			//.field("queries", &self.queries)
			//.field("timers", &self.timers)
			.finish_non_exhaustive()
	}
}

impl<'a, P> Deref for Agent<'a, P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	type Target = Arc<RwLock<P>>;

	fn deref(&self) -> &Self::Target {
		&self.props
	}
}

impl<'a, P> Agent<'a, P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// Create an instance of an agent.
	pub fn new(config: crate::config::Config, properties: P) -> Self {
		let com = Arc::new(Communicator::new(config));
		let pd = PhantomData {};
		Self {
			pd,
			com,
			props: Arc::new(RwLock::new(properties)),
			#[cfg(feature = "liveliness")]
			liveliness: false,
			#[cfg(feature = "liveliness")]
			liveliness_token: RwLock::new(None),
			#[cfg(feature = "liveliness")]
			liveliness_subscriber: Arc::new(RwLock::new(None)),
			#[cfg(feature = "subscriber")]
			subscribers: Arc::new(RwLock::new(HashMap::new())),
			#[cfg(feature = "queryable")]
			queryables: Arc::new(RwLock::new(HashMap::new())),
			#[cfg(feature = "publisher")]
			publishers: Arc::new(RwLock::new(HashMap::new())),
			#[cfg(feature = "query")]
			queries: Arc::new(RwLock::new(HashMap::new())),
			#[cfg(feature = "timer")]
			timers: Arc::new(RwLock::new(HashMap::new())),
		}
	}

	/// Create an instance of an agent with a standard prefix for the topics.
	pub fn new_with_prefix(
		config: crate::config::Config,
		properties: P,
		prefix: impl Into<String>,
	) -> Self {
		let com = Arc::new(Communicator::new_with_prefix(config, prefix));
		let pd = PhantomData {};
		Self {
			pd,
			com,
			props: Arc::new(RwLock::new(properties)),
			#[cfg(feature = "liveliness")]
			liveliness: false,
			#[cfg(feature = "liveliness")]
			liveliness_token: RwLock::new(None),
			#[cfg(feature = "liveliness")]
			liveliness_subscriber: Arc::new(RwLock::new(None)),
			#[cfg(feature = "subscriber")]
			subscribers: Arc::new(RwLock::new(HashMap::new())),
			#[cfg(feature = "queryable")]
			queryables: Arc::new(RwLock::new(HashMap::new())),
			#[cfg(feature = "publisher")]
			publishers: Arc::new(RwLock::new(HashMap::new())),
			#[cfg(feature = "query")]
			queries: Arc::new(RwLock::new(HashMap::new())),
			#[cfg(feature = "timer")]
			timers: Arc::new(RwLock::new(HashMap::new())),
		}
	}

	/// get the agents uuid
	#[must_use]
	pub fn uuid(&self) -> String {
		self.com.uuid()
	}

	/// get the agents properties
	#[must_use]
	pub fn props(&self) -> Arc<RwLock<P>> {
		self.props.clone()
	}

	//#[cfg_attr(doc, doc(cfg(feature = "liveliness")))]
	/// activate sending liveliness information
	#[cfg(feature = "liveliness")]
	pub fn liveliness(&mut self, activate: bool) {
		self.liveliness = activate;
	}

	fn get_context(&self) -> Arc<Context<P>> {
		Arc::new(Context {
			communicator: self.com.clone(),
			#[cfg(feature = "publisher")]
			publishers: self.publishers.clone(),
			#[cfg(feature = "query")]
			queries: self.queries.clone(),
		})
	}

	//#[cfg_attr(doc, doc(cfg(feature = "liveliness")))]
	/// get a builder for a subscriber for the liveliness information
	#[cfg(feature = "liveliness")]
	#[must_use]
	pub fn liveliness_subscriber(&self) -> LivelinessSubscriberBuilder<P> {
		LivelinessSubscriberBuilder {
			subscriber: self.liveliness_subscriber.clone(),
			context: self.get_context(),
			props: self.props.clone(),
			key_expr: None,
			put_callback: None,
			delete_callback: None,
		}
	}

	//#[cfg_attr(doc, doc(cfg(feature = "subscriber")))]
	/// get a builder for a Subscriber
	#[cfg(feature = "subscriber")]
	#[must_use]
	pub fn subscriber(&self) -> SubscriberBuilder<P> {
		SubscriberBuilder {
			collection: self.subscribers.clone(),
			context: self.get_context(),
			props: self.props.clone(),
			key_expr: None,
			put_callback: None,
			delete_callback: None,
		}
	}

	//#[cfg_attr(doc, doc(cfg(feature = "queryable")))]
	/// get a builder for a Queryable
	#[cfg(feature = "queryable")]
	#[must_use]
	pub fn queryable(&self) -> QueryableBuilder<P> {
		QueryableBuilder {
			collection: self.queryables.clone(),
			context: self.get_context(),
			props: self.props.clone(),
			key_expr: None,
			callback: None,
		}
	}

	//#[cfg_attr(doc, doc(cfg(feature = "publisher")))]
	/// get a builder for a Publisher
	#[cfg(feature = "publisher")]
	#[must_use]
	pub fn publisher(&self) -> PublisherBuilder<P> {
		PublisherBuilder {
			collection: self.publishers.clone(),
			context: self.get_context(),
			props: self.props.clone(),
			key_expr: None,
		}
	}

	//#[cfg_attr(doc, doc(cfg(feature = "query")))]
	/// get a builder for a Query
	#[cfg(feature = "query")]
	#[must_use]
	pub fn query(&self) -> QueryBuilder<P> {
		QueryBuilder {
			collection: self.queries.clone(),
			context: self.get_context(),
			props: self.props.clone(),
			key_expr: None,
			mode: None,
			callback: None,
		}
	}

	//#[cfg_attr(doc, doc(cfg(feature = "timer")))]
	/// get a builder for a Timer
	#[cfg(feature = "timer")]
	#[must_use]
	pub fn timer(&self) -> TimerBuilder<P> {
		TimerBuilder {
			collection: self.timers.clone(),
			props: self.props.clone(),
			context: self.get_context(),
			name: None,
			delay: None,
			interval: None,
			callback: None,
		}
	}

	/// start the agent
	/// # Panics
	///
	#[tracing::instrument]
	pub async fn start(&mut self) {
		// start all registered queryables
		#[cfg(feature = "queryable")]
		self.queryables
			.write()
			.expect("should never happen")
			.iter_mut()
			.for_each(|queryable| {
				queryable.1.start();
			});
		// start all registered subscribers
		#[cfg(feature = "subscriber")]
		self.subscribers
			.write()
			.expect("should never happen")
			.iter_mut()
			.for_each(|subscriber| {
				subscriber.1.start();
			});
		// start liveliness subscriber
		#[cfg(feature = "liveliness")]
		if self
			.liveliness_subscriber
			.read()
			.expect("should never happen")
			.is_some()
		{
			self.liveliness_subscriber
				.write()
				.expect("should never happen")
				.as_mut()
				.expect("should never happen")
				.start();
		}

		// wait a little bit before starting active part
		tokio::time::sleep(Duration::from_millis(100)).await;

		// activate liveliness
		#[cfg(feature = "liveliness")]
		if self.liveliness {
			let msg_type = "alive";
			let token: LivelinessToken<'a> = self.com.liveliness(msg_type).await;
			self.liveliness_token
				.write()
				.expect("should never happen")
				.replace(token);
		}

		// start all registered timers
		#[cfg(feature = "timer")]
		self.timers
			.write()
			.expect("should never happen")
			.iter_mut()
			.for_each(|timer| {
				timer.1.start();
			});

		// wait for a shutdown signal
		match signal::ctrl_c().await {
			Ok(()) => {
				self.stop();
			}
			Err(err) => {
				eprintln!("Unable to listen for shutdown signal: {err}");
				// we also shut down in case of error
				self.stop();
			}
		}
	}

	/// stop the agent
	/// # Panics
	#[tracing::instrument]
	pub fn stop(&mut self) {
		// reverse order of start!
		// stop all registered timers
		#[cfg(feature = "timer")]
		self.timers
			.write()
			.expect("should never happen")
			.iter_mut()
			.for_each(|timer| {
				timer.1.stop();
			});

		#[cfg(feature = "liveliness")]
		{
			// stop liveliness
			self.liveliness_token
				.write()
				.expect("should never happen")
				.take();
			self.liveliness = false;

			// stop liveliness subscriber
			#[cfg(feature = "liveliness")]
			if self
				.liveliness_subscriber
				.read()
				.expect("should never happen")
				.is_some()
			{
				self.liveliness_subscriber
					.write()
					.expect("should never happen")
					.as_mut()
					.expect("should never happen")
					.stop();
			}
		}

		// stop all registered subscribers
		#[cfg(feature = "subscriber")]
		self.subscribers
			.write()
			.expect("should never happen")
			.iter_mut()
			.for_each(|subscriber| {
				subscriber.1.stop();
			});
		// stop all registered queryables
		#[cfg(feature = "queryable")]
		self.queryables
			.write()
			.expect("should never happen")
			.iter_mut()
			.for_each(|queryable| {
				queryable.1.stop();
			});
	}
}
// endregion:	--- Agent

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[derive(Debug)]
	struct Props {}

	#[test]
	const fn normal_types() {
		is_normal::<Agent<Props>>();
	}

	#[tokio::test]
	//#[serial]
	async fn agent_create_default() {
		let _agent1: Agent<Props> = Agent::new(crate::config::Config::local(), Props {});
		let _agent2: Agent<Props> =
			Agent::new_with_prefix(crate::config::Config::local(), Props {}, "agent2");
	}

	#[tokio::test(flavor = "current_thread")]
	//#[serial]
	async fn agent_create_current() {
		let _agent1: Agent<Props> = Agent::new(crate::config::Config::local(), Props {});
		let _agent2: Agent<Props> =
			Agent::new_with_prefix(crate::config::Config::local(), Props {}, "agent2");
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
	//#[serial]
	async fn agent_create_restricted() {
		let _agent1: Agent<Props> = Agent::new(crate::config::Config::local(), Props {});
		let _agent2: Agent<Props> =
			Agent::new_with_prefix(crate::config::Config::local(), Props {}, "agent2");
	}

	#[tokio::test(flavor = "multi_thread")]
	//#[serial]
	async fn agent_create_multi() {
		let _agent1: Agent<Props> = Agent::new(crate::config::Config::local(), Props {});
		let _agent2: Agent<Props> =
			Agent::new_with_prefix(crate::config::Config::local(), Props {}, "agent2");
	}
}
