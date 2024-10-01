// Copyright © 2024 Stephan Kunz

//! Module handles liveliness.
//!

// region:    	--- modules
/// `LivelinessSubscriber`
mod subscriber;
/// `LivelinessSubscriberBuilder`
mod subscriber_builder;

// flatten
pub use subscriber::*;
pub use subscriber_builder::*;

use dimas_core::{error::Result, traits::Context};
use std::sync::{Arc, Mutex};
// endregion: 	--- modules

// region:    	--- types
/// Type definition for a liveliness atomic reference counted callback
#[cfg(feature = "unstable")]
pub type ArcLivelinessCallback<P> =
	Arc<Mutex<dyn FnMut(Context<P>, &str) -> Result<()> + Send + Sync + 'static>>;
// endregion: 	--- types
