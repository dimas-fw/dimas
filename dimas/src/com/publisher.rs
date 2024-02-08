// Copyright © 2023 Stephan Kunz

// region:		--- modules
use super::communicator::Communicator;
use crate::prelude::*;
use std::sync::{Arc, RwLock};
// endregion:	--- modules

// region:		--- types
//#[allow(clippy::module_name_repetitions)]
//pub type PublisherCallback<P> = fn(Arc<Context>, Arc<RwLock<P>>, sample: Sample);
// endregion:	--- types

// region:		--- PublisherBuilder
/// The builder for a publisher
#[allow(clippy::module_name_repetitions)]
#[derive(Default, Clone)]
pub struct PublisherBuilder<'a, P> {
	pub(crate) collection: Arc<RwLock<Vec<Publisher<'a>>>>,
	pub(crate) communicator: Arc<Communicator>,
	pub(crate) props: Arc<RwLock<P>>,
	pub(crate) key_expr: Option<String>,
}

impl<'a, P> PublisherBuilder<'a, P>
where
	P: Send + Sync + Unpin + 'static,
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
		let key_expr = self.communicator.clone().prefix() + "/" + &msg_type.into();
		self.key_expr.replace(key_expr);
		self
	}

	/// Build the publisher
	/// # Errors
	///
	/// # Panics
	///
	pub async fn build(mut self) -> Result<Publisher<'a>> {
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
		let publ = self.communicator.create_publisher(key_expr).await;
		let p = Publisher { _publisher: publ };

		Ok(p)
	}

	/// Build and add the publisher to the agent
	/// # Errors
	///
	/// # Panics
	///
	pub async fn add(self) -> Result<()> {
		let collection = self.collection.clone();
		let p = self.build().await?;
		collection
			.write()
			.expect("should never happen")
			.push(p);
		Ok(())
	}
}
// endregion:	--- PublisherBuilder

// region:		--- Publisher
/// Publisher
pub struct Publisher<'a> {
	_publisher: zenoh::publication::Publisher<'a>,
}
// endregion:	--- Publisher

#[cfg(test)]
mod tests {
	use super::*;

	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Publisher>();
		is_normal::<PublisherBuilder<Props>>();
	}
}
