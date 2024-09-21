// Copyright © 2023 Stephan Kunz

//! Module `query` provides an information/compute requestor `Query` which can be created using the `QueryBuilder`.

// region:		--- modules
use super::ArcQueryCallback;
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	message_types::{Message, QueryableMsg},
	traits::{Capability, Context},
};
use std::{fmt::Debug, time::Duration};
use tracing::{error, instrument, warn, Level};
use zenoh::{
	query::{ConsolidationMode, QueryTarget},
	sample::{Locality, SampleKind},
	Wait,
};
// endregion:	--- modules

// region:		--- Query
/// Query
pub struct Query<P>
where
	P: Send + Sync + Unpin + 'static,
{
	selector: String,
	/// Context for the Query
	context: Context<P>,
	activation_state: OperationState,
	callback: ArcQueryCallback<P>,
	mode: ConsolidationMode,
	allowed_destination: Locality,
	target: QueryTarget,
	timeout: Option<Duration>,
	key_expr: Option<zenoh::key_expr::KeyExpr<'static>>,
}

impl<P> Debug for Query<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Query")
			.field("selector", &self.selector)
			.field("mode", &self.mode)
			.field("allowed_destination", &self.allowed_destination)
			.finish_non_exhaustive()
	}
}

impl<P> Capability for Query<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn manage_operation_state(&mut self, state: &OperationState) -> Result<()> {
		if state >= &self.activation_state {
			return self.init();
		} else if state < &self.activation_state {
			return self.de_init();
		}
		Ok(())
	}
}

impl<P> Query<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor for a [`Query`]
	#[must_use]
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		selector: String,
		context: Context<P>,
		activation_state: OperationState,
		response_callback: ArcQueryCallback<P>,
		mode: ConsolidationMode,
		allowed_destination: Locality,
		target: QueryTarget,
		timeout: Option<Duration>,
	) -> Self {
		Self {
			selector,
			context,
			activation_state,
			callback: response_callback,
			mode,
			allowed_destination,
			target,
			timeout,
			key_expr: None,
		}
	}

	/// Get `selector`
	#[must_use]
	pub fn selector(&self) -> &str {
		&self.selector
	}

	/// Initialize
	/// # Errors
	#[allow(clippy::unused_self)]
	#[allow(clippy::unnecessary_wraps)]
	fn init(&mut self) -> Result<()>
	where
		P: Send + Sync + Unpin + 'static,
	{
		let key_expr = self
			.context
			.session()
			.declare_keyexpr(self.selector.clone())
			.wait()?;
		self.key_expr.replace(key_expr);
		Ok(())
	}

	/// De-Initialize
	/// # Errors
	#[allow(clippy::unused_self)]
	#[allow(clippy::unnecessary_wraps)]
	fn de_init(&mut self) -> Result<()>
	where
		P: Send + Sync + Unpin + 'static,
	{
		self.key_expr.take();
		Ok(())
	}

	/// Run a Query with an optional [`Message`].
	#[instrument(name="query", level = Level::ERROR, skip_all)]
	pub fn get(
		&self,
		message: Option<Message>,
		mut callback: Option<&dyn Fn(QueryableMsg) -> Result<()>>,
	) -> Result<()> {
		// check Mutex
		{
			if self.callback.lock().is_err() {
				warn!("found poisoned Mutex");
				self.callback.clear_poison();
			}
		}

		let cb = self.callback.clone();
		let session = self.context.session();
		let mut query = message
			.map_or_else(
				|| session.get(&self.selector),
				|msg| session.get(&self.selector).payload(msg.value()),
			)
			.target(self.target)
			.consolidation(self.mode)
			.allowed_destination(self.allowed_destination);

		if let Some(timeout) = self.timeout {
			query = query.timeout(timeout);
		};

		let replies = query
			.wait()
			.map_err(|_| DimasError::ShouldNotHappen)?;

		while let Ok(reply) = replies.recv() {
			match reply.result() {
				Ok(sample) => match sample.kind() {
					SampleKind::Put => {
						let content: Vec<u8> = sample.payload().into();
						let msg = QueryableMsg(content);
						if callback.is_none() {
							let guard = cb.lock();
							match guard {
								Ok(mut lock) => {
									if let Err(error) = lock(&self.context.clone(), msg) {
										error!("callback failed with {error}");
									}
								}
								Err(err) => {
									error!("callback lock failed with {err}");
								}
							}
						} else {
							callback.as_mut().expect("snh")(msg)?;
						}
					}
					SampleKind::Delete => {
						error!("Delete in Query");
					}
				},
				Err(err) => error!("receive error: {:?})", err),
			}
		}
		Ok(())
	}
}
// endregion:	--- Query

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Query<Props>>();
	}
}
