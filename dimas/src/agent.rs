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
//!   let mut agent = Agent::new(properties).config(Config::default())?;
//!
//!   // configuration of the agent
//!   // ...
//!
//!   // start the agent
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
	sync::{mpsc, Mutex},
};
use tokio::{select, signal};
use tracing::{error, info};
use zenoh::{liveliness::LivelinessToken, prelude::sync::SyncResolve, SessionDeclarations};
// endregion:	--- modules

// region:	   --- UnconfiguredAgent
/// This is a new Agent without the necessary configuration input
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct UnconfiguredAgent<P>
where
	P: Send + Sync + Unpin + 'static,
{
	prefix: Option<String>,
	props: P,
}

impl<'a, P> UnconfiguredAgent<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor
	const fn new(properties: P) -> Self {
		Self {
			props: properties,
			prefix: None,
		}
	}

	/// Set a prefix
	#[must_use]
	pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
		self.prefix = Some(prefix.into());
		self
	}

	/// Set the [`Config`]uration.
	///
	/// # Errors
	pub fn config(self, config: Config) -> Result<Agent<'a, P>> {
		let context = Context::new(config, self.props, self.prefix)?.into();
		Ok(Agent {
			context,
			liveliness: false,
			liveliness_token: RwLock::new(None),
		})
	}
}
// endregion:   --- UnconfiguredAgent

// region:	   --- Agent
/// This is a new Agent without the necessary configuration input
#[allow(clippy::module_name_repetitions)]
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

impl<'a, P> Agent<'a, P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Builder
	#[allow(clippy::new_ret_no_self)]
	pub const fn new(properties: P) -> UnconfiguredAgent<P> {
		UnconfiguredAgent::new(properties)
	}

	/// Activate sending liveliness information
	pub fn liveliness(&mut self, activate: bool) {
		self.liveliness = activate;
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
		self.context.liveliness_subscriber()
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
		self.context.liveliness_subscriber()
	}

	/// Get a builder for a [`Publisher`]
	#[cfg(feature = "publisher")]
	#[must_use]
	pub fn publisher(
		&self,
	) -> PublisherBuilder<crate::com::publisher::NoKeyExpression, crate::com::publisher::Storage> {
		self.context.publisher()
	}
	/// Get a builder for a [`Publisher`]
	#[cfg(not(feature = "publisher"))]
	#[must_use]
	pub fn publisher(
		&self,
	) -> PublisherBuilder<crate::com::publisher::NoKeyExpression, crate::com::publisher::NoStorage>
	{
		self.context.publisher()
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
		self.context.query()
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
		self.context.query()
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
		self.context.queryable()
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
		self.context.queryable()
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
		self.context.subscriber()
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
		self.context.subscriber()
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
		self.context.timer()
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
		self.context.timer()
	}

	/// Start the agent.<br>
	/// The agent can be stopped properly using `ctrl-c`
	///
	/// # Errors
	/// Propagation of errors from [`ArcContext::start_registered_tasks()`].
	#[tracing::instrument(skip_all)]
	pub async fn start(self) -> Result<Agent<'a, P>> {
		// we need an mpsc channel with a receiver behind a mutex guard
		let (tx, rx) = mpsc::channel();
		let rx = Mutex::new(rx);

		self.context.start_registered_tasks(&tx)?;

		// activate sending liveliness
		if self.liveliness {
			let session = self.context.communicator.session.clone();
			let uuid = format!("{}/{}", self.context.communicator.key_expr("alive"), session.zid());

			let token = session
				.liveliness()
				.declare_token(&uuid)
				.res_sync()
				.map_err(DimasError::ActivateLiveliness)?;

			self.liveliness_token
				.write()
				.map_err(|_| DimasError::ShouldNotHappen)?
				.replace(token);
		};

		RunningAgent {
			rx,
			tx,
			context: self.context,
			liveliness: self.liveliness,
			liveliness_token: self.liveliness_token,
		}
		.run()
		.await
	}
}
// endregion:   --- Agent

// region:	   --- RunningAgent
/// This is the running Agent
#[allow(clippy::module_name_repetitions)]
pub struct RunningAgent<'a, P>
where
	P: Send + Sync + Unpin + 'static,
{
	rx: Mutex<mpsc::Receiver<TaskSignal>>,
	tx: mpsc::Sender<TaskSignal>,
	/// The agents context structure
	context: ArcContext<P>,
	/// Flag to control whether sending liveliness or not
	liveliness: bool,
	/// The liveliness token - typically the uuid sent to other participants<br>
	/// Is available in the [`LivelinessSubscriber`] callback
	liveliness_token: RwLock<Option<LivelinessToken<'a>>>,
}

impl<'a, P> RunningAgent<'a, P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// run
	async fn run(mut self) -> Result<Agent<'a, P>> {
		#[cfg(not(any(
			feature = "liveliness",
			feature = "publisher",
			feature = "query",
			feature = "queryable",
			feature = "subscriber",
			feature = "timer",
		)))]
		{
			let tx = self.tx.clone();
			std::mem::drop(tx);
		}
		loop {
			// different possibilities that can happen
			select! {
				// `TaskSignal`s
				signal = wait_for_task_signals(&self.rx) => {
					match *signal {
						#[cfg(feature = "liveliness")]
						TaskSignal::RestartLiveliness(key_expr) => {
							self.context.liveliness_subscribers
								.write()
								.map_err(|_| DimasError::WriteProperties)?
								.get_mut(&key_expr)
								.ok_or(DimasError::ShouldNotHappen)?
								.start(self.context.clone(), self.tx.clone());
						},
						#[cfg(feature = "queryable")]
						TaskSignal::RestartQueryable(key_expr) => {
							self.context.queryables
								.write()
								.map_err(|_| DimasError::WriteProperties)?
								.get_mut(&key_expr)
								.ok_or(DimasError::ShouldNotHappen)?
								.start(self.context.clone(), self.tx.clone());
						},
						#[cfg(feature = "subscriber")]
						TaskSignal::RestartSubscriber(key_expr) => {
							self.context.subscribers
								.write()
								.map_err(|_| DimasError::WriteProperties)?
								.get_mut(&key_expr)
								.ok_or(DimasError::ShouldNotHappen)?
								.start(self.context.clone(), self.tx.clone());
						},
						#[cfg(feature = "timer")]
						TaskSignal::RestartTimer(key_expr) => {
							self.context.timers
								.write()
								.map_err(|_| DimasError::WriteProperties)?
								.get_mut(&key_expr)
								.ok_or(DimasError::ShouldNotHappen)?
								.start(self.context.clone(), self.tx.clone());
						},
						TaskSignal::Dummy => {},
					};
				}

				// shutdown signal "ctrl-c"
				signal = signal::ctrl_c() => {
					match signal {
						Ok(()) => {
							info!("shutdown due to 'ctrl-c'");
							self.context.stop_registered_tasks()?;
							// stop liveliness
							if self.liveliness {
								self.liveliness_token
									.write()
									.map_err(|_| DimasError::ShouldNotHappen)?
									.take();
							}
							let r = Agent {
								context: self.context,
								liveliness: self.liveliness,
								liveliness_token: self.liveliness_token,
							};
							return Ok(r);
						}
						Err(err) => {
							error!("Unable to listen for 'Ctrl-C': {err}");
							// we also try to shut down the agent properly
							self.context.stop_registered_tasks()?;
							// stop liveliness
							if self.liveliness {
								self.liveliness_token
									.write()
									.map_err(|_| DimasError::ShouldNotHappen)?
									.take();
							}
							return Err(DimasError::ShouldNotHappen.into());
						}
					}
				}
			}
		}
	}

	/// Stop the agent
	///
	/// # Errors
	/// Propagation of errors from [`ArcContext::stop_registered_tasks()`].
	#[tracing::instrument(skip_all)]
	pub fn stop(mut self) -> Result<Agent<'a, P>> {
		self.context.stop_registered_tasks()?;

		// stop liveliness
		if self.liveliness {
			self.liveliness_token
				.write()
				.map_err(|_| DimasError::ShouldNotHappen)?
				.take();
		}
		let r = Agent {
			context: self.context,
			liveliness: self.liveliness,
			liveliness_token: self.liveliness_token,
		};
		Ok(r)
	}
}
// endregion:   --- RunningAgent

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
	async fn agent_build() -> Result<()> {
		let agent_u = Agent::new(Props {});
		let config = crate::config::Config::local()?;
		let _agent_c = agent_u.prefix("test").config(config)?;
		Ok(())
	}
}
