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
//!   let mut agent = Agent::new(properties).config(&Config::default())?;
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
use crate::com::{
	liveliness::LivelinessSubscriberBuilder,
	publisher::PublisherBuilder,
	query::QueryBuilder,
	queryable::QueryableBuilder,
	subscriber::SubscriberBuilder,
	task_signal::{wait_for_task_signals, TaskSignal},
};
use crate::context::{ArcContext, Context};
use crate::timer::TimerBuilder;
#[cfg(doc)]
use crate::{
	com::{
		liveliness::LivelinessSubscriber, publisher::Publisher, query::Query,
		subscriber::Subscriber,
	},
	timer::Timer,
};
use dimas_com::messages::AboutEntity;
use dimas_com::Request;
use dimas_config::Config;
use dimas_core::{
	error::{DimasError, Result},
	traits::{ManageState, OperationState},
};
use std::{
	fmt::Debug,
	sync::{mpsc, Mutex, RwLock},
};
use tokio::{select, signal};
use tracing::{error, info};
use zenoh::{liveliness::LivelinessToken, prelude::sync::SyncResolve, SessionDeclarations};
// endregion:	--- modules

// region:	   --- UnconfiguredAgent
/// A new Agent without the basic configuration decisions
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct UnconfiguredAgent<P>
where
	P: Send + Sync + Unpin + 'static,
{
	name: Option<String>,
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
			name: None,
			prefix: None,
			props: properties,
		}
	}

	/// Set a name
	#[must_use]
	pub fn name(mut self, name: impl Into<String>) -> Self {
		self.name = Some(name.into());
		self
	}

	/// Set a prefix
	#[must_use]
	pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
		self.prefix = Some(prefix.into());
		self
	}

	/// Set the [`Config`]uration.
	/// An agent with [`OperationState`] `Configured` can be started
	/// and will respond to commands from dimasctl/dimasmon
	///
	/// # Errors
	///
	pub fn config(self, config: &Config) -> Result<Agent<'a, P>> {
		// we need an mpsc channel with a receiver behind a mutex guard
		let (tx, rx) = mpsc::channel();
		let rx = Mutex::new(rx);
		let context: ArcContext<P> =
			Context::new(config, self.props, self.name, tx, self.prefix)?.into();

		let agent = Agent {
			rx,
			context,
			liveliness: false,
			liveliness_token: RwLock::new(None),
		};

		// create "about" queryables
		// for zid
		let key_expr = format!("{}/about", agent.context.uuid());
		agent
			.queryable()
			.key_expr(&key_expr)
			.callback(Agent::about)
			.activation_state(OperationState::Created)
			.add()?;
		// for fully qualified name
		if let Some(fq_name) = agent.context.fq_name() {
			let key_expr = format!("{fq_name}/about");
			agent
				.queryable()
				.key_expr(&key_expr)
				.callback(Agent::about)
				.activation_state(OperationState::Created)
				.add()?;
		}

		// create "state" queryables
		// for zid
		let key_expr = format!("{}/state", agent.context.uuid());
		agent
			.queryable()
			.key_expr(&key_expr)
			.callback(Agent::state)
			.activation_state(OperationState::Created)
			.add()?;
		// for fully qualified name
		if let Some(fq_name) = agent.context.fq_name() {
			let key_expr = format!("{fq_name}/state");
			agent
				.queryable()
				.key_expr(&key_expr)
				.callback(Agent::state)
				.activation_state(OperationState::Created)
				.add()?;
		}

		// set agents [`OperationState`] to Configured
		// This will also start the basic queryables
		agent
			.context
			.set_state(OperationState::Configured)?;

		Ok(agent)
	}
}
// endregion:   --- UnconfiguredAgent

// region:	   --- Agent
/// An Agent with the basic configuration decisions fixed, but not running
#[allow(clippy::module_name_repetitions)]
pub struct Agent<'a, P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// A reciever for signals from tasks
	rx: Mutex<mpsc::Receiver<TaskSignal>>,
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
			.field("prefix", self.context.prefix())
			.field("name", &self.context.name())
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
	#[must_use]
	pub fn liveliness_subscriber(
		&self,
	) -> LivelinessSubscriberBuilder<
		P,
		crate::com::liveliness::NoPutCallback,
		crate::com::liveliness::Storage<P>,
	> {
		self.context.liveliness_subscriber()
	}

	/// Get a builder for a [`Publisher`]
	#[must_use]
	pub fn publisher(
		&self,
	) -> PublisherBuilder<
		P,
		crate::com::publisher::NoKeyExpression,
		crate::com::publisher::Storage<P>,
	> {
		self.context.publisher()
	}

	/// Get a builder for a [`Query`]
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

	/// Get a builder for a [`Queryable`]
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

	/// Get a builder for a [`Subscriber`]
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

	/// Get a builder for a [`Timer`]
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

	fn about(ctx: &ArcContext<P>, request: Request) -> Result<()> {
		let name = ctx
			.fq_name()
			.unwrap_or_else(|| String::from("--"));
		let mode = ctx.communicator.mode().to_string();
		let zid = ctx.communicator.uuid();
		let state = ctx.state();
		let value = AboutEntity::new(name, mode, zid, state);
		request.reply(value)?;
		Ok(())
	}

	fn state(ctx: &ArcContext<P>, request: Request) -> Result<()> {
		let parms = request
			.parameters()
			.to_string()
			.replace("(state=", "")
			.replace(')', "");
		let _ = match parms.as_str() {
			"Created" | "created" => ctx.set_state(OperationState::Created),
			"Configured" | "configured" => ctx.set_state(OperationState::Configured),
			"Inactive" | "inactive" => ctx.set_state(OperationState::Inactive),
			"Standby" | "standby" => ctx.set_state(OperationState::Standby),
			"Active" | "active" => ctx.set_state(OperationState::Active),
			_ => {Ok(())},
		};

		// send back result
		let name = ctx
			.fq_name()
			.unwrap_or_else(|| String::from("--"));
		let mode = ctx.communicator.mode().to_string();
		let zid = ctx.communicator.uuid();
		let state = ctx.state();
		let value = AboutEntity::new(name, mode, zid, state);
		request.reply(value)?;
		Ok(())
	}

	/// Start the agent.<br>
	/// The agent can be stopped properly using `ctrl-c`
	///
	/// # Errors
	/// Propagation of errors from [`ArcContext::start_registered_tasks()`].
	#[tracing::instrument(skip_all)]
	pub async fn start(self) -> Result<Agent<'a, P>> {
		let session = self.context.communicator.session();

		// activate sending liveliness
		if self.liveliness {
			let token_str = self
				.context
				.prefix()
				.clone()
				.map_or(self.context.communicator.uuid(), |prefix| {
					format!("{}/{}", prefix, self.context.communicator.uuid())
				});

			let token = session
				.liveliness()
				.declare_token(&token_str)
				.res_sync()
				.map_err(DimasError::ActivateLiveliness)?;

			self.liveliness_token
				.write()
				.map_err(|_| DimasError::ModifyContext("liveliness".into()))?
				.replace(token);
		};

		self.context.set_state(OperationState::Active)?;

		RunningAgent {
			rx: self.rx,
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
/// A running Agent, which can't be modified while running
#[allow(clippy::module_name_repetitions)]
pub struct RunningAgent<'a, P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// A reciever for signals from tasks
	rx: Mutex<mpsc::Receiver<TaskSignal>>,
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
	async fn run(self) -> Result<Agent<'a, P>> {
		loop {
			// different possibilities that can happen
			select! {
				// `TaskSignal`s
				signal = wait_for_task_signals(&self.rx) => {
					match *signal {
						TaskSignal::RestartLiveliness(key_expr) => {
							self.context.liveliness_subscribers
								.write()
								.map_err(|_| DimasError::WriteProperties)?
								.get_mut(&key_expr)
								.ok_or(DimasError::ShouldNotHappen)?
								.manage_state(&self.context.state())?;
						},
						TaskSignal::RestartQueryable(key_expr) => {
							self.context.queryables
								.write()
								.map_err(|_| DimasError::WriteProperties)?
								.get_mut(&key_expr)
								.ok_or(DimasError::ShouldNotHappen)?
								.manage_state(&self.context.state())?;
						},
						TaskSignal::RestartSubscriber(key_expr) => {
							self.context.subscribers
								.write()
								.map_err(|_| DimasError::WriteProperties)?
								.get_mut(&key_expr)
								.ok_or(DimasError::ShouldNotHappen)?
								.manage_state(&self.context.state())?;
						},
						TaskSignal::RestartTimer(key_expr) => {
							self.context.timers
								.write()
								.map_err(|_| DimasError::WriteProperties)?
								.get_mut(&key_expr)
								.ok_or(DimasError::ShouldNotHappen)?
								.manage_state(&self.context.state())?;
						},
					};
				}

				// shutdown signal "ctrl-c"
				signal = signal::ctrl_c() => {
					match signal {
						Ok(()) => {
							info!("shutdown due to 'ctrl-c'");
							return self.stop();
						}
						Err(err) => {
							error!("Unable to listen for 'Ctrl-C': {err}");
							// we also try to shut down the agent properly
							return self.stop();
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
	pub fn stop(self) -> Result<Agent<'a, P>> {
		self.context.set_state(OperationState::Created)?;

		// stop liveliness
		if self.liveliness {
			self.liveliness_token
				.write()
				.map_err(|_| DimasError::ModifyContext("liveliness".into()))?
				.take();
		}
		let r = Agent {
			rx: self.rx,
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
}
