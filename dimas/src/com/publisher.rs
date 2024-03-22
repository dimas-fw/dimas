// Copyright © 2023 Stephan Kunz

//! Module `publisher` provides a message sender `Publisher` which can be created using the `PublisherBuilder`.

// region:		--- modules
use crate::prelude::*;
#[allow(unused_imports)]
use std::collections::HashMap;
use std::fmt::Debug;
use tracing::{instrument, Level};
use zenoh::prelude::sync::SyncResolve;
// endregion:	--- modules

// region:		--- states
/// State signaling that the [`PublisherBuilder`] has no storage value set
pub struct NoStorage;
#[cfg(feature = "publisher")]
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
/// The builder for a publisher
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct PublisherBuilder<K, S> {
	prefix: Option<String>,
	pub(crate) key_expr: K,
	pub(crate) storage: S,
}

impl PublisherBuilder<NoKeyExpression, NoStorage> {
	/// Construct a `PublisherBuilder` in initial state
	#[must_use]
	pub const fn new(prefix: Option<String>) -> Self {
		Self {
			prefix,
			key_expr: NoKeyExpression,
			storage: NoStorage,
		}
	}
}

#[cfg(feature = "publisher")]
impl<K> PublisherBuilder<K, NoStorage> {
	/// Provide agents storage for the publisher
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Publisher>>>,
	) -> PublisherBuilder<K, Storage> {
		let Self {
			prefix, key_expr, ..
		} = self;
		PublisherBuilder {
			prefix,
			key_expr,
			storage: Storage { storage },
		}
	}
}

impl<S> PublisherBuilder<NoKeyExpression, S> {
	/// Set the full expression for the publisher
	#[must_use]
	pub fn key_expr(self, key_expr: &str) -> PublisherBuilder<KeyExpression, S> {
		let Self {
			prefix, storage, ..
		} = self;
		PublisherBuilder {
			prefix,
			key_expr: KeyExpression {
				key_expr: key_expr.into(),
			},
			storage,
		}
	}

	/// Set only the message qualifing part of the publisher.
	/// Will be prefixed with agents prefix.
	#[must_use]
	pub fn topic(mut self, topic: &str) -> PublisherBuilder<KeyExpression, S> {
		let key_expr = self
			.prefix
			.take()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		let Self {
			prefix, storage, ..
		} = self;
		PublisherBuilder {
			prefix,
			key_expr: KeyExpression { key_expr },
			storage,
		}
	}
}

impl<S> PublisherBuilder<KeyExpression, S> {
	/// Build the publisher
	/// # Errors
	///
	pub fn build(self) -> Result<Publisher> {
		dbg!(&self.key_expr.key_expr);
		Ok(Publisher {
			key_expr: self.key_expr.key_expr,
			publisher: None,
		})
	}
}

#[cfg(feature = "publisher")]
impl PublisherBuilder<KeyExpression, Storage> {
	/// Build and add the publisher to the agents context
	/// # Errors
	///
	#[cfg_attr(any(nightly, docrs), doc, doc(cfg(feature = "publisher")))]
	#[cfg(feature = "publisher")]
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
	/// Initialize
	/// # Errors
	pub fn init<P>(&mut self, context: &ArcContext<P>) -> Result<()>
	where
		P: Send + Sync + Unpin + 'static,
	{
		let publ = context.create_publisher(&self.key_expr)?;
		self.publisher.replace(publ);
		Ok(())
	}

	/// De-Initialize
	/// # Errors
	pub fn de_init(&mut self) -> Result<()> {
		self.publisher.take();
		Ok(())
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
			.expect("snh")
			.put(value)
			.res_sync()
		{
			Ok(()) => Ok(()),
			Err(_) => Err(DimasError::PutMessage.into()),
		}
	}

	/// Send a "delete" message - method currently does not work!!
	/// # Errors
	///
	#[instrument(level = Level::ERROR, skip_all)]
	pub fn delete(&self) -> Result<()> {
		match self
			.publisher
			.clone()
			.expect("snh")
			.delete()
			.res_sync()
		{
			Ok(()) => Ok(()),
			Err(_) => Err(DimasError::DeleteMessage.into()),
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
		is_normal::<Publisher>();
		is_normal::<PublisherBuilder<NoKeyExpression, NoStorage>>();
	}
}
