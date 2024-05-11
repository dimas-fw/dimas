// Copyright © 2023 Stephan Kunz

//! Module `queryable` provides an information/compute provider `Queryable` which can be created using the `QueryableBuilder`.

// region:		--- modules
use super::task_signal::TaskSignal;
use crate::context::ArcContext;
use dimas_com::Request;
use dimas_core::error::{DimasError, Result};
use std::{
	fmt::Debug,
	marker::PhantomData,
	sync::{mpsc::Sender, Arc, Mutex, RwLock},
};
use tokio::task::JoinHandle;
use tracing::{error, info, instrument, warn, Level};
use zenoh::prelude::{r#async::AsyncResolve, SessionDeclarations};
use zenoh::sample::Locality;
// endregion:	--- modules

// region:		--- types
/// type defnition for the queryables callback function.
#[allow(clippy::module_name_repetitions)]
pub type QueryableCallback<P> = Arc<
	Mutex<Box<dyn FnMut(&ArcContext<P>, Request) -> Result<()> + Send + Sync + Unpin + 'static>>,
>;
// endregion:	--- types

// region:		--- states
/// State signaling that the [`QueryableBuilder`] has no storage value set
pub struct NoStorage;
/// State signaling that the [`QueryableBuilder`] has the storage value set
pub struct Storage<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Thread safe reference to a [`HashMap`] to store the created [`Queryable`]
	pub storage: Arc<RwLock<std::collections::HashMap<String, Queryable<P>>>>,
}

/// State signaling that the [`QueryableBuilder`] has no key expression set
pub struct NoKeyExpression;
/// State signaling that the [`QueryableBuilder`] has the key expression set
pub struct KeyExpression {
	/// The key expression
	key_expr: String,
}

/// State signaling that the [`QueryableBuilder`] has no request callback set
pub struct NoRequestCallback;
/// State signaling that the [`QueryableBuilder`] has the request callback set
pub struct RequestCallback<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Request callback for the [`Queryable`]
	pub request: QueryableCallback<P>,
}

// region:		--- QueryableBuilder
/// The builder for a queryable.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct QueryableBuilder<P, K, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	completeness: bool,
	allowed_origin: Locality,
	prefix: Option<String>,
	pub(crate) key_expr: K,
	pub(crate) request_callback: C,
	pub(crate) storage: S,
	phantom: PhantomData<P>,
}

impl<P, K, C, S> QueryableBuilder<P, K, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the completeness of the [`Queryable`].
	#[must_use]
	pub const fn completeness(mut self, completeness: bool) -> Self {
		self.completeness = completeness;
		self
	}

	/// Set the allowed origin of the [`Queryable`].
	#[must_use]
	pub const fn allowed_origin(mut self, allowed_origin: Locality) -> Self {
		self.allowed_origin = allowed_origin;
		self
	}
}

impl<P> QueryableBuilder<P, NoKeyExpression, NoRequestCallback, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a `QueryableBuilder` in initial state
	#[must_use]
	pub const fn new(prefix: Option<String>) -> Self {
		Self {
			completeness: true,
			allowed_origin: Locality::Any,
			prefix,
			key_expr: NoKeyExpression,
			request_callback: NoRequestCallback,
			storage: NoStorage,
			phantom: PhantomData,
		}
	}
}

impl<P, C, S> QueryableBuilder<P, NoKeyExpression, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full expression for the [`Queryable`].
	#[must_use]
	pub fn key_expr(self, key_expr: &str) -> QueryableBuilder<P, KeyExpression, C, S> {
		let Self {
			completeness,
			allowed_origin,
			prefix,
			storage,
			request_callback: callback,
			phantom,
			..
		} = self;
		QueryableBuilder {
			completeness,
			allowed_origin,
			prefix,
			key_expr: KeyExpression {
				key_expr: key_expr.into(),
			},
			request_callback: callback,
			storage,
			phantom,
		}
	}

	/// Set only the topic of the [`Queryable`].
	/// Will be prefixed with agents prefix.
	#[must_use]
	pub fn topic(mut self, topic: &str) -> QueryableBuilder<P, KeyExpression, C, S> {
		let key_expr = self
			.prefix
			.take()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		let Self {
			completeness,
			allowed_origin,
			prefix,
			storage,
			request_callback: callback,
			phantom,
			..
		} = self;
		QueryableBuilder {
			completeness,
			allowed_origin,
			prefix,
			key_expr: KeyExpression { key_expr },
			request_callback: callback,
			storage,
			phantom,
		}
	}
}

impl<P, K, S> QueryableBuilder<P, K, NoRequestCallback, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set callback for request messages
	#[must_use]
	pub fn callback<F>(self, callback: F) -> QueryableBuilder<P, K, RequestCallback<P>, S>
	where
		F: FnMut(&ArcContext<P>, Request) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			completeness,
			allowed_origin,
			prefix,
			key_expr,
			storage,
			phantom,
			..
		} = self;
		let request: QueryableCallback<P> = Arc::new(Mutex::new(Box::new(callback)));
		QueryableBuilder {
			completeness,
			allowed_origin,
			prefix,
			key_expr,
			request_callback: RequestCallback { request },
			storage,
			phantom,
		}
	}
}

impl<P, K, C> QueryableBuilder<P, K, C, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Provide agents storage for the queryable
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Queryable<P>>>>,
	) -> QueryableBuilder<P, K, C, Storage<P>> {
		let Self {
			completeness,
			allowed_origin,
			prefix,
			key_expr,
			request_callback: callback,
			phantom,
			..
		} = self;
		QueryableBuilder {
			completeness,
			allowed_origin,
			prefix,
			key_expr,
			request_callback: callback,
			storage: Storage { storage },
			phantom,
		}
	}
}

impl<P, S> QueryableBuilder<P, KeyExpression, RequestCallback<P>, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the [`Queryable`]
	/// # Errors
	///
	pub fn build(self) -> Result<Queryable<P>> {
		let Self {
			completeness,
			allowed_origin,
			key_expr,
			request_callback,
			..
		} = self;
		let key_expr = key_expr.key_expr;
		Ok(Queryable::new(
			key_expr,
			request_callback.request,
			completeness,
			allowed_origin,
		))
	}
}

impl<P> QueryableBuilder<P, KeyExpression, RequestCallback<P>, Storage<P>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the queryable to the agents context
	/// # Errors
	///
	pub fn add(self) -> Result<Option<Queryable<P>>> {
		let collection = self.storage.storage.clone();
		let q = self.build()?;

		let r = collection
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(q.key_expr.clone(), q);
		Ok(r)
	}
}
// endregion:	--- QueryableBuilder

// region:		--- Queryable
/// Queryable
pub struct Queryable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	key_expr: String,
	request_callback: QueryableCallback<P>,
	completeness: bool,
	allowed_origin: Locality,
	handle: Option<JoinHandle<()>>,
}

impl<P> Debug for Queryable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Queryable")
			.field("key_expr", &self.key_expr)
			.field("complete", &self.completeness)
			.finish_non_exhaustive()
	}
}

impl<P> Queryable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor for a [`Queryable`]
	#[must_use]
	pub fn new(
		key_expr: String,
		request_callback: QueryableCallback<P>,
		completeness: bool,
		allowed_origin: Locality,
	) -> Self {
		Self {
			key_expr,
			request_callback,
			completeness,
			allowed_origin,
			handle: None,
		}
	}

	/// Start or restart the queryable.
	/// An already running queryable will be stopped, eventually damaged Mutexes will be repaired
	#[instrument(level = Level::TRACE, skip_all)]
	pub fn start(&mut self, ctx: ArcContext<P>, tx: Sender<TaskSignal>) {
		self.stop();

		{
			if self.request_callback.lock().is_err() {
				warn!("found poisoned put Mutex");
				self.request_callback.clear_poison();
			}
		}

		let completeness = self.completeness;
		let allowed_origin = self.allowed_origin;
		let key_expr = self.key_expr.clone();
		let cb = self.request_callback.clone();

		self.handle
			.replace(tokio::task::spawn(async move {
				let key = key_expr.clone();
				std::panic::set_hook(Box::new(move |reason| {
					error!("queryable panic: {}", reason);
					if let Err(reason) = tx.send(TaskSignal::RestartQueryable(key.clone())) {
						error!("could not restart queryable: {}", reason);
					} else {
						info!("restarting queryable!");
					};
				}));
				if let Err(error) =
					run_queryable(key_expr, cb, completeness, allowed_origin, ctx).await
				{
					error!("queryable failed with {error}");
				};
			}));
	}

	/// Stop a running Queryable
	#[instrument(level = Level::TRACE)]
	pub fn stop(&mut self) {
		if let Some(handle) = self.handle.take() {
			handle.abort();
		}
	}
}

#[instrument(name="queryable", level = Level::ERROR, skip_all)]
async fn run_queryable<P>(
	key_expr: String,
	cb: QueryableCallback<P>,
	completeness: bool,
	allowed_origin: Locality,
	ctx: ArcContext<P>,
) -> Result<()>
where
	P: Send + Sync + Unpin + 'static,
{
	let subscriber = ctx
		.communicator
		.session()
		.declare_queryable(&key_expr)
		.complete(completeness)
		.allowed_origin(allowed_origin)
		.res_async()
		.await
		.map_err(|_| DimasError::ShouldNotHappen)?;

	loop {
		let query = subscriber
			.recv_async()
			.await
			.map_err(|_| DimasError::ShouldNotHappen)?;
		let request = Request(query);

		match cb.lock() {
			Ok(mut lock) => {
				if let Err(error) = lock(&ctx, request) {
					error!("queryable callback failed with {error}");
				}
			}
			Err(err) => {
				error!("queryable callback failed with {err}");
			}
		}
	}
}
// endregion:	--- Queryable

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Queryable<Props>>();
		is_normal::<QueryableBuilder<Props, NoKeyExpression, NoRequestCallback, NoStorage>>();
	}
}
