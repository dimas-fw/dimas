// Copyright © 2023 Stephan Kunz

//! Module `liveliness_subscriber` provides a `LivelinessSubscriber` which can be created using the `LivelinessSubscriberBuilder`.
//! A `LivelinessSubscriber` can optional subscribe on a delete message.

// region:		--- modules
use crate::{agent::Command, prelude::*};
use std::{
	fmt::Debug,
	sync::{mpsc::Sender, Mutex},
	time::Duration,
};
use tokio::task::JoinHandle;
use tracing::{error, instrument, warn, Level};
#[cfg(feature = "liveliness")]
use tracing::info;
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

// region:		--- LivelinessSubscriberBuilder
/// The builder for the liveliness subscriber
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct LivelinessSubscriberBuilder<P>
where
	P: std::fmt::Debug + Send + Sync + Unpin + 'static,
{
	pub(crate) subscriber: Arc<RwLock<Option<LivelinessSubscriber<P>>>>,
	pub(crate) context: ArcContext<P>,
	pub(crate) key_expr: Option<String>,
	pub(crate) put_callback: Option<LivelinessCallback<P>>,
	pub(crate) delete_callback: Option<LivelinessCallback<P>>,
}

impl<P> LivelinessSubscriberBuilder<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// Set the full expression for the liveliness subscriber
	#[must_use]
	pub fn key_expr(mut self, key_expr: impl Into<String>) -> Self {
		self.key_expr.replace(key_expr.into());
		self
	}

	/// Set only the message qualifing part of the liveliness subscriber.
	/// Will be prefixed with agents prefix.
	#[must_use]
	pub fn msg_type(mut self, msg_type: impl Into<String>) -> Self {
		let key_expr = self
			.context
			.communicator
			.clone()
			.key_expr(msg_type)
			+ "/*";
		self.key_expr.replace(key_expr);
		self
	}

	/// Set liveliness subscribers callback for `put` messages
	#[must_use]
	pub fn put_callback<F>(mut self, callback: F) -> Self
	where
		F: FnMut(&ArcContext<P>, &str) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		self.put_callback
			.replace(Arc::new(Mutex::new(Some(Box::new(callback)))));
		self
	}

	/// Set liveliness subscribers callback for `delete` messages
	#[must_use]
	pub fn delete_callback<F>(mut self, callback: F) -> Self
	where
		F: FnMut(&ArcContext<P>, &str) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		self.delete_callback
			.replace(Arc::new(Mutex::new(Some(Box::new(callback)))));
		self
	}

	/// Build the liveliness subscriber
	/// # Errors
	///
	pub fn build(self) -> Result<LivelinessSubscriber<P>> {
		let key_expr = if self.key_expr.is_none() {
			return Err(DimasError::NoKeyExpression.into());
		} else {
			self.key_expr.ok_or(DimasError::ShouldNotHappen)?
		};
		if self.put_callback.is_none() {
			return Err(DimasError::NoCallback.into());
		};

		let s = LivelinessSubscriber {
			key_expr,
			put_callback: self.put_callback,
			delete_callback: self.delete_callback,
			handle: None,
			context: self.context,
		};

		Ok(s)
	}

	/// Build and add the liveliness subscriber to the agent
	/// # Errors
	///
	#[cfg_attr(any(nightly, docrs), doc, doc(cfg(feature = "liveliness")))]
	#[cfg(feature = "liveliness")]
	pub fn add(self) -> Result<()> {
		let c = self.subscriber.clone();
		let s = self.build()?;

		c.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.replace(s);
		Ok(())
	}
}
// endregion:	--- LivelinessSubscriberBuilder

// region:		--- LivelinessSubscriber
pub struct LivelinessSubscriber<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	key_expr: String,
	put_callback: Option<LivelinessCallback<P>>,
	delete_callback: Option<LivelinessCallback<P>>,
	handle: Option<JoinHandle<()>>,
	context: ArcContext<P>,
}

impl<P> std::fmt::Debug for LivelinessSubscriber<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("LivelinessSubscriber")
			.field("key_expr", &self.key_expr)
			.finish_non_exhaustive()
	}
}

impl<P> LivelinessSubscriber<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// Start or restart the liveliness subscriber.
	/// An already running subscriber will be stopped, eventually damaged Mutexes will be repaired
	#[instrument(level = Level::TRACE, skip_all)]
	pub fn start(&mut self, tx: Sender<Command>) {
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
		let key_expr = self.key_expr.clone();
		let p_cb = self.put_callback.clone();
		let ctx = self.context.clone();
		tokio::spawn(async move {
			if let Err(error) = run_initial(key_expr, p_cb, ctx).await {
				error!("spawning initial liveliness failed with {error}");
			};
		});

		// the liveliness subscriber
		let key_expr = self.key_expr.clone();
		let p_cb = self.put_callback.clone();
		let d_cb = self.delete_callback.clone();
		let ctx = self.context.clone();

		self.handle.replace(tokio::spawn(async move {
			std::panic::set_hook(Box::new(move |reason| {
				error!("liveliness subscriber panic: {}", reason);
				#[cfg(feature = "liveliness")]
				if let Err(reason) = tx.send(Command::RestartLivelinessSubscriber) {
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
	mut key_expr: String,
	p_cb: Option<LivelinessCallback<P>>,
	d_cb: Option<LivelinessCallback<P>>,
	ctx: ArcContext<P>,
) -> Result<()>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	let subscriber = ctx
		.communicator
		.session
		.liveliness()
		.declare_subscriber(&key_expr)
		.res_async()
		.await
		.map_err(|_| DimasError::ShouldNotHappen)?;

	key_expr
		.pop()
		.ok_or(DimasError::ShouldNotHappen)?;

	loop {
		let result = subscriber.recv_async().await;
		match result {
			Ok(sample) => {
				let id = sample.key_expr.to_string().replace(&key_expr, "");
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
									if let Err(error) = cb.as_deref_mut().expect("snh")(&ctx, &id) {
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
									if let Err(error) = cb.as_deref_mut().expect("snh")(&ctx, &id) {
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
	mut key_expr: String,
	p_cb: Option<LivelinessCallback<P>>,
	ctx: ArcContext<P>,
) -> Result<()>
where
	P: Debug + Send + Sync + Unpin + 'static,
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
			key_expr
				.pop()
				.ok_or(DimasError::ShouldNotHappen)?;
			while let Ok(reply) = replies.recv_async().await {
				match reply.sample {
					Ok(sample) => {
						let id = sample.key_expr.to_string().replace(&key_expr, "");
						// skip own live message
						if id == ctx.uuid() {
							continue;
						};
						if let Some(cb) = p_cb.clone() {
							let result = cb.lock();
							match result {
								Ok(mut cb) => {
									if let Err(error) = cb.as_deref_mut().expect("snh")(&ctx, &id) {
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
		is_normal::<LivelinessSubscriberBuilder<Props>>();
	}
}
