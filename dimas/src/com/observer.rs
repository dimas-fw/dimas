// Copyright © 2024 Stephan Kunz

// region:		--- modules
use bitcode::decode;
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	message_types::{Message, ResponseType},
	traits::{Capability, Context},
};
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use tracing::{error, instrument, warn, Level};
use zenoh::{
	core::Wait,
	query::{ConsolidationMode, QueryTarget},
	sample::{Locality, SampleKind},
	subscriber::Reliability,
};

use super::{subscriber::Subscriber, ArcPutCallback};
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
	/// callback for feedback including result
	callback: ArcPutCallback<P>,
	/// handle for the asynchronous feedback subscriber
	feedback: Arc<Mutex<Option<Subscriber<P>>>>,
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
		callback: ArcPutCallback<P>,
	) -> Self {
		Self {
			selector,
			context,
			activation_state,
			callback,
			feedback: Arc::new(Mutex::new(None)),
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

	/// Run an observation with an optional [`Message`].
	#[instrument(name="observer", level = Level::ERROR, skip_all)]
	pub fn observe(&self, message: Option<Message>) -> Result<()> {
		let cb = self.callback.clone();
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
						let response: ResponseType = decode(&content)?;
						match response {
							ResponseType::Accepted(content) => {
								// create the subscriber for feedback
								// use "<query_selector>/feedback/<replier_id>" as key
								// in case there is no replier_id, listen on all id's
								let replier_id = reply
									.replier_id()
									.map_or_else(|| "*".to_string(), |id| id.to_string());
								let subscriber_selector =
									format!("{}/feedback/{}", &self.selector, &replier_id);
								dbg!(&subscriber_selector);
								let mut sub = Subscriber::new(
									subscriber_selector,
									self.context.clone(),
									OperationState::Created,
									self.callback.clone(),
									Reliability::Reliable,
									None,
								);
								sub.manage_operation_state(&OperationState::Active)?;
								self.feedback
									.lock()
									.map_or_else(|_| todo!(), |mut fb| fb.replace(sub));
								// TODO:
								// clarify if this is needed
								let msg = Message(content);
								let guard = cb.lock();
								match guard {
									Ok(mut lock) => {
										if let Err(error) = lock(&self.context.clone(), msg) {
											error!("callback failed with {error}");
										}
									}
									Err(err) => {
										error!("callback lock failed with {err}");
									}
								}
							}
							ResponseType::Declined => {
								todo!()
							}
						}
					}
					SampleKind::Delete => {
						error!("Delete in observe");
					}
				},
				Err(err) => error!("receive error: {:?})", err),
			}
		}
		Ok(())
	}

	/// Cancel a running observation
	#[instrument(name="observer", level = Level::ERROR, skip_all)]
	pub fn cancel(&self) -> Result<()> {
		let cb = self.callback.clone();
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
						let content: Vec<u8> = sample.payload().into();
						let msg = Message(content);
						let guard = cb.lock();
						match guard {
							Ok(mut lock) => {
								if let Err(error) = lock(&self.context.clone(), msg) {
									error!("callback failed with {error}");
								}
							}
							Err(err) => {
								error!("callback lock failed with {err}");
							}
						}
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
}
// endregion:	--- Observer

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
