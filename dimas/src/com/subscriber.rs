//! Copyright © 2023 Stephan Kunz

// region:    --- modules
use super::communicator::Communicator;
use crate::prelude::*;
use std::sync::{Arc, RwLock};
use zenoh::{sample::Sample, subscriber::Subscriber};
// endregion: --- modules

// region:    --- types
pub type SubscriberCallback = fn(Sample);
// endregion: --- types

// region:    --- SubscriberBuilder
#[derive(Default, Clone)]
pub struct SubscriberBuilder<'a> {
	collection: Option<Arc<RwLock<Vec<Subscriber<'a, ()>>>>>,
	communicator: Option<Arc<Communicator>>,
	key_expr: Option<String>,
	msg_type: Option<String>,
	callback: Option<SubscriberCallback>,
}

impl<'a> SubscriberBuilder<'a> {
	pub fn collection(mut self, collection: Arc<RwLock<Vec<Subscriber<'a, ()>>>>) -> Self {
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

	pub fn msg_type(mut self, msg_type: impl Into<String>) -> Self {
		self.msg_type.replace(msg_type.into());
		self
	}

	pub fn callback(mut self, callback: SubscriberCallback) -> Self {
		self.callback.replace(callback);
		self
	}

	pub(crate) fn build(mut self) -> Result<Subscriber<'a, ()>> {
		if self.communicator.is_none() {
			return Err("No communicator given".into());
		}
		if self.key_expr.is_none() && self.msg_type.is_none() {
			return Err("No key expression or msg type given".into());
		}
		if self.callback.is_none() {
			return Err("No callback given".into());
		}
		let key_expr = if self.key_expr.is_some() {
			self.key_expr.take().unwrap()
		} else {
			self.communicator.clone().unwrap().prefix() + "/" + &self.msg_type.unwrap() + "/*"
		};
		//dbg!(&key_expr);
		let s = self
			.communicator
			.unwrap()
			.subscriber(&key_expr, self.callback.take().unwrap());

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
// endregion: --- SubscriberBuilder

// region:    --- Subscriber
//pub struct Subscriber {}
// endregion: --- Subscriber

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	fn normal_types() {
		is_normal::<SubscriberBuilder>();
	}

	#[test]
	fn subscriber_create() {
		let _builder = SubscriberBuilder::default();
	}
}
