// Copyright © 2023 Stephan Kunz

// region:		--- modules
use super::communicator::Communicator;
use crate::prelude::*;
use std::sync::{Arc, RwLock};
// endregion:	--- modules

// region:		--- types
/// type definition for the queries callback function
#[allow(clippy::module_name_repetitions)]
pub type QueryCallback<P> = fn(&Arc<Context>, &Arc<RwLock<P>>, answer: &[u8]);
// endregion:	--- types

// region:		--- QueryBuilder
/// The builder for a query
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct QueryBuilder<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub(crate) collection: Arc<RwLock<Vec<Query>>>,
	pub(crate) communicator: Arc<Communicator>,
	pub(crate) props: Arc<RwLock<P>>,
	pub(crate) key_expr: Option<String>,
	pub(crate) callback: Option<QueryCallback<P>>,
}

impl<P> QueryBuilder<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full expression for the query
	#[must_use]
	pub fn key_expr(mut self, key_expr: impl Into<String>) -> Self {
		self.key_expr.replace(key_expr.into());
		self
	}

	/// Set only the message qualifing part of the query.
	/// Will be prefixed with agents prefix.
	#[must_use]
	pub fn msg_type(mut self, msg_type: impl Into<String>) -> Self {
		let key_expr = self.communicator.clone().prefix() + "/" + &msg_type.into();
		self.key_expr.replace(key_expr);
		self
	}

	/// Set the queries callback function
	#[must_use]
	pub fn callback(mut self, callback: QueryCallback<P>) -> Self {
		self.callback.replace(callback);
		self
	}

	/// Build the query
	/// # Errors
	///
	/// # Panics
	///
	pub fn build(mut self) -> Result<Query> {
		if self.key_expr.is_none() {
			return Err("No key expression or msg type given".into());
		}
		let _callback = if self.callback.is_none() {
			return Err("No callback given".into());
		} else {
			self.callback.expect("should never happen")
		};
		let key_expr = if self.key_expr.is_some() {
			self.key_expr.take().expect("should never happen")
		} else {
			String::new()
		};

		let communicator = self.communicator;
		let _ctx = Arc::new(Context { communicator });
		let _props = self.props.clone();

		let s = Query {
			_key_expr: key_expr,
		};

		Ok(s)
	}

	/// Build and add the query to the agent
	/// # Errors
	///
	/// # Panics
	///
	pub fn add(self) -> Result<()> {
		let collection = self.collection.clone();
		let q = self.build()?;

		collection
			.write()
			.expect("should never happen")
			.push(q);
		Ok(())
	}
}
// endregion:	--- QueryBuilder

// region:		--- Query
/// Query
pub struct Query {
	_key_expr: String,
}
// endregion:	--- Query

#[cfg(test)]
mod tests {
	use super::*;
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Query>();
		is_normal::<QueryBuilder<Props>>();
	}
}
