// Copyright © 2023 Stephan Kunz

//! Module `publisher` provides a message sender `Publisher` which can be created using the `PublisherBuilder`.

// region:		--- modules
// these ones are only for doc needed
#[cfg(doc)]
use crate::agent::Agent;
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	traits::Context,
	utils::selector_from,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use zenoh::publication::{CongestionControl, Priority};

use super::publisher::Publisher;
// endregion:	--- modules

// region:		--- states
/// State signaling that the [`PublisherBuilder`] has no storage value set
pub struct NoStorage;
/// State signaling that the [`PublisherBuilder`] has the storage value set
pub struct Storage<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Thread safe reference to a [`HashMap`] to store the created [`Publisher`]
	pub storage: Arc<RwLock<HashMap<String, Publisher<P>>>>,
}

/// State signaling that the [`PublisherBuilder`] has no selector set
pub struct NoSelector;
/// State signaling that the [`PublisherBuilder`] has the selector set
pub struct Selector {
	/// The selector
	selector: String,
}
// endregion:	--- states

// region:		--- PublisherBuilder
/// The builder for a [`Publisher`]
#[allow(clippy::module_name_repetitions)]
pub struct PublisherBuilder<P, K, S>
where
	P: Send + Sync + Unpin + 'static,
{
	context: Context<P>,
	activation_state: OperationState,
	priority: Priority,
	congestion_control: CongestionControl,
	selector: K,
	storage: S,
}

impl<P> PublisherBuilder<P, NoSelector, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a [`PublisherBuilder`] in initial state
	#[must_use]
	pub const fn new(context: Context<P>) -> Self {
		Self {
			context,
			activation_state: OperationState::Standby,
			priority: Priority::Data,
			congestion_control: CongestionControl::Drop,
			selector: NoSelector,
			storage: NoStorage,
		}
	}
}

impl<P, K, S> PublisherBuilder<P, K, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the activation state.
	#[must_use]
	pub const fn activation_state(mut self, state: OperationState) -> Self {
		self.activation_state = state;
		self
	}

	/// Set the publishers priority
	#[must_use]
	pub const fn set_priority(mut self, priority: Priority) -> Self {
		self.priority = priority;
		self
	}

	/// Set the publishers congestion control
	#[must_use]
	pub const fn set_congestion_control(mut self, congestion_control: CongestionControl) -> Self {
		self.congestion_control = congestion_control;
		self
	}
}

impl<P, K> PublisherBuilder<P, K, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Provide agents storage for the publisher
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Publisher<P>>>>,
	) -> PublisherBuilder<P, K, Storage<P>> {
		let Self {
			context,
			activation_state,
			priority,
			congestion_control,
			selector,
			..
		} = self;
		PublisherBuilder {
			context,
			activation_state,
			priority,
			congestion_control,
			selector,
			storage: Storage { storage },
		}
	}
}

impl<P, S> PublisherBuilder<P, NoSelector, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full key expression for the [`Publisher`]
	#[must_use]
	pub fn selector(self, selector: &str) -> PublisherBuilder<P, Selector, S> {
		let Self {
			context,
			activation_state,
			priority,
			congestion_control,
			storage,
			..
		} = self;
		PublisherBuilder {
			context,
			activation_state,
			priority,
			congestion_control,
			selector: Selector {
				selector: selector.into(),
			},
			storage,
		}
	}

	/// Set only the message qualifing part of the [`Publisher`].
	/// Will be prefixed with [`Agent`]s prefix.
	#[must_use]
	pub fn topic(self, topic: &str) -> PublisherBuilder<P, Selector, S> {
		let selector = selector_from(topic, self.context.prefix());
		self.selector(&selector)
	}
}

impl<P, S> PublisherBuilder<P, Selector, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the [`Publisher`]
	///
	/// # Errors
	/// Currently none
	pub fn build(self) -> Result<Publisher<P>> {
		Ok(Publisher::new(
			self.selector.selector,
			self.context,
			self.activation_state,
			self.priority,
			self.congestion_control,
		))
	}
}

impl<P> PublisherBuilder<P, Selector, Storage<P>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the [Publisher] to the [`Agent`]s context
	///
	/// # Errors
	/// Currently none
	pub fn add(self) -> Result<Option<Publisher<P>>> {
		let collection = self.storage.storage.clone();
		let p = self.build()?;
		let r = collection
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(p.selector().to_string(), p);
		Ok(r)
	}
}
// endregion:	--- PublisherBuilder

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<PublisherBuilder<Props, NoSelector, NoStorage>>();
	}
}
