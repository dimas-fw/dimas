// Copyright © 2023 Stephan Kunz

//! Module `queryable` provides an information/compute provider `Queryable` which can be created using the `QueryableBuilder`.

// region:		--- modules
use crate::{prelude::*, utils::TaskSignal};
use std::{
	fmt::Debug,
	marker::PhantomData,
	sync::{mpsc::Sender, Mutex},
};
use tokio::task::JoinHandle;
#[cfg(feature = "queryable")]
use tracing::info;
use tracing::{error, instrument, warn, Level};
use zenoh::{prelude::r#async::AsyncResolve, SessionDeclarations};
// endregion:	--- modules

// region:		--- types
/// type defnition for the queryables callback function.
#[allow(clippy::module_name_repetitions)]
pub type QueryableCallback<P> = Arc<
	Mutex<
		Option<
			Box<dyn FnMut(&ArcContext<P>, Request) -> Result<()> + Send + Sync + Unpin + 'static>,
		>,
	>,
>;
// endregion:	--- types

// region:		--- states
pub struct NoStorage;
#[cfg(feature = "queryable")]
pub struct Storage<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub storage: Arc<RwLock<std::collections::HashMap<String, Queryable<P>>>>,
}

pub struct NoKeyExpression;
pub struct KeyExpression {
	key_expr: String,
}

pub struct NoRequestCallback;
pub struct RequestCallback<P>
where
	P: Send + Sync + Unpin + 'static,
{
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
	prefix: Option<String>,
	pub(crate) key_expr: K,
	pub(crate) callback: C,
	pub(crate) storage: S,
	phantom: PhantomData<P>,
}

impl<P> QueryableBuilder<P, NoKeyExpression, NoRequestCallback, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a `QueryableBuilder` in initial state
	#[must_use]
	pub const fn new(prefix: Option<String>) -> Self {
		Self {
			prefix,
			key_expr: NoKeyExpression,
			callback: NoRequestCallback,
			storage: NoStorage,
			phantom: PhantomData,
		}
	}
}

impl<P, C, S> QueryableBuilder<P, NoKeyExpression, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full expression for the queryable
	#[must_use]
	pub fn key_expr(self, key_expr: &str) -> QueryableBuilder<P, KeyExpression, C, S> {
		let Self {
			prefix,
			storage,
			callback,
			phantom,
			..
		} = self;
		QueryableBuilder {
			prefix,
			key_expr: KeyExpression {
				key_expr: key_expr.into(),
			},
			callback,
			storage,
			phantom,
		}
	}

	/// Set only the message qualifing part of the queryable.
	/// Will be prefixed with agents prefix.
	#[must_use]
	pub fn topic(mut self, topic: &str) -> QueryableBuilder<P, KeyExpression, C, S> {
		let key_expr = self
			.prefix
			.take()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		let Self {
			prefix,
			storage,
			callback,
			phantom,
			..
		} = self;
		QueryableBuilder {
			prefix,
			key_expr: KeyExpression { key_expr },
			callback,
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
			prefix,
			key_expr,
			storage,
			phantom,
			..
		} = self;
		let request: QueryableCallback<P> = Arc::new(Mutex::new(Some(Box::new(callback))));
		QueryableBuilder {
			prefix,
			key_expr,
			callback: RequestCallback { request },
			storage,
			phantom,
		}
	}
}

#[cfg(feature = "queryable")]
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
			prefix,
			key_expr,
			callback,
			phantom,
			..
		} = self;
		QueryableBuilder {
			prefix,
			key_expr,
			callback,
			storage: Storage { storage },
			phantom,
		}
	}
}

impl<P, S> QueryableBuilder<P, KeyExpression, RequestCallback<P>, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the queryable
	/// # Errors
	///
	pub fn build(self) -> Result<Queryable<P>> {
		let Self {
			key_expr, callback, ..
		} = self;
		let key_expr = key_expr.key_expr;
		Ok(Queryable {
			key_expr,
			callback: Some(callback.request),
			handle: None,
		})
	}
}

#[cfg(feature = "queryable")]
impl<P> QueryableBuilder<P, KeyExpression, RequestCallback<P>, Storage<P>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the queryable to the agents context
	/// # Errors
	///
	#[cfg_attr(any(nightly, docrs), doc, doc(cfg(feature = "query")))]
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
	callback: Option<QueryableCallback<P>>,
	handle: Option<JoinHandle<()>>,
}

impl<P> Debug for Queryable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Queryable")
			.field("key_expr", &self.key_expr)
			.finish_non_exhaustive()
	}
}

impl<P> Queryable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Start or restart the queryable.
	/// An already running queryable will be stopped, eventually damaged Mutexes will be repaired
	#[instrument(level = Level::TRACE, skip_all)]
	pub fn start(&mut self, ctx: ArcContext<P>, tx: Sender<TaskSignal>) {
		self.stop();

		#[cfg(not(feature = "queryable"))]
		drop(tx);

		{
			if let Some(cb) = self.callback.clone() {
				if let Err(err) = cb.lock() {
					warn!("found poisoned put Mutex");
					self.callback
						.replace(Arc::new(Mutex::new(err.into_inner().take())));
				}
			}
		}

		let key_expr = self.key_expr.clone();
		let cb = self.callback.clone();

		self.handle
			.replace(tokio::task::spawn(async move {
				#[cfg(feature = "queryable")]
				let key = key_expr.clone();
				std::panic::set_hook(Box::new(move |reason| {
					error!("queryable panic: {}", reason);
					#[cfg(feature = "queryable")]
					if let Err(reason) = tx.send(TaskSignal::RestartQueryable(key.clone())) {
						error!("could not restart queryable: {}", reason);
					} else {
						info!("restarting queryable!");
					};
				}));
				if let Err(error) = run_queryable(key_expr, cb, ctx).await {
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
	cb: Option<QueryableCallback<P>>,
	ctx: ArcContext<P>,
) -> Result<()>
where
	P: Send + Sync + Unpin + 'static,
{
	let subscriber = ctx
		.communicator
		.session
		.declare_queryable(&key_expr)
		.res_async()
		.await
		.map_err(|_| DimasError::ShouldNotHappen)?;

	loop {
		let query = subscriber
			.recv_async()
			.await
			.map_err(|_| DimasError::ShouldNotHappen)?;
		let request = Request(query);

		if let Some(cb) = cb.clone() {
			let result = cb.lock();
			match result {
				Ok(mut cb) => {
					if let Err(error) = cb.as_deref_mut().expect("snh")(&ctx, request) {
						error!("callback failed with {error}");
					}
				}
				Err(err) => {
					error!("callback lock failed with {err}");
				}
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
