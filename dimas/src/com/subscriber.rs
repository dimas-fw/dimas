// Copyright © 2023 Stephan Kunz

//! Module `subscriber` provides a message `Subscriber` which can be created using the `SubscriberBuilder`.
//! A `Subscriber` can optional subscribe on a delete message.

// region:		--- modules
use crate::prelude::*;
use std::{fmt::Debug, sync::Mutex};
use tokio::task::JoinHandle;
use tracing::{span, Level};
use zenoh::prelude::{r#async::AsyncResolve, SampleKind};
// endregion:	--- modules

// region:		--- types
/// Type definition for a subscribers `publish` callback function
#[allow(clippy::module_name_repetitions)]
pub type SubscriberPutCallback<P> =
	Arc<Mutex<dyn FnMut(&ArcContext<P>, &Message) + Send + Sync + Unpin + 'static>>;
/// Type definition for a subscribers `delete` callback function
#[allow(clippy::module_name_repetitions)]
pub type SubscriberDeleteCallback<P> =
	Arc<Mutex<dyn FnMut(&ArcContext<P>) + Send + Sync + Unpin + 'static>>;
// endregion:	--- types

// region:		--- SubscriberBuilder
/// A builder for a subscriber
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct SubscriberBuilder<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	pub(crate) context: ArcContext<P>,
	pub(crate) key_expr: Option<String>,
	pub(crate) put_callback: Option<SubscriberPutCallback<P>>,
	pub(crate) delete_callback: Option<SubscriberDeleteCallback<P>>,
}

impl<P> SubscriberBuilder<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// Set the full expression to subscribe on.
	#[must_use]
	pub fn key_expr(mut self, key_expr: impl Into<String>) -> Self {
		self.key_expr.replace(key_expr.into());
		self
	}

	/// Set only the message qualifying part of the expression to subscribe on.
	/// Will be prefixed by the agents prefix.
	#[must_use]
	pub fn msg_type(mut self, msg_type: impl Into<String>) -> Self {
		let key_expr = self.context.key_expr(msg_type);
		self.key_expr.replace(key_expr);
		self
	}

	/// Set subscribers callback for `put` messages
	#[must_use]
	pub fn put_callback<F>(mut self, callback: F) -> Self
	where
		F: FnMut(&ArcContext<P>, &Message) + Send + Sync + Unpin + 'static,
	{
		self.put_callback
			.replace(Arc::new(Mutex::new(callback)));
		self
	}

	/// Set subscribers callback for `delete` messages
	#[must_use]
	pub fn delete_callback<F>(mut self, callback: F) -> Self
	where
		F: FnMut(&ArcContext<P>) + Send + Sync + Unpin + 'static,
	{
		self.delete_callback
			.replace(Arc::new(Mutex::new(callback)));
		self
	}

	/// Build the subscriber
	/// # Errors
	///
	/// # Panics
	///
	pub fn build(mut self) -> Result<Subscriber<P>> {
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

		let s = Subscriber {
			key_expr,
			put_callback,
			delete_callback: self.delete_callback,
			handle: None,
			context: self.context,
		};

		Ok(s)
	}

	/// Build and add the subscriber to the agents context
	/// # Errors
	///
	/// # Panics
	///
	#[cfg_attr(any(nightly, docrs), doc, doc(cfg(feature = "subscriber")))]
	#[cfg(feature = "subscriber")]
	pub fn add(self) -> Result<()> {
		let collection = self.context.subscribers.clone();
		let s = self.build()?;

		collection
			.write()
			.expect("should never happen")
			.insert(s.key_expr.clone(), s);
		Ok(())
	}
}
// endregion:	--- SubscriberBuilder

// region:		--- Subscriber
/// Subscriber
pub struct Subscriber<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	key_expr: String,
	put_callback: SubscriberPutCallback<P>,
	delete_callback: Option<SubscriberDeleteCallback<P>>,
	handle: Option<JoinHandle<()>>,
	context: ArcContext<P>,
}

impl<P> std::fmt::Debug for Subscriber<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Subscriber")
			.field("key_expr", &self.key_expr)
			.finish_non_exhaustive()
	}
}

impl<P> Subscriber<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// Start Subscriber
	/// # Panics
	///
	pub fn start(&mut self) {
		let key_expr = self.key_expr.clone();
		let p_cb = self.put_callback.clone();
		let d_cb = self.delete_callback.clone();
		let ctx = self.context.clone();
		self.handle.replace(tokio::spawn(async move {
			run_subscriber(key_expr, p_cb, d_cb, ctx).await;
		}));
	}

	/// Stop Subscriber
	/// # Panics
	///
	pub fn stop(&mut self) {
		self.handle
			.take()
			.expect("should never happen")
			.abort();
	}
}

//#[tracing::instrument(level = tracing::Level::DEBUG)]
async fn run_subscriber<P>(
	key_expr: String,
	p_cb: SubscriberPutCallback<P>,
	d_cb: Option<SubscriberDeleteCallback<P>>,
	ctx: ArcContext<P>,
) where
	P: Debug + Send + Sync + Unpin + 'static,
{
	let session = ctx.communicator.session.clone();
	let subscriber = session
		.declare_subscriber(&key_expr)
		.res_async()
		.await
		.expect("should never happen");

	loop {
		let sample = subscriber
			.recv_async()
			.await
			.expect("should never happen");

		let span = span!(Level::DEBUG, "run_subscriber");
		let _guard = span.enter();
		match sample.kind {
			SampleKind::Put => {
				let value: Vec<u8> = sample
					.value
					.try_into()
					.expect("should not happen");

				let msg = Message {
					key_expr: sample.key_expr.to_string(),
					value,
				};
				p_cb.lock().expect("should not happen")(&ctx, &msg);
			}
			SampleKind::Delete => {
				if let Some(cb) = d_cb.clone() {
					cb.lock().expect("should not happen")(&ctx);
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
		is_normal::<SubscriberBuilder<Props>>();
	}
}
