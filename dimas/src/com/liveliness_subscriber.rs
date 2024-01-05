//! Copyright © 2023 Stephan Kunz

// region:		--- modules
use super::communicator::Communicator;
use crate::{context::Context, prelude::*};
use std::sync::{Arc, RwLock};
use tokio::task::JoinHandle;
use zenoh::prelude::{Sample, r#async::AsyncResolve};
// endregion:	--- modules

// region:		--- types
pub type LivelinessSubscriberCallback<P> = fn(Arc<Context>, Arc<RwLock<P>>, sample: Sample);
// msg: Box<dyn DimasMessage<Msg=dyn Any>>
// endregion:	--- types

// region:		--- LivelinessSubscriberBuilder
#[derive(Clone, Debug)]
pub struct LivelinessSubscriberBuilder<P>
where
	P: std::fmt::Debug + Send + Sync + Unpin + 'static,
{
	pub(crate) subscriber: Arc<RwLock<Option<LivelinessSubscriber<P>>>>,
	pub(crate) communicator: Arc<Communicator>,
	pub(crate) props: Arc<RwLock<P>>,
	pub(crate) key_expr: Option<String>,
	pub(crate) msg_type: Option<String>,
	pub(crate) callback: Option<LivelinessSubscriberCallback<P>>,
}

impl<P> LivelinessSubscriberBuilder<P>
where
	P: std::fmt::Debug + Send + Sync + Unpin + 'static,
{
	pub fn key_expr(mut self, key_expr: impl Into<String>) -> Self {
		self.key_expr.replace(key_expr.into());
		self
	}

	pub fn msg_type(mut self, msg_type: impl Into<String>) -> Self {
		self.msg_type.replace(msg_type.into());
		self
	}

	pub fn callback(mut self, callback: LivelinessSubscriberCallback<P>) -> Self {
		self.callback.replace(callback);
		self
	}

	pub async fn add(mut self) -> Result<()> {
		//dbg!(&self);
		if self.key_expr.is_none() && self.msg_type.is_none() {
			return Err("No key expression or msg type given".into());
		}
		if self.callback.is_none() {
			return Err("No callback given".into());
		}
		let key_expr = if self.key_expr.is_some() {
			self.key_expr.take().unwrap()
		} else {
			self.communicator.clone().prefix() + "/" + &self.msg_type.unwrap() + "/*"
		};

		let communicator = self.communicator;
		let ctx = Arc::new(Context { communicator });

		let s = LivelinessSubscriber {
			key_expr,
			callback: self.callback.take().unwrap(),
			handle: None,
			context: ctx,
			props: self.props,
		};

		self.subscriber.write().unwrap().replace(s);
		Ok(())
	}
}
// endregion:	--- LivelinessSubscriberBuilder

// region:		--- LivelinessSubscriber
#[derive(Debug)]
pub struct LivelinessSubscriber<P>
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
	pub fn start(&mut self) -> Result<()> {
		let key_expr = self.key_expr.clone();
		dbg!(&key_expr);
		let cb = self.callback;
		let ctx = self.context.clone();
		let props = self.props.clone();
		self.handle.replace(tokio::spawn(async move {
			let session = ctx.communicator.session();
			let subscriber = session
				.liveliness()
				.declare_subscriber(&key_expr)
				.res_async()
				.await
				.unwrap();
			loop {
				let sample = subscriber.recv_async().await.unwrap();
				cb(ctx.clone(), props.clone(), sample);
			}

		}));
		Ok(())
	}

	pub fn stop(&mut self) -> Result<()> {
		self.handle.take().unwrap().abort();
		Ok(())
	}
}
// endregion:	--- Subscriber

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	fn normal_types() {
		is_normal::<LivelinessSubscriber<Props>>();
		is_normal::<LivelinessSubscriberBuilder<Props>>();
	}
}
