// Copyright © 2024 Stephan Kunz

// region:		--- modules
use super::{observer::Observer, ArcObserverControlCallback, ArcObserverResponseCallback};
#[cfg(doc)]
use crate::agent::Agent;
use crate::{Callback, NoCallback, NoSelector, NoStorage, Selector, Storage};
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	message_types::{ControlResponse, ObservableResponse},
	traits::Context,
	utils::selector_from,
};
use std::sync::{Arc, Mutex, RwLock};
// endregion:	--- modules

// region:		--- ObserverBuilder
/// The builder for an [`Observer`]
#[allow(clippy::module_name_repetitions)]
pub struct ObserverBuilder<P, K, CC, RC, S>
where
	P: Send + Sync + 'static,
{
	/// Context for the `ObserverBuilder`
	context: Context<P>,
	activation_state: OperationState,
	selector: K,
	/// callback for observer request and cancelation
	control_callback: CC,
	/// callback for observer result
	response_callback: RC,
	storage: S,
}

impl<P> ObserverBuilder<P, NoSelector, NoCallback, NoCallback, NoStorage>
where
	P: Send + Sync + 'static,
{
	/// Construct an `ObserverBuilder` in initial state
	#[must_use]
	pub const fn new(context: Context<P>) -> Self {
		Self {
			context,
			activation_state: OperationState::Standby,
			selector: NoSelector,
			control_callback: NoCallback,
			response_callback: NoCallback,
			storage: NoStorage,
		}
	}
}

impl<P, K, CC, RC, S> ObserverBuilder<P, K, CC, RC, S>
where
	P: Send + Sync + 'static,
{
	/// Set the activation state.
	#[must_use]
	pub const fn activation_state(mut self, state: OperationState) -> Self {
		self.activation_state = state;
		self
	}
}

impl<P, CC, RC, S> ObserverBuilder<P, NoSelector, CC, RC, S>
where
	P: Send + Sync + 'static,
{
	/// Set the full key expression for the [`Observer`].
	#[must_use]
	pub fn selector(self, selector: &str) -> ObserverBuilder<P, Selector, CC, RC, S> {
		let Self {
			context,
			activation_state,
			control_callback,
			response_callback,
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
			response_callback,
			storage,
		}
	}

	/// Set only the message qualifing part of the [`Observer`].
	/// Will be prefixed with [`Agent`]s prefix.
	#[must_use]
	pub fn topic(self, topic: &str) -> ObserverBuilder<P, Selector, CC, RC, S> {
		let selector = selector_from(topic, self.context.prefix());
		self.selector(&selector)
	}
}

impl<P, K, RC, S> ObserverBuilder<P, K, NoCallback, RC, S>
where
	P: Send + Sync + 'static,
{
	/// Set callback for messages
	#[must_use]
	pub fn control_callback<F>(
		self,
		callback: F,
	) -> ObserverBuilder<P, K, Callback<ArcObserverControlCallback<P>>, RC, S>
	where
		F: FnMut(Context<P>, ControlResponse) -> Result<()> + Send + Sync + 'static,
	{
		let Self {
			context,
			activation_state,
			selector,
			response_callback,
			storage,
			..
		} = self;
		let callback: ArcObserverControlCallback<P> = Arc::new(Mutex::new(callback));
		ObserverBuilder {
			context,
			activation_state,
			selector,
			control_callback: Callback { callback },
			response_callback,
			storage,
		}
	}
}

impl<P, K, CC, S> ObserverBuilder<P, K, CC, NoCallback, S>
where
	P: Send + Sync + 'static,
{
	/// Set callback for response messages
	#[must_use]
	pub fn response_callback<F>(
		self,
		callback: F,
	) -> ObserverBuilder<P, K, CC, Callback<ArcObserverResponseCallback<P>>, S>
	where
		F: FnMut(Context<P>, ObservableResponse) -> Result<()> + Send + Sync + 'static,
	{
		let Self {
			context,
			activation_state,
			selector,
			control_callback,
			storage,
			..
		} = self;
		let callback: ArcObserverResponseCallback<P> = Arc::new(Mutex::new(callback));
		ObserverBuilder {
			context,
			activation_state,
			selector,
			control_callback,
			response_callback: Callback { callback },
			storage,
		}
	}
}

impl<P, K, CC, RC> ObserverBuilder<P, K, CC, RC, NoStorage>
where
	P: Send + Sync + 'static,
{
	/// Provide agents storage for the subscriber
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Observer<P>>>>,
	) -> ObserverBuilder<P, K, CC, RC, Storage<Observer<P>>> {
		let Self {
			context,
			activation_state,
			selector,
			control_callback,
			response_callback,
			..
		} = self;
		ObserverBuilder {
			context,
			activation_state,
			selector,
			control_callback,
			response_callback,
			storage: Storage { storage },
		}
	}
}

impl<P, S>
	ObserverBuilder<
		P,
		Selector,
		Callback<ArcObserverControlCallback<P>>,
		Callback<ArcObserverResponseCallback<P>>,
		S,
	>
where
	P: Send + Sync + 'static,
{
	/// Build the [`Observer`].
	///
	/// # Errors
	/// Currently none
	pub fn build(self) -> Result<Observer<P>> {
		let Self {
			context,
			selector,
			activation_state,
			control_callback,
			response_callback,
			..
		} = self;
		let selector = selector.selector;
		Ok(Observer::new(
			selector,
			context,
			activation_state,
			control_callback.callback,
			response_callback.callback,
		))
	}
}

impl<P>
	ObserverBuilder<
		P,
		Selector,
		Callback<ArcObserverControlCallback<P>>,
		Callback<ArcObserverResponseCallback<P>>,
		Storage<Observer<P>>,
	>
where
	P: Send + Sync + 'static,
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
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<ObserverBuilder<Props, NoSelector, NoCallback, NoCallback, NoStorage>>();
	}
}
