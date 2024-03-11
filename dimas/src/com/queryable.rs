// Copyright © 2023 Stephan Kunz

//! Module `queryable` provides an information/compute provider `Queryable` which can be created using the `QueryableBuilder`.

// region:		--- modules
use crate::{agent::Command, prelude::*};
use std::{
	fmt::Debug,
	sync::{mpsc::Sender, Mutex},
};
use tokio::task::JoinHandle;
use tracing::{error, info, instrument, warn, Level};
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

// region:		--- QueryableBuilder
/// The builder fo a queryable.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
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
		F: FnMut(&ArcContext<P>, Request) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		self.callback
			.replace(Arc::new(Mutex::new(Some(Box::new(callback)))));
		self
	}

	/// Build the queryable
	/// # Errors
	///
	pub fn build(self) -> Result<Queryable<P>> {
		let key_expr = if self.key_expr.is_none() {
			return Err(DimasError::NoKeyExpression.into());
		} else {
			self.key_expr.ok_or(DimasError::ShouldNotHappen)?
		};
		if self.callback.is_none() {
			return Err(DimasError::NoCallback.into());
		};

		let q = Queryable {
			key_expr,
			callback: self.callback,
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
	pub fn add(self) -> Result<()> {
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
	callback: Option<QueryableCallback<P>>,
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
	/// Start or restart the queryable.
	/// An already running queryable will be stopped, eventually damaged Mutexes will be repaired
	#[instrument(level = Level::TRACE)]
	pub fn start(&mut self, tx: Sender<Command>) {
		self.stop();

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
		let ctx = self.context.clone();

		self.handle.replace(tokio::spawn(async move {
			let key = key_expr.clone();
			std::panic::set_hook(Box::new(move |reason| {
				error!("queryable panic: {}", reason);
				if let Err(reason) = tx.send(Command::RestartQueryable(key.clone())) {
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
	P: Debug + Send + Sync + Unpin + 'static,
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
		is_normal::<QueryableBuilder<Props>>();
	}
}
