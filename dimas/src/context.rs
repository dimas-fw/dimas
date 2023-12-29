//! Copyright © 2023 Stephan Kunz

// region:    --- modules
use serde::Serialize;
use std::sync::Arc;
use zenoh::prelude::sync::SyncResolve;

use crate::{com::communicator::Communicator, prelude::*};
// endregion: --- modules

// region:    --- Context
#[derive(Debug, Clone)]
pub struct Context<'a> {
	pub communicator: Arc<Communicator<'a>>,
}

impl<'a> Context<'a> {
	pub fn _publish<T>(&self, key_expr: impl Into<String>, message: T) -> Result<()>
	where
		T: Serialize,
	{
		let value = serde_json::to_string(&message).unwrap();
		match self
			.communicator
			.clone()
			.session()
			.put(&key_expr.into(), value)
			.res()
		{
			Ok(_) => Ok(()),
			Err(_) => Err("Context publish failed".into()),
		}
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
