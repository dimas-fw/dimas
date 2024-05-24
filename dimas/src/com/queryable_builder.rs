// Copyright © 2023 Stephan Kunz

//! Module `queryable` provides an information/compute provider `Queryable` which can be created using the `QueryableBuilder`.

// region:		--- modules
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	message_types::Request,
	traits::Context,
};
use std::{
	collections::HashMap,
	sync::{Arc, Mutex, RwLock},
};
use zenoh::sample::Locality;

use super::queryable::{Queryable, QueryableCallback};
// endregion:	--- modules

// region:		--- states
/// State signaling that the [`QueryableBuilder`] has no storage value set
pub struct NoStorage;
/// State signaling that the [`QueryableBuilder`] has the storage value set
pub struct Storage<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Thread safe reference to a [`HashMap`] to store the created [`Queryable`]
	pub storage: Arc<RwLock<HashMap<String, Queryable<P>>>>,
}

/// State signaling that the [`QueryableBuilder`] has no key expression set
pub struct NoKeyExpression;
/// State signaling that the [`QueryableBuilder`] has the key expression set
pub struct KeyExpression {
	/// The key expression
	key_expr: String,
}

/// State signaling that the [`QueryableBuilder`] has no request callback set
pub struct NoRequestCallback;
/// State signaling that the [`QueryableBuilder`] has the request callback set
pub struct RequestCallback<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Request callback for the [`Queryable`]
	pub request: QueryableCallback<P>,
}
// endregion:   --- states

// region:		--- QueryableBuilder
/// The builder for a queryable.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct QueryableBuilder<P, K, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	context: Context<P>,
	activation_state: OperationState,
	completeness: bool,
	allowed_origin: Locality,
	key_expr: K,
	request_callback: C,
	storage: S,
}

impl<P, K, C, S> QueryableBuilder<P, K, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the activation state.
	#[must_use]
	pub const fn activation_state(mut self, state: OperationState) -> Self {
		self.activation_state = state;
		self
	}

	/// Set the completeness of the [`Queryable`].
	#[must_use]
	pub const fn completeness(mut self, completeness: bool) -> Self {
		self.completeness = completeness;
		self
	}

	/// Set the allowed origin of the [`Queryable`].
	#[must_use]
	pub const fn allowed_origin(mut self, allowed_origin: Locality) -> Self {
		self.allowed_origin = allowed_origin;
		self
	}
}

impl<P> QueryableBuilder<P, NoKeyExpression, NoRequestCallback, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a `QueryableBuilder` in initial state
	#[must_use]
	pub const fn new(context: Context<P>) -> Self {
		Self {
			context,
			activation_state: OperationState::Standby,
			completeness: true,
			allowed_origin: Locality::Any,
			key_expr: NoKeyExpression,
			request_callback: NoRequestCallback,
			storage: NoStorage,
		}
	}
}

impl<P, C, S> QueryableBuilder<P, NoKeyExpression, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full expression for the [`Queryable`].
	#[must_use]
	pub fn key_expr(self, key_expr: &str) -> QueryableBuilder<P, KeyExpression, C, S> {
		let Self {
			context,
			activation_state,
			completeness,
			allowed_origin,
			storage,
			request_callback: callback,
			..
		} = self;
		QueryableBuilder {
			context,
			activation_state,
			completeness,
			allowed_origin,
			key_expr: KeyExpression {
				key_expr: key_expr.into(),
			},
			request_callback: callback,
			storage,
		}
	}

	/// Set only the topic of the [`Queryable`].
	/// Will be prefixed with agents prefix.
	#[must_use]
	pub fn topic(self, topic: &str) -> QueryableBuilder<P, KeyExpression, C, S> {
		let key_expr = self
			.context
			.prefix()
			.clone()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		let Self {
			context,
			activation_state,
			completeness,
			allowed_origin,
			storage,
			request_callback: callback,
			..
		} = self;
		QueryableBuilder {
			context,
			activation_state,
			completeness,
			allowed_origin,
			key_expr: KeyExpression { key_expr },
			request_callback: callback,
			storage,
		}
	}
}

impl<P, K, S> QueryableBuilder<P, K, NoRequestCallback, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set callback for request messages
	#[must_use]
	pub fn callback<F>(self, callback: F) -> QueryableBuilder<P, K, RequestCallback<P>, S>
	where
		F: FnMut(&Context<P>, Request) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			context,
			activation_state,
			completeness,
			allowed_origin,
			key_expr,
			storage,
			..
		} = self;
		let request: QueryableCallback<P> = Arc::new(Mutex::new(Box::new(callback)));
		QueryableBuilder {
			context,
			activation_state,
			completeness,
			allowed_origin,
			key_expr,
			request_callback: RequestCallback { request },
			storage,
		}
	}
}

impl<P, K, C> QueryableBuilder<P, K, C, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Provide agents storage for the queryable
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Queryable<P>>>>,
	) -> QueryableBuilder<P, K, C, Storage<P>> {
		let Self {
			context,
			activation_state,
			completeness,
			allowed_origin,
			key_expr,
			request_callback: callback,
			..
		} = self;
		QueryableBuilder {
			context,
			activation_state,
			completeness,
			allowed_origin,
			key_expr,
			request_callback: callback,
			storage: Storage { storage },
		}
	}
}

impl<P, S> QueryableBuilder<P, KeyExpression, RequestCallback<P>, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the [`Queryable`]
	/// # Errors
	///
	pub fn build(self) -> Result<Queryable<P>> {
		let Self {
			context,
			activation_state,
			completeness,
			allowed_origin,
			key_expr,
			request_callback,
			..
		} = self;
		let key_expr = key_expr.key_expr;
		Ok(Queryable::new(
			key_expr,
			context,
			activation_state,
			request_callback.request,
			completeness,
			allowed_origin,
		))
	}
}

impl<P> QueryableBuilder<P, KeyExpression, RequestCallback<P>, Storage<P>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the queryable to the agents context
	/// # Errors
	///
	pub fn add(self) -> Result<Option<Queryable<P>>> {
		let collection = self.storage.storage.clone();
		let q = self.build()?;

		let r = collection
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(q.key_expr().to_string(), q);
		Ok(r)
	}
}
// endregion:	--- QueryableBuilder

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<QueryableBuilder<Props, NoKeyExpression, NoRequestCallback, NoStorage>>();
	}
}
