// Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas_core::{
	enums::{OperationState, TaskSignal},
	error::{DimasError, Result},
	message_types::ObserverMsg,
	traits::{Capability, Context},
};
use tokio::task::JoinHandle;
use tracing::{error, info, instrument, warn, Level};
use zenoh::sample::Locality;
use zenoh::session::SessionDeclarations;

use super::ArcObservableCallback;
// endregion:	--- modules

// region:		--- Observable
/// Observable
pub struct Observable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// The observables key expression
	selector: String,
	/// Context for the Observable
	context: Context<P>,
	activation_state: OperationState,
	/// callback for observation request and cancelation
	callback: ArcObservableCallback<P>,
	/// handle for the asynchronous feedback publisher
	feedback: Option<JoinHandle<()>>,
	handle: Option<JoinHandle<()>>,
}

impl<P> std::fmt::Debug for Observable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Observable")
			.finish_non_exhaustive()
	}
}

impl<P> Capability for Observable<P>
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

impl<P> Observable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor for an [`Observable`]
	#[must_use]
	pub fn new(
		selector: String,
		context: Context<P>,
		activation_state: OperationState,
		callback: ArcObservableCallback<P>,
	) -> Self {
		Self {
			selector,
			context,
			activation_state,
			callback,
			feedback: None,
			handle: None,
		}
	}

	/// Get `selector`
	#[must_use]
	pub fn selector(&self) -> &str {
		&self.selector
	}

	/// Start or restart the Observable.
	/// An already running Observable will be stopped, eventually damaged Mutexes will be repaired
	#[instrument(level = Level::TRACE, skip_all)]
	fn start(&mut self) -> Result<()> {
		self.stop();

		{
			if self.callback.lock().is_err() {
				warn!("found poisoned put Mutex");
				self.callback.clear_poison();
			}
		}

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
						.send(TaskSignal::RestartObservable(key.clone()))
					{
						error!("could not restart observable: {}", reason);
					} else {
						info!("restarting observable!");
					};
				}));
				if let Err(error) = run_observable(selector, cb, ctx2).await {
					error!("observable failed with {error}");
				};
			}));

		Ok(())
	}

	/// Stop a running Observable
	#[instrument(level = Level::TRACE, skip_all)]
	fn stop(&mut self) {
		if let Some(handle) = self.handle.take() {
			handle.abort();
		}
	}
}

#[instrument(name="observable", level = Level::ERROR, skip_all)]
async fn run_observable<P>(
	selector: String,
	callback: ArcObservableCallback<P>,
	ctx: Context<P>,
) -> Result<()>
where
	P: Send + Sync + Unpin + 'static,
{
	let queryable = ctx
		.session()
		.declare_queryable(&selector)
		.complete(true)
		.allowed_origin(Locality::Any)
		.await?;

	loop {
		let query = queryable
			.recv_async()
			.await
			.map_err(|_| DimasError::ShouldNotHappen)?;
		let p = query.parameters().as_str();
		// TODO: make a proper "key: value" implementation
		if p == "request" {
			let request = ObserverMsg(query);
			match callback.lock() {
				Ok(mut lock) => {
					if let Err(error) = lock(&ctx, request) {
						error!("observable callback failed with {error}");
					}
				}
				Err(err) => {
					error!("observable callback failed with {err}");
				}
			}
		} else {
			error!("observable got unknown parameter: {p}");
		}
	}
}
// endregion:	--- Observable

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Observable<Props>>();
	}
}
