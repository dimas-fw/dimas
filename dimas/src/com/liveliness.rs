// Copyright © 2023 Stephan Kunz

//! Module `liveliness` provides a `LivelinessSubscriber` which can be created using the `LivelinessSubscriberBuilder`.
//! A `LivelinessSubscriber` can optional subscribe on a delete message.

// region:		--- modules
use dimas_core::{
	error::{DimasError, Result},
	task_signal::TaskSignal,
	traits::{Capability, CommunicationCapability, Context, OperationState},
};
#[cfg(doc)]
use std::collections::HashMap;
use std::{
	sync::{Arc, Mutex, RwLock},
	time::Duration,
};
use tokio::task::JoinHandle;
use tracing::info;
use tracing::{error, instrument, warn, Level};
use zenoh::prelude::{r#async::AsyncResolve, SampleKind, SessionDeclarations};
// endregion:	--- modules

// region:		--- types
/// Type definition for liveliness callback function
#[allow(clippy::module_name_repetitions)]
pub type LivelinessCallback<P> =
	Box<dyn FnMut(&Context<P>, &str) -> Result<()> + Send + Sync + Unpin + 'static>;
type ArcCallback<P> =
	Arc<Mutex<dyn FnMut(&Context<P>, &str) -> Result<()> + Send + Sync + Unpin + 'static>>;
// endregion:	--- types

// region:		--- states
/// State signaling that the [`LivelinessSubscriberBuilder`] has no storage value set
pub struct NoStorage;
/// State signaling that the [`LivelinessSubscriberBuilder`] has the storage value set
pub struct Storage<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Thread safe reference to a [`HashMap`] to store the created [`LivelinessSubscriber`]
	pub storage: Arc<RwLock<std::collections::HashMap<String, LivelinessSubscriber<P>>>>,
}

/// State signaling that the [`LivelinessSubscriberBuilder`] has no put callback set
pub struct NoPutCallback;
/// State signaling that the [`LivelinessSubscriberBuilder`] has the put callback set
pub struct PutCallback<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// The callback to use when receiving a put message
	pub callback: ArcCallback<P>,
}
// endregion:	--- states

// region:		--- LivelinessSubscriberBuilder
/// The builder for the liveliness subscriber
#[allow(clippy::module_name_repetitions)]
pub struct LivelinessSubscriberBuilder<P, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	token: String,
	context: Context<P>,
	activation_state: OperationState,
	put_callback: C,
	storage: S,
	delete_callback: Option<ArcCallback<P>>,
}

impl<P> LivelinessSubscriberBuilder<P, NoPutCallback, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a `LivelinessSubscriberBuilder` in initial state
	#[must_use]
	pub fn new(context: Context<P>) -> Self {
		let token = context
			.prefix()
			.clone()
			.map_or("*".to_string(), |prefix| format!("{prefix}/*"));
		Self {
			token,
			context,
			activation_state: OperationState::Configured,
			put_callback: NoPutCallback,
			storage: NoStorage,
			delete_callback: None,
		}
	}
}

impl<P, C, S> LivelinessSubscriberBuilder<P, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the activation state.
	#[must_use]
	pub const fn activation_state(mut self, state: OperationState) -> Self {
		self.activation_state = state;
		self
	}

	/// Set a different prefix for the liveliness subscriber.
	#[must_use]
	pub fn prefix(self, prefix: &str) -> Self {
		let token = format!("{prefix}/*");
		let Self {
			context,
			activation_state,
			put_callback,
			storage,
			delete_callback,
			..
		} = self;
		Self {
			token,
			context,
			activation_state,
			put_callback,
			storage,
			delete_callback,
		}
	}

	/// Set an explicite token for the liveliness subscriber.
	#[must_use]
	pub fn token(self, token: impl Into<String>) -> Self {
		let Self {
			context,
			activation_state,
			put_callback,
			storage,
			delete_callback,
			..
		} = self;
		Self {
			token: token.into(),
			context,
			activation_state,
			put_callback,
			storage,
			delete_callback,
		}
	}

	/// Set liveliness subscribers callback for `delete` messages
	#[must_use]
	pub fn delete_callback(self, callback: LivelinessCallback<P>) -> Self {
		let Self {
			token,
			context,
			activation_state,
			put_callback,
			storage,
			..
		} = self;
		let delete_callback: Option<ArcCallback<P>> =
			Some(Arc::new(Mutex::new(Box::new(callback))));
		Self {
			token,
			context,
			activation_state,
			put_callback,
			storage,
			delete_callback,
		}
	}
}

impl<P, S> LivelinessSubscriberBuilder<P, NoPutCallback, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set liveliness subscribers callback for `put` messages
	#[must_use]
	pub fn put_callback(
		self,
		callback: LivelinessCallback<P>,
	) -> LivelinessSubscriberBuilder<P, PutCallback<P>, S> {
		let Self {
			token,
			context,
			activation_state,
			storage,
			delete_callback,
			..
		} = self;
		let put_callback: ArcCallback<P> = Arc::new(Mutex::new(Box::new(callback)));
		LivelinessSubscriberBuilder {
			token,
			context,
			activation_state,
			put_callback: PutCallback {
				callback: put_callback,
			},
			storage,
			delete_callback,
		}
	}
}

impl<P, C> LivelinessSubscriberBuilder<P, C, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Provide agents storage for the liveliness subscriber
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, LivelinessSubscriber<P>>>>,
	) -> LivelinessSubscriberBuilder<P, C, Storage<P>> {
		let Self {
			token,
			context,
			activation_state,
			put_callback,
			delete_callback,
			..
		} = self;
		LivelinessSubscriberBuilder {
			token,
			context,
			activation_state,
			put_callback,
			storage: Storage { storage },
			delete_callback,
		}
	}
}

impl<P, S> LivelinessSubscriberBuilder<P, PutCallback<P>, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the [`LivelinessSubscriber`]
	/// # Errors
	///
	pub fn build(self) -> Result<LivelinessSubscriber<P>> {
		let Self {
			token,
			context,
			activation_state,
			put_callback,
			delete_callback,
			..
		} = self;
		Ok(LivelinessSubscriber::new(
			token,
			context,
			activation_state,
			put_callback.callback,
			delete_callback,
		))
	}
}

impl<P> LivelinessSubscriberBuilder<P, PutCallback<P>, Storage<P>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the liveliness subscriber to the agent
	/// # Errors
	///
	pub fn add(self) -> Result<Option<LivelinessSubscriber<P>>> {
		let c = self.storage.storage.clone();
		let s = self.build()?;

		let r = c
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(s.token.clone(), s);
		Ok(r)
	}
}
// endregion:	--- LivelinessSubscriberBuilder

// region:		--- LivelinessSubscriber
/// Liveliness Subscriber
#[allow(clippy::module_name_repetitions)]
pub struct LivelinessSubscriber<P>
where
	P: Send + Sync + Unpin + 'static,
{
	token: String,
	context: Context<P>,
	activation_state: OperationState,
	put_callback: ArcCallback<P>,
	delete_callback: Option<ArcCallback<P>>,
	handle: Option<JoinHandle<()>>,
}

impl<P> std::fmt::Debug for LivelinessSubscriber<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("LivelinessSubscriber")
			.finish_non_exhaustive()
	}
}

impl<P> Capability for LivelinessSubscriber<P>
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

impl<P> CommunicationCapability for LivelinessSubscriber<P> where P: Send + Sync + Unpin + 'static {}

impl<P> LivelinessSubscriber<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor for a [`LivelinessSubscriber`]
	pub fn new(
		token: String,
		context: Context<P>,
		activation_state: OperationState,
		put_callback: ArcCallback<P>,
		delete_callback: Option<ArcCallback<P>>,
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

	/// Start or restart the liveliness subscriber.
	/// An already running subscriber will be stopped before,
	/// eventually damaged Mutexes will be repaired
	#[instrument(level = Level::TRACE, skip_all)]
	fn start(&mut self) -> Result<()> {
		self.stop();

		{
			if self.put_callback.lock().is_err() {
				warn!("found poisoned put Mutex");
				self.put_callback.clear_poison();
			}
		}
		{
			if let Some(dcb) = self.delete_callback.clone() {
				if dcb.lock().is_err() {
					warn!("found poisoned delete Mutex");
					dcb.clear_poison();
				}
			}
		}

		// the initial liveliness query
		let p_cb = self.put_callback.clone();
		let ctx = self.context.clone();
		let token = self.token.clone();
		tokio::task::spawn(async move {
			if let Err(error) = run_initial(token, p_cb, ctx).await {
				error!("spawning initial liveliness failed with {error}");
			};
		});

		// the liveliness subscriber
		let token = self.token.clone();
		let p_cb = self.put_callback.clone();
		let d_cb = self.delete_callback.clone();
		let ctx1 = self.context.clone();
		let ctx2 = self.context.clone();

		self.handle
			.replace(tokio::task::spawn(async move {
				let key = token.clone();
				std::panic::set_hook(Box::new(move |reason| {
					error!("liveliness subscriber panic: {}", reason);
					if let Err(reason) = ctx1
						.sender()
						.send(TaskSignal::RestartLiveliness(key.clone()))
					{
						error!("could not restart liveliness subscriber: {}", reason);
					} else {
						info!("restarting liveliness subscriber!");
					};
				}));
				if let Err(error) = run_liveliness(token, p_cb, d_cb, ctx2).await {
					error!("spawning liveliness subscriber failed with {error}");
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
	p_cb: ArcCallback<P>,
	d_cb: Option<ArcCallback<P>>,
	ctx: Context<P>,
) -> Result<()> {
	let subscriber = ctx
		.session()
		.liveliness()
		.declare_subscriber(&token)
		.res_async()
		.await
		.map_err(|_| DimasError::ShouldNotHappen)?;

	loop {
		let result = subscriber.recv_async().await;
		match result {
			Ok(sample) => {
				let id = sample.key_expr.split('/').last().unwrap_or("");
				// skip own live message
				if id == ctx.uuid() {
					continue;
				};
				match sample.kind {
					SampleKind::Put => match p_cb.lock() {
						Ok(mut lock) => {
							if let Err(error) = lock(&ctx, id) {
								error!("liveliness put callback failed with {error}");
							}
						}
						Err(err) => {
							error!("liveliness put callback lock failed with {err}");
						}
					},
					SampleKind::Delete => {
						if let Some(cb) = d_cb.clone() {
							match cb.lock() {
								Ok(mut lock) => {
									if let Err(err) = lock(&ctx, id) {
										error!("liveliness delete callback failed with {err}");
									}
								}
								Err(err) => {
									error!("liveliness delete callback lock failed with {err}");
								}
							}
						}
					}
				}
			}
			Err(error) => {
				error!("receive failed with {error}");
			}
		}
	}
}

#[instrument(name="initial liveliness", level = Level::ERROR, skip_all)]
async fn run_initial<P>(token: String, p_cb: ArcCallback<P>, ctx: Context<P>) -> Result<()> {
	let result = ctx
		.session()
		.liveliness()
		.get(&token)
		.timeout(Duration::from_millis(100))
		.res()
		.await;

	match result {
		Ok(replies) => {
			while let Ok(reply) = replies.recv_async().await {
				match reply.sample {
					Ok(sample) => {
						let id = sample.key_expr.split('/').last().unwrap_or("");
						// skip own live message
						if id == ctx.uuid() {
							continue;
						};
						match p_cb.lock() {
							Ok(mut lock) => {
								if let Err(error) = lock(&ctx, id) {
									error!("lveliness put callback failed with {error}");
								}
							}
							Err(err) => {
								error!("liveliness put callback failed with {err}");
							}
						}
					}
					Err(err) => error!(">> liveliness subscriber delete error: {err})"),
				}
			}
		}
		Err(error) => {
			error!("livelieness subscriber failed with {error}");
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
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<LivelinessSubscriber<Props>>();
		is_normal::<LivelinessSubscriberBuilder<Props, NoPutCallback, NoStorage>>();
	}
}
