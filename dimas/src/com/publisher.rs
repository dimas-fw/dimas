//! Copyright © 2023 Stephan Kunz

use zenoh::publication::Publisher;

// region:    --- modules
use super::communicator::Communicator;
use crate::prelude::*;
use std::sync::{Arc, RwLock};
// endregion: --- modules

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

	pub(crate) fn build(mut self) -> Result<Publisher<'a>> {
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
		let s = self.communicator.unwrap().publisher(&key_expr);
		Ok(s)
	}

	pub fn add(mut self) -> Result<()> {
		if self.collection.is_none() {
			return Err("No collection given".into());
		}

		let c = self.collection.take();
		let subscriber = self.build()?;
		c.unwrap().write().unwrap().push(subscriber);
		Ok(())
	}
}
// endregion: --- PublisherBuilder

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	fn normal_types() {
		is_normal::<PublisherBuilder>();
	}

	#[test]
	fn publisher_create() {
		let _builder = PublisherBuilder::default();
	}
}
