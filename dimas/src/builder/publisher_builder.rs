// Copyright © 2023 Stephan Kunz

//! Module `publisher` provides a message sender `Publisher` which can be created using the `PublisherBuilder`.

// region:		--- modules
// these ones are only for doc needed
use super::{NoSelector, NoStorage, Selector, Storage};
#[cfg(doc)]
use crate::agent::Agent;
use crate::error::Error;
use dimas_com::traits::Publisher as PublisherTrait;
use dimas_com::zenoh::publisher::Publisher;
use dimas_core::{enums::OperationState, traits::Context, utils::selector_from, Result};
use std::sync::{Arc, RwLock};
use zenoh::bytes::Encoding;
use zenoh::qos::CongestionControl;
use zenoh::qos::Priority;
#[cfg(feature = "unstable")]
use zenoh::{qos::Reliability, sample::Locality};
// endregion:	--- modules

// region:		--- PublisherBuilder
/// The builder for a [`Publisher`]
pub struct PublisherBuilder<P, K, S>
where
	P: Send + Sync + 'static,
{
	context: Context<P>,
	activation_state: OperationState,
	#[cfg(feature = "unstable")]
	allowed_destination: Locality,
	congestion_control: CongestionControl,
	encoding: String,
	express: bool,
	priority: Priority,
	#[cfg(feature = "unstable")]
	reliability: Reliability,
	selector: K,
	storage: S,
}

impl<P> PublisherBuilder<P, NoSelector, NoStorage>
where
	P: Send + Sync + 'static,
{
	/// Construct a [`PublisherBuilder`] in initial state
	#[must_use]
	pub fn new(context: Context<P>) -> Self {
		Self {
			context,
			activation_state: OperationState::Active,
			#[cfg(feature = "unstable")]
			allowed_destination: Locality::Any,
			congestion_control: CongestionControl::Drop,
			encoding: Encoding::default().to_string(),
			express: false,
			priority: Priority::Data,
			#[cfg(feature = "unstable")]
			reliability: Reliability::BestEffort,
			selector: NoSelector,
			storage: NoStorage,
		}
	}
}

impl<P, K, S> PublisherBuilder<P, K, S>
where
	P: Send + Sync + 'static,
{
	/// Set the activation state.
	#[must_use]
	pub const fn activation_state(mut self, state: OperationState) -> Self {
		self.activation_state = state;
		self
	}

	/// Set the publishers alllowed destinations
	#[cfg(feature = "unstable")]
	#[must_use]
	pub const fn set_allowed_destination(mut self, allowed_destination: Locality) -> Self {
		self.allowed_destination = allowed_destination;
		self
	}

	/// Set the publishers congestion control
	#[must_use]
	pub const fn set_congestion_control(mut self, congestion_control: CongestionControl) -> Self {
		self.congestion_control = congestion_control;
		self
	}

	/// Set the publishers encoding
	#[must_use]
	pub fn encoding(mut self, encoding: String) -> Self {
		self.encoding = encoding;
		self
	}

	/// Set the publishers enexpress policy
	#[must_use]
	pub const fn set_express(mut self, express: bool) -> Self {
		self.express = express;
		self
	}

	/// Set the publishers priority
	#[must_use]
	pub const fn set_priority(mut self, priority: Priority) -> Self {
		self.priority = priority;
		self
	}

	/// Set the publishers reliability
	#[cfg(feature = "unstable")]
	#[must_use]
	pub const fn set_reliability(mut self, reliability: Reliability) -> Self {
		self.reliability = reliability;
		self
	}
}

impl<P, K> PublisherBuilder<P, K, NoStorage>
where
	P: Send + Sync + 'static,
{
	/// Provide agents storage for the publisher
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Box<dyn PublisherTrait>>>>,
	) -> PublisherBuilder<P, K, Storage<Box<dyn PublisherTrait>>> {
		let Self {
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			congestion_control,
			encoding,
			express,
			priority,
			#[cfg(feature = "unstable")]
			reliability,
			selector,
			..
		} = self;
		PublisherBuilder {
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			congestion_control,
			encoding,
			express,
			priority,
			#[cfg(feature = "unstable")]
			reliability,
			selector,
			storage: Storage { storage },
		}
	}
}

impl<P, S> PublisherBuilder<P, NoSelector, S>
where
	P: Send + Sync + 'static,
{
	/// Set the full key expression for the [`Publisher`]
	#[must_use]
	pub fn selector(self, selector: &str) -> PublisherBuilder<P, Selector, S> {
		let Self {
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			congestion_control,
			encoding,
			express,
			priority,
			#[cfg(feature = "unstable")]
			reliability,
			storage,
			..
		} = self;
		PublisherBuilder {
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			congestion_control,
			encoding,
			express,
			priority,
			#[cfg(feature = "unstable")]
			reliability,
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
	P: Send + Sync + 'static,
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
			#[cfg(feature = "unstable")]
			self.allowed_destination,
			self.congestion_control,
			self.encoding,
			self.express,
			self.priority,
			#[cfg(feature = "unstable")]
			self.reliability,
		))
	}
}

impl<P> PublisherBuilder<P, Selector, Storage<Box<dyn PublisherTrait>>>
where
	P: Send + Sync + 'static,
{
	/// Build and add the [Publisher] to the [`Agent`]s context
	///
	/// # Errors
	/// Currently none
	pub fn add(self) -> Result<Option<Box<dyn PublisherTrait>>> {
		let collection = self.storage.storage.clone();
		let p = self.build()?;
		let r = collection
			.write()
			.map_err(|_| Error::MutexPoison(String::from("PublisherBuilder")))?
			.insert(p.selector().to_string(), Box::new(p));
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
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<PublisherBuilder<Props, NoSelector, NoStorage>>();
	}
}
