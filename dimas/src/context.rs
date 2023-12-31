//! Copyright © 2023 Stephan Kunz

// region:    --- modules
use crate::{com::communicator::Communicator, prelude::*};
use serde::Serialize;
use std::sync::Arc;
// endregion: --- modules

// region:    --- Context
#[derive(Debug, Clone, Default)]
pub struct Context {
	pub communicator: Arc<Communicator>,
}

impl Context {
	pub fn publish<T>(&self, msg_name: impl Into<String>, message: T) -> Result<()>
	where
		T: Serialize,
	{
		self.communicator.publish(msg_name, message)
	}

	pub fn query<T>(&self, msg_name: impl Into<String>, message: T) -> Result<()>
	where
		T: Serialize,
	{
		self.communicator.publish(msg_name, message)
	}
}
// endregion: --- Context

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	fn normal_types() {
		is_normal::<Context>();
	}
}
