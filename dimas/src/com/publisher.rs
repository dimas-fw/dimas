// Copyright © 2023 Stephan Kunz

//! Module `publisher` provides a message sender `Publisher` which can be created using the `PublisherBuilder`.

// region:		--- modules
use crate::prelude::*;
use std::fmt::Debug;
use zenoh::prelude::sync::SyncResolve;
// endregion:	--- modules

// region:		--- PublisherBuilder
/// The builder for a publisher
#[allow(clippy::module_name_repetitions)]
#[derive(Default, Clone)]
pub struct PublisherBuilder<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	pub(crate) context: ArcContext<P>,
	pub(crate) key_expr: Option<String>,
}

impl<P> PublisherBuilder<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// Set the full expression for the publisher
	#[must_use]
	pub fn key_expr(mut self, key_expr: impl Into<String>) -> Self {
		self.key_expr.replace(key_expr.into());
		self
	}

	/// Set only the message qualifing part of the query.
	/// Will be prefixed with agents prefix.
	#[must_use]
	pub fn msg_type(mut self, msg_type: impl Into<String>) -> Self {
		let key_expr = self.context.key_expr(msg_type);
		self.key_expr.replace(key_expr);
		self
	}

	/// Build the publisher
	/// # Errors
	///
	pub fn build(self) -> Result<Publisher, DimasError> {
		let key_expr = if self.key_expr.is_none() {
			return Err(DimasError::NoKeyExpression);
		} else {
			self.key_expr.ok_or(DimasError::ShouldNotHappen)?
		};

		//dbg!(&key_expr);
		let publ = self.context.create_publisher(key_expr);
		let p = Publisher { publisher: publ };

		Ok(p)
	}

	/// Build and add the publisher to the agents context
	/// # Errors
	///
	#[cfg_attr(any(nightly, docrs), doc, doc(cfg(feature = "publisher")))]
	#[cfg(feature = "publisher")]
	pub fn add(self) -> Result<(), DimasError> {
		let collection = self.context.publishers.clone();
		let p = self.build()?;
		collection
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(p.publisher.key_expr().to_string(), p);
		Ok(())
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
	/// # Panics
	///
	#[tracing::instrument(level = tracing::Level::DEBUG)]
	pub fn put<T>(&self, message: T) -> Result<(), DimasError>
	where
		T: Debug + Encode,
	{
		let value: Vec<u8> = encode(&message).map_err(|_| DimasError::EncodingFailed)?;
		match self.publisher.put(value).res_sync() {
			Ok(()) => Ok(()),
			Err(_) => Err(DimasError::PutFailed),
		}
	}

	// TODO! This currently does not work - it sends a put message
	/// Send a "delete" message - method currently does not work!!
	/// # Errors
	///
	/// # Panics
	///
	#[tracing::instrument(level = tracing::Level::DEBUG)]
	pub fn delete(&self) -> Result<(), DimasError> {
		match self.publisher.delete().res_sync() {
			Ok(()) => Ok(()),
			Err(_) => Err(DimasError::DeleteFailed),
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
		is_normal::<PublisherBuilder<Props>>();
	}
}
