// Copyright © 2023 Stephan Kunz

// region:		--- modules
use crate::prelude::*;
use std::{collections::HashMap, fmt::Debug};
use tokio::task::JoinHandle;
use zenoh::prelude::r#async::AsyncResolve;
// endregion:	--- modules

// region:		--- types
/// type defnition for the queryables callback function.
#[allow(clippy::module_name_repetitions)]
pub type QueryableCallback<P> = fn(&Arc<Context<P>>, request: &Request);
// endregion:	--- types

// region:		--- QueryableBuilder
/// The builder fo a queryable.
#[allow(clippy::module_name_repetitions)]
#[derive(Default, Clone)]
pub struct QueryableBuilder<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	pub(crate) collection: Arc<RwLock<HashMap<String, Queryable<P>>>>,
	pub(crate) context: Arc<Context<P>>,
	pub(crate) key_expr: Option<String>,
	pub(crate) callback: Option<QueryableCallback<P>>,
}

impl<P> QueryableBuilder<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// Set the full expression for the queryable.
	#[must_use]
	pub fn key_expr(mut self, key_expr: impl Into<String>) -> Self {
		self.key_expr.replace(key_expr.into());
		self
	}

	/// Set the message qualifying part only.
	/// Will be prefixed by the agents prefix.
	#[must_use]
	pub fn msg_type(mut self, msg_type: impl Into<String>) -> Self {
		let key_expr = self.context.key_expr(msg_type);
		self.key_expr.replace(key_expr);
		self
	}

	/// Set the queryables callback function.
	#[must_use]
	pub fn callback(mut self, callback: QueryableCallback<P>) -> Self {
		self.callback.replace(callback);
		self
	}

	/// Build the queryable
	/// # Errors
	///
	/// # Panics
	///
	pub fn build(mut self) -> Result<Queryable<P>> {
		if self.key_expr.is_none() {
			return Err(Error::NoKeyExpression);
		}
		let callback = if self.callback.is_none() {
			return Err(Error::NoCallback);
		} else {
			self.callback.expect("should never happen")
		};
		let key_expr = if self.key_expr.is_some() {
			self.key_expr.take().expect("should never happen")
		} else {
			String::new()
		};

		//dbg!(&key_expr);
		let q = Queryable {
			key_expr,
			callback,
			handle: None,
			context: self.context,
		};

		Ok(q)
	}

	/// Build and add the queryable to the agent
	/// # Errors
	///
	/// # Panics
	///
	pub fn add(self) -> Result<()> {
		let collection = self.collection.clone();
		let q = self.build()?;

		collection
			.write()
			.expect("should never happen")
			.insert(q.key_expr.clone(), q);
		Ok(())
	}
}
// endregion:	--- QueryableBuilder

// region:		--- Queryable
/// Queryable
pub struct Queryable<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	key_expr: String,
	callback: QueryableCallback<P>,
	handle: Option<JoinHandle<()>>,
	context: Arc<Context<P>>,
}

impl<P> Queryable<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// Start Queryable
	/// # Panics
	///
	pub fn start(&mut self) {
		let key_expr = self.key_expr.clone();
		//dbg!(&key_expr);
		let cb = self.callback;
		let ctx = self.context.clone();
		self.handle.replace(tokio::spawn(async move {
			run_queryable(key_expr, cb, ctx).await;
		}));
	}

	/// Stop Queryable
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
async fn run_queryable<P>(
	key_expr: String,
	cb: QueryableCallback<P>,
	ctx: Arc<Context<P>>,
) where
	P: Debug + Send + Sync + Unpin + 'static,
{
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
		let request = Request { query };
		cb(&ctx, &request);
	}
}
// endregion:	--- Queryable

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Queryable<Props>>();
		is_normal::<QueryableBuilder<Props>>();
	}
}
