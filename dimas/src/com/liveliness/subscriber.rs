// Copyright © 2023 Stephan Kunz

//! Module `liveliness` provides a `LivelinessSubscriber` which can be created using the `LivelinessSubscriberBuilder`.
//! A `LivelinessSubscriber` can optional subscribe on a delete message.

// region:		--- modules
use super::ArcLivelinessCallback;
use core::time::Duration;
use dimas_core::{
	enums::{OperationState, TaskSignal},
	error::Result,
	traits::{Capability, Context},
};
#[cfg(doc)]
use std::collections::HashMap;
use tokio::task::JoinHandle;
use tracing::info;
use tracing::{error, instrument, warn, Level};
use zenoh::sample::SampleKind;
// endregion:	--- modules

// region:		--- LivelinessSubscriber
/// Liveliness Subscriber
#[allow(clippy::module_name_repetitions)]
pub struct LivelinessSubscriber<P>
where
	P: Send + Sync + 'static,
{
	token: String,
	context: Context<P>,
	activation_state: OperationState,
	put_callback: ArcLivelinessCallback<P>,
	delete_callback: Option<ArcLivelinessCallback<P>>,
	handle: Option<JoinHandle<()>>,
}

impl<P> core::fmt::Debug for LivelinessSubscriber<P>
where
	P: Send + Sync + 'static,
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("LivelinessSubscriber")
			.finish_non_exhaustive()
	}
}

impl<P> Capability for LivelinessSubscriber<P>
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

impl<P> LivelinessSubscriber<P>
where
	P: Send + Sync + 'static,
{
	/// Constructor for a [`LivelinessSubscriber`]
	pub fn new(
		token: String,
		context: Context<P>,
		activation_state: OperationState,
		put_callback: ArcLivelinessCallback<P>,
		delete_callback: Option<ArcLivelinessCallback<P>>,
	) -> Self {
		Self {
			token,
			context,
			activation_state,
			put_callback,
			delete_callback,
			handle: None,
		}
	}

	/// get token
	#[must_use]
	pub fn token(&self) -> String {
		self.token.clone()
	}

	/// Start or restart the liveliness subscriber.
	/// An already running subscriber will be stopped before,
	/// eventually damaged Mutexes will be repaired
	#[instrument(level = Level::TRACE, skip_all)]
	fn start(&mut self) -> Result<()> {
		self.stop();

		// liveliness handling
		let key = self.token.clone();
		let token1 = self.token.clone();
		let token2 = self.token.clone();
		let p_cb1 = self.put_callback.clone();
		let p_cb2 = self.put_callback.clone();
		let d_cb = self.delete_callback.clone();
		let ctx = self.context.clone();
		let ctx1 = self.context.clone();
		let ctx2 = self.context.clone();

		self.handle
			.replace(tokio::task::spawn(async move {
				std::panic::set_hook(Box::new(move |reason| {
					error!("liveliness subscriber panic: {}", reason);
					if let Err(reason) = ctx
						.sender()
						.blocking_send(TaskSignal::RestartLiveliness(key.clone()))
					{
						error!("could not restart liveliness subscriber: {}", reason);
					} else {
						info!("restarting liveliness subscriber!");
					};
				}));

				// the initial liveliness query
				if let Err(error) = run_initial(token1, p_cb1, ctx1).await {
					error!("running initial liveliness failed with {error}");
				};

				tokio::time::sleep(Duration::from_millis(1)).await;

				// the liveliness subscriber
				if let Err(error) = run_liveliness(token2, p_cb2, d_cb, ctx2).await {
					error!("running liveliness subscriber failed with {error}");
				};
			}));
		Ok(())
	}

	/// Stop a running LivelinessSubscriber
	#[instrument(level = Level::TRACE)]
	fn stop(&mut self) {
		if let Some(handle) = self.handle.take() {
			handle.abort();
		}
	}
}

#[instrument(name="liveliness", level = Level::ERROR, skip_all)]
async fn run_liveliness<P>(
	token: String,
	p_cb: ArcLivelinessCallback<P>,
	d_cb: Option<ArcLivelinessCallback<P>>,
	ctx: Context<P>,
) -> Result<()> {
	let subscriber = ctx
		.session()
		.liveliness()
		.declare_subscriber(&token)
		.await?;

	loop {
		let result = subscriber.recv_async().await;
		match result {
			Ok(sample) => {
				let id = sample.key_expr().split('/').last().unwrap_or("");
				// skip own live message
				if id == ctx.uuid() {
					continue;
				};
				match sample.kind() {
					SampleKind::Put => {
						let ctx = ctx.clone();
						let mut lock = p_cb.lock().await;
						if let Err(error) = lock(ctx, id.to_string()).await {
							error!("liveliness put callback failed with {error}");
						}
					}
					SampleKind::Delete => {
						if let Some(cb) = d_cb.clone() {
							let ctx = ctx.clone();
							let mut lock = cb.lock().await;
							if let Err(err) = lock(ctx, id.to_string()).await {
								error!("liveliness delete callback failed with {err}");
							}
						}
					}
				}
			}
			Err(error) => {
				error!("liveliness receive failed with {error}");
			}
		}
	}
}

#[instrument(name="initial liveliness", level = Level::ERROR, skip_all)]
async fn run_initial<P>(
	token: String,
	p_cb: ArcLivelinessCallback<P>,
	ctx: Context<P>,
) -> Result<()> {
	let result = ctx
		.session()
		.liveliness()
		.get(&token)
		.timeout(Duration::from_millis(100))
		.await;

	match result {
		Ok(replies) => {
			while let Ok(reply) = replies.recv_async().await {
				match reply.result() {
					Ok(sample) => {
						let id = sample.key_expr().split('/').last().unwrap_or("");
						// skip own live message
						if id == ctx.uuid() {
							continue;
						};
						let ctx = ctx.clone();
						let mut lock = p_cb.lock().await;
						if let Err(error) = lock(ctx, id.to_string()).await {
							error!("lveliness initial query put callback failed with {error}");
						}
					}
					Err(err) => error!(">> liveliness initial query failed with {:?})", err),
				}
			}
		}
		Err(error) => {
			error!("livelieness initial query receive failed with {error}");
		}
	}
	Ok(())
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
		is_normal::<LivelinessSubscriber<Props>>();
	}
}
