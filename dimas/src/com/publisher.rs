// Copyright © 2023 Stephan Kunz

// region:		--- modules
use crate::{context::Context, error::Result};
use std::{
	collections::HashMap,
	fmt::Debug,
	sync::{Arc, RwLock},
};
use zenoh::prelude::sync::SyncResolve;
// endregion:	--- modules

// region:		--- types
//#[allow(clippy::module_name_repetitions)]
//pub type PublisherCallback<P> = fn(Arc<Context<P>>, Arc<RwLock<P>>, sample: Sample);
// endregion:	--- types

// region:		--- PublisherBuilder
/// The builder for a publisher
#[allow(clippy::module_name_repetitions)]
#[derive(Default, Clone)]
pub struct PublisherBuilder<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	pub(crate) collection: Arc<RwLock<HashMap<String, Publisher>>>,
	pub(crate) context: Arc<Context<P>>,
	pub(crate) props: Arc<RwLock<P>>,
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
	/// # Panics
	///
	pub fn build(mut self) -> Result<Publisher> {
		if self.key_expr.is_none() {
			return Err("No key expression or msg type given".into());
		}

		let key_expr = if self.key_expr.is_some() {
			self.key_expr.take().expect("should never happen")
		} else {
			String::new()
		};

		//dbg!(&key_expr);
		let _props = self.props.clone();
		let publ = self.context.create_publisher(key_expr);
		let p = Publisher { publisher: publ };

		Ok(p)
	}

	/// Build and add the publisher to the agent
	/// # Errors
	///
	/// # Panics
	///
	pub fn add(self) -> Result<()> {
		let collection = self.collection.clone();
		let p = self.build()?;
		collection
			.write()
			.expect("should never happen")
			.insert(p.publisher.key_expr().to_string(), p);
		Ok(())
	}
}
// endregion:	--- PublisherBuilder

// region:		--- Publisher
/// Publisher
#[derive(Debug)]
pub struct Publisher {
	publisher: zenoh::publication::Publisher<'static>,
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
	pub fn put<T>(&self, message: T) -> Result<()>
	where
		T: Debug + bitcode::Encode,
	{
		let value: Vec<u8> = bitcode::encode(&message).expect("should never happen");
		//let _ = self.publisher.put(value).res_sync();
		match self.publisher.put(value).res_sync() {
			Ok(()) => Ok(()),
			Err(_) => Err("Publish failed".into()),
		}
	}

	// TODO!
	/// Send a "delete" message - method currently does not work!!
	/// # Errors
	///
	/// # Panics
	///
	#[tracing::instrument(level = tracing::Level::DEBUG)]
	pub fn delete(&self) -> Result<()> {
		match self.publisher.delete().res_sync() {
			Ok(()) => Ok(()),
			Err(_) => Err("Delete failed".into()),
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
