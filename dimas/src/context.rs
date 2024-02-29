// Copyright © 2023 Stephan Kunz

//! The `Context` provides access to an `Agent`'s internal data and its defined properties.

// region:		--- modules
use crate::com::communicator::Communicator;
use crate::prelude::*;
#[cfg(any(
	feature = "publisher",
	feature = "query",
	feature = "queryable",
	feature = "subscriber",
	feature = "timer",
))]
use std::collections::HashMap;
use std::{fmt::Debug, ops::Deref};
use zenoh::publication::Publisher;
use zenoh::query::ConsolidationMode;
// endregion:	--- modules

// region:		--- types
/// Type definition for a thread safe `Context`
#[allow(clippy::module_name_repetitions)]
pub type ArcContext<P> = Arc<Context<P>>;
// endregion:	--- types

// region:		--- Context
/// Context makes all relevant data of the agent accessible via accessor methods.
#[derive(Debug, Clone, Default)]
pub struct Context<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// The agents property structure
	pub(crate) props: Arc<RwLock<P>>,
	pub(crate) communicator: Arc<Communicator>,
	#[cfg(feature = "publisher")]
	pub(crate) publishers: Arc<RwLock<HashMap<String, crate::com::publisher::Publisher>>>,
	#[cfg(feature = "query")]
	pub(crate) queries: Arc<RwLock<HashMap<String, crate::com::query::Query<P>>>>,
	// registered queryables
	#[cfg(feature = "queryable")]
	pub(crate) queryables: Arc<RwLock<HashMap<String, Queryable<P>>>>,
	// registered subscribers
	#[cfg(feature = "subscriber")]
	pub(crate) subscribers: Arc<RwLock<HashMap<String, Subscriber<P>>>>,
	// registered timer
	#[cfg(feature = "timer")]
	pub(crate) timers: Arc<RwLock<HashMap<String, Timer<P>>>>,
}

impl<P> Deref for Context<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	type Target = Arc<RwLock<P>>;

	fn deref(&self) -> &Self::Target {
		&self.props
	}
}

impl<P> Context<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// Constructor for the `Context`
	pub(crate) fn new(config: Config, props: P) -> Arc<Self> {
		let communicator = Arc::new(Communicator::new(config));
		Arc::new(Self {
			communicator,
			props: Arc::new(RwLock::new(props)),
			#[cfg(feature = "publisher")]
			publishers: Arc::new(RwLock::new(HashMap::new())),
			#[cfg(feature = "query")]
			queries: Arc::new(RwLock::new(HashMap::new())),
			#[cfg(feature = "query")]
			queryables: Arc::new(RwLock::new(HashMap::new())),
			#[cfg(feature = "subscriber")]
			subscribers: Arc::new(RwLock::new(HashMap::new())),
			#[cfg(feature = "timer")]
			timers: Arc::new(RwLock::new(HashMap::new())),
		})
	}

	/// Constructor for the `Context` with a prefix
	pub(crate) fn new_with_prefix(
		config: Config,
		props: P,
		prefix: impl Into<String>,
	) -> Arc<Self> {
		let communicator = Arc::new(Communicator::new_with_prefix(config, prefix));
		Arc::new(Self {
			communicator,
			props: Arc::new(RwLock::new(props)),
			#[cfg(feature = "publisher")]
			publishers: Arc::new(RwLock::new(HashMap::new())),
			#[cfg(feature = "query")]
			queries: Arc::new(RwLock::new(HashMap::new())),
			#[cfg(feature = "query")]
			queryables: Arc::new(RwLock::new(HashMap::new())),
			#[cfg(feature = "subscriber")]
			subscribers: Arc::new(RwLock::new(HashMap::new())),
			#[cfg(feature = "timer")]
			timers: Arc::new(RwLock::new(HashMap::new())),
		})
	}

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
		M: Encode,
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
		M: Debug + Encode,
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
	pub fn get<F>(&self, ctx: Arc<Self>, query_name: impl Into<String>, callback: F)
	where
		P: Debug + Send + Sync + Unpin + 'static,
		F: Fn(&ArcContext<P>, &Message) + Send + Sync + Unpin + 'static,
	{
		self.communicator
			.get(ctx, query_name, ConsolidationMode::None, callback);
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
