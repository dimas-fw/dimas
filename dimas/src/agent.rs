// Copyright © 2023 Stephan Kunz

//! Module `agent` provides the `Agent`.

// region:		--- modules
use crate::context::Context;
use crate::prelude::*;
use std::{
	fmt::Debug,
	ops::Deref,
	sync::{
		mpsc::{self, Receiver, Sender},
		Mutex,
	},
	time::Duration,
};
use tokio::{select, signal};
use tracing::{error, info};
use zenoh::liveliness::LivelinessToken;
// endregion:	--- modules

// region:		--- TaskSignal
#[derive(Debug, Clone)]
pub enum TaskSignal {
	#[cfg(feature = "liveliness")]
	RestartLiveliness(String),
	#[cfg(feature = "queryable")]
	RestartQueryable(String),
	#[cfg(feature = "subscriber")]
	RestartSubscriber(String),
	#[cfg(feature = "timer")]
	RestartTimer(String),
	Dummy,
}

async fn handle_signals(rx: &Mutex<Receiver<TaskSignal>>) -> Box<TaskSignal> {
	loop {
		if let Ok(signal) = rx.lock().expect("").try_recv() {
			return Box::new(signal);
		};

		tokio::time::sleep(Duration::from_millis(1)).await;
	}
}
// endregion:	--- TaskSignal

// region:		--- Agent
/// Agent
pub struct Agent<'a, P>
where
	P: Send + Sync + Unpin + 'static,
{
	// The agents context structure
	context: ArcContext<P>,
	// flag if sending liveliness is active
	liveliness: bool,
	// the liveliness token
	liveliness_token: RwLock<Option<LivelinessToken<'a>>>,
}

impl<'a, P> Debug for Agent<'a, P>
where
	P: Send + Sync + Unpin + 'static,
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
	P: Send + Sync + Unpin + 'static,
{
	type Target = Arc<RwLock<P>>;

	fn deref(&self) -> &Self::Target {
		&self.context.props
	}
}

impl<'a, P> Agent<'a, P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Create an instance of an agent.
	/// # Errors
	///
	pub fn new(config: crate::config::Config, properties: P) -> Result<Self> {
		Ok(Self {
			context: Context::new(config, properties)?.into(),
			liveliness: false,
			liveliness_token: RwLock::new(None),
		})
	}

	/// Create an instance of an agent with a standard prefix for the topics.
	/// # Errors
	///
	pub fn new_with_prefix(
		config: crate::config::Config,
		properties: P,
		prefix: &str,
	) -> Result<Self> {
		Ok(Self {
			context: Context::new_with_prefix(config, properties, prefix)?.into(),
			liveliness: false,
			liveliness_token: RwLock::new(None),
		})
	}

	/// Get the agents uuid
	#[must_use]
	pub fn uuid(&self) -> String {
		self.context.uuid()
	}

	/// Get the agents properties
	#[must_use]
	pub fn props(&self) -> Arc<RwLock<P>> {
		self.context.props.clone()
	}

	/// Activate sending liveliness information
	pub fn liveliness(&mut self, activate: bool) {
		self.liveliness = activate;
	}

	/// Get a `Context` of the `Agent`
	pub fn get_context(&self) -> ArcContext<P> {
		self.context.clone()
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
		self.get_context().liveliness_subscriber()
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
		self.get_context().liveliness_subscriber()
	}

	/// Get a builder for a [`Publisher`]
	#[cfg(feature = "publisher")]
	#[must_use]
	pub fn publisher(
		&self,
	) -> PublisherBuilder<P, crate::com::publisher::NoKeyExpression, crate::com::publisher::Storage>
	{
		self.get_context().publisher()
	}
	/// Get a builder for a [`Publisher`]
	#[cfg(not(feature = "publisher"))]
	#[must_use]
	pub fn publisher(
		&self,
	) -> PublisherBuilder<P, crate::com::publisher::NoKeyExpression, crate::com::publisher::NoStorage>
	{
		self.get_context().publisher()
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
		self.get_context().query()
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
		self.get_context().query()
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
		self.get_context().queryable()
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
		self.get_context().queryable()
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
		self.get_context().subscriber()
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
		self.get_context().subscriber()
	}

	/// Get a builder for a [`Timer`]
	#[cfg(feature = "timer")]
	#[must_use]
	pub fn timer(
		&self,
	) -> TimerBuilder<
		P,
		crate::timer::NoName,
		crate::timer::NoInterval,
		crate::timer::NoIntervalCallback,
		crate::timer::Storage<P>,
	> {
		self.get_context().timer()
	}
	/// Get a builder for a [`Timer`]
	#[cfg(not(feature = "timer"))]
	#[must_use]
	pub fn timer(
		&self,
	) -> TimerBuilder<
		P,
		crate::timer::NoName,
		crate::timer::NoInterval,
		crate::timer::NoIntervalCallback,
		crate::timer::NoStorage,
	> {
		self.get_context().timer()
	}

	/// Internal function for starting all registered tasks
	/// # Errors
	/// Currently none
	#[allow(unused_variables)]
	async fn start_tasks(&mut self, tx: &Sender<TaskSignal>) -> Result<()> {
		// start all registered queryables
		#[cfg(feature = "queryable")]
		self.context
			.queryables
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|queryable| {
				queryable.1.start(tx.clone());
			});

		// start all registered subscribers
		#[cfg(feature = "subscriber")]
		self.context
			.subscribers
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|subscriber| {
				subscriber.1.start(tx.clone());
			});

		// start liveliness subscriber
		#[cfg(feature = "liveliness")]
		self.context
			.liveliness_subscribers
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|subscriber| {
				subscriber.1.start(tx.clone());
			});

		// wait a little bit before starting active part
		//tokio::time::sleep(Duration::from_millis(10)).await;

		// start all registered timers
		#[cfg(feature = "timer")]
		self.context
			.timers
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|timer| {
				timer.1.start(tx.clone());
			});

		// activate liveliness
		if self.liveliness {
			let token: LivelinessToken<'a> = self
				.context
				.communicator
				.send_liveliness("alive")
				.await?;
			self.liveliness_token
				.write()
				.map_err(|_| DimasError::ShouldNotHappen)?
				.replace(token);
		}

		Ok(())
	}

	/// Start the agent
	/// # Errors
	/// Currently none
	#[tracing::instrument(skip_all)]
	pub async fn start(&mut self) -> Result<()> {
		// we need an mpsc channel with a receiver behind a `Mutex`
		let (tx, rx) = mpsc::channel();
		let rx = Mutex::new(rx);

		self.start_tasks(&tx).await?;

		loop {
			// different possibilities that can happen
			select! {
				// Commands
				command = handle_signals(&rx) => {
					match *command {
						#[cfg(feature = "liveliness")]
						TaskSignal::RestartLiveliness(key_expr) => {
							self.context.liveliness_subscribers
								.write()
								.map_err(|_| DimasError::WriteProperties)?
								.get_mut(&key_expr)
								.ok_or(DimasError::ShouldNotHappen)?
								.start(tx.clone());
						},
						#[cfg(feature = "queryable")]
						TaskSignal::RestartQueryable(key_expr) => {
							self.context.queryables
								.write()
								.map_err(|_| DimasError::WriteProperties)?
								.get_mut(&key_expr)
								.ok_or(DimasError::ShouldNotHappen)?
								.start(tx.clone());
						},
						#[cfg(feature = "subscriber")]
						TaskSignal::RestartSubscriber(key_expr) => {
							self.context.subscribers
								.write()
								.map_err(|_| DimasError::WriteProperties)?
								.get_mut(&key_expr)
								.ok_or(DimasError::ShouldNotHappen)?
								.start(tx.clone());
						},
						#[cfg(feature = "timer")]
						TaskSignal::RestartTimer(key_expr) => {
							self.context.timers
								.write()
								.map_err(|_| DimasError::WriteProperties)?
								.get_mut(&key_expr)
								.ok_or(DimasError::ShouldNotHappen)?
								.start(tx.clone());
						},
						TaskSignal::Dummy => {},
					};
				}

				// shutdown signal "ctrl-c"
				signal = signal::ctrl_c() => {
					match signal {
						Ok(()) => {
							info!("shutdown due to 'Ctrl-C'");
							self.stop()?;
							return Ok(());
						}
						Err(err) => {
							error!("Unable to listen for 'Ctrl-C': {err}");
							// we also try to shut down the agent properly
							self.stop()?;
							return Err(DimasError::ShouldNotHappen.into());
						}
					}
				}
			}
		}
	}

	/// Stop the agent
	/// # Errors
	/// Currently none
	#[tracing::instrument(skip_all)]
	pub fn stop(&mut self) -> Result<()> {
		// reverse order of start!
		// stop liveliness
		if self.liveliness {
			self.liveliness_token
				.write()
				.map_err(|_| DimasError::ShouldNotHappen)?
				.take();
		}

		// stop all registered timers
		#[cfg(feature = "timer")]
		self.context
			.timers
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|timer| {
				timer.1.stop();
			});

		#[cfg(feature = "liveliness")]
		{
			// stop all registered liveliness subscribers
			#[cfg(feature = "liveliness")]
			self.context
				.liveliness_subscribers
				.write()
				.map_err(|_| DimasError::ShouldNotHappen)?
				.iter_mut()
				.for_each(|subscriber| {
					subscriber.1.stop();
				});
		}

		// stop all registered subscribers
		#[cfg(feature = "subscriber")]
		self.context
			.subscribers
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|subscriber| {
				subscriber.1.stop();
			});

		// stop all registered queryables
		#[cfg(feature = "queryable")]
		self.context
			.queryables
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.iter_mut()
			.for_each(|queryable| {
				queryable.1.stop();
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
		is_normal::<TaskSignal>();
	}

	#[tokio::test]
	//#[serial]
	async fn agent_create_default() -> Result<()> {
		let _agent1 = Agent::new(crate::config::Config::local()?, Props {});
		let _agent2 = Agent::new_with_prefix(crate::config::Config::local()?, Props {}, "agent2");
		Ok(())
	}

	#[tokio::test(flavor = "current_thread")]
	//#[serial]
	async fn agent_create_current() -> Result<()> {
		let _agent1 = Agent::new(crate::config::Config::local()?, Props {});
		let _agent2 = Agent::new_with_prefix(crate::config::Config::local()?, Props {}, "agent2");
		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
	//#[serial]
	async fn agent_create_restricted() -> Result<()> {
		let _agent1 = Agent::new(crate::config::Config::local()?, Props {});
		let _agent2 = Agent::new_with_prefix(crate::config::Config::local()?, Props {}, "agent2");
		Ok(())
	}

	#[tokio::test(flavor = "multi_thread")]
	//#[serial]
	async fn agent_create_multi() -> Result<()> {
		let _agent1 = Agent::new(crate::config::Config::local()?, Props {});
		let _agent2 = Agent::new_with_prefix(crate::config::Config::local()?, Props {}, "agent2");
		Ok(())
	}
}
