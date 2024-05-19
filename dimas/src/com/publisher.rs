// Copyright © 2023 Stephan Kunz

//! Module `publisher` provides a message sender `Publisher` which can be created using the `PublisherBuilder`.

// region:		--- modules
// these ones are only for doc needed
#[cfg(doc)]
use crate::agent::Agent;
use dimas_core::{
	error::{DimasError, Result},
	message_types::Message,
	traits::{Capability, CommunicationCapability, Context, OperationState},
};
#[cfg(doc)]
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use tracing::{instrument, Level};
use zenoh::{
	prelude::sync::SyncResolve,
	publication::{CongestionControl, Priority},
	SessionDeclarations,
};
// endregion:	--- modules

// region:		--- states
/// State signaling that the [`PublisherBuilder`] has no storage value set
pub struct NoStorage;
/// State signaling that the [`PublisherBuilder`] has the storage value set
pub struct Storage<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Thread safe reference to a [`HashMap`] to store the created [`Publisher`]
	pub storage: Arc<RwLock<std::collections::HashMap<String, Publisher<P>>>>,
}

/// State signaling that the [`PublisherBuilder`] has no key expression set
pub struct NoKeyExpression;
/// State signaling that the [`PublisherBuilder`] has the key expression set
pub struct KeyExpression {
	/// The key expression
	key_expr: String,
}
// endregion:	--- states

// region:		--- PublisherBuilder
/// The builder for a [`Publisher`]
#[allow(clippy::module_name_repetitions)]
pub struct PublisherBuilder<P, K, S>
where
	P: Send + Sync + Unpin + 'static,
{
	context: Context<P>,
	activation_state: OperationState,
	priority: Priority,
	congestion_control: CongestionControl,
	key_expr: K,
	storage: S,
}

impl<P> PublisherBuilder<P, NoKeyExpression, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a [`PublisherBuilder`] in initial state
	#[must_use]
	pub const fn new(context: Context<P>) -> Self {
		Self {
			context,
			activation_state: OperationState::Active,
			priority: Priority::Data,
			congestion_control: CongestionControl::Drop,
			key_expr: NoKeyExpression,
			storage: NoStorage,
		}
	}
}

impl<P, K, S> PublisherBuilder<P, K, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the activation state.
	#[must_use]
	pub const fn activation_state(mut self, state: OperationState) -> Self {
		self.activation_state = state;
		self
	}

	/// Set the publishers priority
	#[must_use]
	pub const fn set_priority(mut self, priority: Priority) -> Self {
		self.priority = priority;
		self
	}

	/// Set the publishers congestion control
	#[must_use]
	pub const fn set_congestion_control(mut self, congestion_control: CongestionControl) -> Self {
		self.congestion_control = congestion_control;
		self
	}
}

impl<P, K> PublisherBuilder<P, K, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Provide agents storage for the publisher
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Publisher<P>>>>,
	) -> PublisherBuilder<P, K, Storage<P>> {
		let Self {
			context,
			activation_state,
			priority,
			congestion_control,
			key_expr,
			..
		} = self;
		PublisherBuilder {
			context,
			activation_state,
			priority,
			congestion_control,
			key_expr,
			storage: Storage { storage },
		}
	}
}

impl<P, S> PublisherBuilder<P, NoKeyExpression, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full key expression for the [`Publisher`]
	#[must_use]
	pub fn key_expr(self, key_expr: &str) -> PublisherBuilder<P, KeyExpression, S> {
		let Self {
			context,
			activation_state,
			priority,
			congestion_control,
			storage,
			..
		} = self;
		PublisherBuilder {
			context,
			activation_state,
			priority,
			congestion_control,
			key_expr: KeyExpression {
				key_expr: key_expr.into(),
			},
			storage,
		}
	}

	/// Set only the message qualifing part of the [`Publisher`].
	/// Will be prefixed with [`Agent`]s prefix.
	#[must_use]
	pub fn topic(self, topic: &str) -> PublisherBuilder<P, KeyExpression, S> {
		let key_expr = self
			.context
			.prefix()
			.clone()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		let Self {
			context,
			activation_state,
			priority,
			congestion_control,
			storage,
			..
		} = self;
		PublisherBuilder {
			context,
			activation_state,
			priority,
			congestion_control,
			key_expr: KeyExpression { key_expr },
			storage,
		}
	}
}

impl<P, S> PublisherBuilder<P, KeyExpression, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the [`Publisher`]
	///
	/// # Errors
	/// Currently none
	pub fn build(self) -> Result<Publisher<P>> {
		Ok(Publisher::new(
			self.key_expr.key_expr,
			self.context,
			self.activation_state,
			self.priority,
			self.congestion_control,
		))
	}
}

impl<P> PublisherBuilder<P, KeyExpression, Storage<P>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the [Publisher] to the [`Agent`]s context
	///
	/// # Errors
	/// Currently none
	pub fn add(self) -> Result<Option<Publisher<P>>> {
		let collection = self.storage.storage.clone();
		let p = self.build()?;
		let r = collection
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(p.key_expr.to_string(), p);
		Ok(r)
	}
}
// endregion:	--- PublisherBuilder

// region:		--- Publisher
/// Publisher
pub struct Publisher<P>
where
	P: Send + Sync + Unpin + 'static,
{
	key_expr: String,
	/// Context for the Publisher
	context: Context<P>,
	activation_state: OperationState,
	priority: Priority,
	congestion_control: CongestionControl,
	publisher: Option<zenoh::publication::Publisher<'static>>,
}

impl<P> Debug for Publisher<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Publisher")
			.field("key_expr", &self.key_expr)
			.field("initialized", &self.publisher.is_some())
			.finish_non_exhaustive()
	}
}

impl<P> Capability for Publisher<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn manage_operation_state(&mut self, state: &OperationState) -> Result<()> {
		if state >= &self.activation_state {
			return self.init();
		} else if state < &self.activation_state {
			return self.de_init();
		}
		Ok(())
	}
}

impl<P> CommunicationCapability for Publisher<P> where P: Send + Sync + Unpin + 'static {}

impl<P> Publisher<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor for a [`Publisher`]
	#[must_use]
	pub const fn new(
		key_expr: String,
		context: Context<P>,
		activation_state: OperationState,
		priority: Priority,
		congestion_control: CongestionControl,
	) -> Self {
		Self {
			key_expr,
			context,
			activation_state,
			priority,
			congestion_control,
			publisher: None,
		}
	}

	/// Get `key_expr`
	#[must_use]
	pub fn key_expr(&self) -> &str {
		&self.key_expr
	}

	/// Initialize
	/// # Errors
	///
	fn init(&mut self) -> Result<()>
	where
		P: Send + Sync + Unpin + 'static,
	{
		let publ = self
			.context
			.session()
			.declare_publisher(self.key_expr.clone())
			.congestion_control(self.congestion_control)
			.priority(self.priority)
			.res_sync()?;
		//.map_err(|_| DimasError::Put.into())?;
		self.publisher.replace(publ);
		Ok(())
	}

	/// De-Initialize
	/// # Errors
	///
	#[allow(clippy::unnecessary_wraps)]
	fn de_init(&mut self) -> Result<()> {
		self.publisher.take();
		Ok(())
	}

	/// Send a "put" message
	/// # Errors
	///
	#[instrument(name="publish", level = Level::ERROR, skip_all)]
	pub fn put(&self, message: Message) -> Result<()> {
		match self
			.publisher
			.clone()
			.ok_or(DimasError::ShouldNotHappen)?
			.put(message.0)
			.res_sync()
		{
			Ok(()) => Ok(()),
			Err(_) => Err(DimasError::Put.into()),
		}
	}

	/// Send a "delete" message
	/// # Errors
	///
	#[instrument(level = Level::ERROR, skip_all)]
	pub fn delete(&self) -> Result<()> {
		match self
			.publisher
			.clone()
			.ok_or(DimasError::ShouldNotHappen)?
			.delete()
			.res_sync()
		{
			Ok(()) => Ok(()),
			Err(_) => Err(DimasError::Delete.into()),
		}
	}
}
// endregion:	--- Publisher

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Publisher<Props>>();
		is_normal::<PublisherBuilder<Props, NoKeyExpression, NoStorage>>();
	}
}
