// Copyright © 2023 Stephan Kunz

//! Module `subscriber` provides a message `Subscriber` which can be created using the `SubscriberBuilder`.
//! A `Subscriber` can optional subscribe on a delete message.

// region:		--- modules
use crate::{prelude::*, utils::TaskSignal};
use std::sync::{mpsc::Sender, Mutex};
use tokio::task::JoinHandle;
#[cfg(feature = "subscriber")]
use tracing::info;
use tracing::{error, instrument, warn, Level};
use zenoh::{
	prelude::{r#async::AsyncResolve, SampleKind},
	SessionDeclarations,
};
// endregion:	--- modules

// region:		--- types
/// Type definition for a subscribers `publish` callback function
#[allow(clippy::module_name_repetitions)]
pub type SubscriberPutCallback<P> = Arc<
	Mutex<
		Option<
			Box<dyn FnMut(&ArcContext<P>, Message) -> Result<()> + Send + Sync + Unpin + 'static>,
		>,
	>,
>;
/// Type definition for a subscribers `delete` callback function
#[allow(clippy::module_name_repetitions)]
pub type SubscriberDeleteCallback<P> = Arc<
	Mutex<Option<Box<dyn FnMut(&ArcContext<P>) -> Result<()> + Send + Sync + Unpin + 'static>>>,
>;
// endregion:	--- types

// region:		--- states
pub struct NoStorage;
#[cfg(feature = "subscriber")]
pub struct Storage<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub storage: Arc<RwLock<std::collections::HashMap<String, Subscriber<P>>>>,
}

pub struct NoKeyExpression;
pub struct KeyExpression {
	key_expr: String,
}

pub struct NoPutCallback;
pub struct PutCallback<P>
where
	P: Send + Sync + Unpin + 'static,
{
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
			delete_callback: None,
		}
	}
}

impl<P, K, C, S> SubscriberBuilder<P, K, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
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
			..
		} = self;
		let delete_callback: Option<SubscriberDeleteCallback<P>> =
			Some(Arc::new(Mutex::new(Some(Box::new(callback)))));
		Self {
			prefix,
			key_expr,
			put_callback,
			storage,
			delete_callback,
		}
	}
}

impl<P, C, S> SubscriberBuilder<P, NoKeyExpression, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full expression for the subscriber
	#[must_use]
	pub fn key_expr(self, key_expr: &str) -> SubscriberBuilder<P, KeyExpression, C, S> {
		let Self {
			prefix,
			storage,
			put_callback,
			delete_callback,
			..
		} = self;
		SubscriberBuilder {
			prefix,
			key_expr: KeyExpression {
				key_expr: key_expr.into(),
			},
			put_callback,
			storage,
			delete_callback,
		}
	}

	/// Set only the message qualifing part of the subscriber.
	/// Will be prefixed with agents prefix.
	#[must_use]
	pub fn topic(mut self, topic: &str) -> SubscriberBuilder<P, KeyExpression, C, S> {
		let key_expr = self
			.prefix
			.take()
			.unwrap_or_else(|| String::from(topic))
			+ "/" + topic;
		let Self {
			prefix,
			storage,
			put_callback,
			delete_callback,
			..
		} = self;
		SubscriberBuilder {
			prefix,
			key_expr: KeyExpression { key_expr },
			put_callback,
			storage,
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
			delete_callback,
			..
		} = self;
		let callback: SubscriberPutCallback<P> = Arc::new(Mutex::new(Some(Box::new(callback))));
		SubscriberBuilder {
			prefix,
			key_expr,
			put_callback: PutCallback { callback },
			storage,
			delete_callback,
		}
	}
}

#[cfg(feature = "subscriber")]
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
			delete_callback,
			..
		} = self;
		SubscriberBuilder {
			prefix,
			key_expr,
			put_callback,
			storage: Storage { storage },
			delete_callback,
		}
	}
}

impl<P, S> SubscriberBuilder<P, KeyExpression, PutCallback<P>, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the subscriber
	/// # Errors
	///
	pub fn build(self) -> Result<Subscriber<P>> {
		let Self {
			key_expr,
			put_callback,
			delete_callback,
			..
		} = self;
		Ok(Subscriber {
			key_expr: key_expr.key_expr,
			put_callback: Some(put_callback.callback),
			delete_callback,
			handle: None,
		})
	}
}

#[cfg(feature = "subscriber")]
impl<P> SubscriberBuilder<P, KeyExpression, PutCallback<P>, Storage<P>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the subscriber to the agent
	/// # Errors
	///
	#[cfg_attr(any(nightly, docrs), doc, doc(cfg(feature = "subscriber")))]
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
	put_callback: Option<SubscriberPutCallback<P>>,
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
	/// Start or restart the subscriber.
	/// An already running subscriber will be stopped, eventually damaged Mutexes will be repaired
	#[instrument(level = Level::TRACE, skip_all)]
	pub fn start(&mut self, ctx: ArcContext<P>, tx: Sender<TaskSignal>) {
		self.stop();

		#[cfg(not(feature = "subscriber"))]
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

		let key_expr = self.key_expr.clone();
		let p_cb = self.put_callback.clone();
		let d_cb = self.delete_callback.clone();

		self.handle
			.replace(tokio::task::spawn(async move {
				#[cfg(feature = "subscriber")]
				let key = key_expr.clone();
				std::panic::set_hook(Box::new(move |reason| {
					error!("subscriber panic: {}", reason);
					#[cfg(feature = "subscriber")]
					if let Err(reason) = tx.send(TaskSignal::RestartSubscriber(key.clone())) {
						error!("could not restart subscriber: {}", reason);
					} else {
						info!("restarting subscriber!");
					};
				}));
				if let Err(error) = run_subscriber(key_expr, p_cb, d_cb, ctx).await {
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
	p_cb: Option<SubscriberPutCallback<P>>,
	d_cb: Option<SubscriberDeleteCallback<P>>,
	ctx: ArcContext<P>,
) -> Result<()>
where
	P: Send + Sync + Unpin + 'static,
{
	let subscriber = ctx
		.communicator
		.session
		.declare_subscriber(&key_expr)
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
				if let Some(cb) = p_cb.clone() {
					let result = cb.lock();
					match result {
						Ok(mut cb) => {
							if let Err(error) = cb.as_deref_mut().expect("snh")(&ctx, msg) {
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
							if let Err(error) = cb.as_deref_mut().expect("snh")(&ctx) {
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
