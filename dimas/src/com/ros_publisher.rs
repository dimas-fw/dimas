// Copyright © 2024 Stephan Kunz

// region:		--- modules
use crate::prelude::*;
use std::fmt::Debug;
// endregion:	--- modules

// region:		--- states
/// State signaling that the [`RosPublisherBuilder`] has no storage value set
pub struct NoStorage;
/// State signaling that the [`RosPublisherBuilder`] has the storage value set
#[cfg(feature = "ros_publisher")]
pub struct Storage {
	/// Thread safe reference to a [`HashMap`] to store the created [`RosPublisher`]
	pub storage: Arc<RwLock<std::collections::HashMap<String, RosPublisher>>>,
}

/// State signaling that the [`RosPublisherBuilder`] has no key expression set
pub struct NoKeyExpression;
/// State signaling that the [`RosPublisherBuilder`] has the key expression set
pub struct KeyExpression {
	/// The key expression
	key_expr: String,
}
// endregion:	--- states

// region:		--- RosPublisherBuilder
/// `RosPublisherBuilder`
#[allow(clippy::module_name_repetitions)]
pub struct RosPublisherBuilder<K, S> {
	prefix: Option<String>,
	pub(crate) key_expr: K,
	pub(crate) storage: S,
}

impl RosPublisherBuilder<NoKeyExpression, NoStorage> {
	/// Construct a `RosPublisherBuilder` in initial state
	#[must_use]
	pub const fn new(prefix: Option<String>) -> Self {
		Self {
			prefix,
			key_expr: NoKeyExpression,
			storage: NoStorage,
		}
	}
}

#[cfg(feature = "ros_publisher")]
impl<K> RosPublisherBuilder<K, NoStorage> {
	/// Provide agents storage for the publisher
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, RosPublisher>>>,
	) -> RosPublisherBuilder<K, Storage> {
		let Self {
			prefix, key_expr, ..
		} = self;
		RosPublisherBuilder {
			prefix,
			key_expr,
			storage: Storage { storage },
		}
	}
}

impl<S> RosPublisherBuilder<NoKeyExpression, S> {
	/// Set the full key expression for the [`RosPublisher`]
	#[must_use]
	pub fn key_expr(self, key_expr: &str) -> RosPublisherBuilder<KeyExpression, S> {
		let Self {
			prefix, storage, ..
		} = self;
		RosPublisherBuilder {
			prefix,
			key_expr: KeyExpression {
				key_expr: key_expr.into(),
			},
			storage,
		}
	}

	/// Set only the message qualifing part of the [`Publisher`].
	/// Will be prefixed with [`Agent`]s prefix.
	#[must_use]
	pub fn topic(mut self, topic: &str) -> RosPublisherBuilder<KeyExpression, S> {
		let key_expr = self
			.prefix
			.take()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		let Self {
			prefix, storage, ..
		} = self;
		RosPublisherBuilder {
			prefix,
			key_expr: KeyExpression { key_expr },
			storage,
		}
	}
}

impl<S> RosPublisherBuilder<KeyExpression, S> {
	/// Build the [`RosPublisher`]
	///
	/// # Errors
	/// Currently none
	pub fn build(self) -> Result<RosPublisher> {
		Ok(RosPublisher {
			key_expr: self.key_expr.key_expr,
		})
	}
}

#[cfg(feature = "ros_publisher")]
impl RosPublisherBuilder<KeyExpression, Storage> {
	/// Build and add the [`RosPublisher`] to the [`Agent`]s context
	///
	/// # Errors
	/// Currently none
	#[cfg_attr(any(nightly, docrs), doc, doc(cfg(feature = "ros_publisher")))]
	pub fn add(self) -> Result<Option<RosPublisher>> {
		let collection = self.storage.storage.clone();
		let p = self.build()?;
		let r = collection
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(p.key_expr.to_string(), p);
		Ok(r)
	}
}
// endregion:	--- RosPublisherBuilder

// region:		--- RosPublisher
/// `RosPublisher`
pub struct RosPublisher {
	pub(crate) key_expr: String,
}

impl Debug for RosPublisher {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("RosPublisher")
			.field("key_expr", &self.key_expr)
			//.field("initialized", &self.publisher.is_some())
			.finish_non_exhaustive()
	}
}
// endregion:	--- RosPublisher

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<RosPublisher>();
		is_normal::<RosPublisherBuilder<NoKeyExpression, NoStorage>>();
	}
}
