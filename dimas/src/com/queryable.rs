// Copyright © 2023 Stephan Kunz

//! Module `queryable` provides an information/compute provider `Queryable` which can be created using the `QueryableBuilder`.

// region:		--- modules
use crate::prelude::*;
use std::{fmt::Debug, sync::Mutex};
use tokio::task::JoinHandle;
use tracing::{error, span, Level};
use zenoh::prelude::r#async::AsyncResolve;
// endregion:	--- modules

// region:		--- types
/// type defnition for the queryables callback function.
#[allow(clippy::module_name_repetitions)]
pub type QueryableCallback<P> = Arc<
	Mutex<
		dyn FnMut(&ArcContext<P>, Request) -> Result<(), DimasError>
			+ Send
			+ Sync
			+ Unpin
			+ 'static,
	>,
>;
// endregion:	--- types

// region:		--- QueryableBuilder
/// The builder fo a queryable.
#[allow(clippy::module_name_repetitions)]
#[derive(Default, Clone)]
pub struct QueryableBuilder<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	pub(crate) context: ArcContext<P>,
	pub(crate) key_expr: Option<String>,
	pub(crate) callback: Option<QueryableCallback<P>>,
}

impl<P> QueryableBuilder<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// Set the full expression for the queryable.
	#[must_use]
	pub fn key_expr(mut self, key_expr: impl Into<String>) -> Self {
		self.key_expr.replace(key_expr.into());
		self
	}

	/// Set the message qualifying part only.
	/// Will be prefixed by the agents prefix.
	#[must_use]
	pub fn msg_type(mut self, msg_type: impl Into<String>) -> Self {
		let key_expr = self.context.key_expr(msg_type);
		self.key_expr.replace(key_expr);
		self
	}

	/// Set the queryables callback function.
	#[must_use]
	pub fn callback<F>(mut self, callback: F) -> Self
	where
		F: FnMut(&ArcContext<P>, Request) -> Result<(), DimasError>
			+ Send
			+ Sync
			+ Unpin
			+ 'static,
	{
		self.callback
			.replace(Arc::new(Mutex::new(callback)));
		self
	}

	/// Build the queryable
	/// # Errors
	///
	pub fn build(self) -> Result<Queryable<P>, DimasError> {
		let key_expr = if self.key_expr.is_none() {
			return Err(DimasError::NoKeyExpression);
		} else {
			self.key_expr.ok_or(DimasError::ShouldNotHappen)?
		};
		let callback = if self.callback.is_none() {
			return Err(DimasError::NoCallback);
		} else {
			self.callback.ok_or(DimasError::ShouldNotHappen)?
		};

		//dbg!(&key_expr);
		let q = Queryable {
			key_expr,
			callback,
			handle: None,
			context: self.context,
		};

		Ok(q)
	}

	/// Build and add the queryable to the agents context
	/// # Errors
	///
	#[cfg_attr(any(nightly, docrs), doc, doc(cfg(feature = "queryable")))]
	#[cfg(feature = "queryable")]
	pub fn add(self) -> Result<(), DimasError> {
		let collection = self.context.queryables.clone();
		let q = self.build()?;

		collection
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(q.key_expr.clone(), q);
		Ok(())
	}
}
// endregion:	--- QueryableBuilder

// region:		--- Queryable
/// Queryable
pub struct Queryable<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	key_expr: String,
	callback: QueryableCallback<P>,
	handle: Option<JoinHandle<()>>,
	context: ArcContext<P>,
}

impl<P> Debug for Queryable<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Queryable")
			.field("key_expr", &self.key_expr)
			.finish_non_exhaustive()
	}
}

impl<P> Queryable<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// Start Queryable
	/// # Panics
	///
	pub fn start(&mut self) {
		let key_expr = self.key_expr.clone();
		//dbg!(&key_expr);
		let cb = self.callback.clone();
		let ctx = self.context.clone();
		self.handle.replace(tokio::spawn(async move {
			run_queryable(key_expr, cb, ctx).await;
		}));
	}

	/// Stop Queryable
	/// # Panics
	///
	pub fn stop(&mut self) {
		self.handle
			.take()
			.expect("should never happen")
			.abort();
	}
}

//#[tracing::instrument(level = tracing::Level::DEBUG)]
async fn run_queryable<P>(key_expr: String, cb: QueryableCallback<P>, ctx: ArcContext<P>)
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	let session = ctx.communicator.session.clone();
	let subscriber = session
		.declare_queryable(&key_expr)
		.res_async()
		.await
		.expect("should never happen");

	loop {
		let query = subscriber
			.recv_async()
			.await
			.expect("should never happen");
		let request = Request(query);

		let span = span!(Level::DEBUG, "run_queryable");
		let _guard = span.enter();
		if let Err(error) = cb.lock().expect("should not happen")(&ctx, request) {
			error!("call failed with {error}");
		};
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
		is_normal::<QueryableBuilder<Props>>();
	}
}
