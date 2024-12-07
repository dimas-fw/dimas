// Copyright © 2024 Stephan Kunz

//! Builder for an [`Observable`]

// region:		--- modules
use anyhow::Result;
use dimas_com::zenoh::observable::{
	ArcControlCallback, ArcExecutionCallback, ArcFeedbackCallback, ControlCallback,
	ExecutionCallback, FeedbackCallback, Observable, ObservableParameter,
};
use dimas_core::{
	message_types::{Message, ObservableControlResponse}, traits::Context, utils::selector_from, ActivityType, Component, ComponentType, OperationState, OperationalType
};
use futures::future::{BoxFuture, Future};
use std::sync::Arc;
use tokio::{sync::Mutex, time::Duration};

use super::{
	builder_states::{Callback, NoCallback, NoSelector, NoStorage, Selector, Storage},
	error::Error,
};
// endregion:	--- modules

// region:		--- ObservableBuilder
/// The builder for an [`Observable`]
pub struct ObservableBuilder<P, K, CC, FC, EF, S>
where
	P: Send + Sync + 'static,
{
	session_id: String,
	/// Context for the `ObservableBuilder`
	context: Context<P>,
	activation_state: OperationState,
	feedback_interval: Duration,
	selector: K,
	control_callback: CC,
	feedback_callback: FC,
	execution_callback: EF,
	storage: S,
}

impl<P> ObservableBuilder<P, NoSelector, NoCallback, NoCallback, NoCallback, NoStorage>
where
	P: Send + Sync + 'static,
{
	/// Construct a `ObservableBuilder` in initial state
	#[must_use]
	pub fn new(session_id: impl Into<String>, context: Context<P>) -> Self {
		Self {
			session_id: session_id.into(),
			context,
			activation_state: OperationState::Active,
			feedback_interval: Duration::from_millis(100),
			selector: NoSelector,
			control_callback: NoCallback,
			feedback_callback: NoCallback,
			execution_callback: NoCallback,
			storage: NoStorage,
		}
	}
}

impl<P, K, CC, FC, EC, S> ObservableBuilder<P, K, CC, FC, EC, S>
where
	P: Send + Sync + 'static,
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

	/// Set the session id.
	#[must_use]
	pub fn session_id(mut self, session_id: &str) -> Self {
		self.session_id = session_id.into();
		self
	}
}

impl<P, CC, FC, EF, S> ObservableBuilder<P, NoSelector, CC, FC, EF, S>
where
	P: Send + Sync + 'static,
{
	/// Set the full expression for the [`Observable`].
	#[must_use]
	pub fn selector(self, selector: &str) -> ObservableBuilder<P, Selector, CC, FC, EF, S> {
		let Self {
			session_id,
			context,
			activation_state,
			feedback_interval,
			storage,
			control_callback,
			feedback_callback,
			execution_callback,
			..
		} = self;
		ObservableBuilder {
			session_id,
			context,
			activation_state,
			feedback_interval,
			selector: Selector {
				selector: selector.into(),
			},
			control_callback,
			feedback_callback,
			execution_callback,
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
	P: Send + Sync + 'static,
{
	/// Set callback for control messages
	#[must_use]
	pub fn control_callback<C, F>(
		self,
		mut callback: C,
	) -> ObservableBuilder<P, K, Callback<ArcControlCallback<P>>, FC, EF, S>
	where
		C: FnMut(Context<P>, Message) -> F + Send + Sync + 'static,
		F: Future<Output = Result<ObservableControlResponse>> + Send + Sync + 'static,
	{
		let Self {
			session_id,
			context,
			activation_state,
			feedback_interval,
			selector,
			storage,
			feedback_callback,
			execution_callback,
			..
		} = self;
		let callback: ControlCallback<P> = Box::new(move |ctx, msg| Box::pin(callback(ctx, msg)));
		let callback: ArcControlCallback<P> = Arc::new(Mutex::new(callback));
		ObservableBuilder {
			session_id,
			context,
			activation_state,
			feedback_interval,
			selector,
			control_callback: Callback { callback },
			feedback_callback,
			execution_callback,
			storage,
		}
	}
}

impl<P, K, CC, EF, S> ObservableBuilder<P, K, CC, NoCallback, EF, S>
where
	P: Send + Sync + 'static,
{
	/// Set callback for feedback messages
	#[must_use]
	pub fn feedback_callback<C, F>(
		self,
		mut callback: C,
	) -> ObservableBuilder<P, K, CC, Callback<ArcFeedbackCallback<P>>, EF, S>
	where
		C: FnMut(Context<P>) -> F + Send + Sync + 'static,
		F: Future<Output = Result<Message>> + Send + Sync + 'static,
	{
		let Self {
			session_id,
			context,
			activation_state,
			feedback_interval,
			selector,
			storage,
			control_callback,
			execution_callback,
			..
		} = self;
		let callback: FeedbackCallback<P> = Box::new(move |ctx| Box::pin(callback(ctx)));
		let callback: ArcFeedbackCallback<P> = Arc::new(Mutex::new(callback));
		ObservableBuilder {
			session_id,
			context,
			activation_state,
			feedback_interval,
			selector,
			control_callback,
			feedback_callback: Callback { callback },
			execution_callback,
			storage,
		}
	}
}

impl<P, K, CC, FC, S> ObservableBuilder<P, K, CC, FC, NoCallback, S>
where
	P: Send + Sync + 'static,
{
	/// Set execution function
	#[must_use]
	pub fn execution_callback<C, F>(
		self,
		mut callback: C,
	) -> ObservableBuilder<P, K, CC, FC, Callback<ArcExecutionCallback<P>>, S>
	where
		C: FnMut(Context<P>) -> F + Send + Sync + 'static,
		F: Future<Output = Result<Message>> + Send + Sync + 'static,
	{
		let Self {
			session_id,
			context,
			activation_state,
			feedback_interval,
			selector,
			storage,
			control_callback,
			feedback_callback,
			..
		} = self;
		let callback: ExecutionCallback<P> = Box::new(move |ctx| Box::pin(callback(ctx)));
		let callback = Arc::new(Mutex::new(callback));
		ObservableBuilder {
			session_id,
			context,
			activation_state,
			feedback_interval,
			selector,
			control_callback,
			feedback_callback,
			execution_callback: Callback { callback },
			storage,
		}
	}
}

impl<P, K, CC, FC, EF> ObservableBuilder<P, K, CC, FC, EF, NoStorage>
where
	P: Send + Sync + 'static,
{
	/// Provide agents storage for the observable
	#[must_use]
	pub fn storage(
		self,
		storage: &mut ComponentType,
	) -> ObservableBuilder<P, K, CC, FC, EF, Storage> {
		let Self {
			session_id,
			context,
			activation_state,
			feedback_interval,
			selector,
			control_callback,
			feedback_callback,
			execution_callback,
			..
		} = self;
		ObservableBuilder {
			session_id,
			context,
			activation_state,
			feedback_interval,
			selector,
			control_callback,
			feedback_callback,
			execution_callback,
			storage: Storage { storage },
		}
	}
}

impl<P, S>
	ObservableBuilder<
		P,
		Selector,
		Callback<ArcControlCallback<P>>,
		Callback<ArcFeedbackCallback<P>>,
		Callback<
			Arc<
				Mutex<
					Box<dyn FnMut(Context<P>) -> BoxFuture<'static, Result<Message>> + Send + Sync>,
				>,
			>,
		>,
		S,
	>
where
	P: Send + Sync + 'static,
{
	/// Build the [`Observable`]
	/// # Errors
	///
	pub fn build(self) -> Result<Observable<P>> {
		let Self {
			session_id,
			context,
			activation_state,
			feedback_interval,
			selector,
			control_callback,
			feedback_callback,
			execution_callback,
			..
		} = self;

		let session = context
			.session(&session_id)
			.ok_or(Error::NoZenohSession)?;

		let selector = selector.selector;
		let activity = ActivityType::new(selector.clone());
		let operational = OperationalType::new(activation_state);
		let parameter = ObservableParameter::new(feedback_interval);

		Ok(Observable::new(
			activity,
			operational,
			selector,
			parameter,
			session,
			context,
			control_callback.callback,
			feedback_callback.callback,
			execution_callback.callback,
		))
	}
}

impl<'a, P>
	ObservableBuilder<
		P,
		Selector,
		Callback<ArcControlCallback<P>>,
		Callback<ArcFeedbackCallback<P>>,
		Callback<ArcExecutionCallback<P>>,
		Storage<'a>,
	>
where
	P: Send + Sync + 'static,
{
	/// Build and add the observable to the agents context
	/// # Errors
	///
	pub fn add(self) -> Result<()> {
		let mut collection = self.storage.storage.clone();
		let o = self.build()?;
		collection.add_activity(Box::new(o));
		Ok(())
	}
}
// endregion:	--- ObservableBuilder
