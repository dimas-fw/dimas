// Copyright © 2023 Stephan Kunz

//! Module `liveliness_subscriber` provides a `LivelinessSubscriber` which can be created using the `LivelinessSubscriberBuilder`.
//! A `LivelinessSubscriber` can optional subscribe on a delete message.

// region:		--- modules
use crate::prelude::*;
use std::{fmt::Debug, sync::Mutex};
use tokio::task::JoinHandle;
use tracing::{error, span, Level};
use zenoh::prelude::{r#async::AsyncResolve, SampleKind};
// endregion:	--- modules

// region:		--- types
/// Type definition for liveliness callback function
pub type LivelinessCallback<P> = Arc<
	Mutex<
		dyn FnMut(&ArcContext<P>, &str) -> Result<(), DimasError> + Send + Sync + Unpin + 'static,
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
		F: FnMut(&ArcContext<P>, &str) -> Result<(), DimasError> + Send + Sync + Unpin + 'static,
	{
		self.put_callback
			.replace(Arc::new(Mutex::new(callback)));
		self
	}

	/// Set liveliness subscribers callback for `delete` messages
	#[must_use]
	pub fn delete_callback<F>(mut self, callback: F) -> Self
	where
		F: FnMut(&ArcContext<P>, &str) -> Result<(), DimasError> + Send + Sync + Unpin + 'static,
	{
		self.delete_callback
			.replace(Arc::new(Mutex::new(callback)));
		self
	}

	/// Build the liveliness subscriber
	/// # Errors
	///
	pub fn build(self) -> Result<LivelinessSubscriber<P>, DimasError> {
		let key_expr = if self.key_expr.is_none() {
			return Err(DimasError::NoKeyExpression);
		} else {
			self.key_expr.ok_or(DimasError::ShouldNotHappen)?
		};
		let put_callback = if self.put_callback.is_none() {
			return Err(DimasError::NoCallback);
		} else {
			self.put_callback
				.ok_or(DimasError::ShouldNotHappen)?
		};

		let s = LivelinessSubscriber {
			key_expr,
			put_callback,
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
	pub fn add(self) -> Result<(), DimasError> {
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
	put_callback: LivelinessCallback<P>,
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
	pub fn start(&mut self) {
		let key_expr = self.key_expr.clone();
		let p_cb = self.put_callback.clone();
		let d_cb = self.delete_callback.clone();
		let ctx = self.context.clone();
		//dbg!(&key_expr);
		self.handle.replace(tokio::spawn(async move {
			if let Err(error) = run_liveliness(key_expr, p_cb, d_cb, ctx).await {
				error!("spawning liveliness subscriber failed with {error}");
			};
		}));

		// the initial liveliness query
		let key_expr = self.key_expr.clone();
		let p_cb = self.put_callback.clone();
		let ctx = self.context.clone();
		tokio::spawn(async move {
			if let Err(error) = run_initial(key_expr, p_cb, ctx).await {
				error!("spawning initial liveliness failed with {error}");
			};
		});
	}

	pub fn stop(&mut self) -> Result<(), DimasError> {
		self.handle
			.take()
			.ok_or(DimasError::ShouldNotHappen)?
			.abort();
		Ok(())
	}
}

//#[tracing::instrument(level = tracing::Level::DEBUG)]
async fn run_liveliness<P>(
	mut key_expr: String,
	p_cb: LivelinessCallback<P>,
	d_cb: Option<LivelinessCallback<P>>,
	ctx: ArcContext<P>,
) -> Result<(), DimasError>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	let session = ctx.communicator.session.clone();
	let subscriber = session
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
		let span = span!(Level::DEBUG, "run_liveliness");
		let _guard = span.enter();
		match result {
			Ok(sample) => {
				let id = sample.key_expr.to_string().replace(&key_expr, "");
				match sample.kind {
					SampleKind::Put => {
						let guard = p_cb.lock();
						match guard {
							Ok(mut lock) => {
								if let Err(error) = lock(&ctx, &id) {
									error!("liveliness put callback failed with {error}");
								}
							}
							Err(err) => {
								error!("liveliness put callback lock failed with {err}");
							}
						}
					}
					SampleKind::Delete => {
						if let Some(cb) = d_cb.clone() {
							let guard = cb.lock();
							match guard {
								Ok(mut lock) => {
									if let Err(error) = lock(&ctx, &id) {
										error!("liveliness delete callback failed with {error}");
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
				error!("livelieness subscriber failed with {error}");
			}
		}
	}
}

//#[tracing::instrument(level = tracing::Level::DEBUG)]
async fn run_initial<P>(
	key_expr: String,
	p_cb: LivelinessCallback<P>,
	ctx: ArcContext<P>,
) -> Result<(), DimasError>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	let span = span!(Level::DEBUG, "run_initial_liveliness");
	let _guard = span.enter();
	let session = ctx.communicator.session.clone();
	let result = session
		.liveliness()
		.get(&key_expr)
		//.timeout(Duration::from_millis(500))
		.res()
		.await;

	match result {
		Ok(replies) => {
			let span = span!(Level::DEBUG, "run_initial_liveliness");
			let _guard = span.enter();
			while let Ok(reply) = replies.recv_async().await {
				match reply.sample {
					Ok(sample) => {
						let id = sample.key_expr.to_string().replace(&key_expr, "");
						let guard = p_cb.lock();
						match guard {
							Ok(mut lock) => {
								if let Err(error) = lock(&ctx, &id) {
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
		is_normal::<LivelinessSubscriberBuilder<Props>>();
	}
}
