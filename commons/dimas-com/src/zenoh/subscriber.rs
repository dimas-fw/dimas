// Copyright © 2023 Stephan Kunz

//! Module `subscriber` provides a message `Subscriber` which can be created using the `SubscriberBuilder`.
//! A `Subscriber` can optional subscribe on a delete message.

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use crate::error::Error;
use alloc::sync::Arc;
use alloc::{boxed::Box, string::String, vec::Vec};
use dimas_core::{
	Result,
	enums::{OperationState, TaskSignal},
	message_types::Message,
	traits::{Capability, Context},
};
use futures::future::BoxFuture;
#[cfg(feature = "std")]
use tokio::{sync::Mutex, task::JoinHandle};
use tracing::{Level, error, info, instrument, warn};
use zenoh::Session;
#[cfg(feature = "unstable")]
use zenoh::sample::Locality;
use zenoh::sample::SampleKind;
// endregion:	--- modules

// region:    	--- types
/// Type definition for a subscribers `put` callback
pub type PutCallback<P> =
	Box<dyn FnMut(Context<P>, Message) -> BoxFuture<'static, Result<()>> + Send + Sync>;
/// Type definition for a subscribers atomic reference counted `put` callback
pub type ArcPutCallback<P> = Arc<Mutex<PutCallback<P>>>;
/// Type definition for a subscribers `delete` callback
pub type DeleteCallback<P> =
	Box<dyn FnMut(Context<P>) -> BoxFuture<'static, Result<()>> + Send + Sync>;
/// Type definition for a subscribers atomic reference counted `delete` callback
pub type ArcDeleteCallback<P> = Arc<Mutex<DeleteCallback<P>>>;
// endregion: 	--- types

// region:		--- Subscriber
/// Subscriber
pub struct Subscriber<P>
where
	P: Send + Sync + 'static,
{
	/// the zenoh session this subscriber belongs to
	session: Arc<Session>,
	/// The subscribers key expression
	selector: String,
	/// Context for the Subscriber
	context: Context<P>,
	/// [`OperationState`] on which this subscriber is started
	activation_state: OperationState,
	#[cfg(feature = "unstable")]
	allowed_origin: Locality,
	put_callback: ArcPutCallback<P>,
	delete_callback: Option<ArcDeleteCallback<P>>,
	handle: std::sync::Mutex<Option<JoinHandle<()>>>,
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

impl<P> crate::traits::Responder for Subscriber<P>
where
	P: Send + Sync + 'static,
{
	/// Get `selector`
	fn selector(&self) -> &str {
		&self.selector
	}
}

impl<P> Capability for Subscriber<P>
where
	P: Send + Sync + 'static,
{
	fn manage_operation_state(&self, state: &OperationState) -> Result<()> {
		if state >= &self.activation_state {
			self.start()
		} else if state < &self.activation_state {
			self.stop()
		} else {
			Ok(())
		}
	}
}

impl<P> Subscriber<P>
where
	P: Send + Sync + 'static,
{
	/// Constructor for a [`Subscriber`].
	#[must_use]
	pub fn new(
		session: Arc<Session>,
		selector: String,
		context: Context<P>,
		activation_state: OperationState,
		#[cfg(feature = "unstable")] allowed_origin: Locality,
		put_callback: ArcPutCallback<P>,
		delete_callback: Option<ArcDeleteCallback<P>>,
	) -> Self {
		Self {
			session,
			selector,
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_origin,
			put_callback,
			delete_callback,
			handle: std::sync::Mutex::new(None),
		}
	}
	/// Start or restart the subscriber.
	/// An already running subscriber will be stopped.
	#[instrument(level = Level::TRACE, skip_all)]
	fn start(&self) -> Result<()> {
		self.stop()?;

		let selector = self.selector.clone();
		let p_cb = self.put_callback.clone();
		let d_cb = self.delete_callback.clone();
		let ctx1 = self.context.clone();
		let ctx2 = self.context.clone();
		let session = self.session.clone();
		#[cfg(feature = "unstable")]
		let allowed_origin = self.allowed_origin;

		self.handle.lock().map_or_else(
			|_| todo!(),
			|mut handle| {
				handle.replace(tokio::task::spawn(async move {
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
						session,
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
			},
		)
	}

	/// Stop a running Subscriber
	#[instrument(level = Level::TRACE, skip_all)]
	fn stop(&self) -> Result<()> {
		self.handle.lock().map_or_else(
			|_| todo!(),
			|mut handle| {
				handle.take();
				Ok(())
			},
		)
	}
}

#[instrument(name="subscriber", level = Level::ERROR, skip_all)]
async fn run_subscriber<P>(
	session: Arc<Session>,
	selector: String,
	#[cfg(feature = "unstable")] allowed_origin: Locality,
	p_cb: ArcPutCallback<P>,
	d_cb: Option<ArcDeleteCallback<P>>,
	ctx: Context<P>,
) -> Result<()>
where
	P: Send + Sync + 'static,
{
	let builder = session.declare_subscriber(&selector);

	#[cfg(feature = "unstable")]
	let builder = builder.allowed_origin(allowed_origin);

	let subscriber = builder.await?;

	loop {
		let sample = subscriber
			.recv_async()
			.await
			.map_err(|source| Error::SubscriberCreation { source })?;

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
