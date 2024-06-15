// Copyright © 2024 Stephan Kunz

// region:		--- modules
use crate::{
	builder::{Callback, NoCallback, NoSelector, NoStorage, Selector, Storage},
	com::{observer::Observer, ArcFeedbackCallback, ArcResultCallback},
};
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	message_types::FeedbackMsg,
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
	result_callback: C,
	storage: S,
	feedback_callback: Option<ArcFeedbackCallback<P>>,
}

impl<P> ObserverBuilder<P, NoSelector, NoCallback, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a `SubscriberBuilder` in initial state
	#[must_use]
	pub const fn new(context: Context<P>) -> Self {
		Self {
			context,
			activation_state: OperationState::Standby,
			selector: NoSelector,
			result_callback: NoCallback,
			storage: NoStorage,
			feedback_callback: None,
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

	/// Set observers callback for `feedback` messages
	#[must_use]
	pub fn monitor<F>(mut self, callback: F) -> Self
	where
		F: Fn(&Context<P>, FeedbackMsg) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		self.feedback_callback
			.replace(Arc::new(Mutex::new(callback)));
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
			result_callback,
			feedback_callback,
			..
		} = self;
		ObserverBuilder {
			context,
			activation_state,
			selector: Selector {
				selector: selector.into(),
			},
			result_callback,
			storage,
			feedback_callback,
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
	/// Set callback for result messages
	#[must_use]
	pub fn callback<F>(
		self,
		callback: F,
	) -> ObserverBuilder<P, K, Callback<ArcResultCallback<P>>, S>
	where
		F: Fn(&Context<P>, dimas_core::message_types::ResultMsg) -> Result<()>
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
			feedback_callback,
			..
		} = self;
		let callback: ArcResultCallback<P> = Arc::new(Mutex::new(callback));
		ObserverBuilder {
			context,
			activation_state,
			selector,
			result_callback: Callback { callback },
			storage,
			feedback_callback,
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
			result_callback,
			feedback_callback,
			..
		} = self;
		ObserverBuilder {
			context,
			activation_state,
			selector,
			result_callback,
			storage: Storage { storage },
			feedback_callback,
		}
	}
}

impl<P, S> ObserverBuilder<P, Selector, Callback<ArcResultCallback<P>>, S>
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
			result_callback,
			feedback_callback,
			..
		} = self;
		Ok(Observer::new(
			selector.selector,
			context,
			activation_state,
			result_callback.callback,
			feedback_callback,
		))
	}
}

impl<P> ObserverBuilder<P, Selector, Callback<ArcResultCallback<P>>, Storage<Observer<P>>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the [`Subscriber`] to the [`Agent`].
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
