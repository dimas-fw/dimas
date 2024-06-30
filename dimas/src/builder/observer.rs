// Copyright © 2024 Stephan Kunz

// region:		--- modules
use crate::{
	builder::{Callback, NoCallback, NoSelector, NoStorage, Selector, Storage},
	com::{observer::Observer, ArcPutCallback},
};
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	message_types::Message,
	traits::Context,
	utils::selector_from,
};
use std::sync::{Arc, Mutex, RwLock};
// endregion:	--- modules

// region:		--- ObserverBuilder
/// The builder for an [`Observer`]
#[allow(clippy::module_name_repetitions)]
pub struct ObserverBuilder<P, K, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Context for the ObserverBuilder
	context: Context<P>,
	activation_state: OperationState,
	selector: K,
	callback: C,
	storage: S,
}

impl<P> ObserverBuilder<P, NoSelector, NoCallback, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct an `ObserverBuilder` in initial state
	#[must_use]
	pub const fn new(context: Context<P>) -> Self {
		Self {
			context,
			activation_state: OperationState::Standby,
			selector: NoSelector,
			callback: NoCallback,
			storage: NoStorage,
		}
	}
}

impl<P, K, C, S> ObserverBuilder<P, K, C, S>
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

impl<P, C, S> ObserverBuilder<P, NoSelector, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full key expression for the [`Observer`].
	#[must_use]
	pub fn selector(self, selector: &str) -> ObserverBuilder<P, Selector, C, S> {
		let Self {
			context,
			activation_state,
			storage,
			callback,
			..
		} = self;
		ObserverBuilder {
			context,
			activation_state,
			selector: Selector {
				selector: selector.into(),
			},
			callback,
			storage,
		}
	}

	/// Set only the message qualifing part of the [`Observer`].
	/// Will be prefixed with [`Agent`]s prefix.
	#[must_use]
	pub fn topic(self, topic: &str) -> ObserverBuilder<P, Selector, C, S> {
		let selector = selector_from(topic, self.context.prefix());
		self.selector(&selector)
	}
}

impl<P, K, S> ObserverBuilder<P, K, NoCallback, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set callback for messages
	#[must_use]
	pub fn callback<F>(self, callback: F) -> ObserverBuilder<P, K, Callback<ArcPutCallback<P>>, S>
	where
		F: FnMut(&Context<P>, Message) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			context,
			activation_state,
			selector,
			storage,
			..
		} = self;
		let callback: ArcPutCallback<P> = Arc::new(Mutex::new(callback));
		ObserverBuilder {
			context,
			activation_state,
			selector,
			callback: Callback { callback },
			storage,
		}
	}
}

impl<P, K, C> ObserverBuilder<P, K, C, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Provide agents storage for the subscriber
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Observer<P>>>>,
	) -> ObserverBuilder<P, K, C, Storage<Observer<P>>> {
		let Self {
			context,
			activation_state,
			selector,
			callback,
			..
		} = self;
		ObserverBuilder {
			context,
			activation_state,
			selector,
			callback,
			storage: Storage { storage },
		}
	}
}

impl<P, S> ObserverBuilder<P, Selector, Callback<ArcPutCallback<P>>, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the [`Subscriber`].
	///
	/// # Errors
	/// Currently none
	pub fn build(self) -> Result<Observer<P>> {
		let Self {
			context,
			selector,
			activation_state,
			callback,
			..
		} = self;
		let selector = selector.selector;
		Ok(Observer::new(
			selector,
			context,
			activation_state,
			callback.callback,
		))
	}
}

impl<P> ObserverBuilder<P, Selector, Callback<ArcPutCallback<P>>, Storage<Observer<P>>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the [`Observer`] to the [`Agent`].
	///
	/// # Errors
	/// Currently none
	pub fn add(self) -> Result<Option<Observer<P>>> {
		let c = self.storage.storage.clone();
		let s = self.build()?;

		let r = c
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(s.selector().to_string(), s);
		Ok(r)
	}
}
// endregion:	--- ObserverBuilder

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<ObserverBuilder<Props, NoSelector, NoCallback, NoStorage>>();
	}
}
