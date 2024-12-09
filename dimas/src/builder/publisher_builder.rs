// Copyright © 2023 Stephan Kunz

//! Module `publisher_builder`.

// region:		--- modules
use anyhow::Result;
use dimas_com::zenoh::publisher::{Publisher, PublisherParameter};
use dimas_core::{
	traits::Context, utils::selector_from, ActivityType, Component, ComponentType, OperationState,
	OperationalType,
};
use zenoh::{
	bytes::Encoding,
	qos::{CongestionControl, Priority},
};
#[cfg(feature = "unstable")]
use zenoh::{qos::Reliability, sample::Locality};

use super::{
	builder_states::{NoSelector, NoStorage, Selector, Storage},
	error::Error,
};
// endregion:	--- modules

// region:		--- PublisherBuilder
/// The builder for a [`Publisher`]
pub struct PublisherBuilder<P, K, S>
where
	P: Send + Sync + 'static,
{
	session_id: String,
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
	pub fn new(session_id: impl Into<String>, context: Context<P>) -> Self {
		Self {
			session_id: session_id.into(),
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

	/// Set the session id.
	#[must_use]
	pub fn session_id(mut self, session_id: &str) -> Self {
		self.session_id = session_id.into();
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
	pub fn encoding(mut self, encoding: impl Into<String>) -> Self {
		self.encoding = encoding.into();
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
	pub fn storage(self, storage: &mut ComponentType) -> PublisherBuilder<P, K, Storage> {
		let Self {
			session_id,
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
			session_id,
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
			session_id,
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
			session_id,
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
	/// Will be prefixed with `Agent`s prefix.
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
	pub fn build(self) -> Result<Publisher> {
		let session = self
			.context
			.session(&self.session_id)
			.ok_or(Error::NoZenohSession)?;

		let activity = ActivityType::new(self.selector.selector.clone());
		let operational = OperationalType::new(self.activation_state);
		let encoding = Encoding::from(self.encoding);
		#[cfg(not(feature = "unstable"))]
		let parameter = PublisherParameter::new(
			self.congestion_control,
			encoding,
			self.express,
			self.priority,
		);
		#[cfg(feature = "unstable")]
		let parameter = PublisherParameter::new(
			self.congestion_control,
			encoding,
			self.express,
			self.priority,
			self.reliability,
			self.allowed_destination,
		);
		Ok(Publisher::new(
			activity,
			operational,
			self.selector.selector,
			parameter,
			session,
		))
	}
}

impl<'a, P> PublisherBuilder<P, Selector, Storage<'a>>
where
	P: Send + Sync + 'static,
{
	/// Build and add the [Publisher] to the `Agent`s context
	///
	/// # Errors
	/// Currently none
	pub fn add(self) -> Result<()> {
		let Self {
			session_id,
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
			storage,
		} = self;

		let builder = PublisherBuilder {
			session_id,
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
			storage: NoStorage,
		};

		let p = builder.build()?;
		storage.storage.add_activity(Box::new(p));
		Ok(())
	}
}
// endregion:	--- PublisherBuilder
