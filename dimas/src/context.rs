//! Copyright © 2023 Stephan Kunz

// region:		--- modules
#[cfg(feature = "query")]
use crate::com::query::QueryCallback;
use crate::{com::communicator::Communicator, prelude::*};
use serde::Serialize;
use std::sync::{Arc, RwLock};
use zenoh::query::ConsolidationMode;
// endregion:	--- modules

// region:		--- Context
/// Context makes all relevant data of the agent accessible via accessor methods.
#[derive(Debug, Clone, Default)]
pub struct Context {
	pub(crate) communicator: Arc<Communicator>,
}

impl Context {
	/// get the agents uuid
	#[must_use]
	pub fn uuid(&self) -> String {
		self.communicator.uuid()
	}

	/// get the agents prefix
	#[must_use]
	pub fn prefix(&self) -> String {
		self.communicator.prefix()
	}

	/// method to do an ad hoc publishing
	/// # Errors
	///   Error is propagated from Communicator
	#[cfg(feature = "publisher")]
	pub fn publish<P>(&self, msg_name: impl Into<String>, message: P) -> Result<()>
	where
		P: Serialize,
	{
		self.communicator.publish(msg_name, message)
	}

	/// method to do an ad hoc query
	/// # Errors
	///   Error is propagated from Communicator
	#[cfg(feature = "query")]
	pub fn query<P>(
		&self,
		ctx: Arc<Self>,
		props: Arc<RwLock<P>>,
		query_name: impl Into<String>,
		mode: ConsolidationMode,
		callback: QueryCallback<P>,
	)
	where
		P: Send + Sync + Unpin + 'static,
	{
		self.communicator
			.query(ctx, props, query_name, mode, callback);
	}
}
// endregion:	--- Context

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Context>();
	}
}
