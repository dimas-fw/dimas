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
	liveliness::LivelinessSubscriberBuilder, publisher::PublisherBuilder, query::QueryBuilder,
	queryable::QueryableBuilder, subscriber::SubscriberBuilder,
};
use crate::context::ContextImpl;
use crate::timer::TimerBuilder;
#[cfg(doc)]
use crate::{
	com::{
		liveliness::LivelinessSubscriber, publisher::Publisher, query::Query, queryable::Queryable,
		subscriber::Subscriber,
	},
	timer::Timer,
};
use dimas_com::messages::AboutEntity;
use dimas_config::Config;
use dimas_core::{
	enums::{OperationState, Signal},
	error::{DimasError, Result},
	message_types::{Message, Request},
	task_signal::{wait_for_task_signals, TaskSignal},
	traits::{Capability, Context, ContextAbstraction},
};
use std::sync::Arc;
use std::time::Duration;
use std::{
	fmt::Debug,
	sync::{mpsc, Mutex, RwLock},
};
use tokio::{select, signal};
use tracing::{error, info, warn};
use zenoh::{liveliness::LivelinessToken, prelude::sync::SyncResolve, SessionDeclarations};
// endregion:	--- modules

// region:	   --- callbacks
fn callback_dispatcher<P>(ctx: &Context<P>, request: Request) -> Result<()>
where
	P: Send + Sync + Unpin + 'static,
{
	if let Some(value) = request.value() {
		let content: Vec<u8> = value.try_into()?;
		let msg = Message(content);
		let signal: Signal = Message::decode(msg)?;
		match signal {
			Signal::About => about_handler(ctx, request)?,
			Signal::Shutdown => shutdown_handler(ctx, request)?,
			Signal::State { state } => state_handler(ctx, request, state)?,
		}
	}
	Ok(())
}

fn about_handler<P>(ctx: &Context<P>, request: Request) -> Result<()>
where
	P: Send + Sync + Unpin + 'static,
{
	let name = ctx
		.fq_name()
		.unwrap_or_else(|| String::from("--"));
	let mode = ctx.mode().to_string();
	let zid = ctx.uuid();
	let state = ctx.state();
	let value = AboutEntity::new(name, mode, zid, state);
	request.reply(value)?;
	Ok(())
}

fn shutdown_handler<P>(ctx: &Context<P>, request: Request) -> Result<()>
where
	P: Send + Sync + Unpin + 'static,
{
	// send back current infos
	let name = ctx
		.fq_name()
		.unwrap_or_else(|| String::from("--"));
	let mode = ctx.mode().to_string();
	let zid = ctx.uuid();
	let state = ctx.state();
	let value = AboutEntity::new(name, mode, zid, state);
	request.reply(value)?;
	
	// shutdown agent after a short wait time to be able to send response
	let ctx = ctx.clone();
	tokio::task::spawn(async move {
		tokio::time::sleep(Duration::from_millis(2)).await;
		let _ = ctx.sender().send(TaskSignal::Shutdown);
	});
	Ok(())
}

fn state_handler<P>(ctx: &Context<P>, request: Request, state: Option<OperationState>) -> Result<()>
where
	P: Send + Sync + Unpin + 'static,
{
	// is a state value given?
	if let Some(value) = state {
		let _ = ctx.set_state(value);
	}

	// send back result
	let name = ctx
		.fq_name()
		.unwrap_or_else(|| String::from("--"));
	let mode = ctx.mode().to_string();
	let zid = ctx.uuid();
	let state = ctx.state();
	let value = AboutEntity::new(name, mode, zid, state);
	request.reply(value)?;
	Ok(())
}
// endregion:	--- callbacks

// region:	   --- UnconfiguredAgent
/// A new Agent without the basic configuration decisions
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct UnconfiguredAgent<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	name: Option<String>,
	prefix: Option<String>,
	props: P,
}

impl<'a, P> UnconfiguredAgent<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
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
		let context: Arc<ContextImpl<P>> = Arc::new(ContextImpl::new(
			config,
			self.props,
			self.name,
			tx,
			self.prefix,
		)?);

		let agent = Agent {
			rx,
			context,
			liveliness: false,
			liveliness_token: RwLock::new(None),
		};

		// add signal queryables
		// for zid
		let key_expr = format!("{}/*", agent.context.uuid());
		agent
			.queryable()
			.key_expr(&key_expr)
			.callback(callback_dispatcher)
			.activation_state(OperationState::Created)
			.add()?;
		// for fully qualified name
		if let Some(fq_name) = agent.context.fq_name() {
			let key_expr = format!("{fq_name}/*");
			agent
				.queryable()
				.key_expr(&key_expr)
				.callback(callback_dispatcher)
				.activation_state(OperationState::Created)
				.add()?;
		}

		// set [`OperationState`] to Created
		// This will also start the basic queryables
		agent.context.set_state(OperationState::Created)?;

		Ok(agent)
	}
}
// endregion:   --- UnconfiguredAgent

// region:	   --- Agent
/// An Agent with the basic configuration decisions fixed, but not running
#[allow(clippy::module_name_repetitions)]
pub struct Agent<'a, P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// A reciever for signals from tasks
	rx: Mutex<mpsc::Receiver<TaskSignal>>,
	/// The agents context structure
	context: Arc<ContextImpl<P>>,
	/// Flag to control whether sending liveliness or not
	liveliness: bool,
	/// The liveliness token - typically the uuid sent to other participants<br>
	/// Is available in the [`LivelinessSubscriber`] callback
	liveliness_token: RwLock<Option<LivelinessToken<'a>>>,
}

impl<'a, P> Debug for Agent<'a, P>
where
	P: Debug + Send + Sync + Unpin + 'static,
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
	P: Debug + Send + Sync + Unpin + 'static,
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

	/// Get a [`LivelinessSubscriberBuilder`], the builder for a [`LivelinessSubscriber`].
	#[must_use]
	pub fn liveliness_subscriber(
		&self,
	) -> LivelinessSubscriberBuilder<
		P,
		crate::com::liveliness::NoPutCallback,
		crate::com::liveliness::Storage<P>,
	> {
		LivelinessSubscriberBuilder::new(self.context.clone())
			.storage(self.context.liveliness_subscribers().clone())
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
		PublisherBuilder::new(self.context.clone()).storage(self.context.publishers().clone())
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
		QueryBuilder::new(self.context.clone()).storage(self.context.queries().clone())
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
		QueryableBuilder::new(self.context.clone()).storage(self.context.queryables().clone())
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
		SubscriberBuilder::new(self.context.clone()).storage(self.context.subscribers().clone())
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
		TimerBuilder::new(self.context.clone()).storage(self.context.timers().clone())
	}

	/// Start the agent.<br>
	/// The agent can be stopped properly using `ctrl-c`
	///
	/// # Errors
	#[tracing::instrument(skip_all)]
	pub async fn start(self) -> Result<Agent<'a, P>> {
		let session = self.context.session();

		// activate sending liveliness
		if self.liveliness {
			let token_str = self
				.context
				.prefix()
				.clone()
				.map_or(self.context.uuid(), |prefix| {
					format!("{}/{}", prefix, self.context.uuid())
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
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// A reciever for signals from tasks
	rx: Mutex<mpsc::Receiver<TaskSignal>>,
	/// The agents context structure
	context: Arc<ContextImpl<P>>,
	/// Flag to control whether sending liveliness or not
	liveliness: bool,
	/// The liveliness token - typically the uuid sent to other participants<br>
	/// Is available in the [`LivelinessSubscriber`] callback
	liveliness_token: RwLock<Option<LivelinessToken<'a>>>,
}

impl<'a, P> RunningAgent<'a, P>
where
	P: Debug + Send + Sync + Unpin + 'static,
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
							self.context.liveliness_subscribers()
								.write()
								.map_err(|_| DimasError::WriteProperties)?
								.get_mut(&key_expr)
								.ok_or(DimasError::ShouldNotHappen)?
								.manage_operation_state(&self.context.state())?;
						},
						TaskSignal::RestartQueryable(key_expr) => {
							self.context.queryables()
								.write()
								.map_err(|_| DimasError::WriteProperties)?
								.get_mut(&key_expr)
								.ok_or(DimasError::ShouldNotHappen)?
								.manage_operation_state(&self.context.state())?;
						},
						TaskSignal::RestartSubscriber(key_expr) => {
							self.context.subscribers()
								.write()
								.map_err(|_| DimasError::WriteProperties)?
								.get_mut(&key_expr)
								.ok_or(DimasError::ShouldNotHappen)?
								.manage_operation_state(&self.context.state())?;
						},
						TaskSignal::RestartTimer(key_expr) => {
							self.context.timers()
								.write()
								.map_err(|_| DimasError::WriteProperties)?
								.get_mut(&key_expr)
								.ok_or(DimasError::ShouldNotHappen)?
								.manage_operation_state(&self.context.state())?;
						},
						TaskSignal::Shutdown => {
							return self.stop();
						}
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
	/// Propagation of errors from [`Context::stop_registered_tasks()`].
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
		is_normal::<UnconfiguredAgent<Props>>();
		is_normal::<Agent<Props>>();
		is_normal::<RunningAgent<Props>>();
		is_normal::<TaskSignal>();
	}
}
