// Copyright © 2023 Stephan Kunz

//! Module `publisher` provides a message sender `Publisher` which can be created using the `PublisherBuilder`.

// region:		--- modules
// these ones are only for doc needed
#[cfg(doc)]
use crate::agent::Agent;
use crate::context::ArcContext;
use bitcode::{encode, Encode};
use dimas_core::error::{DimasError, Result};
#[cfg(doc)]
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use tracing::{instrument, Level};
use zenoh::{
	prelude::sync::SyncResolve,
	publication::{CongestionControl, Priority},
};
// endregion:	--- modules

// region:		--- states
/// State signaling that the [`PublisherBuilder`] has no storage value set
pub struct NoStorage;
/// State signaling that the [`PublisherBuilder`] has the storage value set
pub struct Storage {
	/// Thread safe reference to a [`HashMap`] to store the created [`Publisher`]
	pub storage: Arc<RwLock<std::collections::HashMap<String, Publisher>>>,
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
pub struct PublisherBuilder<K, S> {
	prefix: Option<String>,
	priority: Priority,
	congestion_control: CongestionControl,
	pub(crate) key_expr: K,
	pub(crate) storage: S,
}

impl PublisherBuilder<NoKeyExpression, NoStorage> {
	/// Construct a [`PublisherBuilder`] in initial state
	#[must_use]
	pub const fn new(prefix: Option<String>) -> Self {
		Self {
			prefix,
			priority: Priority::Data,
			congestion_control: CongestionControl::Drop,
			key_expr: NoKeyExpression,
			storage: NoStorage,
		}
	}
}

impl<K, S> PublisherBuilder<K, S> {
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

impl<K> PublisherBuilder<K, NoStorage> {
	/// Provide agents storage for the publisher
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Publisher>>>,
	) -> PublisherBuilder<K, Storage> {
		let Self {
			prefix,
			priority,
			congestion_control,
			key_expr,
			..
		} = self;
		PublisherBuilder {
			prefix,
			priority,
			congestion_control,
			key_expr,
			storage: Storage { storage },
		}
	}
}

impl<S> PublisherBuilder<NoKeyExpression, S> {
	/// Set the full key expression for the [`Publisher`]
	#[must_use]
	pub fn key_expr(self, key_expr: &str) -> PublisherBuilder<KeyExpression, S> {
		let Self {
			prefix,
			priority,
			congestion_control,
			storage,
			..
		} = self;
		PublisherBuilder {
			prefix,
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
	pub fn topic(mut self, topic: &str) -> PublisherBuilder<KeyExpression, S> {
		let key_expr = self
			.prefix
			.take()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		let Self {
			prefix,
			priority,
			congestion_control,
			storage,
			..
		} = self;
		PublisherBuilder {
			prefix,
			priority,
			congestion_control,
			key_expr: KeyExpression { key_expr },
			storage,
		}
	}
}

impl<S> PublisherBuilder<KeyExpression, S> {
	/// Build the [`Publisher`]
	///
	/// # Errors
	/// Currently none
	pub fn build(self) -> Result<Publisher> {
		Ok(Publisher::new(
			self.key_expr.key_expr,
			self.priority,
			self.congestion_control,
		))
	}
}

impl PublisherBuilder<KeyExpression, Storage> {
	/// Build and add the [Publisher] to the [`Agent`]s context
	///
	/// # Errors
	/// Currently none
	pub fn add(self) -> Result<Option<Publisher>> {
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
pub struct Publisher {
	pub(crate) key_expr: String,
	priority: Priority,
	congestion_control: CongestionControl,
	publisher: Option<zenoh::publication::Publisher<'static>>,
}

impl Debug for Publisher {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Publisher")
			.field("key_expr", &self.key_expr)
			.field("initialized", &self.publisher.is_some())
			.finish_non_exhaustive()
	}
}

impl Publisher
//where
//	P: Send + Sync + Unpin + 'a,
{
	/// Constructor for a [`Publisher`]
	#[must_use]
	pub const fn new(
		key_expr: String,
		priority: Priority,
		congestion_control: CongestionControl,
	) -> Self {
		Self {
			key_expr,
			priority,
			congestion_control,
			publisher: None,
		}
	}

	/// Initialize
	/// # Errors
	///
	pub fn init<P>(&mut self, context: &ArcContext<P>) -> Result<()>
	where
		P: Send + Sync + Unpin + 'static,
	{
		let publ = context
			.communicator
			.create_publisher(&self.key_expr)?
			.congestion_control(self.congestion_control)
			.priority(self.priority);
		self.publisher.replace(publ);
		Ok(())
	}

	/// De-Initialize
	pub fn de_init(&mut self) {
		self.publisher.take();
	}

	/// Send a "put" message
	/// # Errors
	///
	#[instrument(name="publish", level = Level::ERROR, skip_all)]
	pub fn put<T>(&self, message: T) -> Result<()>
	where
		T: Debug + Encode,
	{
		let value: Vec<u8> = encode(&message);
		match self
			.publisher
			.clone()
			.ok_or(DimasError::ShouldNotHappen)?
			.put(value)
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

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Publisher>();
		is_normal::<PublisherBuilder<NoKeyExpression, NoStorage>>();
	}
}
