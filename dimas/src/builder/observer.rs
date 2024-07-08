// Copyright © 2024 Stephan Kunz

// region:		--- modules
use crate::{
	builder::{Callback, NoCallback, NoSelector, NoStorage, Selector, Storage},
	com::{
		observer::Observer, ArcObserverControlCallback, ArcObserverFeedbackCallback,
		ArcObserverResultCallback,
	},
};
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	message_types::{ControlResponse, Message, ResultResponse},
	traits::Context,
	utils::selector_from,
};
use std::sync::{Arc, Mutex, RwLock};
// endregion:	--- modules

// region:		--- ObserverBuilder
/// The builder for an [`Observer`]
#[allow(clippy::module_name_repetitions)]
pub struct ObserverBuilder<P, K, CC, FC, RC, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Context for the ObserverBuilder
	context: Context<P>,
	activation_state: OperationState,
	selector: K,
	/// callback for observer request and cancelation
	control_callback: CC,
	/// callback for observer feedback
	feedback_callback: FC,
	/// callback for observer result
	result_callback: RC,
	storage: S,
}

impl<P> ObserverBuilder<P, NoSelector, NoCallback, NoCallback, NoCallback, NoStorage>
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
			control_callback: NoCallback,
			feedback_callback: NoCallback,
			result_callback: NoCallback,
			storage: NoStorage,
		}
	}
}

impl<P, K, CC, FC, RC, S> ObserverBuilder<P, K, CC, FC, RC, S>
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

impl<P, CC, FC, RC, S> ObserverBuilder<P, NoSelector, CC, FC, RC, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full key expression for the [`Observer`].
	#[must_use]
	pub fn selector(self, selector: &str) -> ObserverBuilder<P, Selector, CC, FC, RC, S> {
		let Self {
			context,
			activation_state,
			control_callback,
			feedback_callback,
			result_callback,
			storage,
			..
		} = self;
		ObserverBuilder {
			context,
			activation_state,
			selector: Selector {
				selector: selector.into(),
			},
			control_callback,
			feedback_callback,
			result_callback,
			storage,
		}
	}

	/// Set only the message qualifing part of the [`Observer`].
	/// Will be prefixed with [`Agent`]s prefix.
	#[must_use]
	pub fn topic(self, topic: &str) -> ObserverBuilder<P, Selector, CC, FC, RC, S> {
		let selector = selector_from(topic, self.context.prefix());
		self.selector(&selector)
	}
}

impl<P, K, FC, RC, S> ObserverBuilder<P, K, NoCallback, FC, RC, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set callback for messages
	#[must_use]
	pub fn control_callback<F>(
		self,
		callback: F,
	) -> ObserverBuilder<P, K, Callback<ArcObserverControlCallback<P>>, FC, RC, S>
	where
		F: FnMut(&Context<P>, ControlResponse) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			context,
			activation_state,
			selector,
			feedback_callback,
			result_callback,
			storage,
			..
		} = self;
		let callback: ArcObserverControlCallback<P> = Arc::new(Mutex::new(callback));
		ObserverBuilder {
			context,
			activation_state,
			selector,
			control_callback: Callback { callback },
			feedback_callback,
			result_callback,
			storage,
		}
	}
}

impl<P, K, CC, RC, S> ObserverBuilder<P, K, CC, NoCallback, RC, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set callback for feedback
	#[must_use]
	pub fn feedback_callback<F>(
		self,
		callback: F,
	) -> ObserverBuilder<P, K, CC, Callback<ArcObserverFeedbackCallback<P>>, RC, S>
	where
		F: FnMut(&Context<P>, Message) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			context,
			activation_state,
			selector,
			control_callback,
			result_callback,
			storage,
			..
		} = self;
		let callback: ArcObserverFeedbackCallback<P> = Arc::new(Mutex::new(callback));
		ObserverBuilder {
			context,
			activation_state,
			selector,
			control_callback,
			feedback_callback: Callback { callback },
			result_callback,
			storage,
		}
	}
}

impl<P, K, CC, FC, S> ObserverBuilder<P, K, CC, FC, NoCallback, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set callback for result messages
	#[must_use]
	pub fn result_callback<F>(
		self,
		callback: F,
	) -> ObserverBuilder<P, K, CC, FC, Callback<ArcObserverResultCallback<P>>, S>
	where
		F: FnMut(&Context<P>, ResultResponse) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			context,
			activation_state,
			selector,
			control_callback,
			feedback_callback,
			storage,
			..
		} = self;
		let callback: ArcObserverResultCallback<P> = Arc::new(Mutex::new(callback));
		ObserverBuilder {
			context,
			activation_state,
			selector,
			control_callback,
			feedback_callback,
			result_callback: Callback { callback },
			storage,
		}
	}
}

impl<P, K, CC, FC, RC> ObserverBuilder<P, K, CC, FC, RC, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Provide agents storage for the subscriber
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Observer<P>>>>,
	) -> ObserverBuilder<P, K, CC, FC, RC, Storage<Observer<P>>> {
		let Self {
			context,
			activation_state,
			selector,
			control_callback,
			feedback_callback,
			result_callback,
			..
		} = self;
		ObserverBuilder {
			context,
			activation_state,
			selector,
			control_callback,
			feedback_callback,
			result_callback,
			storage: Storage { storage },
		}
	}
}

impl<P, S>
	ObserverBuilder<
		P,
		Selector,
		Callback<ArcObserverControlCallback<P>>,
		Callback<ArcObserverFeedbackCallback<P>>,
		Callback<ArcObserverResultCallback<P>>,
		S,
	> where
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
			control_callback,
			feedback_callback,
			result_callback,
			..
		} = self;
		let selector = selector.selector;
		Ok(Observer::new(
			selector,
			context,
			activation_state,
			control_callback.callback,
			feedback_callback.callback,
			result_callback.callback,
		))
	}
}

impl<P>
	ObserverBuilder<
		P,
		Selector,
		Callback<ArcObserverControlCallback<P>>,
		Callback<ArcObserverFeedbackCallback<P>>,
		Callback<ArcObserverResultCallback<P>>,
		Storage<Observer<P>>,
	> where
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
		is_normal::<
			ObserverBuilder<Props, NoSelector, NoCallback, NoCallback, NoCallback, NoStorage>,
		>();
	}
}
