// Copyright © 2023 Stephan Kunz

//! Module `queryable` provides an information/compute provider `Queryable` which can be created using the `QueryableBuilder`.

// region:		--- modules
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	message_types::QueryMsg,
	traits::Context,
	utils::selector_from,
};
use std::sync::{Arc, Mutex, RwLock};
use zenoh::sample::Locality;

use crate::builder::{Callback, NoCallback, NoSelector, NoStorage, Selector, Storage};
use crate::com::{queryable::Queryable, ArcQueryableCallback};
// endregion:	--- modules

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
	selector: K,
	callback: C,
	storage: S,
}

impl<P> QueryableBuilder<P, NoSelector, NoCallback, NoStorage>
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
			selector: NoSelector,
			callback: NoCallback,
			storage: NoStorage,
		}
	}
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

impl<P, C, S> QueryableBuilder<P, NoSelector, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full expression for the [`Queryable`].
	#[must_use]
	pub fn selector(self, selector: &str) -> QueryableBuilder<P, Selector, C, S> {
		let Self {
			context,
			activation_state,
			completeness,
			allowed_origin,
			storage,
			callback,
			..
		} = self;
		QueryableBuilder {
			context,
			activation_state,
			completeness,
			allowed_origin,
			selector: Selector {
				selector: selector.into(),
			},
			callback,
			storage,
		}
	}

	/// Set only the topic of the [`Queryable`].
	/// Will be prefixed with agents prefix.
	#[must_use]
	pub fn topic(self, topic: &str) -> QueryableBuilder<P, Selector, C, S> {
		let selector = selector_from(topic, self.context.prefix());
		self.selector(&selector)
	}
}

impl<P, K, S> QueryableBuilder<P, K, NoCallback, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set callback for request messages
	#[must_use]
	pub fn callback<F>(
		self,
		callback: F,
	) -> QueryableBuilder<P, K, Callback<ArcQueryableCallback<P>>, S>
	where
		F: FnMut(&Context<P>, QueryMsg) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			context,
			activation_state,
			completeness,
			allowed_origin,
			selector,
			storage,
			..
		} = self;
		let callback: ArcQueryableCallback<P> = Arc::new(Mutex::new(callback));
		QueryableBuilder {
			context,
			activation_state,
			completeness,
			allowed_origin,
			selector,
			callback: Callback { callback },
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
	) -> QueryableBuilder<P, K, C, Storage<Queryable<P>>> {
		let Self {
			context,
			activation_state,
			completeness,
			allowed_origin,
			selector,
			callback,
			..
		} = self;
		QueryableBuilder {
			context,
			activation_state,
			completeness,
			allowed_origin,
			selector,
			callback,
			storage: Storage { storage },
		}
	}
}

impl<P, S> QueryableBuilder<P, Selector, Callback<ArcQueryableCallback<P>>, S>
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
			selector,
			callback,
			..
		} = self;
		let selector = selector.selector;
		Ok(Queryable::new(
			selector,
			context,
			activation_state,
			callback.callback,
			completeness,
			allowed_origin,
		))
	}
}

impl<P> QueryableBuilder<P, Selector, Callback<ArcQueryableCallback<P>>, Storage<Queryable<P>>>
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
			.insert(q.selector().to_string(), q);
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
		is_normal::<QueryableBuilder<Props, NoSelector, NoCallback, NoStorage>>();
	}
}
