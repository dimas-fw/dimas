// Copyright © 2024 Stephan Kunz

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use alloc::sync::Arc;
use bitcode::decode;
use core::time::Duration;
use dimas_core::{
	enums::OperationState,
	message_types::{ControlResponse, Message, ObservableResponse},
	traits::{Capability, Context},
	utils::{cancel_selector_from, feedback_selector_from, request_selector_from},
	Result,
};
use futures::future::BoxFuture;
#[cfg(feature = "std")]
use std::prelude::rust_2021::*;
#[cfg(feature = "std")]
use tokio::sync::Mutex;
#[cfg(feature = "std")]
use tokio::task::JoinHandle;
use tracing::{error, instrument, warn, Level};
#[cfg(feature = "unstable")]
use zenoh::sample::Locality;
use zenoh::{
	query::{ConsolidationMode, QueryTarget},
	sample::SampleKind,
	Wait,
};

use crate::error::Error;
// endregion:	--- modules

// region:    	--- types
/// Type definition for an observers `control` callback
pub type ControlCallback<P> =
	Box<dyn FnMut(Context<P>, ControlResponse) -> BoxFuture<'static, Result<()>> + Send + Sync>;
/// Type definition for an observers atomic reference counted `control` callback
pub type ArcControlCallback<P> = Arc<Mutex<ControlCallback<P>>>;
/// Type definition for an observers `response` callback
pub type ResponseCallback<P> =
	Box<dyn FnMut(Context<P>, ObservableResponse) -> BoxFuture<'static, Result<()>> + Send + Sync>;
/// Type definition for an observers atomic reference counted `response` callback
pub type ArcResponseCallback<P> = Arc<Mutex<ResponseCallback<P>>>;
// endregion: 	--- types

// region:		--- Observer
/// Observer
pub struct Observer<P>
where
	P: Send + Sync + 'static,
{
	/// The observers key expression
	selector: String,
	/// Context for the Observer
	context: Context<P>,
	activation_state: OperationState,
	/// callback for control request results
	control_callback: ArcControlCallback<P>,
	/// callback for responses
	response_callback: ArcResponseCallback<P>,
	/// timeout value
	timeout: Duration,
	handle: Option<JoinHandle<()>>,
}

impl<P> core::fmt::Debug for Observer<P>
where
	P: Send + Sync + 'static,
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("Observer").finish_non_exhaustive()
	}
}

impl<P> crate::traits::Observer for Observer<P>
where
	P: Send + Sync + 'static,
{
	/// Get `selector`
	fn selector(&self) -> &str {
		&self.selector
	}

	/// Cancel a running observation
	#[instrument(level = Level::ERROR, skip_all)]
	fn cancel(&self) -> Result<()> {
		let session = self.context.session();
		// TODO: make a proper "key: value" implementation
		let selector = cancel_selector_from(&self.selector);
		let builder = session
			.get(&selector)
			.target(QueryTarget::All)
			.consolidation(ConsolidationMode::None)
			.timeout(self.timeout);

		#[cfg(feature = "unstable")]
		let builder = builder.allowed_destination(Locality::Any);

		let query = builder
			.wait()
			.map_err(|source| Error::QueryCreation { source })?;

		let mut unreached = true;
		let mut retry_count = 0u8;

		while unreached && retry_count <= 5 {
			retry_count += 1;
			while let Ok(reply) = query.recv() {
				match reply.result() {
					Ok(sample) => match sample.kind() {
						SampleKind::Put => {
							let ccb = self.control_callback.clone();
							let ctx = self.context.clone();
							let content: Vec<u8> = sample.payload().to_bytes().into_owned();
							let response: ControlResponse = decode(&content)?;
							if matches!(response, ControlResponse::Canceled) {
								// without spawning possible deadlock when called inside an control response
								tokio::spawn(async move {
									let mut lock = ccb.lock().await;
									if let Err(error) = lock(ctx.clone(), response).await {
										error!("callback failed with {error}");
									}
								});
							} else {
								error!("unexpected response on cancelation");
							};
						}
						SampleKind::Delete => {
							error!("Delete in cancel");
						}
					},
					Err(err) => error!("receive error: {:?})", err),
				}
				unreached = false;
			}
			if unreached {
				if retry_count < 5 {
					std::thread::sleep(self.timeout);
				} else {
					return Err(Error::AccessingObservable {
						selector: self.selector.to_string(),
					}
					.into());
				}
			}
		}
		Ok(())
	}

	/// Request an observation with an optional [`Message`].
	#[instrument(level = Level::ERROR, skip_all)]
	fn request(&self, message: Option<Message>) -> Result<()> {
		let session = self.context.session();
		// TODO: make a proper "key: value" implementation
		let selector = request_selector_from(&self.selector);
		let mut query = session
			.get(&selector)
			.target(QueryTarget::All)
			.consolidation(ConsolidationMode::None)
			.timeout(self.timeout);

		if let Some(message) = message {
			let value = message.value().to_owned();
			query = query.payload(value);
		};

		#[cfg(feature = "unstable")]
		let query = query.allowed_destination(Locality::Any);

		let query = query
			.wait()
			.map_err(|source| Error::QueryCreation { source })?;

		let mut unreached = true;
		let mut retry_count = 0u8;

		while unreached && retry_count <= 5 {
			retry_count += 1;
			while let Ok(reply) = query.recv() {
				match reply.result() {
					Ok(sample) => match sample.kind() {
						SampleKind::Put => {
							let content: Vec<u8> = sample.payload().to_bytes().into_owned();
							decode::<ControlResponse>(&content).map_or_else(
								|_| todo!(),
								|response| {
									if matches!(response, ControlResponse::Accepted) {
										let ctx = self.context.clone();
										// use "<query_selector>/feedback/<source_id/replier_id>" as key
										// in case there is no source_id/replier_id, listen on all id's
										#[cfg(not(feature = "unstable"))]
										let source_id = "*".to_string();
										#[cfg(feature = "unstable")]
										let source_id = reply.result().map_or_else(
											|_| {
												reply.replier_id().map_or_else(
													|| "*".to_string(),
													|id| id.to_string(),
												)
											},
											|sample| {
												sample.source_info().source_id().map_or_else(
													|| {
														reply.replier_id().map_or_else(
															|| "*".to_string(),
															|id| id.to_string(),
														)
													},
													|id| id.zid().to_string(),
												)
											},
										);
										let selector =
											feedback_selector_from(&self.selector, &source_id);

										let rcb = self.response_callback.clone();
										tokio::task::spawn(async move {
											if let Err(error) =
												run_observation(selector, ctx, rcb).await
											{
												error!("observation failed with {error}");
											};
										});
									};
									// call control callback
									let ctx = self.context.clone();
									let ccb = self.control_callback.clone();
									tokio::task::spawn(async move {
										let mut lock = ccb.lock().await;
										if let Err(error) = lock(ctx, response).await {
											error!("control callback failed with {error}");
										}
									});
								},
							);
						}
						SampleKind::Delete => {
							error!("Delete in request response");
						}
					},
					Err(err) => error!("request response error: {:?})", err),
				};
				unreached = false;
			}
			if unreached {
				if retry_count < 5 {
					std::thread::sleep(self.timeout);
				} else {
					return Err(Error::AccessingObservable {
						selector: self.selector.to_string(),
					}
					.into());
				}
			}
		}
		Ok(())
	}
}

impl<P> Capability for Observer<P>
where
	P: Send + Sync + 'static,
{
	fn manage_operation_state(&mut self, state: &OperationState) -> Result<()> {
		if state >= &self.activation_state {
			return self.init();
		} else if state < &self.activation_state {
			return self.de_init();
		}
		Ok(())
	}
}

impl<P> Observer<P>
where
	P: Send + Sync + 'static,
{
	/// Constructor for an [`Observer`]
	#[must_use]
	pub fn new(
		selector: String,
		context: Context<P>,
		activation_state: OperationState,
		control_callback: ArcControlCallback<P>,
		response_callback: ArcResponseCallback<P>,
		timeout: Duration,
	) -> Self {
		Self {
			selector,
			context,
			activation_state,
			control_callback,
			response_callback,
			timeout,
			handle: None,
		}
	}

	/// Initialize
	/// # Errors
	///
	#[instrument(level = Level::TRACE, skip_all)]
	fn init(&mut self) -> Result<()> {
		Ok(())
	}

	/// De-Initialize
	/// # Errors
	///
	#[allow(clippy::unnecessary_wraps)]
	fn de_init(&mut self) -> Result<()> {
		// cancel current request before stopping
		let _ = crate::traits::Observer::cancel(self);
		self.handle.take();
		Ok(())
	}
}
// endregion:	--- Observer

// region:		--- functions
#[allow(clippy::significant_drop_in_scrutinee)]
#[instrument(name="observation", level = Level::ERROR, skip_all)]
async fn run_observation<P>(
	selector: String,
	ctx: Context<P>,
	rcb: ArcResponseCallback<P>,
) -> Result<()> {
	// create the feedback subscriber
	let subscriber = ctx
		.session()
		.declare_subscriber(&selector)
		.await?;

	loop {
		match subscriber.recv_async().await {
			// feedback from observable
			Ok(sample) => {
				match sample.kind() {
					SampleKind::Put => {
						let content: Vec<u8> = sample.payload().to_bytes().into_owned();
						match decode::<ObservableResponse>(&content) {
							Ok(response) => {
								// remember to stop loop on anything that is not feedback
								let stop = !matches!(response, ObservableResponse::Feedback(_));
								let ctx = ctx.clone();
								if let Err(error) = rcb.lock().await(ctx, response).await {
									error!("response callback failed with {error}");
								};
								if stop {
									break;
								};
							}
							Err(_) => todo!(),
						};
					}
					SampleKind::Delete => {
						error!("unexpected delete in observation response");
					}
				}
			}
			Err(err) => {
				error!("observation response with {err}");
			}
		}
	}
	Ok(())
}
// endregion:	--- functions

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Observer<Props>>();
	}
}
