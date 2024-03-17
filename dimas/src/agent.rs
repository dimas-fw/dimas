// Copyright © 2023 Stephan Kunz

//! Primary module of `DiMAS` implementing [`Agent`]
//!
//! An agent is a physical or virtual unit that
//! - can act in an environment
//! - communicates directly with other units/agents
//! - is driven by a set of tendecies (individual goals or satisfaction-/survival-mechanisms)
//! - has own resources
//! - can perceive its environment to a limited extent
//! - has no or only a partial representation of its environment
//! - has  capabilities and offers services
//! - can possibly reproduce itself
//! - whose behaviour is aimed at fulfilling its objectives,
//!   taking into account the resources and capabilities available to it and
//!   depending on its perception, representations and abilities
//!

// region:		--- modules
use crate::context::Context;
use crate::prelude::*;
use std::{
	fmt::Debug,
	ops::Deref,
	sync::{
		mpsc::{self, Receiver},
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
/// Internal signals, used by panic hooks to inform the [`Agent`] that someting has happened
pub(crate) enum TaskSignal {
	/// Restart a certain liveliness subscriber, identified by its key expression
	#[cfg(feature = "liveliness")]
	RestartLiveliness(String),
	/// Restart a certain queryable, identified by its key expression
	#[cfg(feature = "queryable")]
	RestartQueryable(String),
	/// Restart a certain lsubscriber, identified by its key expression
	#[cfg(feature = "subscriber")]
	RestartSubscriber(String),
	/// Restart a certain timer, identified by its key expression
	#[cfg(feature = "timer")]
	RestartTimer(String),
	/// just to avoid warning messages when no feature is selected
	#[allow(dead_code)]
	Dummy,
}

/// Wait for [`TaskSignal`]s.
/// Necessary for the `select!` macro within the [`Agent`]s main loop
async fn wait_for_signals(rx: &Mutex<Receiver<TaskSignal>>) -> Box<TaskSignal> {
	loop {
		if let Ok(signal) = rx.lock().expect("").try_recv() {
			return Box::new(signal);
		};

		tokio::time::sleep(Duration::from_millis(1)).await;
	}
}
// endregion:	--- TaskSignal

// region:		--- Agent
/// `Agent`
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
	type Target = ArcContext<P>;

	fn deref(&self) -> &Self::Target {
		&self.context
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

	/// Start the agent
	/// # Errors
	/// Currently none
	#[tracing::instrument(skip_all)]
	pub async fn start(&mut self) -> Result<()> {
		// we need an mpsc channel with a receiver behind a `Mutex`
		let (tx, rx) = mpsc::channel();
		let rx = Mutex::new(rx);

		self.start_tasks(&tx)?;

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

		loop {
			// different possibilities that can happen
			select! {
				// `TaskSignal`s
				signal = wait_for_signals(&rx) => {
					match *signal {
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
