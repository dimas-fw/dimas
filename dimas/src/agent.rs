// Copyright © 2023 Stephan Kunz

//! Primary module of `DiMAS` implementing [`Agent`]
//!
//! An agent is a physical or virtual unit that
//! - can act in an environment
//! - communicates directly with other units/agents
//! - is driven by a set of tendencies (individual goals and/or satisfaction-/survival-mechanisms)
//! - has own resources
//! - can perceive its environment to a limited extent
//! - has no or only a partial representation of its environment
//! - has  capabilities and offers services
//! - can possibly reproduce itself
//! - whose behaviour is aimed at fulfilling its objectives,
//!   taking into account the resources and capabilities available to it and
//!   depending on its perception, representations and abilities
//!
//! # Examples
//! ```rust,no_run
//! use dimas::prelude::*;
//! use std::time::Duration;
//!
//! #[derive(Debug)]
//! struct AgentProps {}
//!
//! // we need an async runtime, preferably tokio
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!   // create & initialize agents properties
//!   let properties = AgentProps {};
//!
//!   // create an agent with the properties and a default configuration
//!   let mut agent = Agent::new(Config::default(), properties)?;
//!
//!   // configuration of the agent
//!   // ...
//!
//!   // run the agent
//!   agent.start().await?;
//!   Ok(())
//! }
//! ```
//!
//! A running agent can be properly stopped with `ctrl-c`
//!

// region:		--- modules
use crate::context::Context;
use crate::prelude::*;
use crate::utils::{wait_for_task_signals, TaskSignal};
use std::{
	fmt::Debug,
	ops::Deref,
	sync::{mpsc, Mutex},
};
use tokio::{select, signal};
use tracing::{error, info};
use zenoh::liveliness::LivelinessToken;
// endregion:	--- modules

// region:		--- Agent
/// Representation of an [`Agent`].<br>
/// Available constructors: [`Agent::new`] and [`Agent::new_with_prefix`]
pub struct Agent<'a, P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// The agents context structure
	context: ArcContext<P>,
	/// Flag to control whether sending liveliness or not
	liveliness: bool,
	/// The liveliness token - typically the uuid sent to other participants<br>
	/// Is available in the [`LivelinessSubscriber`] callback
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

/// Enables thread safe access to [`Context`] including [`Agent`]s properties.
impl<'a, P> Deref for Agent<'a, P>
where
	P: Send + Sync + Unpin + 'static,
{
	type Target = ArcContext<P>;
	/// Enables thread safe access to [`Context`] including properties.
	fn deref(&self) -> &Self::Target {
		&self.context
	}
}

/// Directly accessible methods.
impl<'a, P> Agent<'a, P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Create an instance of an agent.
	/// # Errors
	/// Propagation of errors creating the [`Context`].
	pub fn new(config: crate::config::Config, properties: P) -> Result<Self> {
		Ok(Self {
			context: Context::new(config, properties)?.into(),
			liveliness: false,
			liveliness_token: RwLock::new(None),
		})
	}

	/// Create an instance of an agent with a prefix.<br>
	/// The prefix is used in communication to prefix the topics.
	/// It is an easy way to separate groups of agents within the same environment.<br>
	/// See [`LivelinessSubscriberBuilder`], [`PublisherBuilder`], [`QueryBuilder`], [`QueryableBuilder`], [`SubscriberBuilder`].
	/// # Errors
	/// Propagation of errors creating the [`Context`].
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

	/// Activate sending liveliness information
	pub fn liveliness(&mut self, activate: bool) {
		self.liveliness = activate;
	}

	/// Start the agent.<br>
	/// The agent can be stopped properly using `ctrl-c`
	/// # Errors
	/// Propagation of errors from [`Agent::stop`],
	/// [`Context::start_registered_tasks`] and
	/// [`Communicator::send_liveliness`].
	#[tracing::instrument(skip_all)]
	pub async fn start(&mut self) -> Result<()> {
		// we need an mpsc channel with a receiver behind a mutex guard
		let (tx, rx) = mpsc::channel();
		let rx = Mutex::new(rx);

		self.context.start_registered_tasks(&tx)?;

		// activate liveliness
		if self.liveliness {
			let token: LivelinessToken<'a> = self
				.context
				.communicator
				.send_liveliness()
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
				signal = wait_for_task_signals(&rx) => {
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
							info!("shutdown due to 'ctrl-c'");
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

	#[tokio::test(flavor = "multi_thread")]
	//#[serial]
	async fn agent_create_multi() -> Result<()> {
		let _agent1 = Agent::new(crate::config::Config::local()?, Props {});
		let _agent2 = Agent::new_with_prefix(crate::config::Config::local()?, Props {}, "agent2");
		Ok(())
	}
}
