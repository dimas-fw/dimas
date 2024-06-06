// Copyright © 2023 Stephan Kunz

//! Module `query` provides an information/compute requestor `Query` which can be created using the `QueryBuilder`.

// region:		--- modules
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	message_types::{Message, Response},
	traits::{Capability, Context},
};
use std::{
	fmt::Debug,
	sync::{Arc, Mutex},
	time::Duration,
};
use tracing::{error, instrument, Level};
use zenoh::{
	prelude::{sync::SyncResolve, SampleKind},
	query::{ConsolidationMode, QueryTarget},
	sample::Locality,
};
// endregion:	--- modules

// region:		--- types
/// type definition for the queries callback function
#[allow(clippy::module_name_repetitions)]
pub type QueryCallback<P> =
	Arc<Mutex<dyn FnMut(&Context<P>, Response) -> Result<()> + Send + Sync + Unpin + 'static>>;
// endregion:	--- types

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
	response_callback: QueryCallback<P>,
	mode: ConsolidationMode,
	allowed_destination: Locality,
	target: QueryTarget,
	timeout: Option<Duration>,
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
		response_callback: QueryCallback<P>,
		mode: ConsolidationMode,
		allowed_destination: Locality,
		target: QueryTarget,
		timeout: Option<Duration>,
	) -> Self {
		Self {
			selector,
			context,
			activation_state,
			response_callback,
			mode,
			allowed_destination,
			target,
			timeout,
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
		Ok(())
	}

	/// Run a Query with an optional [`Message`].
	#[instrument(name="query with message", level = Level::ERROR, skip_all)]
	pub fn get(
		&self,
		message: Option<Message>,
		mut callback: Option<&dyn Fn(Response) -> Result<()>>,
	) -> Result<()> {
		let cb = self.response_callback.clone();
		let session = self.context.session();
		let mut query = session
			.get(&self.selector)
			.target(self.target)
			.consolidation(self.mode)
			.allowed_destination(self.allowed_destination);

		if let Some(timeout) = self.timeout {
			query = query.timeout(timeout);
		};

		if let Some(message) = message {
			let value = message.value().to_owned();
			query = query.with_value(value);
		};

		let replies = query
			.res_sync()
			.map_err(|_| DimasError::ShouldNotHappen)?;

		while let Ok(reply) = replies.recv() {
			match reply.sample {
				Ok(sample) => match sample.kind {
					SampleKind::Put => {
						let content: Vec<u8> = sample.value.try_into()?;
						let msg = Response(content);
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
				Err(err) => error!("receive error: {err})"),
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
