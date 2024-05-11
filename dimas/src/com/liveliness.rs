// Copyright © 2023 Stephan Kunz

//! Module `liveliness` provides a `LivelinessSubscriber` which can be created using the `LivelinessSubscriberBuilder`.
//! A `LivelinessSubscriber` can optional subscribe on a delete message.

// region:		--- modules
use dimas_core::error::{DimasError, Result};
#[cfg(doc)]
use std::collections::HashMap;
use std::{
	sync::{mpsc::Sender, Arc, Mutex, RwLock},
	time::Duration,
};
use tokio::task::JoinHandle;
use tracing::info;
use tracing::{error, instrument, warn, Level};
use zenoh::prelude::{r#async::AsyncResolve, SampleKind, SessionDeclarations};

use crate::prelude::ArcContext;

use super::task_signal::TaskSignal;
// endregion:	--- modules

// region:		--- types
/// Type definition for liveliness callback function
#[allow(clippy::module_name_repetitions)]
pub type LivelinessCallback<P> =
	Arc<Mutex<Box<dyn FnMut(&ArcContext<P>, &str) -> Result<()> + Send + Sync + Unpin + 'static>>>;
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
	pub callback: LivelinessCallback<P>,
}
// endregion:	--- states

// region:		--- LivelinessSubscriberBuilder
/// The builder for the liveliness subscriber
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct LivelinessSubscriberBuilder<P, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	token: String,
	pub(crate) put_callback: C,
	pub(crate) storage: S,
	pub(crate) delete_callback: Option<LivelinessCallback<P>>,
}

impl<P> LivelinessSubscriberBuilder<P, NoPutCallback, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a `LivelinessSubscriberBuilder` in initial state
	#[must_use]
	pub fn new(prefix: Option<String>) -> Self {
		let token = prefix.map_or("*".to_string(), |prefix| format!("{prefix}/*"));
		Self {
			token,
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
	/// Set a different prefix for the liveliness subscriber.
	#[must_use]
	pub fn prefix(self, prefix: &str) -> Self {
		let key_expr = format!("{prefix}/*");
		let Self {
			put_callback,
			storage,
			delete_callback,
			..
		} = self;
		Self {
			token: key_expr,
			put_callback,
			storage,
			delete_callback,
		}
	}

	/// Set an explicite token for the liveliness subscriber.
	#[must_use]
	pub fn token(self, token: impl Into<String>) -> Self {
		let Self {
			put_callback,
			storage,
			delete_callback,
			..
		} = self;
		Self {
			token: token.into(),
			put_callback,
			storage,
			delete_callback,
		}
	}

	/// Set liveliness subscribers callback for `delete` messages
	#[must_use]
	pub fn delete_callback<F>(self, callback: F) -> Self
	where
		F: FnMut(&ArcContext<P>, &str) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			token: key_expr,
			put_callback,
			storage,
			..
		} = self;
		let delete_callback: Option<LivelinessCallback<P>> =
			Some(Arc::new(Mutex::new(Box::new(callback))));
		Self {
			token: key_expr,
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
	pub fn put_callback<F>(self, callback: F) -> LivelinessSubscriberBuilder<P, PutCallback<P>, S>
	where
		F: FnMut(&ArcContext<P>, &str) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			token: key_expr,
			storage,
			delete_callback,
			..
		} = self;
		let put_callback: LivelinessCallback<P> = Arc::new(Mutex::new(Box::new(callback)));
		LivelinessSubscriberBuilder {
			token: key_expr,
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
			token: key_expr,
			put_callback,
			delete_callback,
			..
		} = self;
		LivelinessSubscriberBuilder {
			token: key_expr,
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
			token: key_expr,
			put_callback,
			delete_callback,
			..
		} = self;
		Ok(LivelinessSubscriber::new(
			key_expr,
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
			.insert(s.key_expr.clone(), s);
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
	key_expr: String,
	put_callback: LivelinessCallback<P>,
	delete_callback: Option<LivelinessCallback<P>>,
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

impl<P> LivelinessSubscriber<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor for a [`LivelinessSubscriber`]
	pub fn new(
		key_expr: String,
		put_callback: LivelinessCallback<P>,
		delete_callback: Option<LivelinessCallback<P>>,
	) -> Self {
		Self {
			key_expr,
			put_callback,
			delete_callback,
			handle: None,
		}
	}

	/// Start or restart the liveliness subscriber.
	/// An already running subscriber will be stopped, eventually damaged Mutexes will be repaired
	#[instrument(level = Level::TRACE, skip_all)]
	pub fn start(&mut self, context: ArcContext<P>, tx: Sender<TaskSignal>) {
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
		let ctx = context.clone();
		let key_expr = self.key_expr.clone();
		tokio::task::spawn(async move {
			if let Err(error) = run_initial(key_expr, p_cb, ctx).await {
				error!("spawning initial liveliness failed with {error}");
			};
		});

		// the liveliness subscriber
		let p_cb = self.put_callback.clone();
		let d_cb = self.delete_callback.clone();
		let ctx = context;
		let key_expr = self.key_expr.clone();

		self.handle
			.replace(tokio::task::spawn(async move {
				let key = key_expr.clone();
				std::panic::set_hook(Box::new(move |reason| {
					error!("liveliness subscriber panic: {}", reason);
					if let Err(reason) = tx.send(TaskSignal::RestartLiveliness(key.clone())) {
						error!("could not restart liveliness subscriber: {}", reason);
					} else {
						info!("restarting liveliness subscriber!");
					};
				}));
				if let Err(error) = run_liveliness(key_expr, p_cb, d_cb, ctx).await {
					error!("spawning liveliness subscriber failed with {error}");
				};
			}));
	}

	/// Stop a running LivelinessSubscriber
	#[instrument(level = Level::TRACE)]
	pub fn stop(&mut self) {
		if let Some(handle) = self.handle.take() {
			handle.abort();
		}
	}
}

#[instrument(name="liveliness", level = Level::ERROR, skip_all)]
async fn run_liveliness<P>(
	key_expr: String,
	p_cb: LivelinessCallback<P>,
	d_cb: Option<LivelinessCallback<P>>,
	ctx: ArcContext<P>,
) -> Result<()>
where
	P: Send + Sync + Unpin + 'static,
{
	let subscriber = ctx
		.communicator
		.session()
		.liveliness()
		.declare_subscriber(&key_expr)
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
async fn run_initial<P>(
	key_expr: String,
	p_cb: LivelinessCallback<P>,
	ctx: ArcContext<P>,
) -> Result<()>
where
	P: Send + Sync + Unpin + 'static,
{
	let result = ctx
		.communicator
		.session()
		.liveliness()
		.get(&key_expr)
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
