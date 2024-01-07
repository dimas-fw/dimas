//! Copyright © 2023 Stephan Kunz

// region:		--- modules
use crate::{com::communicator::Communicator, prelude::*};
use serde::*;
use std::sync::Arc;
// endregion:	--- modules

// region:		--- Context
#[derive(Debug, Clone, Default)]
pub struct Context {
	pub communicator: Arc<Communicator>,
}

impl Context {
	pub fn uuid(&self) -> String {
		self.communicator.uuid()
	}

	pub fn publish<P>(&self, msg_name: impl Into<String>, message: P) -> Result<()>
	where
		P: Serialize,
	{
		self.communicator.publish(msg_name, message)
	}

	pub async fn query<Q>(&self, _query_name: impl Into<String>, _message: Q) -> Result<()>
	where
		Q: Serialize,
	{
		todo!("not yet implemented")
	}
}
// endregion:	--- Context

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
