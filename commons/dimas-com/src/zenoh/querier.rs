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
	Result,
	enums::OperationState,
	message_types::{Message, QueryableMsg},
	traits::{Capability, Context},
};
use futures::future::BoxFuture;
#[cfg(feature = "std")]
use std::{
	boxed::Box,
	string::{String, ToString},
	vec::Vec,
};
#[cfg(feature = "std")]
use tokio::sync::Mutex;
use tracing::{Level, error, instrument, warn};
#[cfg(feature = "unstable")]
use zenoh::sample::Locality;
use zenoh::{
	Session, Wait,
	query::{ConsolidationMode, QueryTarget},
	sample::SampleKind,
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
	/// the zenoh session this querier belongs to
	session: Arc<Session>,
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
	key_expr: std::sync::Mutex<Option<zenoh::key_expr::KeyExpr<'static>>>,
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

impl<P> crate::traits::Querier for Querier<P>
where
	P: Send + Sync + 'static,
{
	/// Get `selector`
	fn selector(&self) -> &str {
		&self.selector
	}

	/// Run a Querier with an optional [`Message`].
	#[instrument(name="Querier", level = Level::ERROR, skip_all)]
	fn get(
		&self,
		message: Option<Message>,
		mut callback: Option<&mut dyn FnMut(QueryableMsg) -> Result<()>>,
	) -> Result<()> {
		let cb = self.callback.clone();
		self.key_expr.lock().map_or_else(
			|_| todo!(),
			|key_expr| {
				let key_expr = key_expr
					.clone()
					.ok_or_else(|| Error::InvalidSelector("querier".into()))?;

				let builder = message
					.map_or_else(
						|| self.session.get(&key_expr),
						|msg| {
							self.session
								.get(&self.selector)
								.payload(msg.value())
						},
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
										let callback = callback.as_mut().ok_or_else(|| {
											Error::AccessingQuerier {
												selector: key_expr.to_string(),
											}
										})?;
										callback(msg)
											.map_err(|source| Error::QueryCallback { source })?;
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
			},
		)
	}
}

impl<P> Capability for Querier<P>
where
	P: Send + Sync + 'static,
{
	fn manage_operation_state(&self, state: &OperationState) -> Result<()> {
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
		session: Arc<Session>,
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
			session,
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
			key_expr: std::sync::Mutex::new(None),
		}
	}

	/// Initialize
	/// # Errors
	fn init(&self) -> Result<()>
	where
		P: Send + Sync + 'static,
	{
		self.de_init()?;

		self.key_expr.lock().map_or_else(
			|_| todo!(),
			|mut key_expr| {
				let new_key_expr = self
					.session
					.declare_keyexpr(self.selector.clone())
					.wait()?;
				key_expr.replace(new_key_expr);
				Ok(())
			},
		)
	}

	/// De-Initialize
	/// # Errors
	#[allow(clippy::unnecessary_wraps)]
	fn de_init(&self) -> Result<()>
	where
		P: Send + Sync + 'static,
	{
		self.key_expr.lock().map_or_else(
			|_| todo!(),
			|mut key_expr| {
				key_expr.take();
				Ok(())
			},
		)
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
