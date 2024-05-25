// Copyright © 2023 Stephan Kunz

//! Module `queryable` provides an information/compute provider `Queryable` which can be created using the `QueryableBuilder`.

// region:		--- modules
use dimas_core::{
	enums::{OperationState, TaskSignal},
	error::{DimasError, Result},
	message_types::Request,
	traits::{Capability, Context},
};
use std::{
	fmt::Debug,
	sync::{Arc, Mutex},
};
use tokio::task::JoinHandle;
use tracing::{error, info, instrument, warn, Level};
use zenoh::prelude::{r#async::AsyncResolve, SessionDeclarations};
use zenoh::sample::Locality;
// endregion:	--- modules

// region:		--- types
/// type defnition for the queryables atomic reference counted callback function.
pub type ArcQueryableCallback<P> =
	Arc<Mutex<dyn FnMut(&Context<P>, Request) -> Result<()> + Send + Sync + Unpin + 'static>>;
// endregion:	--- types

// region:		--- Queryable
/// Queryable
pub struct Queryable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	selector: String,
	/// Context for the Subscriber
	context: Context<P>,
	activation_state: OperationState,
	request_callback: ArcQueryableCallback<P>,
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
			.field("selector", &self.selector)
			.field("complete", &self.completeness)
			.finish_non_exhaustive()
	}
}

impl<P> Capability for Queryable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn manage_operation_state(&mut self, state: &OperationState) -> Result<()> {
		if (state >= &self.activation_state) && self.handle.is_none() {
			return self.start();
		} else if (state < &self.activation_state) && self.handle.is_some() {
			self.stop();
			return Ok(());
		}
		Ok(())
	}
}

impl<P> Queryable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor for a [`Queryable`]
	#[must_use]
	pub fn new(
		selector: String,
		context: Context<P>,
		activation_state: OperationState,
		request_callback: ArcQueryableCallback<P>,
		completeness: bool,
		allowed_origin: Locality,
	) -> Self {
		Self {
			selector,
			context,
			activation_state,
			request_callback,
			completeness,
			allowed_origin,
			handle: None,
		}
	}

	/// Get `selector`
	#[must_use]
	pub fn selector(&self) -> &str {
		&self.selector
	}

	/// Start or restart the queryable.
	/// An already running queryable will be stopped, eventually damaged Mutexes will be repaired
	#[instrument(level = Level::TRACE, skip_all)]
	fn start(&mut self) -> Result<()> {
		self.stop();

		{
			if self.request_callback.lock().is_err() {
				warn!("found poisoned put Mutex");
				self.request_callback.clear_poison();
			}
		}

		let completeness = self.completeness;
		let allowed_origin = self.allowed_origin;
		let selector = self.selector.clone();
		let cb = self.request_callback.clone();
		let ctx1 = self.context.clone();
		let ctx2 = self.context.clone();

		self.handle
			.replace(tokio::task::spawn(async move {
				let key = selector.clone();
				std::panic::set_hook(Box::new(move |reason| {
					error!("queryable panic: {}", reason);
					if let Err(reason) = ctx1
						.sender()
						.send(TaskSignal::RestartQueryable(key.clone()))
					{
						error!("could not restart queryable: {}", reason);
					} else {
						info!("restarting queryable!");
					};
				}));
				if let Err(error) =
					run_queryable(selector, cb, completeness, allowed_origin, ctx2).await
				{
					error!("queryable failed with {error}");
				};
			}));
		Ok(())
	}

	/// Stop a running Queryable
	#[instrument(level = Level::TRACE)]
	fn stop(&mut self) {
		if let Some(handle) = self.handle.take() {
			handle.abort();
		}
	}
}

#[instrument(name="queryable", level = Level::ERROR, skip_all)]
async fn run_queryable<P>(
	selector: String,
	cb: ArcQueryableCallback<P>,
	completeness: bool,
	allowed_origin: Locality,
	ctx: Context<P>,
) -> Result<()>
where
	P: Send + Sync + Unpin + 'static,
{
	let subscriber = ctx
		.session()
		.declare_queryable(&selector)
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
	}
}
