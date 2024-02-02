// Copyright © 2023 Stephan Kunz

// region:		--- modules
use crate::com::communicator::Communicator;
use std::sync::Arc;

#[cfg(feature = "query")]
use crate::com::query::QueryCallback;
#[cfg(feature = "publisher")]
use crate::prelude::*;
#[cfg(feature = "query")]
use std::sync::RwLock;
#[cfg(feature = "query")]
use zenoh::query::ConsolidationMode;
// endregion:	--- modules

// region:		--- Context
/// Context makes all relevant data of the agent accessible via accessor methods.
#[derive(Debug, Clone, Default)]
pub struct Context {
	pub(crate) communicator: Arc<Communicator>,
}

impl Context {
	/// Get the agents uuid
	#[must_use]
	pub fn uuid(&self) -> String {
		self.communicator.uuid()
	}

	/// Get the agents prefix
	#[must_use]
	pub fn prefix(&self) -> String {
		self.communicator.prefix()
	}

	/// Method to do an ad hoc publishing
	/// # Errors
	///   Error is propagated from Communicator
	#[cfg(feature = "publisher")]
	pub fn publish<P>(&self, msg_name: impl Into<String>, message: P) -> Result<()>
	where
		P: bincode::Encode,
	{
		self.communicator.publish(msg_name, message)
	}

	/// Method to do an ad hoc deleteion
	/// # Errors
	///   Error is propagated from Communicator
	#[cfg(feature = "publisher")]
	pub fn delete(&self, msg_name: impl Into<String>) -> Result<()> {
		self.communicator.delete(msg_name)
	}

	/// Method to do an ad hoc query without any consolodation of answers.
	/// Multiple answers may be received for the same timestamp.
	#[cfg(feature = "query")]
	pub fn query<P>(
		&self,
		ctx: Arc<Self>,
		props: Arc<RwLock<P>>,
		query_name: impl Into<String>,
		callback: QueryCallback<P>,
	) where
		P: Send + Sync + Unpin + 'static,
	{
		self.communicator
			.query(ctx, props, query_name, ConsolidationMode::None, callback);
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
