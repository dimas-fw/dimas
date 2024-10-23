// Copyright © 2023 Stephan Kunz

//! Module `query` provides an information/compute requestor `Query` which can be created using the `QuerierBuilder`.

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use dimas_core::builder_states::{Callback, NoCallback, NoSelector, NoStorage, Selector, Storage};
use crate::error::Error;
use core::time::Duration;
use crate::traits::Querier as QuerierTrait;
use crate::zenoh::querier::Querier;
use dimas_core::{
	enums::OperationState, message_types::QueryableMsg, traits::Context, utils::selector_from,
	Result,
};
use std::{
	future::Future,
	sync::{Arc, RwLock},
};
use std::boxed::Box;
use std::string::{String, ToString};
use tokio::sync::Mutex;
#[cfg(feature = "unstable")]
use zenoh::sample::Locality;
use zenoh::{
	bytes::Encoding,
	query::{ConsolidationMode, QueryTarget},
};

use crate::zenoh::querier::{ArcGetCallback, GetCallback};
// endregion:	--- modules

// region:		--- QuerierBuilder
/// The builder for a query
#[derive(Clone)]
pub struct QuerierBuilder<P, K, C, S>
where
	P: Send + Sync + 'static,
{
	context: Context<P>,
	activation_state: OperationState,
	#[cfg(feature = "unstable")]
	allowed_destination: Locality,
	encoding: String,
	timeout: Duration,
	selector: K,
	callback: C,
	storage: S,
	mode: ConsolidationMode,
	target: QueryTarget,
}

impl<P> QuerierBuilder<P, NoSelector, NoCallback, NoStorage>
where
	P: Send + Sync + 'static,
{
	/// Construct a `QuerierBuilder` in initial state
	#[must_use]
	pub fn new(context: Context<P>) -> Self {
		Self {
			context,
			activation_state: OperationState::Active,
			#[cfg(feature = "unstable")]
			allowed_destination: Locality::Any,
			encoding: Encoding::default().to_string(),
			timeout: Duration::from_millis(1000),
			selector: NoSelector,
			callback: NoCallback,
			storage: NoStorage,
			mode: ConsolidationMode::None,
			target: QueryTarget::All,
		}
	}
}

impl<P, K, C, S> QuerierBuilder<P, K, C, S>
where
	P: Send + Sync + 'static,
{
	/// Set the activation state.
	#[must_use]
	pub const fn activation_state(mut self, state: OperationState) -> Self {
		self.activation_state = state;
		self
	}

	/// Set the [`ConsolidationMode`] of the [`Querier`].
	#[must_use]
	pub const fn mode(mut self, mode: ConsolidationMode) -> Self {
		self.mode = mode;
		self
	}

	/// Set the [`QueryTarget`] of the [`Querier`].
	#[must_use]
	pub const fn target(mut self, target: QueryTarget) -> Self {
		self.target = target;
		self
	}

	/// Set the allowed destination of the [`Querier`].
	#[cfg(feature = "unstable")]
	#[must_use]
	pub const fn allowed_destination(mut self, allowed_destination: Locality) -> Self {
		self.allowed_destination = allowed_destination;
		self
	}

	/// Set the [`Querier`]s encoding
	#[must_use]
	pub fn encoding(mut self, encoding: String) -> Self {
		self.encoding = encoding;
		self
	}

	/// Set a timeout for the [`Querier`].
	/// Default is 1000ms
	#[must_use]
	pub const fn timeout(mut self, timeout: Duration) -> Self {
		self.timeout = timeout;
		self
	}
}

impl<P, C, S> QuerierBuilder<P, NoSelector, C, S>
where
	P: Send + Sync + 'static,
{
	/// Set the full expression for the query
	#[must_use]
	pub fn selector(self, selector: &str) -> QuerierBuilder<P, Selector, C, S> {
		let Self {
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			encoding,
			timeout,
			storage,
			callback,
			mode,
			target,
			..
		} = self;
		QuerierBuilder {
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			encoding,
			timeout,
			selector: Selector {
				selector: selector.into(),
			},
			callback,
			storage,
			mode,
			target,
		}
	}

	/// Set only the message qualifing part of the query.
	/// Will be prefixed with agents prefix.
	#[must_use]
	pub fn topic(self, topic: &str) -> QuerierBuilder<P, Selector, C, S> {
		let selector = selector_from(topic, self.context.prefix());
		self.selector(&selector)
	}
}

impl<P, K, S> QuerierBuilder<P, K, NoCallback, S>
where
	P: Send + Sync + 'static,
{
	/// Set query callback for response messages
	#[must_use]
	pub fn callback<C, F>(
		self,
		mut callback: C,
	) -> QuerierBuilder<P, K, Callback<ArcGetCallback<P>>, S>
	where
		C: FnMut(Context<P>, QueryableMsg) -> F + Send + Sync + 'static,
		F: Future<Output = Result<()>> + Send + Sync + 'static,
	{
		let Self {
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			encoding,
			timeout,
			selector,
			storage,
			mode,
			target,
			..
		} = self;
		let callback: GetCallback<P> = Box::new(move |ctx, msg| Box::pin(callback(ctx, msg)));
		let callback: ArcGetCallback<P> = Arc::new(Mutex::new(callback));
		QuerierBuilder {
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			encoding,
			timeout,
			selector,
			callback: Callback { callback },
			storage,
			mode,
			target,
		}
	}
}

impl<P, K, C> QuerierBuilder<P, K, C, NoStorage>
where
	P: Send + Sync + 'static,
{
	/// Provide agents storage for the query
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Box<dyn QuerierTrait>>>>,
	) -> QuerierBuilder<P, K, C, Storage<Box<dyn QuerierTrait>>> {
		let Self {
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			encoding,
			timeout,
			selector,
			callback,
			mode,
			target,
			..
		} = self;
		QuerierBuilder {
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			encoding,
			timeout,
			selector,
			callback,
			storage: Storage { storage },
			mode,
			target,
		}
	}
}

impl<P, S> QuerierBuilder<P, Selector, Callback<ArcGetCallback<P>>, S>
where
	P: Send + Sync + 'static,
{
	/// Build the [`Querier`]
	/// # Errors
	///
	pub fn build(self) -> Result<Querier<P>> {
		let Self {
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			encoding,
			timeout,
			selector,
			callback: response,
			mode,
			target,
			..
		} = self;
		let selector = selector.selector;
		Ok(Querier::new(
			selector,
			context,
			activation_state,
			response.callback,
			mode,
			#[cfg(feature = "unstable")]
			allowed_destination,
			encoding,
			target,
			timeout,
		))
	}
}

impl<P> QuerierBuilder<P, Selector, Callback<ArcGetCallback<P>>, Storage<Box<dyn QuerierTrait>>>
where
	P: Send + Sync + 'static,
{
	/// Build and add the query to the agents context
	/// # Errors
	pub fn add(self) -> Result<Option<Box<dyn QuerierTrait>>> {
		let collection = self.storage.storage.clone();
		let q = self.build()?;

		let r = collection
			.write()
			.map_err(|_| Error::MutexPoison(String::from("QuerierBuilder")))?
			.insert(q.selector().to_string(), Box::new(q));
		Ok(r)
	}
}
// endregion:	--- QuerierBuilder

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<QuerierBuilder<Props, NoSelector, NoCallback, NoStorage>>();
	}
}
