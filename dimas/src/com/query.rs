// Copyright © 2023 Stephan Kunz

//! Module `query` provides an information/compute requestor `Query` which can be created using the `QueryBuilder`.

// region:		--- modules
use crate::prelude::*;
use std::{fmt::Debug, sync::Mutex};
use tracing::{error, instrument, Level};
use zenoh::{
	prelude::{sync::SyncResolve, SampleKind},
	query::ConsolidationMode,
};

// endregion:	--- modules

// region:		--- types
/// type definition for the queries callback function
#[allow(clippy::module_name_repetitions)]
pub type QueryCallback<P> =
	Arc<Mutex<dyn FnMut(&ArcContext<P>, Response) -> Result<()> + Send + Sync + Unpin + 'static>>;
// endregion:	--- types

// region:		--- states
pub struct NoStorage;
#[cfg(feature = "query")]
pub struct Storage<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub storage: Arc<RwLock<std::collections::HashMap<String, Query<P>>>>,
}

pub struct NoKeyExpression;
pub struct KeyExpression {
	key_expr: String,
}

pub struct NoResponseCallback;
pub struct ResponseCallback<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub response: QueryCallback<P>,
}
// endregion:	--- states

// region:		--- QueryBuilder
/// The builder for a query
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct QueryBuilder<P, K, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	pub(crate) context: ArcContext<P>,
	pub(crate) key_expr: K,
	pub(crate) callback: C,
	pub(crate) storage: S,
	pub(crate) mode: ConsolidationMode,
}

impl<P> QueryBuilder<P, NoKeyExpression, NoResponseCallback, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a `QueryBuilder` in initial state
	#[must_use]
	pub const fn new(context: ArcContext<P>) -> Self {
		Self {
			context,
			key_expr: NoKeyExpression,
			callback: NoResponseCallback,
			storage: NoStorage,
			mode: ConsolidationMode::None,
		}
	}
}

impl<P, K, C, S> QueryBuilder<P, K, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the consolidation mode
	#[must_use]
	pub const fn mode(mut self, mode: ConsolidationMode) -> Self {
		self.mode = mode;
		self
	}
}

impl<P, C, S> QueryBuilder<P, NoKeyExpression, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full expression for the query
	#[must_use]
	pub fn key_expr(self, key_expr: &str) -> QueryBuilder<P, KeyExpression, C, S> {
		let Self {
			context,
			storage,
			callback,
			mode,
			..
		} = self;
		QueryBuilder {
			context,
			key_expr: KeyExpression {
				key_expr: key_expr.into(),
			},
			callback,
			storage,
			mode,
		}
	}

	/// Set only the message qualifing part of the query.
	/// Will be prefixed with agents prefix.
	#[must_use]
	pub fn msg_type(self, msg_type: &str) -> QueryBuilder<P, KeyExpression, C, S> {
		let key_expr = self.context.key_expr(msg_type);
		let Self {
			context,
			storage,
			callback,
			mode,
			..
		} = self;
		QueryBuilder {
			context,
			key_expr: KeyExpression { key_expr },
			callback,
			storage,
			mode,
		}
	}
}

impl<P, K, S> QueryBuilder<P, K, NoResponseCallback, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set query callback for response messages
	#[must_use]
	pub fn callback<F>(self, callback: F) -> QueryBuilder<P, K, ResponseCallback<P>, S>
	where
		F: FnMut(&ArcContext<P>, Response) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			context,
			key_expr,
			storage,
			mode,
			..
		} = self;
		let callback: QueryCallback<P> = Arc::new(Mutex::new(callback));
		QueryBuilder {
			context,
			key_expr,
			callback: ResponseCallback { response: callback },
			storage,
			mode,
		}
	}
}

#[cfg(feature = "query")]
impl<P, K, C> QueryBuilder<P, K, C, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Provide agents storage for the query
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Query<P>>>>,
	) -> QueryBuilder<P, K, C, Storage<P>> {
		let Self {
			context,
			key_expr,
			callback,
			mode,
			..
		} = self;
		QueryBuilder {
			context,
			key_expr,
			callback,
			storage: Storage { storage },
			mode,
		}
	}
}

impl<P, S> QueryBuilder<P, KeyExpression, ResponseCallback<P>, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the query
	/// # Errors
	///
	pub fn build(self) -> Result<Query<P>> {
		let Self {
			context,
			key_expr,
			callback,
			mode,
			..
		} = self;
		let key_expr = key_expr.key_expr;
		Ok(Query {
			context,
			key_expr,
			mode,
			callback: callback.response,
		})
	}
}

#[cfg(feature = "query")]
impl<P> QueryBuilder<P, KeyExpression, ResponseCallback<P>, Storage<P>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the query to the agents context
	/// # Errors
	///
	#[cfg_attr(any(nightly, docrs), doc, doc(cfg(feature = "query")))]
	pub fn add(self) -> Result<Option<Query<P>>> {
		let collection = self.storage.storage.clone();
		let q = self.build()?;

		let r = collection
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(q.key_expr.clone(), q);
		Ok(r)
	}
}
// endregion:	--- QueryBuilder

// region:		--- Query
/// Query
pub struct Query<P>
where
	P: Send + Sync + Unpin + 'static,
{
	context: ArcContext<P>,
	key_expr: String,
	mode: ConsolidationMode,
	callback: QueryCallback<P>,
}

impl<P> Debug for Query<P>
where
	P: Send + Sync + Unpin + 'static,
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
	P: Send + Sync + Unpin + 'static,
{
	/// run a query
	#[instrument(name="query", level = Level::ERROR, skip_all)]
	pub fn get(&self) -> Result<()> {
		let cb = self.callback.clone();
		let replies = self
			.context
			.communicator
			.session
			.get(&self.key_expr)
			.consolidation(self.mode)
			//.timeout(Duration::from_millis(1000))
			.res_sync()
			.map_err(|_| DimasError::ShouldNotHappen)?;

		while let Ok(reply) = replies.recv() {
			match reply.sample {
				Ok(sample) => match sample.kind {
					SampleKind::Put => {
						let msg = Response(sample);
						let guard = cb.lock();
						match guard {
							Ok(mut lock) => {
								if let Err(error) = lock(&self.context, msg) {
									error!("callback failed with {error}");
								}
							}
							Err(err) => {
								error!("callback lock failed with {err}");
							}
						}
					}
					SampleKind::Delete => {
						error!("Delete in Query");
					}
				},
				Err(err) => error!("receive error: {err})"),
			}
		}
		Ok(())
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
		is_normal::<QueryBuilder<Props, NoKeyExpression, NoResponseCallback, NoStorage>>();
	}
}
