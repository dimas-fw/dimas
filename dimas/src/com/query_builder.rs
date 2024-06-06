// Copyright © 2023 Stephan Kunz

//! Module `query` provides an information/compute requestor `Query` which can be created using the `QueryBuilder`.

// region:		--- modules
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	message_types::Response,
	traits::Context,
	utils::selector_from,
};
use std::{
	collections::HashMap,
	sync::{Arc, Mutex, RwLock},
	time::Duration,
};
use zenoh::{
	query::{ConsolidationMode, QueryTarget},
	sample::Locality,
};

use super::query::{Query, QueryCallback};
// endregion:	--- modules

// region:		--- states
/// State signaling that the [`QueryBuilder`] has no storage value set
pub struct NoStorage;
/// State signaling that the [`QueryBuilder`] has the storage value set
pub struct Storage<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Thread safe reference to a [`HashMap`] to store the created [`Query`]
	pub storage: Arc<RwLock<HashMap<String, Query<P>>>>,
}

/// State signaling that the [`QueryBuilder`] has no selector set
pub struct NoSelector;
/// State signaling that the [`QueryBuilder`] has the selector set
pub struct Selector {
	/// The selector
	selector: String,
}

/// State signaling that the [`QueryBuilder`] has no response callback set
pub struct NoResponseCallback;
/// State signaling that the [`QueryBuilder`] has the response callback set
pub struct ResponseCallback<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Response callback for the [`Query`]
	pub response: QueryCallback<P>,
}
// endregion:	--- states

// region:		--- QueryBuilder
/// The builder for a query
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct QueryBuilder<P, K, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	context: Context<P>,
	activation_state: OperationState,
	allowed_destination: Locality,
	timeout: Option<Duration>,
	selector: K,
	response_callback: C,
	storage: S,
	mode: ConsolidationMode,
	target: QueryTarget,
}

impl<P> QueryBuilder<P, NoSelector, NoResponseCallback, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a `QueryBuilder` in initial state
	#[must_use]
	pub const fn new(context: Context<P>) -> Self {
		Self {
			context,
			activation_state: OperationState::Standby,
			allowed_destination: Locality::Any,
			timeout: None,
			selector: NoSelector,
			response_callback: NoResponseCallback,
			storage: NoStorage,
			mode: ConsolidationMode::None,
			target: QueryTarget::All,
		}
	}
}

impl<P, K, C, S> QueryBuilder<P, K, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the activation state.
	#[must_use]
	pub const fn activation_state(mut self, state: OperationState) -> Self {
		self.activation_state = state;
		self
	}

	/// Set the [`ConsolidationMode`] of the [`Query`].
	#[must_use]
	pub const fn mode(mut self, mode: ConsolidationMode) -> Self {
		self.mode = mode;
		self
	}

	/// Set the [`QueryTarget`] of the [`Query`].
	#[must_use]
	pub const fn target(mut self, target: QueryTarget) -> Self {
		self.target = target;
		self
	}

	/// Set the allowed destination of the [`Query`].
	#[must_use]
	pub const fn allowed_destination(mut self, allowed_destination: Locality) -> Self {
		self.allowed_destination = allowed_destination;
		self
	}

	/// Set a timeout for the [`Query`].
	#[must_use]
	pub const fn timeout(mut self, timeout: Option<Duration>) -> Self {
		self.timeout = timeout;
		self
	}
}

impl<P, C, S> QueryBuilder<P, NoSelector, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full expression for the query
	#[must_use]
	pub fn selector(self, selector: &str) -> QueryBuilder<P, Selector, C, S> {
		let Self {
			context,
			activation_state,
			allowed_destination,
			timeout,
			storage,
			response_callback: callback,
			mode,
			target,
			..
		} = self;
		QueryBuilder {
			context,
			activation_state,
			allowed_destination,
			timeout,
			selector: Selector {
				selector: selector.into(),
			},
			response_callback: callback,
			storage,
			mode,
			target,
		}
	}

	/// Set only the message qualifing part of the query.
	/// Will be prefixed with agents prefix.
	#[must_use]
	pub fn topic(self, topic: &str) -> QueryBuilder<P, Selector, C, S> {
		let selector = selector_from(topic, self.context.prefix());
		self.selector(&selector)
	}
}

impl<P, K, S> QueryBuilder<P, K, NoResponseCallback, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set query callback for response messages
	#[must_use]
	pub fn callback<F>(self, callback: F) -> QueryBuilder<P, K, ResponseCallback<P>, S>
	where
		F: FnMut(&Context<P>, Response) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			context,
			activation_state,
			allowed_destination,
			timeout,
			selector,
			storage,
			mode,
			target,
			..
		} = self;
		let callback: QueryCallback<P> = Arc::new(Mutex::new(callback));
		QueryBuilder {
			context,
			activation_state,
			allowed_destination,
			timeout,
			selector,
			response_callback: ResponseCallback { response: callback },
			storage,
			mode,
			target,
		}
	}
}

impl<P, K, C> QueryBuilder<P, K, C, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Provide agents storage for the query
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Query<P>>>>,
	) -> QueryBuilder<P, K, C, Storage<P>> {
		let Self {
			context,
			activation_state,
			allowed_destination,
			timeout,
			selector,
			response_callback: callback,
			mode,
			target,
			..
		} = self;
		QueryBuilder {
			context,
			activation_state,
			allowed_destination,
			timeout,
			selector,
			response_callback: callback,
			storage: Storage { storage },
			mode,
			target,
		}
	}
}

impl<P, S> QueryBuilder<P, Selector, ResponseCallback<P>, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the [`Query`]
	/// # Errors
	///
	pub fn build(self) -> Result<Query<P>> {
		let Self {
			context,
			activation_state,
			allowed_destination,
			timeout,
			selector,
			response_callback,
			mode,
			target,
			..
		} = self;
		let selector = selector.selector;
		Ok(Query::new(
			selector,
			context,
			activation_state,
			response_callback.response,
			mode,
			allowed_destination,
			target,
			timeout,
		))
	}
}

impl<P> QueryBuilder<P, Selector, ResponseCallback<P>, Storage<P>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the query to the agents context
	/// # Errors
	pub fn add(self) -> Result<Option<Query<P>>> {
		let collection = self.storage.storage.clone();
		let q = self.build()?;

		let r = collection
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(q.selector().to_string(), q);
		Ok(r)
	}
}
// endregion:	--- QueryBuilder

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<QueryBuilder<Props, NoSelector, NoResponseCallback, NoStorage>>();
	}
}
