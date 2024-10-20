// Copyright © 2023 Stephan Kunz

//! Module `Querier` provides an information/compute requestor `Querier` which can be created using the `QuerierBuilder`.

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use crate::error::Error;
use alloc::sync::Arc;
use core::{fmt::Debug, time::Duration};
use dimas_core::{
	enums::OperationState,
	message_types::{Message, QueryableMsg},
	traits::{Capability, Context},
	Result,
};
use futures::future::BoxFuture;
#[cfg(feature = "std")]
use std::prelude::rust_2021::*;
#[cfg(feature = "std")]
use tokio::sync::Mutex;
use tracing::{error, instrument, warn, Level};
#[cfg(feature = "unstable")]
use zenoh::sample::Locality;
use zenoh::{
	query::{ConsolidationMode, QueryTarget},
	sample::SampleKind,
	Wait,
};
// endregion:	--- modules

// region:    	--- types
/// type definition for a queriers `response` callback
pub type GetCallback<P> =
	Box<dyn FnMut(Context<P>, QueryableMsg) -> BoxFuture<'static, Result<()>> + Send + Sync>;
/// type definition for a queriers atomic reference counted `response` callback
pub type ArcGetCallback<P> = Arc<Mutex<GetCallback<P>>>;
// endregion: 	--- types

// region:		--- Querier
/// Querier
pub struct Querier<P>
where
	P: Send + Sync + 'static,
{
	selector: String,
	/// Context for the Querier
	context: Context<P>,
	activation_state: OperationState,
	callback: ArcGetCallback<P>,
	mode: ConsolidationMode,
	#[cfg(feature = "unstable")]
	allowed_destination: Locality,
	encoding: String,
	target: QueryTarget,
	timeout: Duration,
	key_expr: Option<zenoh::key_expr::KeyExpr<'static>>,
}

impl<P> Debug for Querier<P>
where
	P: Send + Sync + 'static,
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		#[cfg(feature = "unstable")]
		let res = f
			.debug_struct("Querier")
			.field("selector", &self.selector)
			.field("mode", &self.mode)
			.field("allowed_destination", &self.allowed_destination)
			.finish_non_exhaustive();
		#[cfg(not(feature = "unstable"))]
		let res = f
			.debug_struct("Querier")
			.field("selector", &self.selector)
			.field("mode", &self.mode)
			.finish_non_exhaustive();
		res
	}
}

impl<P> Capability for Querier<P>
where
	P: Send + Sync + 'static,
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

impl<P> Querier<P>
where
	P: Send + Sync + 'static,
{
	/// Constructor for a [`Querier`]
	#[must_use]
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		selector: String,
		context: Context<P>,
		activation_state: OperationState,
		response_callback: ArcGetCallback<P>,
		mode: ConsolidationMode,
		#[cfg(feature = "unstable")] allowed_destination: Locality,
		encoding: String,
		target: QueryTarget,
		timeout: Duration,
	) -> Self {
		Self {
			selector,
			context,
			activation_state,
			callback: response_callback,
			mode,
			#[cfg(feature = "unstable")]
			allowed_destination,
			encoding,
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
	fn init(&mut self) -> Result<()>
	where
		P: Send + Sync + 'static,
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
	#[allow(clippy::unnecessary_wraps)]
	fn de_init(&mut self) -> Result<()>
	where
		P: Send + Sync + 'static,
	{
		self.key_expr.take();
		Ok(())
	}

	/// Run a Querier with an optional [`Message`].
	#[instrument(name="Querier", level = Level::ERROR, skip_all)]
	pub fn get(
		&self,
		message: Option<Message>,
		mut callback: Option<&dyn Fn(QueryableMsg) -> Result<()>>,
	) -> Result<()> {
		let cb = self.callback.clone();
		let session = self.context.session();
		let key_expr = self
			.key_expr
			.clone()
			.ok_or_else(|| Error::InvalidSelector("querier".into()))?;

		let builder = message
			.map_or_else(
				|| session.get(&key_expr),
				|msg| session.get(&self.selector).payload(msg.value()),
			)
			.encoding(self.encoding.as_str())
			.target(self.target)
			.consolidation(self.mode)
			.timeout(self.timeout);

		#[cfg(feature = "unstable")]
		let builder = builder.allowed_destination(self.allowed_destination);

		let query = builder
			.wait()
			.map_err(|source| Error::QueryCreation { source })?;

		let mut unreached = true;
		let mut retry_count = 0u8;

		while unreached && retry_count <= 5 {
			retry_count += 1;
			while let Ok(reply) = query.recv() {
				match reply.result() {
					Ok(sample) => match sample.kind() {
						SampleKind::Put => {
							let content: Vec<u8> = sample.payload().to_bytes().into_owned();
							let msg = QueryableMsg(content);
							if callback.is_none() {
								let cb = cb.clone();
								let ctx = self.context.clone();
								tokio::task::spawn(async move {
									let mut lock = cb.lock().await;
									if let Err(error) = lock(ctx, msg).await {
										error!("querier callback failed with {error}");
									}
								});
							} else {
								let callback =
									callback
										.as_mut()
										.ok_or_else(|| Error::AccessingQuerier {
											selector: key_expr.to_string(),
										})?;
								callback(msg).map_err(|source| Error::QueryCallback { source })?;
							}
						}
						SampleKind::Delete => {
							error!("Delete in Querier");
						}
					},
					Err(err) => error!("receive error: {:?})", err),
				}
				unreached = false;
			}
			if unreached {
				if retry_count < 5 {
					std::thread::sleep(self.timeout);
				} else {
					return Err(Error::AccessingQueryable {
						selector: key_expr.to_string(),
					}
					.into());
				}
			}
		}

		Ok(())
	}
}
// endregion:	--- Querier

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Querier<Props>>();
	}
}
