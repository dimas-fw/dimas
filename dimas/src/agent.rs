//! Copyright © 2023 Stephan Kunz

// region:		--- modules
use crate::com::communicator::Communicator;
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
use std::{
	marker::PhantomData,
	sync::{Arc, RwLock},
	time::Duration,
};
use tokio::time::sleep;
#[cfg(feature = "liveliness")]
use zenoh::liveliness::LivelinessToken;
// endregion:	--- modules

// region:		--- types
//type AgentProps<P> = std::fmt::Debug + Send + Sync + Unpin + 'static;
// endregion:	--- types

// region:		--- Agent
/// implementation of an agent
pub struct Agent<'a, P>
where
	P: std::fmt::Debug + Send + Sync + Unpin + 'static,
{
	pd: PhantomData<&'a P>,
	#[cfg(feature = "liveliness")]
	liveliness: bool,
	com: Arc<Communicator>,
	// an optional liveliness token
	#[cfg(feature = "liveliness")]
	liveliness_token: RwLock<Option<LivelinessToken<'a>>>,
	// an optional liveliness subscriber
	#[cfg(feature = "liveliness")]
	liveliness_subscriber: Arc<RwLock<Option<LivelinessSubscriber<P>>>>,
	// registered subscribers
	#[cfg(feature = "subscriber")]
	subscribers: Arc<RwLock<Vec<Subscriber<P>>>>,
	// registered queryables
	#[cfg(feature = "queryable")]
	queryables: Arc<RwLock<Vec<Queryable<P>>>>,
	// registered publisher
	#[cfg(feature = "publisher")]
	publishers: Arc<RwLock<Vec<Publisher<'a>>>>,
	// registered queries
	#[cfg(feature = "query")]
	queries: Arc<RwLock<Vec<Query>>>,
	// registered timer
	#[cfg(feature = "timer")]
	timers: Arc<RwLock<Vec<Timer<P>>>>,
	// The agents propertie structure
	props: Arc<RwLock<P>>,
}

impl<'a, P> Agent<'a, P>
where
	P: std::fmt::Debug + Send + Sync + Unpin + 'static,
{
	/// Create an instance of an agent.
	pub fn new(config: crate::config::Config, prefix: impl Into<String>, properties: P) -> Self {
		let com = Arc::new(Communicator::new(config, prefix));
		let pd = PhantomData {};
		Self {
			pd,
			#[cfg(feature = "liveliness")]
			liveliness: false,
			com,
			#[cfg(feature = "liveliness")]
			liveliness_token: RwLock::new(None),
			#[cfg(feature = "liveliness")]
			liveliness_subscriber: Arc::new(RwLock::new(None)),
			#[cfg(feature = "subscriber")]
			subscribers: Arc::new(RwLock::new(Vec::new())),
			#[cfg(feature = "queryable")]
			queryables: Arc::new(RwLock::new(Vec::new())),
			#[cfg(feature = "publisher")]
			publishers: Arc::new(RwLock::new(Vec::new())),
			#[cfg(feature = "query")]
			queries: Arc::new(RwLock::new(Vec::new())),
			#[cfg(feature = "timer")]
			timers: Arc::new(RwLock::new(Vec::new())),
			props: Arc::new(RwLock::new(properties)),
		}
	}

	/// get the agents uuid
	pub fn uuid(&self) -> String {
		self.com.uuid()
	}

	/// activate sending liveliness information
	#[cfg(feature = "liveliness")]
	pub fn liveliness(&mut self, activate: bool) {
		self.liveliness = activate;
	}

	/// get a builder for a subscriber for the liveliness information
	#[cfg(feature = "liveliness")]
	pub fn liveliness_subscriber(&self) -> LivelinessSubscriberBuilder<P> {
		LivelinessSubscriberBuilder {
			subscriber: self.liveliness_subscriber.clone(),
			communicator: self.com.clone(),
			props: self.props.clone(),
			key_expr: None,
			msg_type: None,
			callback: None,
		}
	}

	/// get a builder for a Subscriber
	#[cfg(feature = "subscriber")]
	pub fn subscriber(&self) -> SubscriberBuilder<P> {
		SubscriberBuilder {
			collection: self.subscribers.clone(),
			communicator: self.com.clone(),
			props: self.props.clone(),
			key_expr: None,
			msg_type: None,
			callback: None,
		}
	}

	/// get a builder for a Queryable
	#[cfg(feature = "queryable")]
	pub fn queryable(&self) -> QueryableBuilder<P> {
		QueryableBuilder {
			collection: Some(self.queryables.clone()),
			communicator: Some(self.com.clone()),
			props: Some(self.props.clone()),
			key_expr: None,
			msg_type: None,
			callback: None,
		}
	}

	/// get a builder for a Publisher
	#[cfg(feature = "publisher")]
	pub fn publisher(&self) -> PublisherBuilder<'a, P> {
		PublisherBuilder {
			collection: self.publishers.clone(),
			communicator: self.com.clone(),
			props: self.props.clone(),
			key_expr: None,
			msg_type: None,
		}
	}

	/// get a builder for a Query
	#[cfg(feature = "query")]
	pub fn query(&self) -> QueryBuilder<P> {
		QueryBuilder {
			collection: self.queries.clone(),
			communicator: self.com.clone(),
			props: self.props.clone(),
			key_expr: None,
			msg_type: None,
			callback: None,
		}
	}

	/// get a builder for a Timer
	#[cfg(feature = "timer")]
	pub fn timer(&self) -> TimerBuilder<P>
	where
		P: Default,
	{
		TimerBuilder {
			collection: self.timers.clone(),
			communicator: self.com.clone(),
			delay: None,
			interval: None,
			callback: None,
			props: self.props.clone(),
		}
	}

	/// start the agent
	/// # Panics
	///
	pub async fn start(&mut self) {
		// avoid warning
		let _props = self.props.clone();

		// start all registered queryables
		#[cfg(feature = "queryable")]
		self.queryables
			.write()
			.expect("should never happen")
			.iter_mut()
			.for_each(|queryable| {
				queryable.start();
			});
		// start all registered subscribers
		#[cfg(feature = "subscriber")]
		self.subscribers
			.write()
			.expect("should never happen")
			.iter_mut()
			.for_each(|subscriber| {
				subscriber.start();
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
				timer.start();
			});

		// main loop so that agent stays alive
		loop {
			sleep(Duration::from_secs(1)).await;
		}
	}

	/// stop the agent
	/// # Panics
	pub fn stop(&mut self) {
		// reverse order of start!
		// stop all registered timers
		#[cfg(feature = "timer")]
		self.timers
			.write()
			.expect("should never happen")
			.iter_mut()
			.for_each(|timer| {
				timer.stop();
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
				subscriber.stop();
			});
		// stop all registered queryables
		#[cfg(feature = "queryable")]
		self.queryables
			.write()
			.expect("should never happen")
			.iter_mut()
			.for_each(|queryable| {
				queryable.stop();
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
		let _agent1: Agent<Props> = Agent::new(crate::config::Config::local(), "agent1", Props {});
		let _agent2: Agent<Props> = Agent::new(crate::config::Config::local(), "agent2", Props {});
	}

	#[tokio::test(flavor = "current_thread")]
	//#[serial]
	async fn agent_create_current() {
		let _agent1: Agent<Props> = Agent::new(crate::config::Config::local(), "agent1", Props {});
		let _agent2: Agent<Props> = Agent::new(crate::config::Config::local(), "agent2", Props {});
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
	//#[serial]
	async fn agent_create_restricted() {
		let _agent1: Agent<Props> = Agent::new(crate::config::Config::local(), "agent1", Props {});
		let _agent2: Agent<Props> = Agent::new(crate::config::Config::local(), "agent2", Props {});
	}

	#[tokio::test(flavor = "multi_thread")]
	//#[serial]
	async fn agent_create_multi() {
		let _agent1: Agent<Props> = Agent::new(crate::config::Config::local(), "agent1", Props {});
		let _agent2: Agent<Props> = Agent::new(crate::config::Config::local(), "agent2", Props {});
	}
}
