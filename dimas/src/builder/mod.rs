// Copyright © 2023 Stephan Kunz
#![allow(clippy::module_name_repetitions)]
//! Module provides builder for communication with other Agents.
//! 
//! Currently only communication via `zenoh` is implemented.
//!

// region:    	--- modules
#[cfg(feature = "unstable")]
mod liveliness_subscriber_builder;
mod observable_builder;
mod observer_builder;
mod publisher_builder;
mod querier_builder;
mod queryable_builder;
mod subscriber_builder;
mod timer_builder;

use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
};
use tokio::time::Duration;

// flatten
#[cfg(feature = "unstable")]
pub use liveliness_subscriber_builder::LivelinessSubscriberBuilder;
pub use observable_builder::ObservableBuilder;
pub use observer_builder::ObserverBuilder;
pub use publisher_builder::PublisherBuilder;
pub use querier_builder::QuerierBuilder;
pub use queryable_builder::QueryableBuilder;
pub use subscriber_builder::SubscriberBuilder;
pub use timer_builder::TimerBuilder;
// endregion: 	--- modules

// region:		--- builder_states
/// State signaling that the builder has no storage value set
#[doc(hidden)]
pub struct NoStorage;
/// State signaling that the builder has the storage value set
#[doc(hidden)]
pub struct Storage<S>
where
	S: Send + Sync + 'static,
{
	/// Thread safe reference to a [`HashMap`] to store the created item of type T
	pub storage: Arc<RwLock<HashMap<String, S>>>,
}

/// State signaling that the builder has no selector set
#[doc(hidden)]
pub struct NoSelector;
/// State signaling that the builder has the selector set
#[doc(hidden)]
pub struct Selector {
	/// The selector
	pub selector: String,
}

/// State signaling that the builder has no interval set
#[doc(hidden)]
pub struct NoInterval;
/// State signaling that the builder has the interval set
#[doc(hidden)]
pub struct Interval {
	/// The [`Duration`] of the interval
	pub interval: Duration,
}

/// State signaling that the builder has a callback not set
#[doc(hidden)]
pub struct NoCallback;
/// State signaling that the builder has a callback set
#[doc(hidden)]
pub struct Callback<C>
where
	C: Send + Sync + 'static,
{
	/// The callback to use
	pub callback: C,
}
// endregion:	--- builder_states

#[cfg(test)]
mod tests {}
