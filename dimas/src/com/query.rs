// Copyright © 2023 Stephan Kunz

//! Module `query` provides an information/compute requestor `Query` which can be created using the `QueryBuilder`.

// region:		--- modules
use crate::context::ArcContext;
use dimas_core::error::{DimasError, Result};
#[cfg(doc)]
use std::collections::HashMap;
use std::{
	fmt::Debug,
	marker::PhantomData,
	sync::{Arc, Mutex, RwLock},
	time::Duration,
};
use tracing::{error, instrument, Level};
use zenoh::{
	prelude::{sync::SyncResolve, SampleKind},
	query::{ConsolidationMode, QueryTarget},
	sample::Locality,
};

use dimas_com::Response;
// endregion:	--- modules

// region:		--- types
/// type definition for the queries callback function
#[allow(clippy::module_name_repetitions)]
pub type QueryCallback<P> =
	Arc<Mutex<dyn FnMut(&ArcContext<P>, Response) -> Result<()> + Send + Sync + Unpin + 'static>>;
// endregion:	--- types

// region:		--- states
/// State signaling that the [`QueryBuilder`] has no storage value set
pub struct NoStorage;
/// State signaling that the [`QueryBuilder`] has the storage value set
pub struct Storage<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Thread safe reference to a [`HashMap`] to store the created [`Query`]
	pub storage: Arc<RwLock<std::collections::HashMap<String, Query<P>>>>,
}

/// State signaling that the [`QueryBuilder`] has no key expression set
pub struct NoKeyExpression;
/// State signaling that the [`QueryBuilder`] has the key expression set
pub struct KeyExpression {
	/// The key expression
	key_expr: String,
}

/// State signaling that the [`QueryBuilder`] has no response callback set
pub struct NoResponseCallback;
/// State signaling that the [`QueryBuilder`] has the response callback set
pub struct ResponseCallback<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Response callback for the [`Query`]
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
	pub(crate) allowed_destination: Locality,
	pub(crate) prefix: Option<String>,
	pub(crate) timeout: Option<Duration>,
	pub(crate) key_expr: K,
	pub(crate) response_callback: C,
	pub(crate) storage: S,
	pub(crate) mode: ConsolidationMode,
	pub(crate) target: QueryTarget,
	phantom: PhantomData<P>,
}

impl<P> QueryBuilder<P, NoKeyExpression, NoResponseCallback, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a `QueryBuilder` in initial state
	#[must_use]
	pub const fn new(prefix: Option<String>) -> Self {
		Self {
			allowed_destination: Locality::Any,
			prefix,
			timeout: None,
			key_expr: NoKeyExpression,
			response_callback: NoResponseCallback,
			storage: NoStorage,
			mode: ConsolidationMode::None,
			target: QueryTarget::BestMatching,
			phantom: PhantomData,
		}
	}
}

impl<P, K, C, S> QueryBuilder<P, K, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the [`ConsolidationMode`] of the [`Query`].
	#[must_use]
	pub const fn mode(mut self, mode: ConsolidationMode) -> Self {
		self.mode = mode;
		self
	}

	/// Set the [`QueryTarget`] of the [`Query`].
	#[must_use]
	pub const fn target(mut self, target: QueryTarget) -> Self {
		self.target = target;
		self
	}

	/// Set the allowed destination of the [`Query`].
	#[must_use]
	pub const fn allowed_destination(mut self, allowed_destination: Locality) -> Self {
		self.allowed_destination = allowed_destination;
		self
	}

	/// Set a timeout for the [`Query`].
	#[must_use]
	pub const fn timeout(mut self, timeout: Option<Duration>) -> Self {
		self.timeout = timeout;
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
			allowed_destination,
			prefix,
			timeout,
			storage,
			response_callback: callback,
			mode,
			target,
			phantom,
			..
		} = self;
		QueryBuilder {
			allowed_destination,
			prefix,
			timeout,
			key_expr: KeyExpression {
				key_expr: key_expr.into(),
			},
			response_callback: callback,
			storage,
			mode,
			target,
			phantom,
		}
	}

	/// Set only the message qualifing part of the query.
	/// Will be prefixed with agents prefix.
	#[must_use]
	pub fn topic(mut self, topic: &str) -> QueryBuilder<P, KeyExpression, C, S> {
		let key_expr = self
			.prefix
			.take()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		let Self {
			allowed_destination,
			prefix,
			timeout,
			storage,
			response_callback: callback,
			mode,
			target,
			phantom,
			..
		} = self;
		QueryBuilder {
			allowed_destination,
			prefix,
			timeout,
			key_expr: KeyExpression { key_expr },
			response_callback: callback,
			storage,
			mode,
			target,
			phantom,
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
			allowed_destination,
			prefix,
			timeout,
			key_expr,
			storage,
			mode,
			target,
			phantom,
			..
		} = self;
		let callback: QueryCallback<P> = Arc::new(Mutex::new(callback));
		QueryBuilder {
			allowed_destination,
			prefix,
			timeout,
			key_expr,
			response_callback: ResponseCallback { response: callback },
			storage,
			mode,
			target,
			phantom,
		}
	}
}

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
			allowed_destination,
			prefix,
			timeout,
			key_expr,
			response_callback: callback,
			mode,
			target,
			phantom,
			..
		} = self;
		QueryBuilder {
			allowed_destination,
			prefix,
			timeout,
			key_expr,
			response_callback: callback,
			storage: Storage { storage },
			mode,
			target,
			phantom,
		}
	}
}

impl<P, S> QueryBuilder<P, KeyExpression, ResponseCallback<P>, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the [`Query`]
	/// # Errors
	///
	pub fn build(self) -> Result<Query<P>> {
		let Self {
			allowed_destination,
			timeout,
			key_expr,
			response_callback,
			mode,
			target,
			..
		} = self;
		let key_expr = key_expr.key_expr;
		Ok(Query::new(
			key_expr,
			response_callback.response,
			mode,
			allowed_destination,
			target,
			timeout,
		))
	}
}

impl<P> QueryBuilder<P, KeyExpression, ResponseCallback<P>, Storage<P>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the query to the agents context
	/// # Errors
	///
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
	pub(crate) key_expr: String,
	response_callback: QueryCallback<P>,
	mode: ConsolidationMode,
	allowed_destination: Locality,
	target: QueryTarget,
	timeout: Option<Duration>,
	context: Option<ArcContext<P>>,
}

impl<P> Debug for Query<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Query")
			.field("key_expr", &self.key_expr)
			.field("mode", &self.mode)
			.field("allowed_destination", &self.allowed_destination)
			.finish_non_exhaustive()
	}
}

impl<P> Query<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor for a [`Query`]
	#[must_use]
	pub fn new(
		key_expr: String,
		response_callback: QueryCallback<P>,
		mode: ConsolidationMode,
		allowed_destination: Locality,
		target: QueryTarget,
		timeout: Option<Duration>,
	) -> Self {
		Self {
			key_expr,
			response_callback,
			mode,
			allowed_destination,
			target,
			timeout,
			context: None,
		}
	}

	/// Initialize
	/// # Errors
	pub fn init(&mut self, context: &ArcContext<P>) -> Result<()>
	where
		P: Send + Sync + Unpin + 'static,
	{
		self.context.replace(context.clone());
		Ok(())
	}

	/// De-Initialize
	/// # Errors
	pub fn de_init(&mut self) -> Result<()>
	where
		P: Send + Sync + Unpin + 'static,
	{
		self.context.take();
		Ok(())
	}

	/// run a query
	#[instrument(name="query", level = Level::ERROR, skip_all)]
	pub fn get(&self) -> Result<()> {
		let cb = self.response_callback.clone();
		let communicator = self
			.context
			.clone()
			.ok_or(DimasError::ShouldNotHappen)?
			.communicator
			.clone();

		let session = communicator.session();
		let mut query = session
			.get(&self.key_expr)
			.target(self.target)
			.consolidation(self.mode)
			.allowed_destination(self.allowed_destination);

		if let Some(timeout) = self.timeout {
			query = query.timeout(timeout);
		};

		let replies = query
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
								if let Err(error) = lock(
									&self
										.context
										.clone()
										.ok_or(DimasError::ShouldNotHappen)?,
									msg,
								) {
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
