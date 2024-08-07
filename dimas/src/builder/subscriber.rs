// Copyright © 2023 Stephan Kunz

//! Module `subscriber` provides a message `Subscriber` which can be created using the `SubscriberBuilder`.
//! A `Subscriber` can optional subscribe on a delete message.

// region:		--- modules
// these ones are only for doc needed
#[cfg(doc)]
use crate::agent::Agent;
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	message_types::Message,
	traits::Context,
	utils::selector_from,
};
use std::sync::{Arc, Mutex, RwLock};
use zenoh::pubsub::Reliability;

use crate::builder::{Callback, NoCallback, NoSelector, NoStorage, Selector, Storage};
use crate::com::{subscriber::Subscriber, ArcDeleteCallback, ArcPutCallback};
// endregion:	--- modules

// region:		--- SubscriberBuilder
/// A builder for a subscriber
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct SubscriberBuilder<P, K, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	context: Context<P>,
	activation_state: OperationState,
	selector: K,
	put_callback: C,
	storage: S,
	reliability: Reliability,
	delete_callback: Option<ArcDeleteCallback<P>>,
}

impl<P> SubscriberBuilder<P, NoSelector, NoCallback, NoStorage>
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
			put_callback: NoCallback,
			storage: NoStorage,
			reliability: Reliability::BestEffort,
			delete_callback: None,
		}
	}
}

impl<P, K, C, S> SubscriberBuilder<P, K, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the activation state.
	#[must_use]
	pub const fn activation_state(mut self, state: OperationState) -> Self {
		self.activation_state = state;
		self
	}

	/// Set reliability
	#[must_use]
	pub const fn set_reliability(mut self, reliability: Reliability) -> Self {
		self.reliability = reliability;
		self
	}

	/// Set subscribers callback for `delete` messages
	#[must_use]
	pub fn delete_callback<F>(mut self, callback: F) -> Self
	where
		F: FnMut(&Context<P>) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		self.delete_callback
			.replace(Arc::new(Mutex::new(callback)));
		self
	}
}

impl<P, C, S> SubscriberBuilder<P, NoSelector, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full key expression for the [`Subscriber`].
	#[must_use]
	pub fn selector(self, selector: &str) -> SubscriberBuilder<P, Selector, C, S> {
		let Self {
			context,
			activation_state,
			storage,
			put_callback,
			delete_callback,
			reliability,
			..
		} = self;
		SubscriberBuilder {
			context,
			activation_state,
			selector: Selector {
				selector: selector.into(),
			},
			put_callback,
			storage,
			reliability,
			delete_callback,
		}
	}

	/// Set only the message qualifing part of the [`Subscriber`].
	/// Will be prefixed with [`Agent`]s prefix.
	#[must_use]
	pub fn topic(self, topic: &str) -> SubscriberBuilder<P, Selector, C, S> {
		let selector = selector_from(topic, self.context.prefix());
		self.selector(&selector)
	}
}

impl<P, K, S> SubscriberBuilder<P, K, NoCallback, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set callback for put messages
	#[must_use]
	pub fn put_callback<F>(
		self,
		callback: F,
	) -> SubscriberBuilder<P, K, Callback<ArcPutCallback<P>>, S>
	where
		F: FnMut(&Context<P>, Message) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			context,
			activation_state,
			selector,
			storage,
			reliability,
			delete_callback,
			..
		} = self;
		let callback: ArcPutCallback<P> = Arc::new(Mutex::new(callback));
		SubscriberBuilder {
			context,
			activation_state,
			selector,
			put_callback: Callback { callback },
			storage,
			reliability,
			delete_callback,
		}
	}
}

impl<P, K, C> SubscriberBuilder<P, K, C, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Provide agents storage for the subscriber
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Subscriber<P>>>>,
	) -> SubscriberBuilder<P, K, C, Storage<Subscriber<P>>> {
		let Self {
			context,
			activation_state,
			selector,
			put_callback,
			reliability,
			delete_callback,
			..
		} = self;
		SubscriberBuilder {
			context,
			activation_state,
			selector,
			put_callback,
			storage: Storage { storage },
			reliability,
			delete_callback,
		}
	}
}

impl<P, S> SubscriberBuilder<P, Selector, Callback<ArcPutCallback<P>>, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the [`Subscriber`].
	///
	/// # Errors
	/// Currently none
	pub fn build(self) -> Result<Subscriber<P>> {
		let Self {
			context,
			activation_state,
			selector,
			put_callback,
			reliability,
			delete_callback,
			..
		} = self;
		Ok(Subscriber::new(
			selector.selector,
			context,
			activation_state,
			put_callback.callback,
			reliability,
			delete_callback,
		))
	}
}

impl<P> SubscriberBuilder<P, Selector, Callback<ArcPutCallback<P>>, Storage<Subscriber<P>>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the [`Subscriber`] to the [`Agent`].
	///
	/// # Errors
	/// Currently none
	pub fn add(self) -> Result<Option<Subscriber<P>>> {
		let c = self.storage.storage.clone();
		let s = self.build()?;

		let r = c
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(s.selector().to_string(), s);
		Ok(r)
	}
}
// endregion:	--- SubscriberBuilder

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<SubscriberBuilder<Props, NoSelector, NoCallback, NoStorage>>();
	}
}
