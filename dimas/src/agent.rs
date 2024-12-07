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
#[cfg(feature = "unstable")]
use crate::builder::LivelinessSubscriberBuilder;
use crate::builder::{
	builder_states::{NoCallback, NoInterval, NoSelector, Storage, StorageNew},
	ObservableBuilder, ObserverBuilder, PublisherBuilder, QuerierBuilder, QueryableBuilder,
	SubscriberBuilder, TimerBuilder,
};
use crate::context::ContextImpl;
use crate::error::Error;
use crate::utils::{ComponentRegistryType, LibManager};
use anyhow::Result;
use chrono::Local;
use core::{fmt::Debug, time::Duration};
#[cfg(feature = "unstable")]
use dimas_com::traits::LivelinessSubscriber;
use dimas_com::traits::{Observer, Responder};
use dimas_commands::messages::{AboutEntity, PingEntity};
use dimas_config::Config;
use dimas_core::{
	enums::{Signal, TaskSignal},
	message_types::{Message, QueryMsg},
	traits::{Context, ContextAbstraction},
	Activity, ActivityId, Component, ComponentId, OperationState, Operational, OperationalType,
	System, SystemType, Transitions,
};
#[cfg(feature = "unstable")]
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::{select, signal, sync::mpsc};
use tracing::{error, event, info, instrument, warn, Level};
#[cfg(feature = "unstable")]
use zenoh::liveliness::LivelinessToken;
#[cfg(feature = "unstable")]
use zenoh::Wait;
// endregion:	--- modules

// region:	   --- callbacks
#[instrument(level = Level::DEBUG, skip_all)]
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

#[instrument(level = Level::DEBUG, skip_all)]
fn about_handler<P>(ctx: Context<P>, request: QueryMsg) -> Result<()>
where
	P: Send + Sync + 'static,
{
	let name = ctx
		.fq_name()
		.unwrap_or_else(|| String::from("--"));
	let mode = ctx.mode();
	let zid = ctx.uuid();
	let state = ctx.state_old();
	let value = AboutEntity::new(name, mode, zid, state);
	drop(ctx);
	request.reply(value)?;
	Ok(())
}

#[instrument(level = Level::DEBUG, skip_all)]
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

#[instrument(level = Level::DEBUG, skip_all)]
fn shutdown_handler<P>(ctx: Context<P>, request: QueryMsg) -> Result<()>
where
	P: Send + Sync + 'static,
{
	// send back current infos
	let name = ctx
		.fq_name()
		.unwrap_or_else(|| String::from("--"));
	let mode = ctx.mode();
	let zid = ctx.uuid();
	let state = ctx.state_old();
	let value = AboutEntity::new(name, mode, zid, state);
	request.reply(value)?;

	// shutdown agent after a short wait time to be able to send response
	tokio::task::spawn(async move {
		tokio::time::sleep(Duration::from_millis(10)).await;
		// gracefully end agent
		let _ = ctx.set_state_old(OperationState::Standby);
		tokio::time::sleep(Duration::from_millis(100)).await;
		let _ = ctx.set_state_old(OperationState::Created);
		let _ = ctx.sender().send(TaskSignal::Shutdown).await;
	});
	Ok(())
}

#[instrument(level = Level::DEBUG, skip_all)]
fn state_handler<P>(ctx: Context<P>, request: QueryMsg, state: Option<OperationState>) -> Result<()>
where
	P: Send + Sync + 'static,
{
	// is a state value given?
	if let Some(value) = state {
		let _ = ctx.set_state_old(value);
	}

	// send back result
	let name = ctx
		.fq_name()
		.unwrap_or_else(|| String::from("--"));
	let mode = ctx.mode();
	let zid = ctx.uuid();
	let state = ctx.state_old();
	let value = AboutEntity::new(name, mode, zid, state);
	drop(ctx);
	request.reply(value)?;
	Ok(())
}
// endregion:	--- callbacks

// region:	   --- UnconfiguredAgent
/// A new Agent without the basic configuration decisions
#[allow(clippy::module_name_repetitions)]
pub struct UnconfiguredAgent<P>
where
	P: Send + Sync + 'static,
{
	name: Option<String>,
	prefix: Option<String>,
	props: P,
}

impl<P> UnconfiguredAgent<P>
where
	P: Send + Sync + 'static,
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
	#[instrument(level = Level::DEBUG, skip_all)]
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

		let mut agent = Agent {
			system: SystemType::default(),
			rx,
			context,
			libmanager: LibManager::new(),
			registry: ComponentRegistryType::new(),
			#[cfg(feature = "unstable")]
			liveliness: false,
			#[cfg(feature = "unstable")]
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
		agent
			.context
			.set_state_old(OperationState::Created)?;

		Ok(agent)
	}
}
// endregion:   --- UnconfiguredAgent

// region:	   --- Agent
/// An Agent with the basic configuration decisions fixed, but not running
#[dimas_macros::system]
pub struct Agent<P>
where
	P: Send + Sync + 'static,
{
	/// A reciever for signals from tasks
	rx: mpsc::Receiver<TaskSignal>,
	/// The agents context structure
	context: Arc<ContextImpl<P>>,
	/// Library manager
	libmanager: LibManager,
	/// Component register
	registry: ComponentRegistryType,
	/// Flag to control whether sending liveliness or not
	#[cfg(feature = "unstable")]
	liveliness: bool,
	/// The liveliness token - typically the uuid sent to other participants.
	/// Is available in the [`LivelinessSubscriber`] callback
	#[cfg(feature = "unstable")]
	liveliness_token: RwLock<Option<LivelinessToken>>,
}

impl<P> AsMut<ComponentRegistryType> for Agent<P>
where
	P: Send + Sync + 'static,
{
	fn as_mut(&mut self) -> &mut ComponentRegistryType {
		&mut self.registry
	}
}

impl<P> AsRef<ComponentRegistryType> for Agent<P>
where
	P: Send + Sync + 'static,
{
	fn as_ref(&self) -> &ComponentRegistryType {
		&self.registry
	}
}

impl<P> AsMut<LibManager> for Agent<P>
where
	P: Send + Sync + 'static,
{
	fn as_mut(&mut self) -> &mut LibManager {
		&mut self.libmanager
	}
}

impl<P> AsRef<LibManager> for Agent<P>
where
	P: Send + Sync + 'static,
{
	fn as_ref(&self) -> &LibManager {
		&self.libmanager
	}
}

impl<P> Debug for Agent<P>
where
	P: Send + Sync + 'static,
{
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

impl<P> Transitions for Agent<P> where P: Send + Sync + 'static {}

impl<P> Agent<P>
where
	P: Send + Sync + 'static,
{
	/// Builder
	#[allow(clippy::new_ret_no_self)]
	pub const fn new(properties: P) -> UnconfiguredAgent<P> {
		UnconfiguredAgent::new(properties)
	}

	/// Activate sending liveliness information
	#[cfg(feature = "unstable")]
	pub fn liveliness(&mut self, activate: bool) {
		self.liveliness = activate;
	}

	/// Get a [`LivelinessSubscriberBuilder`], the builder for a `LivelinessSubscriber`.
	#[cfg(feature = "unstable")]
	#[must_use]
	pub fn liveliness_subscriber(
		&self,
	) -> LivelinessSubscriberBuilder<P, NoCallback, Storage<Box<dyn LivelinessSubscriber>>> {
		LivelinessSubscriberBuilder::new("default", self.context.clone())
			.storage(self.context.liveliness_subscribers())
	}

	/// Get a [`LivelinessSubscriberBuilder`], the builder for a `LivelinessSubscriber`.
	#[cfg(feature = "unstable")]
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

	/// Get a [`PublisherBuilder`].
	#[must_use]
	pub fn publisher(&mut self) -> PublisherBuilder<P, NoSelector, StorageNew> {
		PublisherBuilder::new("default", self.context.clone()).storage(self.system_mut())
	}

	/// Get a [`QuerierBuilder`].
	#[must_use]
	pub fn querier(&mut self) -> QuerierBuilder<P, NoSelector, NoCallback, StorageNew> {
		QuerierBuilder::new("default", self.context.clone()).storage(self.system_mut())
	}

	/// Get a [`QueryableBuilder`].
	#[must_use]
	pub fn queryable(&mut self) -> QueryableBuilder<P, NoSelector, NoCallback, StorageNew> {
		QueryableBuilder::new("default", self.context.clone()).storage(self.system_mut())
	}

	/// Get a [`SubscriberBuilder`].
	#[must_use]
	pub fn subscriber(&mut self) -> SubscriberBuilder<P, NoSelector, NoCallback, StorageNew> {
		SubscriberBuilder::new("default", self.context.clone()).storage(self.system_mut())
	}

	/// Get a [`TimerBuilder`].
	#[must_use]
	pub fn timer(&mut self) -> TimerBuilder<P, NoSelector, NoInterval, NoCallback, StorageNew> {
		TimerBuilder::new(self.context.clone()).storage(self.system_mut())
	}

	/// Start the agent.
	///
	/// The agent can be stopped properly using `ctrl-c`
	///
	/// # Errors
	#[instrument(level = Level::INFO, skip_all)]
	pub async fn start(&mut self) -> Result<()> {
		event!(Level::INFO, "start");
		// activate sending liveliness
		#[cfg(feature = "unstable")]
		if self.liveliness {
			//let session = self.context.session("default")?;
			let token_str = self
				.context
				.prefix()
				.map_or(self.context.uuid(), |prefix| {
					format!("{}/{}", prefix, self.context.uuid())
				});

			let token = self
				.context
				.session("default")
				.ok_or_else(|| Error::Unexpected(file!().into(), line!()))?
				.liveliness()
				.declare_token(&token_str)
				.wait()
				.map_err(|source| Error::ActivateLiveliness { source })?;

			self.liveliness_token.write().replace(token);
		};

		// self.context
		// 	.set_state_old(OperationState::Active)?;

		self.manage_operation_state(OperationState::Active)?;

		self.run().await
	}

	/// run
	#[instrument(level = Level::DEBUG, skip_all)]
	async fn run(&mut self) -> Result<()> {
		event!(Level::DEBUG, "run");

		loop {
			// different possibilities that can happen
			select! {
				// `TaskSignal`s
				Some(signal) = self.rx.recv() => {
					match signal {
						#[cfg(feature = "unstable")]
						TaskSignal::RestartLiveliness(selector) => {
							self.context.liveliness_subscribers()
								.write()
								.get_mut(&selector)
								.ok_or(Error::GetMut("liveliness".into()))?
								.state_transitions(self.context.state_old())?;
						},
						TaskSignal::RestartQueryable(selector) => {
							self.context.responders()
								.write()
								.get_mut(&selector)
								.ok_or_else(|| Error::GetMut("queryables".into()))?
								.state_transitions(self.context.state_old())?;
						},
						TaskSignal::RestartObservable(selector) => {
							self.context.responders()
								.write()
								.get_mut(&selector)
								.ok_or_else(|| Error::GetMut("observables".into()))?
								.state_transitions(self.context.state_old())?;
						},
						TaskSignal::RestartSubscriber(selector) => {
							self.context.responders()
								.write()
								.get_mut(&selector)
								.ok_or_else(|| Error::GetMut("subscribers".into()))?
								.state_transitions(self.context.state_old())?;
						},
						TaskSignal::RestartTimer(selector) => {
							self.context.timers()
								.write()
								.get_mut(&selector)
								.ok_or_else(|| Error::GetMut("timers".into()))?
								.state_transitions(self.context.state_old())?;
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
	#[instrument(level = Level::INFO, skip_all)]
	pub fn stop(&mut self) -> Result<()> {
		event!(Level::INFO, "stop");
		self.context
			.set_state_old(OperationState::Created)?;

		// stop liveliness
		#[cfg(feature = "unstable")]
		if self.liveliness {
			self.liveliness_token.write().take();
		}
		Ok(())
	}
}
// endregion:   --- RunningAgent
