// Copyright © 2023 Stephan Kunz

//! Module `subscriber` provides a message `Subscriber` which can be created using the `SubscriberBuilder`.
//! A `Subscriber` can optional subscribe on a delete message.

// region:		--- modules
// these ones are only for doc needed
use super::task_signal::TaskSignal;
#[cfg(doc)]
use crate::agent::Agent;
use crate::context::ArcContext;
use dimas_com::Message;
use dimas_core::error::{DimasError, Result};
use std::sync::{mpsc::Sender, Arc, Mutex, RwLock};
use tokio::task::JoinHandle;
use tracing::{error, info, instrument, warn, Level};
use zenoh::{
	prelude::{r#async::AsyncResolve, SampleKind, SessionDeclarations},
	subscriber::Reliability,
};
// endregion:	--- modules

// region:		--- types
/// Type definition for a subscribers `put` callback function
#[allow(clippy::module_name_repetitions)]
pub type SubscriberPutCallback<P> = Arc<
	Mutex<Box<dyn FnMut(&ArcContext<P>, Message) -> Result<()> + Send + Sync + Unpin + 'static>>,
>;
/// Type definition for a subscribers `delete` callback function
#[allow(clippy::module_name_repetitions)]
pub type SubscriberDeleteCallback<P> =
	Arc<Mutex<Box<dyn FnMut(&ArcContext<P>) -> Result<()> + Send + Sync + Unpin + 'static>>>;
// endregion:	--- types

// region:		--- states
/// State signaling that the [`SubscriberBuilder`] has no storage value set
pub struct NoStorage;
/// State signaling that the [`SubscriberBuilder`] has the storage value set
pub struct Storage<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Thread safe reference to a [`HashMap`] to store the created [`Subscriber`]
	pub storage: Arc<RwLock<std::collections::HashMap<String, Subscriber<P>>>>,
}

/// State signaling that the [`SubscriberBuilder`] has no key expression value set
pub struct NoKeyExpression;
/// State signaling that the [`SubscriberBuilder`] has the key expression value set
pub struct KeyExpression {
	/// The key expression
	key_expr: String,
}

/// State signaling that the [`SubscriberBuilder`] has no put callback value set
pub struct NoPutCallback;
/// State signaling that the [`SubscriberBuilder`] has the put callback value set
pub struct PutCallback<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Put callback for the [`Subscriber`]
	pub callback: SubscriberPutCallback<P>,
}
// endregion:	--- states

// region:		--- SubscriberBuilder
/// A builder for a subscriber
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct SubscriberBuilder<P, K, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	prefix: Option<String>,
	pub(crate) key_expr: K,
	pub(crate) put_callback: C,
	pub(crate) storage: S,
	pub(crate) reliability: Reliability,
	pub(crate) delete_callback: Option<SubscriberDeleteCallback<P>>,
}

impl<P> SubscriberBuilder<P, NoKeyExpression, NoPutCallback, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a `SubscriberBuilder` in initial state
	#[must_use]
	pub const fn new(prefix: Option<String>) -> Self {
		Self {
			prefix,
			key_expr: NoKeyExpression,
			put_callback: NoPutCallback,
			storage: NoStorage,
			reliability: Reliability::BestEffort,
			delete_callback: None,
		}
	}
}

impl<P, K, C, S> SubscriberBuilder<P, K, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set reliability
	#[must_use]
	pub const fn set_reliability(mut self, reliability: Reliability) -> Self {
		self.reliability = reliability;
		self
	}

	/// Set liveliness subscribers callback for `delete` messages
	#[must_use]
	pub fn delete_callback<F>(self, callback: F) -> Self
	where
		F: FnMut(&ArcContext<P>) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			prefix,
			key_expr,
			put_callback,
			storage,
			reliability,
			..
		} = self;
		let delete_callback: Option<SubscriberDeleteCallback<P>> =
			Some(Arc::new(Mutex::new(Box::new(callback))));
		Self {
			prefix,
			key_expr,
			put_callback,
			storage,
			reliability,
			delete_callback,
		}
	}
}

impl<P, C, S> SubscriberBuilder<P, NoKeyExpression, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full key expression for the [`Subscriber`].
	#[must_use]
	pub fn key_expr(self, key_expr: &str) -> SubscriberBuilder<P, KeyExpression, C, S> {
		let Self {
			prefix,
			storage,
			put_callback,
			delete_callback,
			reliability,
			..
		} = self;
		SubscriberBuilder {
			prefix,
			key_expr: KeyExpression {
				key_expr: key_expr.into(),
			},
			put_callback,
			storage,
			reliability,
			delete_callback,
		}
	}

	/// Set only the message qualifing part of the [`Subscriber`].
	/// Will be prefixed with [`Agent`]s prefix.
	#[must_use]
	pub fn topic(mut self, topic: &str) -> SubscriberBuilder<P, KeyExpression, C, S> {
		let key_expr = self
			.prefix
			.take()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		let Self {
			prefix,
			storage,
			put_callback,
			reliability,
			delete_callback,
			..
		} = self;
		SubscriberBuilder {
			prefix,
			key_expr: KeyExpression { key_expr },
			put_callback,
			storage,
			reliability,
			delete_callback,
		}
	}
}

impl<P, K, S> SubscriberBuilder<P, K, NoPutCallback, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set callback for put messages
	#[must_use]
	pub fn put_callback<F>(self, callback: F) -> SubscriberBuilder<P, K, PutCallback<P>, S>
	where
		F: FnMut(&ArcContext<P>, Message) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			prefix,
			key_expr,
			storage,
			reliability,
			delete_callback,
			..
		} = self;
		let callback: SubscriberPutCallback<P> = Arc::new(Mutex::new(Box::new(callback)));
		SubscriberBuilder {
			prefix,
			key_expr,
			put_callback: PutCallback { callback },
			storage,
			reliability,
			delete_callback,
		}
	}
}

impl<P, K, C> SubscriberBuilder<P, K, C, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Provide agents storage for the subscriber
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Subscriber<P>>>>,
	) -> SubscriberBuilder<P, K, C, Storage<P>> {
		let Self {
			prefix,
			key_expr,
			put_callback,
			reliability,
			delete_callback,
			..
		} = self;
		SubscriberBuilder {
			prefix,
			key_expr,
			put_callback,
			storage: Storage { storage },
			reliability,
			delete_callback,
		}
	}
}

impl<P, S> SubscriberBuilder<P, KeyExpression, PutCallback<P>, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the [`Subscriber`].
	///
	/// # Errors
	/// Currently none
	pub fn build(self) -> Result<Subscriber<P>> {
		let Self {
			key_expr,
			put_callback,
			reliability,
			delete_callback,
			..
		} = self;
		Ok(Subscriber::new(
			key_expr.key_expr,
			put_callback.callback,
			reliability,
			delete_callback,
		))
	}
}

impl<P> SubscriberBuilder<P, KeyExpression, PutCallback<P>, Storage<P>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the [`Subscriber`] to the [`Agent`].
	///
	/// # Errors
	/// Currently none
	pub fn add(self) -> Result<Option<Subscriber<P>>> {
		let c = self.storage.storage.clone();
		let s = self.build()?;

		let r = c
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(s.key_expr.clone(), s);
		Ok(r)
	}
}
// endregion:	--- SubscriberBuilder

// region:		--- Subscriber
/// Subscriber
pub struct Subscriber<P>
where
	P: Send + Sync + Unpin + 'static,
{
	key_expr: String,
	put_callback: SubscriberPutCallback<P>,
	reliability: Reliability,
	delete_callback: Option<SubscriberDeleteCallback<P>>,
	handle: Option<JoinHandle<()>>,
}

impl<P> std::fmt::Debug for Subscriber<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Subscriber")
			.field("key_expr", &self.key_expr)
			.finish_non_exhaustive()
	}
}

impl<P> Subscriber<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor for a [`Subscriber`].
	#[must_use]
	pub fn new(
		key_expr: String,
		put_callback: SubscriberPutCallback<P>,
		reliability: Reliability,
		delete_callback: Option<SubscriberDeleteCallback<P>>,
	) -> Self {
		Self {
			key_expr,
			put_callback,
			reliability,
			delete_callback,
			handle: None,
		}
	}

	/// Start or restart the subscriber.
	/// An already running subscriber will be stopped, eventually damaged Mutexes will be repaired
	#[instrument(level = Level::TRACE, skip_all)]
	pub fn start(&mut self, ctx: ArcContext<P>, tx: Sender<TaskSignal>) {
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

		let key_expr = self.key_expr.clone();
		let p_cb = self.put_callback.clone();
		let d_cb = self.delete_callback.clone();
		let reliability = self.reliability;

		self.handle
			.replace(tokio::task::spawn(async move {
				let key = key_expr.clone();
				std::panic::set_hook(Box::new(move |reason| {
					error!("subscriber panic: {}", reason);
					if let Err(reason) = tx.send(TaskSignal::RestartSubscriber(key.clone())) {
						error!("could not restart subscriber: {}", reason);
					} else {
						info!("restarting subscriber!");
					};
				}));
				if let Err(error) = run_subscriber(key_expr, p_cb, d_cb, reliability, ctx).await {
					error!("spawning subscriber failed with {error}");
				};
			}));
	}

	/// Stop a running Subscriber
	#[instrument(level = Level::TRACE, skip_all)]
	pub fn stop(&mut self) {
		if let Some(handle) = self.handle.take() {
			handle.abort();
		}
	}
}

#[instrument(name="subscriber", level = Level::ERROR, skip_all)]
async fn run_subscriber<P>(
	key_expr: String,
	p_cb: SubscriberPutCallback<P>,
	d_cb: Option<SubscriberDeleteCallback<P>>,
	reliability: Reliability,
	ctx: ArcContext<P>,
) -> Result<()>
where
	P: Send + Sync + Unpin + 'static,
{
	let subscriber = ctx
		.communicator
		.session()
		.declare_subscriber(&key_expr)
		.reliability(reliability)
		.res_async()
		.await
		.map_err(|_| DimasError::ShouldNotHappen)?;

	loop {
		let sample = subscriber
			.recv_async()
			.await
			.map_err(|_| DimasError::ShouldNotHappen)?;

		match sample.kind {
			SampleKind::Put => {
				let msg = Message(sample);
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
		is_normal::<SubscriberBuilder<Props, NoKeyExpression, NoPutCallback, NoStorage>>();
	}
}
