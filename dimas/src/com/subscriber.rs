//! Copyright © 2023 Stephan Kunz

// region:    --- modules
use super::communicator::Communicator;
use crate::{context::Context, prelude::*};
use std::sync::{Arc, RwLock};
use tokio::task::JoinHandle;
use zenoh::prelude::{r#async::AsyncResolve, Sample};
// endregion: --- modules

// region:    --- types
pub type SubscriberCallback<P> = fn(Arc<Context>, Arc<RwLock<P>>, sample: Sample);
// endregion: --- types

// region:    --- SubscriberBuilder
#[derive(Clone)]
pub struct SubscriberBuilder<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub(crate) collection: Option<Arc<RwLock<Vec<Subscriber<P>>>>>,
	pub(crate) communicator: Option<Arc<Communicator>>,
	pub(crate) props: Option<Arc<RwLock<P>>>,
	pub(crate) key_expr: Option<String>,
	pub(crate) msg_type: Option<String>,
	pub(crate) callback: Option<SubscriberCallback<P>>,
}

impl<P> SubscriberBuilder<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub fn key_expr(mut self, key_expr: impl Into<String>) -> Self {
		self.key_expr.replace(key_expr.into());
		self
	}

	pub fn msg_type(mut self, msg_type: impl Into<String>) -> Self {
		self.msg_type.replace(msg_type.into());
		self
	}

	pub fn callback(mut self, callback: SubscriberCallback<P>) -> Self {
		self.callback.replace(callback);
		self
	}

	pub async fn add(mut self) -> Result<()> {
		if self.communicator.is_none() {
			return Err("No communicator given".into());
		}
		if self.key_expr.is_none() && self.msg_type.is_none() {
			return Err("No key expression or msg type given".into());
		}
		if self.callback.is_none() {
			return Err("No callback given".into());
		}
		let key_expr = if self.key_expr.is_some() {
			self.key_expr.take().unwrap()
		} else {
			self.communicator.clone().unwrap().prefix() + "/" + &self.msg_type.unwrap() + "/*"
		};

		let mut ctx = Arc::new(Context::default());
		if self.communicator.is_some() {
			let communicator = self.communicator.unwrap();
			ctx = Arc::new(Context { communicator });
		}

		let s = Subscriber {
			key_expr,
			callback: self.callback.take().unwrap(),
			handle: None,
			context: ctx,
			props: self.props.unwrap(),
		};

		let c = self.collection.take();
		c.unwrap().write().unwrap().push(s);
		Ok(())
	}
}
// endregion: --- SubscriberBuilder

// region:    --- Subscriber
pub struct Subscriber<P>
where
	P: Send + Sync + Unpin + 'static,
{
	key_expr: String,
	callback: SubscriberCallback<P>,
	handle: Option<JoinHandle<()>>,
	context: Arc<Context>,
	props: Arc<RwLock<P>>,
}

impl<P> Subscriber<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub fn start(&mut self) -> Result<()> {
		let key_expr = self.key_expr.clone();
		let cb = self.callback;
		let ctx = self.context.clone();
		let props = self.props.clone();
		self.handle.replace(tokio::spawn(async move {
			let session = ctx.communicator.session();
			let subscriber = session
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
// endregion: --- Subscriber

#[cfg(test)]
mod tests {
	use super::*;

	struct Props {}

	// check, that the auto traits are available
	fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	fn normal_types() {
		is_normal::<Subscriber<Props>>();
		is_normal::<SubscriberBuilder<Props>>();
	}
}
