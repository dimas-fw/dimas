// Copyright © 2023 Stephan Kunz

//! Module handles communication with other Agents.
//!

// region:    --- modules
use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
	time::Duration,
};

/// `Liveliness`
pub mod liveliness;
/// `Observable`
pub mod observable;
/// `Observer`
pub mod observer;
/// `Publisher`
pub mod publisher;
/// `Query`
pub mod query;
/// `Queryable`
pub mod queryable;
/// `Subscriber`
pub mod subscriber;
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

/// State signaling that the builder has no callback set
pub struct NoCallback;
/// State signaling that the [`LivelinessSubscriberBuilder`] has the put callback set
pub struct Callback<C>
where
	C: Send + Sync + Unpin + 'static,
{
	/// The callback to use when receiving a put message
	pub callback: C,
}
// endregion:	--- states

#[cfg(test)]
mod tests {}
