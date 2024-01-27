// Copyright © 2023 Stephan Kunz

// region:		--- modules
use super::communicator::Communicator;
use crate::{context::Context, prelude::*};
use std::sync::{Arc, RwLock};
use tokio::task::JoinHandle;
use zenoh::prelude::{r#async::AsyncResolve, Sample};
// endregion:	--- modules

// region:		--- types
/// Type definition for liveliness subscribers callback function
#[allow(clippy::module_name_repetitions)]
pub type LivelinessSubscriberCallback<P> = fn(Arc<Context>, Arc<RwLock<P>>, sample: Sample);
// msg: Box<dyn DimasMessage<Msg=dyn Any>>
// endregion:	--- types

// region:		--- LivelinessSubscriberBuilder
/// The builder for the liveliness subscriber
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct LivelinessSubscriberBuilder<P>
where
	P: std::fmt::Debug + Send + Sync + Unpin + 'static,
{
	pub(crate) subscriber: Arc<RwLock<Option<LivelinessSubscriber<P>>>>,
	pub(crate) communicator: Arc<Communicator>,
	pub(crate) props: Arc<RwLock<P>>,
	pub(crate) key_expr: Option<String>,
	pub(crate) callback: Option<LivelinessSubscriberCallback<P>>,
}

impl<P> LivelinessSubscriberBuilder<P>
where
	P: std::fmt::Debug + Send + Sync + Unpin + 'static,
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
		let key_expr = self.communicator.clone().prefix() + "/" + &msg_type.into() + "/*";
		self.key_expr.replace(key_expr);
		self
	}

	/// Set the liveliness subscribers callback
	#[must_use]
	pub fn callback(mut self, callback: LivelinessSubscriberCallback<P>) -> Self {
		self.callback.replace(callback);
		self
	}

	/// Add the liveliness subscriber to the agent
	/// # Errors
	///
	/// # Panics
	///
	pub fn add(mut self) -> Result<()> {
		if self.key_expr.is_none() {
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
			String::new()
		};

		let communicator = self.communicator;
		let ctx = Arc::new(Context { communicator });

		let s = LivelinessSubscriber {
			key_expr,
			callback,
			handle: None,
			context: ctx,
			props: self.props,
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
#[derive(Debug)]
pub(crate) struct LivelinessSubscriber<P>
where
	P: std::fmt::Debug + Send + Sync + Unpin + 'static,
{
	key_expr: String,
	callback: LivelinessSubscriberCallback<P>,
	handle: Option<JoinHandle<()>>,
	context: Arc<Context>,
	props: Arc<RwLock<P>>,
}

impl<P> LivelinessSubscriber<P>
where
	P: std::fmt::Debug + Send + Sync + Unpin + 'static,
{
	pub fn start(&mut self) {
		let key_expr = self.key_expr.clone();
		let cb = self.callback;
		let ctx = self.context.clone();
		let props = self.props.clone();
		//dbg!(&key_expr);
		self.handle.replace(tokio::spawn(async move {
			let session = ctx.communicator.session.clone();
			let subscriber = session
				.liveliness()
				.declare_subscriber(&key_expr)
				.res_async()
				.await
				.expect("should never happen");
			loop {
				let sample = subscriber
					.recv_async()
					.await
					.expect("should never happen");
				cb(ctx.clone(), props.clone(), sample);
			}
		}));

		// the initial liveliness query
		let key_expr = self.key_expr.clone();
		let cb = self.callback;
		let ctx = self.context.clone();
		let props = self.props.clone();
		tokio::spawn(async move {
			let session = ctx.communicator.session.clone();
			let replies = session
				.liveliness()
				.get(&key_expr)
				//.timeout(Duration::from_millis(500))
				.res()
				.await
				.expect("should never happen");

			while let Ok(reply) = replies.recv_async().await {
				match reply.sample {
					Ok(sample) => {
						//dbg!(&sample);
						cb(ctx.clone(), props.clone(), sample);
					}
					Err(err) => println!(
						">> Received (ERROR: '{}')",
						String::try_from(&err).expect("to be implemented")
					),
				}
			}
		});
	}

	pub fn stop(&mut self) {
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
