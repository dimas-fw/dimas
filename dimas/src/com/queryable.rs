// Copyright © 2023 Stephan Kunz

// region:		--- modules
use super::communicator::Communicator;
use crate::{context::Context, prelude::*};
use std::sync::{Arc, RwLock};
use tokio::task::JoinHandle;
use zenoh::prelude::r#async::AsyncResolve;
// endregion:	--- modules

// region:		--- types
/// type defnition for the queryables callback function.
#[allow(clippy::module_name_repetitions)]
pub type QueryableCallback<P> = fn(&Arc<Context>, &Arc<RwLock<P>>, query: zenoh::queryable::Query);
// endregion:	--- types

// region:		--- QueryableBuilder
/// The builder fo a queryable.
#[allow(clippy::module_name_repetitions)]
#[derive(Default, Clone)]
pub struct QueryableBuilder<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub(crate) collection: Arc<RwLock<Vec<Queryable<P>>>>,
	pub(crate) communicator: Arc<Communicator>,
	pub(crate) props: Arc<RwLock<P>>,
	pub(crate) key_expr: Option<String>,
	pub(crate) msg_type: Option<String>,
	pub(crate) callback: Option<QueryableCallback<P>>,
}

impl<P> QueryableBuilder<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full expression for the queryable.
	pub fn key_expr(mut self, key_expr: impl Into<String>) -> Self {
		self.key_expr.replace(key_expr.into());
		self
	}

	/// Set the message qualifying part only.
	/// Will be prefixed by the agents prefix.
	pub fn msg_type(mut self, msg_type: impl Into<String>) -> Self {
		self.msg_type.replace(msg_type.into());
		self
	}

	/// Set the queryables callback function.
	pub fn callback(mut self, callback: QueryableCallback<P>) -> Self {
		self.callback.replace(callback);
		self
	}

	/// Add the queryable to the agent
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
			self.communicator.prefix() + "/" + &self.msg_type.expect("should never happen")
		};

		let communicator = self.communicator;
		let ctx = Arc::new(Context { communicator });
		//dbg!(&key_expr);
		let q = Queryable {
			key_expr,
			callback,
			handle: None,
			context: ctx,
			props: self.props,
		};

		self.collection
			.write()
			.expect("should never happen")
			.push(q);
		Ok(())
	}
}
// endregion:	--- QueryableBuilder

// region:		--- Queryable
pub(crate) struct Queryable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	key_expr: String,
	callback: QueryableCallback<P>,
	handle: Option<JoinHandle<()>>,
	context: Arc<Context>,
	props: Arc<RwLock<P>>,
}

impl<P> Queryable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub(crate) fn start(&mut self) {
		let key_expr = self.key_expr.clone();
		//dbg!(&key_expr);
		let cb = self.callback;
		let ctx = self.context.clone();
		let props = self.props.clone();
		self.handle.replace(tokio::spawn(async move {
			let session = ctx.communicator.session.clone();
			let subscriber = session
				.declare_queryable(&key_expr)
				.res_async()
				.await
				.expect("should never happen");

			loop {
				let query = subscriber
					.recv_async()
					.await
					.expect("should never happen");
				//dbg!(&query);
				cb(&ctx, &props, query);
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
// endregion:	--- Queryable

#[cfg(test)]
mod tests {
	use super::*;

	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Queryable<Props>>();
		is_normal::<QueryableBuilder<Props>>();
	}
}
