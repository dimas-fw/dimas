// Copyright © 2023 Stephan Kunz

//! Module `liveliness` provides a `LivelinessSubscriber` which can be created using the `LivelinessSubscriberBuilder`.
//!
//! A `LivelinessSubscriber` subscribes on put and delete messages.

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use alloc::sync::Arc;
use alloc::{
	boxed::Box,
	collections::BTreeSet,
	string::{String, ToString},
};
use dimas_core::{
	Result,
	enums::{OperationState, TaskSignal},
	traits::{Capability, Context},
};
use futures::future::BoxFuture;
#[cfg(feature = "std")]
use tokio::{sync::Mutex, task::JoinHandle};
use tracing::info;
use tracing::{Level, error, instrument, warn};
use zenoh::Session;
use zenoh::sample::SampleKind;
// endregion:	--- modules

// region:    	--- types
/// Type definition for a boxed liveliness subscribers callback
#[allow(clippy::module_name_repetitions)]
pub type LivelinessCallback<P> =
	Box<dyn FnMut(Context<P>, String) -> BoxFuture<'static, Result<()>> + Send + Sync>;
/// Type definition for a liveliness subscribers atomic reference counted callback
pub type ArcLivelinessCallback<P> = Arc<Mutex<LivelinessCallback<P>>>;
// endregion: 	--- types

// region:		--- LivelinessSubscriber
/// Liveliness Subscriber
#[allow(clippy::module_name_repetitions)]
pub struct LivelinessSubscriber<P>
where
	P: Send + Sync + 'static,
{
	/// the zenoh session this liveliness subscriber belongs to
	session: Arc<Session>,
	token: String,
	context: Context<P>,
	activation_state: OperationState,
	put_callback: ArcLivelinessCallback<P>,
	delete_callback: Option<ArcLivelinessCallback<P>>,
	handle: std::sync::Mutex<Option<JoinHandle<()>>>,
	known_agents: Arc<Mutex<BTreeSet<String>>>,
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

impl<P> crate::traits::LivelinessSubscriber for LivelinessSubscriber<P>
where
	P: Send + Sync + 'static,
{
	/// get token
	fn token(&self) -> &String {
		&self.token
	}
}

impl<P> Capability for LivelinessSubscriber<P>
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

impl<P> LivelinessSubscriber<P>
where
	P: Send + Sync + 'static,
{
	/// Constructor for a [`LivelinessSubscriber`]
	pub fn new(
		session: Arc<Session>,
		token: String,
		context: Context<P>,
		activation_state: OperationState,
		put_callback: ArcLivelinessCallback<P>,
		delete_callback: Option<ArcLivelinessCallback<P>>,
	) -> Self {
		Self {
			session,
			token,
			context,
			activation_state,
			put_callback,
			delete_callback,
			handle: std::sync::Mutex::new(None),
			#[cfg(feature = "std")]
			known_agents: Arc::new(Mutex::new(BTreeSet::new())),
		}
	}

	/// Start or restart the liveliness subscriber.
	/// An already running subscriber will be stopped before.
	#[instrument(level = Level::TRACE, skip_all)]
	fn start(&self) -> Result<()> {
		self.stop()?;

		// liveliness handling
		let key = self.token.clone();
		let known_agents = self.known_agents.clone();
		let session2 = self.session.clone();
		let token2 = self.token.clone();
		let p_cb2 = self.put_callback.clone();
		let d_cb = self.delete_callback.clone();
		let ctx = self.context.clone();
		let ctx2 = self.context.clone();

		self.handle.lock().map_or_else(
			|_| todo!(),
			|mut handle| {
				handle.replace(tokio::task::spawn(async move {
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

					// the liveliness subscriber with history
					if let Err(error) =
						run_liveliness(session2, token2, p_cb2, d_cb, ctx2, known_agents).await
					{
						error!("running liveliness subscriber failed with {error}");
					};
				}));
				Ok(())
			},
		)
	}

	/// Stop a running LivelinessSubscriber
	#[instrument(level = Level::TRACE)]
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

#[instrument(name="liveliness", level = Level::ERROR, skip_all)]
async fn run_liveliness<P>(
	session: Arc<Session>,
	token: String,
	p_cb: ArcLivelinessCallback<P>,
	d_cb: Option<ArcLivelinessCallback<P>>,
	ctx: Context<P>,
	known_agents: Arc<Mutex<BTreeSet<String>>>,
) -> Result<()> {
	let subscriber = session
		.liveliness()
		.declare_subscriber(&token)
		.history(true)
		.await?;

	while let Ok(sample) = subscriber.recv_async().await {
		let id = sample.key_expr().split('/').last().unwrap_or("");
		// skip own live message
		if id == ctx.uuid() {
			continue;
		};
		// lock known_agents to avoid concurrent access during handling.
		// drop lock directly after last access
		let mut guard = known_agents.lock().await;
		match sample.kind() {
			SampleKind::Put => {
				if guard.get(id).is_none() {
					guard.insert(id.into());
					drop(guard);
					let ctx = ctx.clone();
					let mut lock = p_cb.lock().await;
					if let Err(error) = lock(ctx, id.to_string()).await {
						error!("liveliness put callback failed with {error}");
					}
				}
			}
			SampleKind::Delete => {
				if guard.get(id).is_some() {
					guard.remove(id);
					drop(guard);
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
