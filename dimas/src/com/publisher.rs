//! Copyright © 2023 Stephan Kunz

// region:    --- modules
use super::communicator::Communicator;
use crate::prelude::*;
use std::sync::Arc;
// endregion: --- modules

// region:    --- PublisherBuilder
#[derive(Default, Clone)]
pub struct PublisherBuilder<'a> {
	communicator: Option<Arc<Communicator<'a>>>,
	key_expr: Option<String>,
}

impl<'a> PublisherBuilder<'a> {
	pub fn communicator(mut self, communicator: Arc<Communicator<'a>>) -> Self {
		self.communicator.replace(communicator);
		self
	}

	pub fn key_expr(mut self, key_expr: impl Into<String>) -> Self {
		self.key_expr.replace(key_expr.into());
		self
	}

	pub(crate) fn build(self) -> Result<()> {
		if self.communicator.is_none() {
			return Err("No communicator given".into());
		}
		if self.key_expr.is_none() {
			return Err("No key expression given".into());
		}
		Ok(())
	}

	pub fn _add(self) -> Result<()> {
		self.build()
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
		let _publisher = PublisherBuilder::default().build().unwrap();
		//assert!(publisher.context().session());
	}
}
