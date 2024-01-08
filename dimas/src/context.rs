//! Copyright © 2023 Stephan Kunz

// region:		--- modules
use crate::{
	com::{communicator::Communicator, query::QueryCallback},
	prelude::*,
};
use serde::*;
use std::sync::{Arc, RwLock};
use zenoh::query::ConsolidationMode;
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

	pub fn prefix(&self) -> String {
		self.communicator.prefix()
	}

	pub fn publish<P>(&self, msg_name: impl Into<String>, message: P) -> Result<()>
	where
		P: Serialize,
	{
		self.communicator.publish(msg_name, message)
	}

	pub fn query<P>(
		&self,
		ctx: Arc<Context>,
		props: Arc<RwLock<P>>,
		query_name: impl Into<String>,
		mode: ConsolidationMode,
		callback: QueryCallback<P>,
	) -> Result<()>
	where
		P: Send + Sync + Unpin + 'static,
	{
		self.communicator
			.query(ctx, props, query_name, mode, callback)
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
