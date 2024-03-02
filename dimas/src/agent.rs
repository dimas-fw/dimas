// Copyright © 2023 Stephan Kunz

//! Module `agent` provides the `Agent`.

// region:		--- modules
#[cfg(feature = "liveliness")]
use crate::com::liveliness_subscriber::{LivelinessSubscriber, LivelinessSubscriberBuilder};
use crate::context::Context;
use crate::prelude::*;
use std::{fmt::Debug, ops::Deref, time::Duration};
use tokio::signal;
use zenoh::liveliness::LivelinessToken;
// endregion:	--- modules

// region:		--- Agent
/// Agent
pub struct Agent<'a, P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	// The agents context structure
	context: ArcContext<P>,
	// flag if sending liveliness is active
	liveliness: bool,
	// the liveliness token
	liveliness_token: RwLock<Option<LivelinessToken<'a>>>,
	// an optional liveliness subscriber
	#[cfg(feature = "liveliness")]
	liveliness_subscriber: Arc<RwLock<Option<LivelinessSubscriber<P>>>>,
}

impl<'a, P> Debug for Agent<'a, P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Agent")
			.field("id", &self.context.uuid())
			.field("prefix", &self.context.prefix())
			.finish_non_exhaustive()
	}
}

impl<'a, P> Deref for Agent<'a, P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	type Target = Arc<RwLock<P>>;

	fn deref(&self) -> &Self::Target {
		&self.context.props
	}
}

impl<'a, P> Agent<'a, P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// Create an instance of an agent.
	pub fn new(config: crate::config::Config, properties: P) -> Self {
		Self {
			context: Context::new(config, properties),
			liveliness: false,
			liveliness_token: RwLock::new(None),
			#[cfg(feature = "liveliness")]
			liveliness_subscriber: Arc::new(RwLock::new(None)),
		}
	}

	/// Create an instance of an agent with a standard prefix for the topics.
	pub fn new_with_prefix(
		config: crate::config::Config,
		properties: P,
		prefix: impl Into<String>,
	) -> Self {
		Self {
			context: Context::new_with_prefix(config, properties, prefix),
			liveliness: false,
			liveliness_token: RwLock::new(None),
			#[cfg(feature = "liveliness")]
			liveliness_subscriber: Arc::new(RwLock::new(None)),
		}
	}

	/// get the agents uuid
	#[must_use]
	pub fn uuid(&self) -> String {
		self.context.uuid()
	}

	/// get the agents properties
	#[must_use]
	pub fn props(&self) -> Arc<RwLock<P>> {
		self.context.props.clone()
	}

	/// activate sending liveliness information
	pub fn liveliness(&mut self, activate: bool) {
		self.liveliness = activate;
	}

	/// get a `Context` of the `Agent`
	pub fn get_context(&self) -> ArcContext<P> {
		self.context.clone()
	}

	//#[cfg_attr(doc, doc(cfg(feature = "liveliness")))]
	/// get a builder for a subscriber for the liveliness information
	#[cfg(feature = "liveliness")]
	#[must_use]
	pub fn liveliness_subscriber(&self) -> LivelinessSubscriberBuilder<P> {
		LivelinessSubscriberBuilder {
			subscriber: self.liveliness_subscriber.clone(),
			context: self.get_context(),
			key_expr: None,
			put_callback: None,
			delete_callback: None,
		}
	}

	/// get a builder for a Publisher
	#[must_use]
	pub fn publisher(&self) -> PublisherBuilder<P> {
		PublisherBuilder {
			context: self.get_context(),
			key_expr: None,
		}
	}

	/// get a builder for a Query
	#[must_use]
	pub fn query(&self) -> QueryBuilder<P> {
		QueryBuilder {
			context: self.get_context(),
			key_expr: None,
			mode: None,
			callback: None,
		}
	}

	/// get a builder for a Queryable
	#[must_use]
	pub fn queryable(&self) -> QueryableBuilder<P> {
		QueryableBuilder {
			context: self.get_context(),
			key_expr: None,
			callback: None,
		}
	}

	/// get a builder for a Subscriber
	#[must_use]
	pub fn subscriber(&self) -> SubscriberBuilder<P> {
		SubscriberBuilder {
			context: self.get_context(),
			key_expr: None,
			put_callback: None,
			delete_callback: None,
		}
	}

	/// get a builder for a Timer
	#[must_use]
	pub fn timer(&self) -> TimerBuilder<P> {
		TimerBuilder {
			context: self.get_context(),
			name: None,
			delay: None,
			interval: None,
			callback: None,
		}
	}

	/// start the agent
	#[tracing::instrument]
	pub async fn start(&mut self) -> Result<(), DimasError> {
		// start all registered queryables
		#[cfg(feature = "queryable")]
		self.context
			.queryables
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|queryable| {
				let _ = queryable.1.start();
			});
		// start all registered subscribers
		#[cfg(feature = "subscriber")]
		self.context
			.subscribers
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|subscriber| {
				let _ = subscriber.1.start();
			});
		// start liveliness subscriber
		#[cfg(feature = "liveliness")]
		if self
			.liveliness_subscriber
			.read()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.is_some()
		{
			self.liveliness_subscriber
				.write()
				.map_err(|_| DimasError::ShouldNotHappen)?
				.as_mut()
				.ok_or(DimasError::ShouldNotHappen)?
				.start();
		}

		// wait a little bit before starting active part
		tokio::time::sleep(Duration::from_millis(100)).await;

		// activate liveliness
		if self.liveliness {
			let msg_type = "alive";
			let token: LivelinessToken<'a> = self
				.context
				.communicator
				.send_liveliness(msg_type)
				.await;
			self.liveliness_token
				.write()
				.map_err(|_| DimasError::ShouldNotHappen)?
				.replace(token);
		}

		// start all registered timers
		#[cfg(feature = "timer")]
		self.context
			.timers
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|timer| {
				let _ = timer.1.start();
			});

		// wait for a shutdown signal
		match signal::ctrl_c().await {
			Ok(()) => {
				self.stop()?;
			}
			Err(err) => {
				tracing::error!("Unable to listen for 'Ctrl-C': {err}");
				// we also try to shut down the agent properly
				self.stop()?;
				return Err(DimasError::ShouldNotHappen)
			}
		}
		Ok(())
	}

	/// stop the agent
	#[tracing::instrument]
	pub fn stop(&mut self) -> Result<(), DimasError> {
		// reverse order of start!
		// stop all registered timers
		#[cfg(feature = "timer")]
		self.context
			.timers
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|timer| {
				let _ = timer.1.stop();
			});

		#[cfg(feature = "liveliness")]
		{
			// stop liveliness
			self.liveliness_token
				.write()
				.map_err(|_| DimasError::ShouldNotHappen)?
				.take();
			self.liveliness = false;

			// stop liveliness subscriber
			#[cfg(feature = "liveliness")]
			if self
				.liveliness_subscriber
				.read()
				.map_err(|_| DimasError::ShouldNotHappen)?
				.is_some()
			{
				self.liveliness_subscriber
					.write()
					.map_err(|_| DimasError::ShouldNotHappen)?
					.as_mut()
					.ok_or(DimasError::ShouldNotHappen)?
					.stop();
			}
		}

		// stop all registered subscribers
		#[cfg(feature = "subscriber")]
		self.context
			.subscribers
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|subscriber| {
				let _ = subscriber.1.stop();
			});
		// stop all registered queryables
		#[cfg(feature = "queryable")]
		self.context
			.queryables
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|queryable| {
				let _ = queryable.1.stop();
			});
			Ok(())
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
