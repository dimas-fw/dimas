// Copyright © 2023 Stephan Kunz

//! Module `publisher` provides a message sender `Publisher` which can be created using the `PublisherBuilder`.

// region:		--- modules
use crate::prelude::*;
use std::fmt::Debug;
use tracing::{instrument, Level};
use zenoh::prelude::sync::SyncResolve;
// endregion:	--- modules

// region:		--- states
pub struct NoStorage;
#[cfg(feature = "publisher")]
pub struct Storage {
	pub storage: Arc<RwLock<std::collections::HashMap<String, Publisher>>>,
}

pub struct NoKeyExpression;
pub struct KeyExpression {
	key_expr: String,
}
// endregion:	--- states

// region:		--- PublisherBuilder
/// The builder for a publisher
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct PublisherBuilder<P, K, S>
where
	P: Send + Sync + Unpin + 'static,
{
	pub(crate) context: ArcContext<P>,
	pub(crate) key_expr: K,
	pub(crate) storage: S,
}

impl<P> PublisherBuilder<P, NoKeyExpression, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a `PublisherBuilder` in initial state
	#[must_use]
	pub const fn new(context: ArcContext<P>) -> Self {
		Self {
			context,
			key_expr: NoKeyExpression,
			storage: NoStorage,
		}
	}
}

#[cfg(feature = "publisher")]
impl<P, K> PublisherBuilder<P, K, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Provide agents storage for the publisher
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Publisher>>>,
	) -> PublisherBuilder<P, K, Storage> {
		let Self {
			context, key_expr, ..
		} = self;
		PublisherBuilder {
			context,
			key_expr,
			storage: Storage { storage },
		}
	}
}

impl<P, S> PublisherBuilder<P, NoKeyExpression, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full expression for the publisher
	#[must_use]
	pub fn key_expr(self, key_expr: &str) -> PublisherBuilder<P, KeyExpression, S> {
		let Self {
			context, storage, ..
		} = self;
		PublisherBuilder {
			context,
			key_expr: KeyExpression {
				key_expr: key_expr.into(),
			},
			storage,
		}
	}

	/// Set only the message qualifing part of the publisher.
	/// Will be prefixed with agents prefix.
	#[must_use]
	pub fn topic(self, topic: &str) -> PublisherBuilder<P, KeyExpression, S> {
		let key_expr = self.context.key_expr(topic);
		let Self {
			context, storage, ..
		} = self;
		PublisherBuilder {
			context,
			key_expr: KeyExpression { key_expr },
			storage,
		}
	}
}

impl<P, S> PublisherBuilder<P, KeyExpression, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the publisher
	/// # Errors
	///
	pub fn build(self) -> Result<Publisher> {
		let publ = self
			.context
			.create_publisher(&self.key_expr.key_expr)?;
		Ok(Publisher { publisher: publ })
	}
}

#[cfg(feature = "publisher")]
impl<P> PublisherBuilder<P, KeyExpression, Storage>
where
	P: Send + Sync + Unpin + 'static,
{
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
			.insert(p.publisher.key_expr().to_string(), p);
		Ok(r)
	}
}
// endregion:	--- PublisherBuilder

// region:		--- Publisher
/// Publisher
pub struct Publisher {
	publisher: zenoh::publication::Publisher<'static>,
}

impl Debug for Publisher {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Publisher")
			.field("key_expr", &self.publisher.key_expr())
			.finish_non_exhaustive()
	}
}

impl Publisher
//where
//	P: Send + Sync + Unpin + 'a,
{
	/// Send a "put" message
	/// # Errors
	///
	#[instrument(name="publish", level = Level::ERROR, skip_all)]
	pub fn put<T>(&self, message: T) -> Result<()>
	where
		T: Debug + Encode,
	{
		let value: Vec<u8> = encode(&message);
		match self.publisher.put(value).res_sync() {
			Ok(()) => Ok(()),
			Err(_) => Err(DimasError::PutMessage.into()),
		}
	}

	// TODO! This currently does not work - it sends a put message
	/// Send a "delete" message - method currently does not work!!
	/// # Errors
	///
	#[instrument(level = Level::ERROR, skip_all)]
	pub fn delete(&self) -> Result<()> {
		match self.publisher.delete().res_sync() {
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
		is_normal::<PublisherBuilder<Props, NoKeyExpression, NoStorage>>();
	}
}
