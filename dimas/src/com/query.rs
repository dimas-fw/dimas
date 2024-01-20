//! Copyright © 2023 Stephan Kunz

// region:		--- modules
use super::communicator::Communicator;
use crate::prelude::*;
use std::sync::{Arc, RwLock};
use zenoh::sample::Sample;
// endregion:	--- modules

// region:		--- types
#[allow(clippy::module_name_repetitions)]
pub type QueryCallback<P> = fn(&Arc<Context>, &Arc<RwLock<P>>, sample: Sample);
// endregion:	--- types

// region:		--- QueryBuilder
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
	pub(crate) msg_type: Option<String>,
	pub(crate) callback: Option<QueryCallback<P>>,
}

impl<P> QueryBuilder<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub fn key_expr(mut self, key_expr: impl Into<String>) -> Self {
		self.key_expr.replace(key_expr.into());
		self
	}

	pub fn msg_type(mut self, msg_type: impl Into<String>) -> Self {
		self.msg_type.replace(msg_type.into());
		self
	}

	pub fn callback(mut self, callback: QueryCallback<P>) -> Self {
		self.callback.replace(callback);
		self
	}

	pub fn add(mut self) -> Result<()> {
		if self.key_expr.is_none() && self.msg_type.is_none() {
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
			self.communicator.clone().prefix()
				+ "/" + &self.msg_type.expect("should never happen")
				+ "/*"
		};

		let communicator = self.communicator;
		let _ctx = Arc::new(Context { communicator });
		let _props = self.props.clone();

		let s = Query {
			_key_expr: key_expr,
		};

		self.collection
			.write()
			.expect("should never happen")
			.push(s);
		Ok(())
	}
}
// endregion:	--- QueryBuilder

// region:		--- Query
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
