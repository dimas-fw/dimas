// Copyright © 2024 Stephan Kunz

// region:		--- modules
use crate::{
	builder::{Callback, NoCallback, NoSelector, NoStorage, Selector, Storage},
	com::{observable::Observable, ArcControlCallback},
};
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	message_types::{Message, ObservableResponse},
	traits::Context,
	utils::selector_from,
};
use std::sync::{Arc, Mutex, RwLock};
// endregion:	--- modules

// region:		--- ObservableBuilder
/// The builder for an [`Observable`]
#[allow(clippy::module_name_repetitions)]
pub struct ObservableBuilder<P, K, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Context for the ObservableBuilder
	context: Context<P>,
	activation_state: OperationState,
	selector: K,
	request_callback: C,
	storage: S,
}

impl<P> ObservableBuilder<P, NoSelector, NoCallback, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a `QueryBuilder` in initial state
	#[must_use]
	pub const fn new(context: Context<P>) -> Self {
		Self {
			context,
			activation_state: OperationState::Standby,
			selector: NoSelector,
			request_callback: NoCallback,
			storage: NoStorage,
		}
	}
}

impl<P, K, C, S> ObservableBuilder<P, K, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the activation state.
	#[must_use]
	pub const fn activation_state(mut self, state: OperationState) -> Self {
		self.activation_state = state;
		self
	}
}

impl<P, C, S> ObservableBuilder<P, NoSelector, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full expression for the [`Observable`].
	#[must_use]
	pub fn selector(self, selector: &str) -> ObservableBuilder<P, Selector, C, S> {
		let Self {
			context,
			activation_state,
			storage,
			request_callback: callback,
			..
		} = self;
		ObservableBuilder {
			context,
			activation_state,
			selector: Selector {
				selector: selector.into(),
			},
			request_callback: callback,
			storage,
		}
	}

	/// Set only the topic of the [`Observable`].
	/// Will be prefixed with agents prefix.
	#[must_use]
	pub fn topic(self, topic: &str) -> ObservableBuilder<P, Selector, C, S> {
		let selector = selector_from(topic, self.context.prefix());
		self.selector(&selector)
	}
}

impl<P, K, S> ObservableBuilder<P, K, NoCallback, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set callback for request messages
	#[must_use]
	pub fn callback<F>(
		self,
		callback: F,
	) -> ObservableBuilder<P, K, Callback<ArcControlCallback<P>>, S>
	where
		F: FnMut(&Context<P>, Message) -> Result<ObservableResponse>
			+ Send
			+ Sync
			+ Unpin
			+ 'static,
	{
		let Self {
			context,
			activation_state,
			selector,
			storage,
			..
		} = self;
		let callback: ArcControlCallback<P> = Arc::new(Mutex::new(callback));
		ObservableBuilder {
			context,
			activation_state,
			selector,
			request_callback: Callback { callback },
			storage,
		}
	}
}

impl<P, K, C> ObservableBuilder<P, K, C, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Provide agents storage for the queryable
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Observable<P>>>>,
	) -> ObservableBuilder<P, K, C, Storage<Observable<P>>> {
		let Self {
			context,
			activation_state,
			selector,
			request_callback: callback,
			..
		} = self;
		ObservableBuilder {
			context,
			activation_state,
			selector,
			request_callback: callback,
			storage: Storage { storage },
		}
	}
}

impl<P, S> ObservableBuilder<P, Selector, Callback<ArcControlCallback<P>>, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the [`Observable`]
	/// # Errors
	///
	pub fn build(self) -> Result<Observable<P>> {
		let Self {
			context,
			activation_state,
			selector,
			request_callback,
			..
		} = self;
		let selector = selector.selector;
		Ok(Observable::new(
			selector,
			context,
			activation_state,
			request_callback.callback,
		))
	}
}

impl<P> ObservableBuilder<P, Selector, Callback<ArcControlCallback<P>>, Storage<Observable<P>>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the queryable to the agents context
	/// # Errors
	///
	pub fn add(self) -> Result<Option<Observable<P>>> {
		let collection = self.storage.storage.clone();
		let q = self.build()?;

		let r = collection
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(q.selector().to_string(), q);
		Ok(r)
	}
}
// endregion:	--- ObservableBuilder

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<ObservableBuilder<Props, NoSelector, NoCallback, NoStorage>>();
	}
}
