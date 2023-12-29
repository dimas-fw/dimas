//! Copyright © 2023 Stephan Kunz

// region:    --- modules
use super::communicator::Communicator;
use crate::prelude::*;
use std::sync::Arc;
use zenoh::sample::Sample;
// endregion: --- modules

// region:    --- types
pub type SubscriberCallback = fn(Sample);
// endregion: --- types

// region:    --- SubscriberBuilder
#[derive(Default, Clone)]
pub struct SubscriberBuilder<'a> {
	communicator: Option<Arc<Communicator<'a>>>,
	key_expr: Option<String>,
	msg_type: Option<String>,
	callback: Option<SubscriberCallback>,
}

impl<'a> SubscriberBuilder<'a> {
	pub fn communicator(mut self, communicator: Arc<Communicator<'a>>) -> Self {
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

	pub(crate) fn build(mut self) -> Result<()> {
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
			self.communicator.clone().unwrap().prefix() + "/" + &self.msg_type.unwrap() + "/**"
		};
		//dbg!(&key_expr);
		self.communicator
			.unwrap()
			.add_subscriber(&key_expr, self.callback.take().unwrap());
		Ok(())
	}

	pub fn add(self) -> Result<()> {
		self.build()
	}
}
// endregion: --- SubscriberBuilder

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
		let _subscriber = SubscriberBuilder::default().build().unwrap();
		//assert!(subscriber.context().session());
	}
}
