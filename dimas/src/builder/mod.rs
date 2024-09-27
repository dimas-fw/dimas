// Copyright © 2024 Stephan Kunz

//! Module provides builder.
//!

// region:    --- modules
use core::time::Duration;
use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
};
// endregion: --- modules

// region:		--- states
/// State signaling that the builder has no storage value set
pub struct NoStorage;
/// State signaling that the builderhas the storage value set
pub struct Storage<S>
where
	S: Send + Sync + Unpin + 'static,
{
	/// Thread safe reference to a [`HashMap`] to store the created item of type T
	pub storage: Arc<RwLock<HashMap<String, S>>>,
}

/// State signaling that the builder has no selector set
pub struct NoSelector;
/// State signaling that the builder has the selector set
pub struct Selector {
	/// The selector
	pub selector: String,
}

/// State signaling that the builder has no interval set
pub struct NoInterval;
/// State signaling that the builder has the interval set
pub struct Interval {
	/// The [`Duration`] of the interval
	pub interval: Duration,
}

/// State signaling that the builder has a callback not set
pub struct NoCallback;
/// State signaling that the builder has a callback set
pub struct Callback<C>
where
	C: Send + Sync + Unpin + 'static,
{
	/// The callback to use
	pub callback: C,
}
// endregion:	--- states

/// `LivelinessBuilder`
#[cfg(feature = "unstable")]
pub mod liveliness;
/// `ObservableBuilder`
pub mod observable;
/// `ObserverBuilder`
pub mod observer;
/// `PublisherBuilder`
pub mod publisher;
/// `QueryBuilder`
pub mod querier;
/// `QueryableBuilder`
pub mod queryable;
/// `SubscriberBuilder`
pub mod subscriber;
/// `TimerrBuilder`
pub mod timer;

#[cfg(test)]
mod tests {}
