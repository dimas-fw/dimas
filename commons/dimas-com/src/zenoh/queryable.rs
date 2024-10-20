// Copyright © 2023 Stephan Kunz

//! Module `queryable` provides an information/compute provider `Queryable` which can be created using the `QueryableBuilder`.

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use alloc::sync::Arc;
use core::fmt::Debug;
use dimas_core::{
	enums::{OperationState, TaskSignal},
	message_types::QueryMsg,
	traits::{Capability, Context},
	Result,
};
use futures::future::BoxFuture;
#[cfg(feature = "std")]
use std::prelude::rust_2021::*;
#[cfg(feature = "std")]
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{error, info, instrument, warn, Level};
#[cfg(feature = "unstable")]
use zenoh::sample::Locality;
// endregion:	--- modules

// region:    	--- types
/// type defnition for a queryables `request` callback
pub type GetCallback<P> =
	Box<dyn FnMut(Context<P>, QueryMsg) -> BoxFuture<'static, Result<()>> + Send + Sync>;
/// type defnition for a queryables atomic reference counted `request` callback
pub type ArcGetCallback<P> = Arc<Mutex<GetCallback<P>>>;
// endregion: 	--- types

// region:		--- Queryable
/// Queryable
pub struct Queryable<P>
where
	P: Send + Sync + 'static,
{
	selector: String,
	/// Context for the Subscriber
	context: Context<P>,
	activation_state: OperationState,
	callback: ArcGetCallback<P>,
	completeness: bool,
	#[cfg(feature = "unstable")]
	allowed_origin: Locality,
	handle: Option<JoinHandle<()>>,
}

impl<P> Debug for Queryable<P>
where
	P: Send + Sync + 'static,
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("Queryable")
			.field("selector", &self.selector)
			.field("complete", &self.completeness)
			.finish_non_exhaustive()
	}
}

impl<P> Capability for Queryable<P>
where
	P: Send + Sync + 'static,
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
	P: Send + Sync + 'static,
{
	/// Constructor for a [`Queryable`]
	#[must_use]
	pub fn new(
		selector: String,
		context: Context<P>,
		activation_state: OperationState,
		request_callback: ArcGetCallback<P>,
		completeness: bool,
		#[cfg(feature = "unstable")] allowed_origin: Locality,
	) -> Self {
		Self {
			selector,
			context,
			activation_state,
			callback: request_callback,
			completeness,
			#[cfg(feature = "unstable")]
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

		let completeness = self.completeness;
		#[cfg(feature = "unstable")]
		let allowed_origin = self.allowed_origin;
		let selector = self.selector.clone();
		let cb = self.callback.clone();
		let ctx1 = self.context.clone();
		let ctx2 = self.context.clone();

		self.handle
			.replace(tokio::task::spawn(async move {
				let key = selector.clone();
				std::panic::set_hook(Box::new(move |reason| {
					error!("queryable panic: {}", reason);
					if let Err(reason) = ctx1
						.sender()
						.blocking_send(TaskSignal::RestartQueryable(key.clone()))
					{
						error!("could not restart queryable: {}", reason);
					} else {
						info!("restarting queryable!");
					};
				}));
				if let Err(error) = run_queryable(
					selector,
					cb,
					completeness,
					#[cfg(feature = "unstable")]
					allowed_origin,
					ctx2,
				)
				.await
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
	callback: ArcGetCallback<P>,
	completeness: bool,
	#[cfg(feature = "unstable")] allowed_origin: Locality,
	ctx: Context<P>,
) -> Result<()>
where
	P: Send + Sync + 'static,
{
	let session = ctx.session();
	let builder = session
		.declare_queryable(&selector)
		.complete(completeness);
	#[cfg(feature = "unstable")]
	let builder = builder.allowed_origin(allowed_origin);

	let queryable = builder.await?;

	loop {
		let query = queryable.recv_async().await?;
		let request = QueryMsg(query);

		let ctx = ctx.clone();
		let mut lock = callback.lock().await;
		if let Err(error) = lock(ctx, request).await {
			error!("queryable callback failed with {error}");
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
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Queryable<Props>>();
	}
}
