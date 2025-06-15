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
//! use core::time::Duration;
//!
//! #[derive(Debug)]
//! struct AgentProps {}
//!
//! // we need a tokio runtime
//! #[dimas::main]
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
use crate::context::ContextImpl;
use crate::error::Error;
use chrono::Local;
use core::{fmt::Debug, time::Duration};
use dimas_com::builder::LivelinessSubscriberBuilder;
use dimas_com::builder::{
	ObservableBuilder, ObserverBuilder, PublisherBuilder, QuerierBuilder, QueryableBuilder,
	SubscriberBuilder,
};
use dimas_com::traits::LivelinessSubscriber;
use dimas_com::traits::{Observer, Publisher, Querier, Responder};
use dimas_commands::messages::{AboutEntity, PingEntity};
use dimas_config::Config;
use dimas_core::{
	Result,
	builder_states::{NoCallback, NoInterval, NoSelector, Storage},
	enums::{OperationState, Signal, TaskSignal},
	message_types::{Message, QueryMsg},
	traits::{Capability, Context, ContextAbstraction},
};
use dimas_time::{Timer, TimerBuilder};
use std::sync::Arc;
use std::sync::RwLock;
use tokio::{select, signal, sync::mpsc};
use tracing::{error, info, warn};
use zenoh::liveliness::LivelinessToken;
// endregion:	--- modules

// region:	   --- callbacks
async fn callback_dispatcher<P>(ctx: Context<P>, request: QueryMsg) -> Result<()>
where
	P: Send + Sync + 'static,
{
	if let Some(value) = request.payload() {
		let content: Vec<u8> = value.to_bytes().into_owned();
		let msg = Message::new(content);
		let signal: Signal = Message::decode(msg)?;
		match signal {
			Signal::About => about_handler(ctx, request)?,
			Signal::Ping { sent } => ping_handler(ctx, request, sent)?,
			Signal::Shutdown => shutdown_handler(ctx, request)?,
			Signal::State { state } => state_handler(ctx, request, state)?,
		}
	}
	Ok(())
}

fn about_handler<P>(ctx: Context<P>, request: QueryMsg) -> Result<()>
where
	P: Send + Sync + 'static,
{
	let name = ctx
		.fq_name()
		.unwrap_or_else(|| String::from("--"));
	let mode = ctx.mode().to_string();
	let zid = ctx.uuid();
	let state = ctx.state();
	let value = AboutEntity::new(name, mode, zid, state);
	drop(ctx);
	request.reply(value)?;
	Ok(())
}

fn ping_handler<P>(ctx: Context<P>, request: QueryMsg, sent: i64) -> Result<()>
where
	P: Send + Sync + 'static,
{
	let now = Local::now()
		.naive_utc()
		.and_utc()
		.timestamp_nanos_opt()
		.unwrap_or(0);

	let name = ctx
		.fq_name()
		.unwrap_or_else(|| String::from("--"));
	let zid = ctx.uuid();
	let value = PingEntity::new(name, zid, now - sent);
	drop(ctx);
	request.reply(value)?;
	Ok(())
}

fn shutdown_handler<P>(ctx: Context<P>, request: QueryMsg) -> Result<()>
where
	P: Send + Sync + 'static,
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
	tokio::task::spawn(async move {
		tokio::time::sleep(Duration::from_millis(10)).await;
		// gracefully end agent
		let _ = ctx.set_state(OperationState::Standby);
		tokio::time::sleep(Duration::from_millis(100)).await;
		let _ = ctx.set_state(OperationState::Created);
		let _ = ctx.sender().send(TaskSignal::Shutdown).await;
	});
	Ok(())
}

fn state_handler<P>(ctx: Context<P>, request: QueryMsg, state: Option<OperationState>) -> Result<()>
where
	P: Send + Sync + 'static,
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
	drop(ctx);
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
	P: Debug + Send + Sync + 'static,
{
	name: Option<String>,
	prefix: Option<String>,
	props: P,
}

impl<P> UnconfiguredAgent<P>
where
	P: Debug + Send + Sync + 'static,
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
	pub fn config(self, config: &Config) -> Result<Agent<P>> {
		// we need an mpsc channel with a receiver behind a mutex guard
		let (tx, rx) = mpsc::channel(32);
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
		let selector = format!("{}/signal", agent.context.uuid());
		agent
			.queryable()
			.selector(&selector)
			.callback(callback_dispatcher)
			.activation_state(OperationState::Created)
			.add()?;
		// for fully qualified name
		if let Some(fq_name) = agent.context.fq_name() {
			let selector = format!("{fq_name}/*");
			agent
				.queryable()
				.selector(&selector)
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
pub struct Agent<P>
where
	P: Debug + Send + Sync + 'static,
{
	/// A reciever for signals from tasks
	rx: mpsc::Receiver<TaskSignal>,
	/// The agents context structure
	context: Arc<ContextImpl<P>>,
	/// Flag to control whether sending liveliness or not
	liveliness: bool,
	/// The liveliness token - typically the uuid sent to other participants.
	/// Is available in the [`LivelinessSubscriber`] callback
	liveliness_token: RwLock<Option<LivelinessToken>>,
}

impl<P> Debug for Agent<P>
where
	P: Debug + Send + Sync + 'static,
{
	#[allow(clippy::or_fun_call)]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Agent")
			.field("id", &self.context.uuid())
			.field(
				"prefix",
				self.context
					.prefix()
					.unwrap_or(&"None".to_string()),
			)
			.field("name", &self.context.name())
			.finish_non_exhaustive()
	}
}

impl<P> Agent<P>
where
	P: Debug + Send + Sync + 'static,
{
	/// Builder
	#[allow(clippy::new_ret_no_self)]
	pub const fn new(properties: P) -> UnconfiguredAgent<P> {
		UnconfiguredAgent::new(properties)
	}

	/// Activate sending liveliness information
	pub const fn liveliness(&mut self, activate: bool) {
		self.liveliness = activate;
	}

	/// Get a [`LivelinessSubscriberBuilder`], the builder for a `LivelinessSubscriber`.
	#[must_use]
	pub fn liveliness_subscriber(
		&self,
	) -> LivelinessSubscriberBuilder<P, NoCallback, Storage<Box<dyn LivelinessSubscriber>>> {
		LivelinessSubscriberBuilder::new("default", self.context.clone())
			.storage(self.context.liveliness_subscribers())
	}

	/// Get a [`LivelinessSubscriberBuilder`], the builder for a `LivelinessSubscriber`.
	#[must_use]
	pub fn liveliness_subscriber_for(
		&self,
		session_id: impl Into<String>,
	) -> LivelinessSubscriberBuilder<P, NoCallback, Storage<Box<dyn LivelinessSubscriber>>> {
		LivelinessSubscriberBuilder::new(session_id, self.context.clone())
			.storage(self.context.liveliness_subscribers())
	}

	/// Get an [`ObservableBuilder`], the builder for an `Observable`.
	#[must_use]
	pub fn observable(
		&self,
	) -> ObservableBuilder<
		P,
		NoSelector,
		NoCallback,
		NoCallback,
		NoCallback,
		Storage<Box<dyn Responder>>,
	> {
		ObservableBuilder::new("default", self.context.clone()).storage(self.context.responders())
	}

	/// Get an [`ObservableBuilder`], the builder for an `Observable`.
	#[must_use]
	pub fn observable_for(
		&self,
		session_id: impl Into<String>,
	) -> ObservableBuilder<
		P,
		NoSelector,
		NoCallback,
		NoCallback,
		NoCallback,
		Storage<Box<dyn Responder>>,
	> {
		ObservableBuilder::new(session_id, self.context.clone()).storage(self.context.responders())
	}

	/// Get an [`ObserverBuilder`], the builder for an `Observer`.
	#[must_use]
	pub fn observer(
		&self,
	) -> ObserverBuilder<P, NoSelector, NoCallback, NoCallback, Storage<Box<dyn Observer>>> {
		ObserverBuilder::new("default", self.context.clone()).storage(self.context.observers())
	}

	/// Get an [`ObserverBuilder`], the builder for an `Observer`.
	#[must_use]
	pub fn observer_for(
		&self,
		session_id: impl Into<String>,
	) -> ObserverBuilder<P, NoSelector, NoCallback, NoCallback, Storage<Box<dyn Observer>>> {
		ObserverBuilder::new(session_id, self.context.clone()).storage(self.context.observers())
	}

	/// Get a [`PublisherBuilder`], the builder for a s`Publisher`.
	#[must_use]
	pub fn publisher(&self) -> PublisherBuilder<P, NoSelector, Storage<Box<dyn Publisher>>> {
		PublisherBuilder::new("default", self.context.clone()).storage(self.context.publishers())
	}

	/// Get a [`PublisherBuilder`], the builder for a s`Publisher`.
	#[must_use]
	pub fn publisher_for(
		&self,
		session_id: impl Into<String>,
	) -> PublisherBuilder<P, NoSelector, Storage<Box<dyn Publisher>>> {
		PublisherBuilder::new(session_id, self.context.clone()).storage(self.context.publishers())
	}

	/// Get a [`QuerierBuilder`], the builder for a `Querier`.
	#[must_use]
	pub fn querier(&self) -> QuerierBuilder<P, NoSelector, NoCallback, Storage<Box<dyn Querier>>> {
		QuerierBuilder::new("default", self.context.clone()).storage(self.context.queriers())
	}

	/// Get a [`QuerierBuilder`], the builder for a `Querier`.
	#[must_use]
	pub fn querier_for(
		&self,
		session_id: impl Into<String>,
	) -> QuerierBuilder<P, NoSelector, NoCallback, Storage<Box<dyn Querier>>> {
		QuerierBuilder::new(session_id, self.context.clone()).storage(self.context.queriers())
	}

	/// Get a [`QueryableBuilder`], the builder for a `Queryable`.
	#[must_use]
	pub fn queryable(
		&self,
	) -> QueryableBuilder<P, NoSelector, NoCallback, Storage<Box<dyn Responder>>> {
		QueryableBuilder::new("default", self.context.clone()).storage(self.context.responders())
	}

	/// Get a [`QueryableBuilder`], the builder for a `Queryable`.
	#[must_use]
	pub fn queryable_for(
		&self,
		session_id: impl Into<String>,
	) -> QueryableBuilder<P, NoSelector, NoCallback, Storage<Box<dyn Responder>>> {
		QueryableBuilder::new(session_id, self.context.clone()).storage(self.context.responders())
	}

	/// Get a [`SubscriberBuilder`], the builder for a `Subscriber`.
	#[must_use]
	pub fn subscriber(
		&self,
	) -> SubscriberBuilder<P, NoSelector, NoCallback, Storage<Box<dyn Responder>>> {
		SubscriberBuilder::new("default", self.context.clone()).storage(self.context.responders())
	}

	/// Get a [`SubscriberBuilder`], the builder for a `Subscriber`.
	#[must_use]
	pub fn subscriber_for(
		&self,
		session_id: impl Into<String>,
	) -> SubscriberBuilder<P, NoSelector, NoCallback, Storage<Box<dyn Responder>>> {
		SubscriberBuilder::new(session_id, self.context.clone()).storage(self.context.responders())
	}

	/// Get a [`TimerBuilder`], the builder for a [`Timer`].
	#[must_use]
	pub fn timer(&self) -> TimerBuilder<P, NoSelector, NoInterval, NoCallback, Storage<Timer<P>>> {
		TimerBuilder::new(self.context.clone()).storage(self.context.timers())
	}

	/// Start the agent.
	///
	/// The agent can be stopped properly using `ctrl-c`
	///
	/// # Errors
	/// 
	/// # Panics
	#[tracing::instrument(skip_all)]
	pub async fn start(self) -> Result<Self> {
		// activate sending liveliness
		if self.liveliness {
			let session = self.context.session("default");
			let token_str = self
				.context
				.prefix()
				.map_or(self.context.uuid(), |prefix| {
					format!("{}/{}", prefix, self.context.uuid())
				});

			let token = session
				.expect("snh")
				.liveliness()
				.declare_token(&token_str)
				.await
				.map_err(|source| Error::ActivateLiveliness { source })?;

			self.liveliness_token
				.write()
				.map_err(|_| Error::ModifyStruct("liveliness".into()))?
				.replace(token);
		}

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
pub struct RunningAgent<P>
where
	P: Debug + Send + Sync + 'static,
{
	/// The receiver for signals from tasks
	rx: mpsc::Receiver<TaskSignal>,
	/// The agents context structure
	context: Arc<ContextImpl<P>>,
	/// Flag to control whether sending liveliness or not
	liveliness: bool,
	/// The liveliness token - typically the uuid sent to other participants.
	/// Is available in the [`LivelinessSubscriber`] callback
	liveliness_token: RwLock<Option<LivelinessToken>>,
}

impl<P> RunningAgent<P>
where
	P: Debug + Send + Sync + 'static,
{
	/// run
	async fn run(mut self) -> Result<Agent<P>> {
		loop {
			// different possibilities that can happen
			select! {
				// `TaskSignal`s
				Some(signal) = self.rx.recv() => {
					match signal {
						TaskSignal::RestartLiveliness(selector) => {
							self.context.liveliness_subscribers()
								.write()
								.map_err(|_| Error::WriteAccess)?
								.get_mut(&selector)
								.ok_or_else(|| Error::GetMut("liveliness".into()))?
								.manage_operation_state(&self.context.state())?;
						},
						TaskSignal::RestartQueryable(selector) => {
							self.context.responders()
								.write()
								.map_err(|_| Error::WriteAccess)?
								.get_mut(&selector)
								.ok_or_else(|| Error::GetMut("queryables".into()))?
								.manage_operation_state(&self.context.state())?;
						},
						TaskSignal::RestartObservable(selector) => {
							self.context.responders()
								.write()
								.map_err(|_| Error::WriteAccess)?
								.get_mut(&selector)
								.ok_or_else(|| Error::GetMut("observables".into()))?
								.manage_operation_state(&self.context.state())?;
						},
						TaskSignal::RestartSubscriber(selector) => {
							self.context.responders()
								.write()
								.map_err(|_| Error::WriteAccess)?
								.get_mut(&selector)
								.ok_or_else(|| Error::GetMut("subscribers".into()))?
								.manage_operation_state(&self.context.state())?;
						},
						TaskSignal::RestartTimer(selector) => {
							self.context.timers()
								.write()
								.map_err(|_| Error::WriteAccess)?
								.get_mut(&selector)
								.ok_or_else(|| Error::GetMut("timers".into()))?
								.manage_operation_state(&self.context.state())?;
						},
						TaskSignal::Shutdown => {
							return self.stop();
						}
					}
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
	#[tracing::instrument(skip_all)]
	pub fn stop(self) -> Result<Agent<P>> {
		self.context.set_state(OperationState::Created)?;

		// stop liveliness
		if self.liveliness {
			self.liveliness_token
				.write()
				.map_err(|_| Error::ModifyStruct("liveliness".into()))?
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
	const fn is_normal<T: Sized + Send + Sync>() {}

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
