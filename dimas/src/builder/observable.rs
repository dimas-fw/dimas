// Copyright © 2024 Stephan Kunz

// region:		--- modules
use crate::{
	builder::{Callback, NoCallback, NoSelector, NoStorage, Selector, Storage},
	com::{
		observable::Observable, ArcObservableControlCallback, ArcObservableExecutionFunction,
		ArcObservableFeedbackCallback,
	},
};
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	message_types::{ControlResponse, Message},
	traits::Context,
	utils::selector_from,
};
use std::{
	sync::{Arc, Mutex, RwLock},
	time::Duration,
};
// endregion:	--- modules

// region:		--- ObservableBuilder
/// The builder for an [`Observable`]
#[allow(clippy::module_name_repetitions)]
pub struct ObservableBuilder<P, K, CC, FC, EF, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Context for the ObservableBuilder
	context: Context<P>,
	activation_state: OperationState,
	feedback_interval: Duration,
	selector: K,
	control_callback: CC,
	feedback_callback: FC,
	execution_function: EF,
	storage: S,
}

impl<P> ObservableBuilder<P, NoSelector, NoCallback, NoCallback, NoCallback, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a `QueryBuilder` in initial state
	#[must_use]
	pub const fn new(context: Context<P>) -> Self {
		Self {
			context,
			activation_state: OperationState::Standby,
			feedback_interval: Duration::from_millis(1000),
			selector: NoSelector,
			control_callback: NoCallback,
			feedback_callback: NoCallback,
			execution_function: NoCallback,
			storage: NoStorage,
		}
	}
}

impl<P, K, CC, FC, EC, S> ObservableBuilder<P, K, CC, FC, EC, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the activation state.
	#[must_use]
	pub const fn activation_state(mut self, state: OperationState) -> Self {
		self.activation_state = state;
		self
	}

	/// Set the feedback interval.
	#[must_use]
	pub const fn feedback_interval(mut self, interval: Duration) -> Self {
		self.feedback_interval = interval;
		self
	}
}

impl<P, CC, FC, EF, S> ObservableBuilder<P, NoSelector, CC, FC, EF, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full expression for the [`Observable`].
	#[must_use]
	pub fn selector(self, selector: &str) -> ObservableBuilder<P, Selector, CC, FC, EF, S> {
		let Self {
			context,
			activation_state,
			feedback_interval,
			storage,
			control_callback,
			feedback_callback,
			execution_function,
			..
		} = self;
		ObservableBuilder {
			context,
			activation_state,
			feedback_interval,
			selector: Selector {
				selector: selector.into(),
			},
			control_callback,
			feedback_callback,
			execution_function,
			storage,
		}
	}

	/// Set only the topic of the [`Observable`].
	/// Will be prefixed with agents prefix.
	#[must_use]
	pub fn topic(self, topic: &str) -> ObservableBuilder<P, Selector, CC, FC, EF, S> {
		let selector = selector_from(topic, self.context.prefix());
		self.selector(&selector)
	}
}

impl<P, K, FC, EF, S> ObservableBuilder<P, K, NoCallback, FC, EF, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set callback for control messages
	#[must_use]
	pub fn control_callback<F>(
		self,
		callback: F,
	) -> ObservableBuilder<P, K, Callback<ArcObservableControlCallback<P>>, FC, EF, S>
	where
		F: FnMut(&Context<P>, Message) -> Result<ControlResponse> + Send + Sync + Unpin + 'static,
	{
		let Self {
			context,
			activation_state,
			feedback_interval,
			selector,
			storage,
			feedback_callback,
			execution_function,
			..
		} = self;
		let callback: ArcObservableControlCallback<P> = Arc::new(Mutex::new(callback));
		ObservableBuilder {
			context,
			activation_state,
			feedback_interval,
			selector,
			control_callback: Callback { callback },
			feedback_callback,
			execution_function,
			storage,
		}
	}
}

impl<P, K, CC, EF, S> ObservableBuilder<P, K, CC, NoCallback, EF, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set callback for feedback messages
	#[must_use]
	pub fn feedback_callback<F>(
		self,
		callback: F,
	) -> ObservableBuilder<P, K, CC, Callback<ArcObservableFeedbackCallback<P>>, EF, S>
	where
		F: FnMut(&Context<P>) -> Result<Message> + Send + Sync + Unpin + 'static,
	{
		let Self {
			context,
			activation_state,
			feedback_interval,
			selector,
			storage,
			control_callback,
			execution_function,
			..
		} = self;
		let callback: ArcObservableFeedbackCallback<P> = Arc::new(Mutex::new(callback));
		ObservableBuilder {
			context,
			activation_state,
			feedback_interval,
			selector,
			control_callback,
			feedback_callback: Callback { callback },
			execution_function,
			storage,
		}
	}
}

impl<P, K, CC, FC, S> ObservableBuilder<P, K, CC, FC, NoCallback, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set execution function
	#[must_use]
	pub fn execution_function<F>(
		self,
		function: F,
	) -> ObservableBuilder<P, K, CC, FC, Callback<ArcObservableExecutionFunction<P>>, S>
	where
		F: FnMut(&Context<P>) -> Result<Message> + Send + Sync + Unpin + 'static,
	{
		let Self {
			context,
			activation_state,
			feedback_interval,
			selector,
			storage,
			control_callback,
			feedback_callback,
			..
		} = self;
		let function: ArcObservableExecutionFunction<P> = Arc::new(tokio::sync::Mutex::new(function));
		ObservableBuilder {
			context,
			activation_state,
			feedback_interval,
			selector,
			control_callback,
			feedback_callback,
			execution_function: Callback { callback: function },
			storage,
		}
	}
}

impl<P, K, CC, FC, EF> ObservableBuilder<P, K, CC, FC, EF, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Provide agents storage for the queryable
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Observable<P>>>>,
	) -> ObservableBuilder<P, K, CC, FC, EF, Storage<Observable<P>>> {
		let Self {
			context,
			activation_state,
			feedback_interval,
			selector,
			control_callback,
			feedback_callback,
			execution_function,
			..
		} = self;
		ObservableBuilder {
			context,
			activation_state,
			feedback_interval,
			selector,
			control_callback,
			feedback_callback,
			execution_function,
			storage: Storage { storage },
		}
	}
}

impl<P, S>
	ObservableBuilder<
		P,
		Selector,
		Callback<ArcObservableControlCallback<P>>,
		Callback<ArcObservableFeedbackCallback<P>>,
		Callback<ArcObservableExecutionFunction<P>>,
		S,
	> where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the [`Observable`]
	/// # Errors
	///
	pub fn build(self) -> Result<Observable<P>> {
		let Self {
			context,
			activation_state,
			feedback_interval,
			selector,
			control_callback,
			feedback_callback,
			execution_function,
			..
		} = self;
		Ok(Observable::new(
			selector.selector,
			context,
			activation_state,
			feedback_interval,
			control_callback.callback,
			feedback_callback.callback,
			execution_function.callback,
		))
	}
}

impl<P>
	ObservableBuilder<
		P,
		Selector,
		Callback<ArcObservableControlCallback<P>>,
		Callback<ArcObservableFeedbackCallback<P>>,
		Callback<ArcObservableExecutionFunction<P>>,
		Storage<Observable<P>>,
	> where
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
		is_normal::<
			ObservableBuilder<Props, NoSelector, NoCallback, NoCallback, NoCallback, NoStorage>,
		>();
	}
}
