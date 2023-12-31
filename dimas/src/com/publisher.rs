//! Copyright © 2023 Stephan Kunz

// region:    --- modules
use super::communicator::Communicator;
use crate::prelude::*;
use std::sync::{Arc, RwLock};
// endregion: --- modules

// region:    --- types
//pub type PublisherCallback<P> = fn(Arc<Context>, Arc<RwLock<P>>, sample: Sample);
// endregion: --- types

// region:    --- PublisherBuilder
#[derive(Default, Clone)]
pub struct PublisherBuilder<'a> {
	collection: Option<Arc<RwLock<Vec<Publisher<'a>>>>>,
	communicator: Option<Arc<Communicator>>,
	key_expr: Option<String>,
	msg_type: Option<String>,
}

impl<'a> PublisherBuilder<'a> {
	pub fn collection(mut self, collection: Arc<RwLock<Vec<Publisher<'a>>>>) -> Self {
		self.collection.replace(collection);
		self
	}

	pub fn communicator(mut self, communicator: Arc<Communicator>) -> Self {
		self.communicator.replace(communicator);
		self
	}

	pub fn key_expr(mut self, key_expr: impl Into<String>) -> Self {
		self.key_expr.replace(key_expr.into());
		self
	}

	pub async fn add(mut self) -> Result<()> {
		if self.collection.is_none() {
			return Err("No collection given".into());
		}
		if self.communicator.is_none() {
			return Err("No communicator given".into());
		}
		if self.key_expr.is_none() && self.msg_type.is_none() {
			return Err("No key expression or msg type given".into());
		}

		let key_expr = if self.key_expr.is_some() {
			self.key_expr.take().unwrap()
		} else {
			let c = self.communicator.clone().unwrap();
			c.prefix() + "/" + &self.msg_type.unwrap() + "/" + &c.uuid()
		};
		//dbg!(&key_expr);
		let publ = self
			.communicator
			.take()
			.unwrap()
			.create_publisher(key_expr)
			.await;
		let p = Publisher { _publisher: publ };
		let c = self.collection.take();
		c.unwrap().write().unwrap().push(p);
		Ok(())
	}
}
// endregion: --- PublisherBuilder

// region:    --- Publisher
pub struct Publisher<'a> {
	_publisher: zenoh::publication::Publisher<'a>,
}
// endregion: --- Publisher

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	fn normal_types() {
		is_normal::<Publisher>();
		is_normal::<PublisherBuilder>();
	}

	#[test]
	fn publisher_create() {
		let _builder = PublisherBuilder::default();
	}
}
