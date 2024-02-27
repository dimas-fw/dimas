// Copyright © 2023 Stephan Kunz

// region:		--- modules
use crate::prelude::*;
use std::{collections::HashMap, fmt::Debug};
use tokio::task::JoinHandle;
use zenoh::prelude::{r#async::AsyncResolve, SampleKind};
// endregion:	--- modules

// region:		--- types
/// Type definition for a subscribers `publish` callback function
#[allow(clippy::module_name_repetitions)]
pub type SubscriberPutCallback<P> = fn(&Arc<Context<P>>, &Arc<RwLock<P>>, messsage: &Message);
/// Type definition for a subscribers `delete` callback function
#[allow(clippy::module_name_repetitions)]
pub type SubscriberDeleteCallback<P> = fn(&Arc<Context<P>>, &Arc<RwLock<P>>);
// endregion:	--- types

// region:		--- SubscriberBuilder
/// A builder for a subscriber
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct SubscriberBuilder<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	pub(crate) collection: Arc<RwLock<HashMap<String, Subscriber<P>>>>,
	pub(crate) context: Arc<Context<P>>,
	pub(crate) props: Arc<RwLock<P>>,
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
	pub fn put_callback(mut self, callback: SubscriberPutCallback<P>) -> Self {
		self.put_callback.replace(callback);
		self
	}

	/// Set subscribers callback for `delete` messages
	#[must_use]
	pub fn delete_callback(mut self, callback: SubscriberDeleteCallback<P>) -> Self {
		self.delete_callback.replace(callback);
		self
	}

	/// Build the subscriber
	/// # Errors
	///
	/// # Panics
	///
	pub fn build(mut self) -> Result<Subscriber<P>> {
		if self.key_expr.is_none() {
			return Err(DimasError::NoKeyExpression);
		}
		let put_callback = if self.put_callback.is_none() {
			return Err(DimasError::NoCallback);
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
			props: self.props,
		};

		Ok(s)
	}

	/// Build and add the subscriber to the agent
	/// # Errors
	///
	/// # Panics
	///
	pub fn add(self) -> Result<()> {
		let collection = self.collection.clone();
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
	context: Arc<Context<P>>,
	props: Arc<RwLock<P>>,
}

impl<P> std::fmt::Debug for Subscriber<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Subscriber")
			.field("key_expr", &self.key_expr)
			//.field("put_callback", &self.put_callback)
			//.field("delete_callback", &self.put_callback)
			//.field("handle", &self.handle)
			//.field("context", &self.context)
			//.field("props", &self.props)
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
		let p_cb = self.put_callback;
		let d_cb = self.delete_callback;
		let ctx = self.context.clone();
		let props = self.props.clone();
		self.handle.replace(tokio::spawn(async move {
			run_liveliness(key_expr, p_cb, d_cb, ctx, props).await;
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

#[tracing::instrument(level = tracing::Level::DEBUG)]
async fn run_liveliness<P>(
	key_expr: String,
	p_cb: SubscriberPutCallback<P>,
	d_cb: Option<SubscriberDeleteCallback<P>>,
	ctx: Arc<Context<P>>,
	props: Arc<RwLock<P>>,
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
		//tracing::info!("{}: {}", sample.kind, sample.value);
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
				p_cb(&ctx, &props, &msg);
			}
			SampleKind::Delete => {
				if let Some(cb) = d_cb {
					cb(&ctx, &props);
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
