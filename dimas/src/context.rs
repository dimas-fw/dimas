// Copyright © 2023 Stephan Kunz

// region:		--- modules
use crate::com::communicator::Communicator;
use crate::com::query::QueryCallback;
use crate::prelude::*;
#[cfg(any(feature = "publisher", feature = "query"))]
use std::collections::HashMap;
use std::{fmt::Debug, marker::PhantomData};
use zenoh::publication::Publisher;
use zenoh::query::ConsolidationMode;
// endregion:	--- modules

// region:		--- Context
/// Context makes all relevant data of the agent accessible via accessor methods.
#[derive(Debug, Clone, Default)]
pub struct Context<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	pub(crate) communicator: Arc<Communicator>,
	#[cfg(feature = "publisher")]
	pub(crate) publishers: Arc<RwLock<HashMap<String, crate::com::publisher::Publisher>>>,
	#[cfg(feature = "query")]
	pub(crate) queries: Arc<RwLock<HashMap<String, crate::com::query::Query<P>>>>,
	pub(crate) pd: PhantomData<P>,
}

impl<P> Context<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// Get the agents uuid
	#[must_use]
	pub fn uuid(&self) -> String {
		self.communicator.uuid()
	}

	/// Get the agents prefix
	#[must_use]
	pub fn prefix(&self) -> Option<String> {
		self.communicator.prefix()
	}

	pub(crate) fn key_expr(&self, msg_name: impl Into<String>) -> String {
		match self.prefix() {
			Some(prefix) => prefix + "/" + &msg_name.into(),
			None => msg_name.into(),
		}
	}

	pub(crate) fn create_publisher<'publisher>(
		&self,
		key_expr: impl Into<String> + Send,
	) -> Publisher<'publisher> {
		self.communicator.create_publisher(key_expr)
	}

	/// Method to do an ad hoc publishing
	/// # Errors
	///   Error is propagated from Communicator
	pub fn put<M>(&self, msg_name: impl Into<String>, message: M) -> Result<()>
	where
		M: bitcode::Encode,
	{
		self.communicator.put(msg_name, message)
	}

	/// Method to pubish data with a stored Publisher
	/// # Errors
	///
	/// # Panics
	///
	#[cfg(feature = "publisher")]
	pub fn put_with<M>(&self, msg_name: &str, message: M) -> Result<()>
	where
		M: Debug + bitcode::Encode,
	{
		let key_expr = self.key_expr(msg_name);
		if self
			.publishers
			.read()
			.expect("should not happen")
			.get(&key_expr)
			.is_some()
		{
			self.publishers
				.read()
				.expect("should not happen")
				.get(&key_expr)
				.expect("should not happen")
				.put(message)?;
		};
		Ok(())
	}

	/// Method to do an ad hoc deletion
	/// # Errors
	///   Error is propagated from Communicator
	pub fn delete(&self, msg_name: impl Into<String>) -> Result<()> {
		self.communicator.delete(msg_name)
	}

	/// Method to delete data with a stored Publisher
	/// # Errors
	///
	/// # Panics
	///
	#[cfg(feature = "publisher")]
	pub fn delete_with(&self, msg_name: &str) -> Result<()> {
		let key_expr = self.key_expr(msg_name);
		if self
			.publishers
			.read()
			.expect("should not happen")
			.get(&key_expr)
			.is_some()
		{
			self.publishers
				.read()
				.expect("should not happen")
				.get(&key_expr)
				.expect("should not happen")
				.delete()?;
		}
		Ok(())
	}

	/// Method to do an ad hoc query without any consolodation of answers.
	/// Multiple answers may be received for the same timestamp.
	pub fn get(
		&self,
		ctx: Arc<Self>,
		props: Arc<RwLock<P>>,
		query_name: impl Into<String>,
		callback: QueryCallback<P>,
	) {
		self.communicator
			.get(ctx, props, query_name, ConsolidationMode::None, callback);
	}

	/// Method to query data with a stored Query
	/// # Errors
	///
	/// # Panics
	///
	#[cfg(feature = "query")]
	pub fn get_with(&self, msg_name: &str) {
		let key_expr = self.key_expr(msg_name);
		if self
			.queries
			.read()
			.expect("should not happen")
			.get(&key_expr)
			.is_some()
		{
			self.queries
				.read()
				.expect("should not happen")
				.get(&key_expr)
				.expect("should not happen")
				.get();
		};
	}
}
// endregion:	--- Context

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[derive(Debug)]
	struct Props {}

	#[test]
	const fn normal_types() {
		is_normal::<Context<Props>>();
	}
}
