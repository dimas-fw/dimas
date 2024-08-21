// Copyright © 2024 Stephan Kunz

// region:		--- modules
use bitcode::decode;
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	message_types::{ControlResponse, Message, ObservableResponse},
	traits::{Capability, Context},
};
use tokio::task::JoinHandle;
use tracing::{error, instrument, warn, Level};
use zenoh::{
	pubsub::Reliability,
	query::{ConsolidationMode, QueryTarget},
	sample::{Locality, SampleKind},
	session::SessionDeclarations,
	Wait,
};

use super::{ArcObserverControlCallback, ArcObserverResponseCallback};
// endregion:	--- modules

// region:		--- Observer
/// Observer
pub struct Observer<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// The observers key expression
	selector: String,
	/// Context for the Observer
	context: Context<P>,
	activation_state: OperationState,
	/// callback for control request results
	control_callback: ArcObserverControlCallback<P>,
	/// callback for responses
	response_callback: ArcObserverResponseCallback<P>,
	handle: Option<JoinHandle<()>>,
}

impl<P> std::fmt::Debug for Observer<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Observer").finish_non_exhaustive()
	}
}

impl<P> Capability for Observer<P>
where
	P: Send + Sync + Unpin + 'static,
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
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor for an [`Observer`]
	#[must_use]
	pub fn new(
		selector: String,
		context: Context<P>,
		activation_state: OperationState,
		control_callback: ArcObserverControlCallback<P>,
		response_callback: ArcObserverResponseCallback<P>,
	) -> Self {
		Self {
			selector,
			context,
			activation_state,
			control_callback,
			response_callback,
			handle: None,
		}
	}

	/// Get `selector`
	#[must_use]
	pub fn selector(&self) -> &str {
		&self.selector
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
		self.handle.take();
		Ok(())
	}

	/// Cancel a running observation
	#[instrument(level = Level::ERROR, skip_all)]
	pub fn cancel(&self) -> Result<()> {
		let session = self.context.session();
		// TODO: make a proper "key: value" implementation
		let selector = format!("{}?cancel", &self.selector);
		let query = session
			.get(&selector)
			.target(QueryTarget::All)
			.consolidation(ConsolidationMode::None)
			.allowed_destination(Locality::Any);

		//if let Some(timeout) = self.timeout {
		//	query = query.timeout(timeout);
		//};

		//if let Some(message) = message {
		//	let value = message.value().to_owned();
		//	query = query.with_value(value);
		//};

		let replies = query
			.wait()
			.map_err(|_| DimasError::ShouldNotHappen)?;

		while let Ok(reply) = replies.recv() {
			match reply.result() {
				Ok(sample) => match sample.kind() {
					SampleKind::Put => {
						let ccb = self.control_callback.clone();
						let ctx = self.context.clone();
						let content: Vec<u8> = sample.payload().into();
						let response: ControlResponse = decode(&content)?;
						if matches!(response, ControlResponse::Canceled) {
							// without spawning possible deadlock when called inside an control response
							tokio::spawn(async move {
								match ccb.lock() {
									Ok(mut lock) => {
										if let Err(error) = lock(&ctx.clone(), response) {
											error!("callback failed with {error}");
										}
									}
									Err(err) => {
										error!("callback lock failed with {err}");
									}
								};
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
		}
		Ok(())
	}

	/// Request an observation with an optional [`Message`].
	#[instrument(level = Level::ERROR, skip_all)]
	pub fn request(&self, message: Option<Message>) -> Result<()> {
		let session = self.context.session();
		// TODO: make a proper "key: value" implementation
		let selector = format!("{}?request", &self.selector);
		let mut query = session
			.get(&selector)
			.target(QueryTarget::All)
			.consolidation(ConsolidationMode::None)
			.allowed_destination(Locality::Any);

		//if let Some(timeout) = self.timeout {
		//	query = query.timeout(timeout);
		//};

		if let Some(message) = message {
			let value = message.value().to_owned();
			query = query.payload(value);
		};

		let replies = query
			.wait()
			.map_err(|_| DimasError::ShouldNotHappen)?;

		while let Ok(reply) = replies.recv() {
			match reply.result() {
				Ok(sample) => match sample.kind() {
					SampleKind::Put => {
						let content: Vec<u8> = sample.payload().into();
						decode::<ControlResponse>(&content).map_or_else(
							|_| todo!(),
							|response| {
								if matches!(response, ControlResponse::Accepted) {
									let ctx = self.context.clone();
									// use "<query_selector>/feedback/<source_id/replier_id>" as key
									// in case there is no source_id/replier_id, listen on all id's
									let source_id = reply.result().map_or_else(
										|_| {
											reply.replier_id().map_or_else(
												|| "*".to_string(),
												|id| id.to_string(),
											)
										},
										|sample| {
											sample.source_info().source_id.map_or_else(
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
										format!("{}/feedback/{}", &self.selector, &source_id);

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
								match self.control_callback.lock() {
									Ok(mut lock) => {
										if let Err(error) = lock(&self.context.clone(), response) {
											error!("control callback failed with {error}");
										}
									}
									Err(err) => {
										error!("control callback lock failed with {err}");
									}
								};
							},
						);
					}
					SampleKind::Delete => {
						error!("Delete in request response");
					}
				},
				Err(err) => error!("request response error: {:?})", err),
			};
		}
		Ok(())
	}
}
// endregion:	--- Observer

// region:		--- functions
#[instrument(name="observation", level = Level::ERROR, skip_all)]
async fn run_observation<P>(
	selector: String,
	ctx: Context<P>,
	rcb: ArcObserverResponseCallback<P>,
) -> Result<()> {
	// create the feedback subscriber
	let subscriber = ctx
		.session()
		.declare_subscriber(&selector)
		.reliability(Reliability::Reliable)
		.await?;

	loop {
		match subscriber.recv_async().await {
			// feedback from observable
			Ok(sample) => {
				match sample.kind() {
					SampleKind::Put => {
						let content: Vec<u8> = sample.payload().into();
						match decode::<ObservableResponse>(&content) {
							Ok(response) => {
								// remember to stop loop on anything that is not feedback
								let stop = !matches!(response, ObservableResponse::Feedback(_));
								rcb.lock().map_or_else(
									|_| todo!(),
									|mut cb| {
										if let Err(error) = cb(&ctx, response) {
											error!("response callback failed with {error}");
										};
									},
								);
								if stop {
									break;
								}
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
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Observer<Props>>();
	}
}
