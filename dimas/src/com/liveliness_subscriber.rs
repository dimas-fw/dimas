// Copyright © 2023 Stephan Kunz

// region:		--- modules
use crate::prelude::*;
use std::{fmt::Debug, sync::Mutex};
use tokio::task::JoinHandle;
use tracing::{span, Level};
use zenoh::prelude::{r#async::AsyncResolve, SampleKind};
// endregion:	--- modules

// region:		--- types
/// Type definition for liveliness callback function
pub type LivelinessCallback<P> =
	Arc<Mutex<dyn FnMut(&ArcContext<P>, &str) + Send + Sync + Unpin + 'static>>;
// endregion:	--- types

// region:		--- LivelinessSubscriberBuilder
/// The builder for the liveliness subscriber
#[allow(clippy::module_name_repetitions)]
#[derive(Default, Clone)]
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
		F: FnMut(&ArcContext<P>, &str) + Send + Sync + Unpin + 'static,
	{
		self.put_callback
			.replace(Arc::new(Mutex::new(callback)));
		self
	}

	/// Set liveliness subscribers callback for `delete` messages
	#[must_use]
	pub fn delete_callback<F>(mut self, callback: F) -> Self
	where
		F: FnMut(&ArcContext<P>, &str) + Send + Sync + Unpin + 'static,
	{
		self.delete_callback
			.replace(Arc::new(Mutex::new(callback)));
		self
	}

	/// Add the liveliness subscriber to the agent
	/// # Errors
	///
	/// # Panics
	///
	#[cfg_attr(any(nightly, docrs), doc, doc(cfg(feature = "liveliness")))]
	#[cfg(feature = "liveliness")]
	pub fn add(mut self) -> Result<()> {
		if self.key_expr.is_none() {
			return Err(Error::NoKeyExpression);
		}
		let put_callback = if self.put_callback.is_none() {
			return Err(Error::NoCallback);
		} else {
			self.put_callback.expect("should never happen")
		};
		let key_expr = if self.key_expr.is_some() {
			self.key_expr.take().expect("should never happen")
		} else {
			String::new()
		};

		let s = LivelinessSubscriber {
			key_expr,
			put_callback,
			delete_callback: self.delete_callback,
			handle: None,
			context: self.context,
		};

		self.subscriber
			.write()
			.expect("should never happen")
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
			run_liveliness(key_expr, p_cb, d_cb, ctx).await;
		}));

		// the initial liveliness query
		let key_expr = self.key_expr.clone();
		let p_cb = self.put_callback.clone();
		let ctx = self.context.clone();
		tokio::spawn(async move {
			run_initial(key_expr, p_cb, ctx).await;
		});
	}

	pub fn stop(&mut self) {
		self.handle
			.take()
			.expect("should never happen")
			.abort();
	}
}

//#[tracing::instrument(level = tracing::Level::DEBUG)]
async fn run_liveliness<P>(
	mut key_expr: String,
	p_cb: LivelinessCallback<P>,
	d_cb: Option<LivelinessCallback<P>>,
	ctx: ArcContext<P>,
) where
	P: Debug + Send + Sync + Unpin + 'static,
{
	let session = ctx.communicator.session.clone();
	let subscriber = session
		.liveliness()
		.declare_subscriber(&key_expr)
		.res_async()
		.await
		.expect("should never happen");

	key_expr.pop().expect("should never happen");

	loop {
		let sample = subscriber
			.recv_async()
			.await
			.expect("should never happen");
		let agent_id = sample.key_expr.to_string().replace(&key_expr, "");
		
		let span = span!(Level::DEBUG, "run_liveliness");
		let _guard = span.enter();
		match sample.kind {
			SampleKind::Put => {
				p_cb.lock().expect("should not happen")(&ctx, &agent_id);
			}
			SampleKind::Delete => {
				if let Some(cb) = d_cb.clone() {
					cb.lock().expect("should not happen")(&ctx, &agent_id);
				}
			}
		}
	}
}

//#[tracing::instrument(level = tracing::Level::DEBUG)]
async fn run_initial<P>(mut key_expr: String, p_cb: LivelinessCallback<P>, ctx: ArcContext<P>)
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	let session = ctx.communicator.session.clone();
	let replies = session
		.liveliness()
		.get(&key_expr)
		//.timeout(Duration::from_millis(500))
		.res()
		.await
		.expect("should never happen");

	key_expr.pop().expect("should never happen");

	while let Ok(reply) = replies.recv_async().await {
		let span = span!(Level::DEBUG, "run_initial_liveliness");
		let _guard = span.enter();
		match reply.sample {
			Ok(sample) => {
				//dbg!(&sample);
				let agent_id = sample.key_expr.to_string().replace(&key_expr, "");
				p_cb.lock().expect("should not happen")(&ctx, &agent_id);
			}
			Err(err) => println!(
				">> Received (ERROR: '{}')",
				String::try_from(&err).expect("to be implemented")
			),
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
		is_normal::<LivelinessSubscriber<Props>>();
		is_normal::<LivelinessSubscriberBuilder<Props>>();
	}
}
