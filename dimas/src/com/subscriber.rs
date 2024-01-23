// Copyright © 2023 Stephan Kunz

// region:		--- modules
use super::communicator::Communicator;
use crate::{context::Context, prelude::*};
use std::sync::{Arc, RwLock};
use tokio::task::JoinHandle;
use zenoh::prelude::{r#async::AsyncResolve, Sample};
// endregion:	--- modules

// region:		--- types
/// type definition for a subscriber callback function
#[allow(clippy::module_name_repetitions)]
pub type SubscriberCallback<P> = fn(&Arc<Context>, &Arc<RwLock<P>>, sample: Sample);
// endregion:	--- types

// region:		--- SubscriberBuilder
/// A builder for a subscriber
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct SubscriberBuilder<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub(crate) collection: Arc<RwLock<Vec<Subscriber<P>>>>,
	pub(crate) communicator: Arc<Communicator>,
	pub(crate) props: Arc<RwLock<P>>,
	pub(crate) key_expr: Option<String>,
	pub(crate) msg_type: Option<String>,
	pub(crate) callback: Option<SubscriberCallback<P>>,
}

impl<P> SubscriberBuilder<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full expression to subscribe on.
	pub fn key_expr(mut self, key_expr: impl Into<String>) -> Self {
		self.key_expr.replace(key_expr.into());
		self
	}

	/// Set only the message qualifying part of the expression to subscribe on.
	/// Will be prefixed by the agents prefix.
	pub fn msg_type(mut self, msg_type: impl Into<String>) -> Self {
		self.msg_type.replace(msg_type.into());
		self
	}

	/// Set subscribers callback
	pub fn callback(mut self, callback: SubscriberCallback<P>) -> Self {
		self.callback.replace(callback);
		self
	}

	/// add the subscriber to the agent
	pub fn add(mut self) -> Result<()> {
		if self.key_expr.is_none() && self.msg_type.is_none() {
			return Err("No key expression or msg type given".into());
		}
		let callback = if self.callback.is_none() {
			return Err("No callback given".into());
		} else {
			self.callback.expect("should never happen")
		};
		let key_expr = if self.key_expr.is_some() {
			self.key_expr.take().expect("should never happen")
		} else {
			self.communicator.clone().prefix()
				+ "/" + &self.msg_type.expect("should never happen")
				+ "/*"
		};

		let communicator = self.communicator;
		let ctx = Arc::new(Context { communicator });

		let s = Subscriber {
			key_expr,
			callback,
			handle: None,
			context: ctx,
			props: self.props,
		};

		self.collection
			.write()
			.expect("should never happen")
			.push(s);
		Ok(())
	}
}
// endregion:	--- SubscriberBuilder

// region:		--- Subscriber
pub(crate) struct Subscriber<P>
where
	P: Send + Sync + Unpin + 'static,
{
	key_expr: String,
	callback: SubscriberCallback<P>,
	handle: Option<JoinHandle<()>>,
	context: Arc<Context>,
	props: Arc<RwLock<P>>,
}

impl<P> std::fmt::Debug for Subscriber<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Subscriber")
			.field("key_expr", &self.key_expr)
			//.field("callback", &self.callback)
			//.field("handle", &self.handle)
			//.field("context", &self.context)
			//.field("props", &self.props)
			.finish_non_exhaustive()
	}
}

impl<P> Subscriber<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub(crate) fn start(&mut self) {
		let key_expr = self.key_expr.clone();
		let cb = self.callback;
		let ctx = self.context.clone();
		let props = self.props.clone();
		self.handle.replace(tokio::spawn(async move {
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
				cb(&ctx, &props, sample);
			}
		}));
	}

	pub(crate) fn stop(&mut self) {
		self.handle
			.take()
			.expect("should never happen")
			.abort();
	}
}
// endregion:	--- Subscriber

#[cfg(test)]
mod tests {
	use super::*;

	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Subscriber<Props>>();
		is_normal::<SubscriberBuilder<Props>>();
	}
}
