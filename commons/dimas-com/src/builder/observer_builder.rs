// Copyright © 2024 Stephan Kunz

//! Module

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use crate::error::Error;
use crate::traits::Observer as ObserverTrait;
use crate::zenoh::observer::{
	ArcControlCallback, ArcResponseCallback, ControlCallback, Observer, ResponseCallback,
};
use alloc::{
	boxed::Box,
	string::{String, ToString},
	sync::Arc,
};
use core::time::Duration;
use dimas_core::builder_states::{Callback, NoCallback, NoSelector, NoStorage, Selector, Storage};
use dimas_core::{
	enums::OperationState,
	message_types::{ControlResponse, ObservableResponse},
	traits::Context,
	utils::selector_from,
	Result,
};
use futures::future::Future;
#[cfg(feature = "std")]
use std::{collections::HashMap, sync::RwLock};
#[cfg(feature = "std")]
use tokio::sync::Mutex;
// endregion:	--- modules

// region:		--- ObserverBuilder
/// The builder for an [`Observer`]
pub struct ObserverBuilder<P, K, CC, RC, S>
where
	P: Send + Sync + 'static,
{
	session_id: String,
	/// Context for the `ObserverBuilder`
	context: Context<P>,
	activation_state: OperationState,
	timeout: Duration,
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
	pub fn new(session_id: impl Into<String>, context: Context<P>) -> Self {
		Self {
			session_id: session_id.into(),
			context,
			activation_state: OperationState::Active,
			timeout: Duration::from_millis(1000),
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
			session_id,
			context,
			activation_state,
			timeout,
			control_callback,
			response_callback,
			storage,
			..
		} = self;
		ObserverBuilder {
			session_id,
			context,
			activation_state,
			timeout,
			selector: Selector {
				selector: selector.into(),
			},
			control_callback,
			response_callback,
			storage,
		}
	}

	/// Set a timeout for the [`Observer`].
	/// Default is 1000ms
	#[must_use]
	pub const fn timeout(mut self, timeout: Duration) -> Self {
		self.timeout = timeout;
		self
	}

	/// Set only the message qualifing part of the [`Observer`].
	/// Will be prefixed with `Agent`s prefix.
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
	pub fn control_callback<C, F>(
		self,
		mut callback: C,
	) -> ObserverBuilder<P, K, Callback<ArcControlCallback<P>>, RC, S>
	where
		C: FnMut(Context<P>, ControlResponse) -> F + Send + Sync + 'static,
		F: Future<Output = Result<()>> + Send + Sync + 'static,
	{
		let Self {
			session_id,
			context,
			activation_state,
			timeout,
			selector,
			response_callback,
			storage,
			..
		} = self;
		let callback: ControlCallback<P> = Box::new(move |ctx, msg| Box::pin(callback(ctx, msg)));
		let callback: ArcControlCallback<P> = Arc::new(Mutex::new(callback));
		ObserverBuilder {
			session_id,
			context,
			activation_state,
			timeout,
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
	pub fn result_callback<C, F>(
		self,
		mut callback: C,
	) -> ObserverBuilder<P, K, CC, Callback<ArcResponseCallback<P>>, S>
	where
		C: FnMut(Context<P>, ObservableResponse) -> F + Send + Sync + 'static,
		F: Future<Output = Result<()>> + Send + Sync + 'static,
	{
		let Self {
			session_id,
			context,
			activation_state,
			timeout,
			selector,
			control_callback,
			storage,
			..
		} = self;
		let callback: ResponseCallback<P> = Box::new(move |ctx, msg| Box::pin(callback(ctx, msg)));
		let callback: ArcResponseCallback<P> = Arc::new(Mutex::new(callback));
		ObserverBuilder {
			session_id,
			context,
			activation_state,
			timeout,
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
		storage: Arc<RwLock<HashMap<String, Box<dyn ObserverTrait>>>>,
	) -> ObserverBuilder<P, K, CC, RC, Storage<Box<dyn ObserverTrait>>> {
		let Self {
			session_id,
			context,
			activation_state,
			timeout,
			selector,
			control_callback,
			response_callback,
			..
		} = self;
		ObserverBuilder {
			session_id,
			context,
			activation_state,
			timeout,
			selector,
			control_callback,
			response_callback,
			storage: Storage { storage },
		}
	}
}

impl<P, S>
	ObserverBuilder<P, Selector, Callback<ArcControlCallback<P>>, Callback<ArcResponseCallback<P>>, S>
where
	P: Send + Sync + 'static,
{
	/// Build the [`Observer`].
	///
	/// # Errors
	/// Currently none
	pub fn build(self) -> Result<Observer<P>> {
		let Self {
			session_id,
			context,
			timeout,
			selector,
			activation_state,
			control_callback,
			response_callback,
			..
		} = self;
		let selector = selector.selector;
		let session = context
			.session(&session_id)
			.ok_or_else(|| Error::NoZenohSession)?;
		Ok(Observer::new(
			session,
			selector,
			context,
			activation_state,
			control_callback.callback,
			response_callback.callback,
			timeout,
		))
	}
}

impl<P>
	ObserverBuilder<
		P,
		Selector,
		Callback<ArcControlCallback<P>>,
		Callback<ArcResponseCallback<P>>,
		Storage<Box<dyn ObserverTrait>>,
	>
where
	P: Send + Sync + 'static,
{
	/// Build and add the [`Observer`] to the `Agent`.
	///
	/// # Errors
	/// Currently none
	pub fn add(self) -> Result<Option<Box<dyn ObserverTrait>>> {
		let c = self.storage.storage.clone();
		let s = self.build()?;

		let r = c
			.write()
			.map_err(|_| Error::MutexPoison(String::from("ObserverBuilder")))?
			.insert(s.selector().to_string(), Box::new(s));
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
