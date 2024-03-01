// Copyright © 2023 Stephan Kunz

//! Module `query` provides an information/compute requestor `Query` which can be created using the `QueryBuilder`.

// region:		--- modules
use crate::prelude::*;
use std::{fmt::Debug, sync::Mutex};
use tracing::error;
use zenoh::{
	prelude::{sync::SyncResolve, SampleKind},
	query::ConsolidationMode,
};

// endregion:	--- modules

// region:		--- types
/// type definition for the queries callback function
#[allow(clippy::module_name_repetitions)]
pub type QueryCallback<P> = Arc<
	Mutex<
		dyn FnMut(&ArcContext<P>, &Message) -> Result<(), DimasError>
			+ Send
			+ Sync
			+ Unpin
			+ 'static,
	>,
>;
// endregion:	--- types

// region:		--- QueryBuilder
/// The builder for a query
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct QueryBuilder<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	pub(crate) context: ArcContext<P>,
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
	pub fn callback<F>(mut self, callback: F) -> Self
	where
		F: FnMut(&ArcContext<P>, &Message) -> Result<(), DimasError>
			+ Send
			+ Sync
			+ Unpin
			+ 'static,
	{
		self.callback
			.replace(Arc::new(Mutex::new(callback)));
		self
	}

	/// Build the query
	/// # Errors
	///
	/// # Panics
	///
	pub fn build(mut self) -> Result<Query<P>, DimasError> {
		if self.key_expr.is_none() {
			return Err(DimasError::NoKeyExpression);
		}
		let callback = if self.callback.is_none() {
			return Err(DimasError::NoCallback);
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
			callback,
		};

		Ok(q)
	}

	/// Build and add the query to the agents context
	/// # Errors
	///
	/// # Panics
	///
	#[cfg_attr(any(nightly, docrs), doc, doc(cfg(feature = "query")))]
	#[cfg(feature = "query")]
	pub fn add(self) -> Result<(), DimasError> {
		let collection = self.context.queries.clone();
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
pub struct Query<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	key_expr: String,
	mode: ConsolidationMode,
	ctx: ArcContext<P>,
	callback: QueryCallback<P>,
}

impl<P> Debug for Query<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Query")
			.field("key_expr", &self.key_expr)
			.field("mode", &self.mode)
			.finish_non_exhaustive()
	}
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
		let cb = self.callback.clone();
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
							if let Err(error) =
								cb.lock().expect("should not happen")(&self.ctx, &msg)
							{
								error!("call failed with {error}");
							};
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
