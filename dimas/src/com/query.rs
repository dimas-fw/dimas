// Copyright © 2023 Stephan Kunz

// region:		--- modules
use crate::prelude::*;
use std::{collections::HashMap, fmt::Debug};
use tracing::error;
use zenoh::{
	prelude::{sync::SyncResolve, SampleKind},
	query::ConsolidationMode,
};

// endregion:	--- modules

// region:		--- types
/// type definition for the queries callback function
#[allow(clippy::module_name_repetitions)]
pub type QueryCallback<P> = fn(&Arc<Context<P>>, &Arc<RwLock<P>>, response: &Message);
// endregion:	--- types

// region:		--- QueryBuilder
/// The builder for a query
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct QueryBuilder<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	pub(crate) collection: Arc<RwLock<HashMap<String, Query<P>>>>,
	pub(crate) context: Arc<Context<P>>,
	pub(crate) props: Arc<RwLock<P>>,
	pub(crate) key_expr: Option<String>,
	pub(crate) mode: Option<ConsolidationMode>,
	pub(crate) callback: Option<QueryCallback<P>>,
}

impl<P> QueryBuilder<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
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
		let key_expr = self.context.key_expr(msg_type);
		self.key_expr.replace(key_expr);
		self
	}

	/// Set the consolidation mode
	#[must_use]
	pub fn mode(mut self, mode: ConsolidationMode) -> Self {
		self.mode.replace(mode);
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
	pub fn build(mut self) -> Result<Query<P>> {
		if self.key_expr.is_none() {
			return Err("No key expression or msg type given".into());
		}
		let callback = if self.callback.is_none() {
			return Err("No callback given".into());
		} else {
			self.callback.expect("should never happen")
		};
		let key_expr = if self.key_expr.is_some() {
			self.key_expr.take().expect("should never happen")
		} else {
			String::new()
		};
		let mode = if self.key_expr.is_some() {
			self.mode.take().expect("should never happen")
		} else {
			ConsolidationMode::None
		};

		let q = Query {
			key_expr,
			mode,
			ctx: self.context,
			props: self.props,
			callback,
		};

		Ok(q)
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
			.insert(q.key_expr.clone(), q);
		Ok(())
	}
}
// endregion:	--- QueryBuilder

// region:		--- Query
/// Query
#[derive(Debug)]
pub struct Query<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	key_expr: String,
	mode: ConsolidationMode,
	ctx: Arc<Context<P>>,
	props: Arc<RwLock<P>>,
	callback: QueryCallback<P>,
}

impl<P> Query<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// run a query
	/// # Panics
	///
	#[tracing::instrument(level = tracing::Level::DEBUG)]
	pub fn get(&self) {
		let cb = self.callback;
		let replies = self
			.ctx
			.communicator
			.session
			.get(&self.key_expr)
			.consolidation(self.mode)
			//.timeout(Duration::from_millis(1000))
			.res_sync()
			.expect("should never happen");
		//dbg!(&replies);

		while let Ok(reply) = replies.recv() {
			//dbg!(&reply);
			match reply.sample {
				Ok(sample) => {
					//dbg!(&sample);
					let value: Vec<u8> = sample
						.value
						.try_into()
						.expect("should not happen");
					let msg = Message {
						key_expr: sample.key_expr.to_string(),
						value,
					};
					match sample.kind {
						SampleKind::Put => {
							cb(&self.ctx, &self.props, &msg);
						}
						SampleKind::Delete => {
							error!("Delete in Query");
						}
					}
				}
				Err(err) => error!(
					">> No data (ERROR: '{}')",
					String::try_from(&err).expect("to be implemented")
				),
			}
		}
	}
}
// endregion:	--- Query

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Query<Props>>();
		is_normal::<QueryBuilder<Props>>();
	}
}
