// Copyright © 2023 Stephan Kunz

//! Module `subscriber` provides a message `Subscriber` which can be created using the `SubscriberBuilder`.
//! A `Subscriber` can optional subscribe on a delete message.

// region:		--- modules
// these ones are only for doc needed
use super::{ArcSubscriberDeleteCallback, ArcSubscriberPutCallback};
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
#[cfg(feature = "unstable")]
use zenoh::sample::Locality;
use zenoh::sample::SampleKind;
// endregion:	--- modules

// region:		--- Subscriber
/// Subscriber
pub struct Subscriber<P>
where
	P: Send + Sync + 'static,
{
	/// The subscribers key expression
	selector: String,
	/// Context for the Subscriber
	context: Context<P>,
	/// [`OperationState`] on which this subscriber is started
	activation_state: OperationState,
	#[cfg(feature = "unstable")]
	allowed_origin: Locality,
	put_callback: ArcSubscriberPutCallback<P>,
	delete_callback: Option<ArcSubscriberDeleteCallback<P>>,
	handle: Option<JoinHandle<()>>,
}

impl<P> core::fmt::Debug for Subscriber<P>
where
	P: Send + Sync + 'static,
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("Subscriber")
			.field("selector", &self.selector)
			.finish_non_exhaustive()
	}
}

impl<P> Capability for Subscriber<P>
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

impl<P> Subscriber<P>
where
	P: Send + Sync + 'static,
{
	/// Constructor for a [`Subscriber`].
	#[must_use]
	pub fn new(
		selector: String,
		context: Context<P>,
		activation_state: OperationState,
		#[cfg(feature = "unstable")] allowed_origin: Locality,
		put_callback: ArcSubscriberPutCallback<P>,
		delete_callback: Option<ArcSubscriberDeleteCallback<P>>,
	) -> Self {
		Self {
			selector,
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_origin,
			put_callback,
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

		let selector = self.selector.clone();
		let p_cb = self.put_callback.clone();
		let d_cb = self.delete_callback.clone();
		let ctx1 = self.context.clone();
		let ctx2 = self.context.clone();
		#[cfg(feature = "unstable")]
		let allowed_origin = self.allowed_origin;

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
				if let Err(error) = run_subscriber(
					selector,
					#[cfg(feature = "unstable")]
					allowed_origin,
					p_cb,
					d_cb,
					ctx2.clone(),
				)
				.await
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
	#[cfg(feature = "unstable")] allowed_origin: Locality,
	p_cb: ArcSubscriberPutCallback<P>,
	d_cb: Option<ArcSubscriberDeleteCallback<P>>,
	ctx: Context<P>,
) -> Result<()>
where
	P: Send + Sync + 'static,
{
	let session = ctx.session();
	let builder = session.declare_subscriber(&selector);

	#[cfg(feature = "unstable")]
	let builder = builder.allowed_origin(allowed_origin);

	let subscriber = builder.await?;

	loop {
		let sample = subscriber
			.recv_async()
			.await
			.map_err(|_| DimasError::ShouldNotHappen)?;

		match sample.kind() {
			SampleKind::Put => {
				let content: Vec<u8> = sample.payload().to_bytes().into_owned();
				let msg = Message::new(content);
				let mut lock = p_cb.lock().await;
				let ctx = ctx.clone();
				if let Err(error) = lock(ctx, msg).await {
					error!("subscriber put callback failed with {error}");
				}
			}
			SampleKind::Delete => {
				if let Some(cb) = d_cb.clone() {
					let ctx = ctx.clone();
					let mut lock = cb.lock().await;
					if let Err(error) = lock(ctx).await {
						error!("subscriber delete callback failed with {error}");
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
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Subscriber<Props>>();
	}
}
