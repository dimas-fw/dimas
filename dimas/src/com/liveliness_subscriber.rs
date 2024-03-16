// Copyright © 2023 Stephan Kunz

//! Module `liveliness_subscriber` provides a `LivelinessSubscriber` which can be created using the `LivelinessSubscriberBuilder`.
//! A `LivelinessSubscriber` can optional subscribe on a delete message.

// region:		--- modules
use crate::{agent::TaskSignal, prelude::*};
use std::{
	sync::{mpsc::Sender, Mutex},
	time::Duration,
};
use tokio::task::JoinHandle;
#[cfg(feature = "liveliness")]
use tracing::info;
use tracing::{error, instrument, warn, Level};
use zenoh::{
	prelude::{r#async::AsyncResolve, SampleKind},
	SessionDeclarations,
};
// endregion:	--- modules

// region:		--- types
/// Type definition for liveliness callback function
pub type LivelinessCallback<P> = Arc<
	Mutex<
		Option<Box<dyn FnMut(&ArcContext<P>, &str) -> Result<()> + Send + Sync + Unpin + 'static>>,
	>,
>;
// endregion:	--- types

// region:		--- states
pub struct NoStorage;
#[cfg(feature = "liveliness")]
pub struct Storage<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub storage: Arc<RwLock<std::collections::HashMap<String, LivelinessSubscriber<P>>>>,
}

pub struct NoPutCallback;
pub struct PutCallback<P>
where
	P: Send + Sync + Unpin + 'static,
{
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
	pub(crate) context: ArcContext<P>,
	key_expr: String,
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
	pub fn new(context: ArcContext<P>) -> Self {
		let key_expr = context.key_expr("alive/*");
		Self {
			context,
			key_expr,
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
	/// Set liveliness subscribers callback for `delete` messages
	#[must_use]
	pub fn delete_callback<F>(self, callback: F) -> Self
	where
		F: FnMut(&ArcContext<P>, &str) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			context,
			key_expr,
			put_callback,
			storage,
			..
		} = self;
		let delete_callback: Option<LivelinessCallback<P>> =
			Some(Arc::new(Mutex::new(Some(Box::new(callback)))));
		Self {
			context,
			key_expr,
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
			context,
			key_expr,
			storage,
			delete_callback,
			..
		} = self;
		let put_callback: LivelinessCallback<P> = Arc::new(Mutex::new(Some(Box::new(callback))));
		LivelinessSubscriberBuilder {
			context,
			key_expr,
			put_callback: PutCallback {
				callback: put_callback,
			},
			storage,
			delete_callback,
		}
	}
}

#[cfg(feature = "liveliness")]
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
			context,
			key_expr,
			put_callback,
			delete_callback,
			..
		} = self;
		LivelinessSubscriberBuilder {
			context,
			key_expr,
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
	/// Build the liveliness subscriber
	/// # Errors
	///
	pub fn build(self) -> Result<LivelinessSubscriber<P>> {
		let Self {
			context,
			key_expr,
			put_callback,
			delete_callback,
			..
		} = self;
		Ok(LivelinessSubscriber {
			context,
			key_expr,
			put_callback: Some(put_callback.callback),
			delete_callback,
			handle: None,
		})
	}
}

#[cfg(feature = "liveliness")]
impl<P> LivelinessSubscriberBuilder<P, PutCallback<P>, Storage<P>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the liveliness subscriber to the agent
	/// # Errors
	///
	#[cfg_attr(any(nightly, docrs), doc, doc(cfg(feature = "liveliness")))]
	pub fn add(self) -> Result<()> {
		let c = self.storage.storage.clone();
		let s = self.build()?;

		c.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(s.key_expr.clone(), s);
		Ok(())
	}
}
// endregion:	--- LivelinessSubscriberBuilder

// region:		--- LivelinessSubscriber
/// Liveliness Subscriber
pub struct LivelinessSubscriber<P>
where
	P: Send + Sync + Unpin + 'static,
{
	key_expr: String,
	put_callback: Option<LivelinessCallback<P>>,
	delete_callback: Option<LivelinessCallback<P>>,
	handle: Option<JoinHandle<()>>,
	context: ArcContext<P>,
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
	/// Start or restart the liveliness subscriber.
	/// An already running subscriber will be stopped, eventually damaged Mutexes will be repaired
	#[instrument(level = Level::TRACE, skip_all)]
	pub fn start(&mut self, tx: Sender<TaskSignal>) {
		self.stop();

		#[cfg(not(feature = "liveliness"))]
		drop(tx);

		{
			if let Some(pcb) = self.put_callback.clone() {
				if let Err(err) = pcb.lock() {
					warn!("found poisoned put Mutex");
					self.put_callback
						.replace(Arc::new(Mutex::new(err.into_inner().take())));
				}
			}
		}
		{
			if let Some(dcb) = self.delete_callback.clone() {
				if let Err(err) = dcb.lock() {
					warn!("found poisoned delete Mutex");
					self.delete_callback
						.replace(Arc::new(Mutex::new(err.into_inner().take())));
				}
			}
		}

		// the initial liveliness query
		let p_cb = self.put_callback.clone();
		let ctx = self.context.clone();
		let key_expr = self.key_expr.clone();
		tokio::spawn(async move {
			if let Err(error) = run_initial(key_expr, p_cb, ctx).await {
				error!("spawning initial liveliness failed with {error}");
			};
		});

		// the liveliness subscriber
		let p_cb = self.put_callback.clone();
		let d_cb = self.delete_callback.clone();
		let ctx = self.context.clone();
		let key_expr = self.key_expr.clone();

		self.handle.replace(tokio::spawn(async move {
			#[cfg(feature = "liveliness")]
			let key = key_expr.clone();
			std::panic::set_hook(Box::new(move |reason| {
				error!("liveliness subscriber panic: {}", reason);
				#[cfg(feature = "liveliness")]
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
	p_cb: Option<LivelinessCallback<P>>,
	d_cb: Option<LivelinessCallback<P>>,
	ctx: ArcContext<P>,
) -> Result<()>
where
	P: Send + Sync + Unpin + 'static,
{
	let subscriber = ctx
		.communicator
		.session
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
					SampleKind::Put => {
						if let Some(cb) = p_cb.clone() {
							let result = cb.lock();
							match result {
								Ok(mut cb) => {
									if let Err(error) = cb.as_deref_mut().expect("snh")(&ctx, id) {
										error!("put callback failed with {error}");
									}
								}
								Err(err) => {
									error!("put callback lock failed with {err}");
								}
							}
						}
					}
					SampleKind::Delete => {
						if let Some(cb) = d_cb.clone() {
							let result = cb.lock();
							match result {
								Ok(mut cb) => {
									if let Err(error) = cb.as_deref_mut().expect("snh")(&ctx, id) {
										error!("delete callback failed with {error}");
									}
								}
								Err(err) => {
									error!("delete callback lock failed with {err}");
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
	p_cb: Option<LivelinessCallback<P>>,
	ctx: ArcContext<P>,
) -> Result<()>
where
	P: Send + Sync + Unpin + 'static,
{
	let result = ctx
		.communicator
		.session
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
						if let Some(cb) = p_cb.clone() {
							let result = cb.lock();
							match result {
								Ok(mut cb) => {
									if let Err(error) = cb.as_deref_mut().expect("snh")(&ctx, id) {
										error!("callback failed with {error}");
									}
								}
								Err(err) => {
									error!("callback lock failed with {err}");
								}
							}
						}
					}
					Err(err) => error!("receive error: {err})"),
				}
			}
		}
		Err(error) => {
			error!("failed with {error}");
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
