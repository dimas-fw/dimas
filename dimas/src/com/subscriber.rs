// Copyright © 2023 Stephan Kunz

//! Module `subscriber` provides a message `Subscriber` which can be created using the `SubscriberBuilder`.
//! A `Subscriber` can optional subscribe on a delete message.

// region:		--- modules
// these ones are only for doc needed
use super::{ArcDeleteCallback, ArcPutCallback};
#[cfg(doc)]
use crate::agent::Agent;
use dimas_core::{
	enums::{OperationState, TaskSignal},
	error::{DimasError, Result},
	message_types::Message,
	traits::{Capability, Context},
};
use tokio::task::JoinHandle;
use tracing::{error, info, instrument, warn, Level};
use zenoh::{pubsub::Reliability, sample::SampleKind, session::SessionDeclarations};
// endregion:	--- modules

// region:		--- Subscriber
/// Subscriber
pub struct Subscriber<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// The subscribers key expression
	selector: String,
	/// Context for the Subscriber
	context: Context<P>,
	/// [`OperationState`] on which this subscriber is started
	activation_state: OperationState,
	put_callback: ArcPutCallback<P>,
	reliability: Reliability,
	delete_callback: Option<ArcDeleteCallback<P>>,
	handle: Option<JoinHandle<()>>,
}

impl<P> std::fmt::Debug for Subscriber<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Subscriber")
			.field("selector", &self.selector)
			.finish_non_exhaustive()
	}
}

impl<P> Capability for Subscriber<P>
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

impl<P> Subscriber<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor for a [`Subscriber`].
	#[must_use]
	pub fn new(
		selector: String,
		context: Context<P>,
		activation_state: OperationState,
		put_callback: ArcPutCallback<P>,
		reliability: Reliability,
		delete_callback: Option<ArcDeleteCallback<P>>,
	) -> Self {
		Self {
			selector,
			context,
			activation_state,
			put_callback,
			reliability,
			delete_callback,
			handle: None,
		}
	}

	/// Get `selector`
	#[must_use]
	pub fn selector(&self) -> &str {
		&self.selector
	}

	/// Start or restart the subscriber.
	/// An already running subscriber will be stopped, eventually damaged Mutexes will be repaired
	#[instrument(level = Level::TRACE, skip_all)]
	fn start(&mut self) -> Result<()> {
		self.stop();

		{
			if self.put_callback.lock().is_err() {
				warn!("found poisoned put Mutex");
				self.put_callback.clear_poison();
			}

			if let Some(dcb) = self.delete_callback.clone() {
				if dcb.lock().is_err() {
					warn!("found poisoned delete Mutex");
					dcb.clear_poison();
				}
			}
		}

		let selector = self.selector.clone();
		let p_cb = self.put_callback.clone();
		let d_cb = self.delete_callback.clone();
		let reliability = self.reliability;
		let ctx1 = self.context.clone();
		let ctx2 = self.context.clone();

		self.handle
			.replace(tokio::task::spawn(async move {
				let key = selector.clone();
				std::panic::set_hook(Box::new(move |reason| {
					error!("subscriber panic: {}", reason);
					if let Err(reason) = ctx1
						.sender()
						.blocking_send(TaskSignal::RestartSubscriber(key.clone()))
					{
						error!("could not restart subscriber: {}", reason);
					} else {
						info!("restarting subscriber!");
					};
				}));
				if let Err(error) =
					run_subscriber(selector, p_cb, d_cb, reliability, ctx2.clone()).await
				{
					error!("spawning subscriber failed with {error}");
				};
			}));
		Ok(())
	}

	/// Stop a running Subscriber
	#[instrument(level = Level::TRACE, skip_all)]
	fn stop(&mut self) {
		if let Some(handle) = self.handle.take() {
			handle.abort();
		}
	}
}

#[instrument(name="subscriber", level = Level::ERROR, skip_all)]
async fn run_subscriber<P>(
	selector: String,
	p_cb: ArcPutCallback<P>,
	d_cb: Option<ArcDeleteCallback<P>>,
	reliability: Reliability,
	ctx: Context<P>,
) -> Result<()>
where
	P: Send + Sync + Unpin + 'static,
{
	let subscriber = ctx
		.session()
		.declare_subscriber(&selector)
		.reliability(reliability)
		.await?;

	loop {
		let sample = subscriber
			.recv_async()
			.await
			.map_err(|_| DimasError::ShouldNotHappen)?;

		match sample.kind() {
			SampleKind::Put => {
				let content: Vec<u8> = sample.payload().into();
				let msg = Message::new(content);
				match p_cb.lock() {
					Ok(mut lock) => {
						if let Err(error) = lock(&ctx, msg) {
							error!("subscriber put callback failed with {error}");
						}
					}
					Err(err) => {
						error!("subscriber put callback lock failed with {err}");
					}
				}
			}
			SampleKind::Delete => {
				if let Some(cb) = d_cb.clone() {
					match cb.lock() {
						Ok(mut lock) => {
							if let Err(error) = lock(&ctx) {
								error!("subscriber delete callback failed with {error}");
							}
						}
						Err(err) => {
							error!("subscriber delete callback lock failed with {err}");
						}
					}
				}
			}
		}
	}
}
// endregion:	--- Subscriber

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Subscriber<Props>>();
	}
}
