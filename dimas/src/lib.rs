// Copyright © 2023 Stephan Kunz
#![crate_type = "lib"]
#![crate_name = "dimas"]
//#![no_panic]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

#[cfg(doctest)]
doc_comment::doctest!("../README.md");

// region:    --- modules
pub mod agent;
pub mod com;
pub mod context;
pub mod time;

// Simplified usage of dimas.
pub mod prelude;

#[cfg(doc)]
use crate::agent::Agent;
use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
};
use tokio::time::Duration;
// endregion: --- modules

// region:		--- builder_states
/// State signaling that the builder has no storage value set
pub struct NoStorage;
/// State signaling that the builderhas the storage value set
pub struct Storage<S>
where
	S: Send + Sync + 'static,
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
	C: Send + Sync + 'static,
{
	/// The callback to use
	pub callback: C,
}
// endregion:	--- builder_states
