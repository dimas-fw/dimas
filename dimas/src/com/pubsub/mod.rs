// Copyright © 2024 Stephan Kunz

//! Module handles publish/subscribe.
//!

// region:    	--- modules
mod publisher;
mod publisher_builder;
mod subscriber;
mod subscriber_builder;

// flatten
pub use publisher::*;
pub use publisher_builder::*;
pub use subscriber::*;
pub use subscriber_builder::*;

use dimas_core::{error::Result, message_types::Message, traits::Context};
use std::sync::{Arc, Mutex};
// endregion: 	--- modules

// region:    	--- types
// ------ Subscriber
/// Type definition for a subscribers atomic reference counted `put` callback
pub type ArcPutCallback<P> =
	Arc<Mutex<dyn FnMut(Context<P>, Message) -> Result<()> + Send + Sync + 'static>>;
/// Type definition for a subscribers atomic reference counted `delete` callback
pub type ArcDeleteCallback<P> =
	Arc<Mutex<dyn FnMut(Context<P>) -> Result<()> + Send + Sync + 'static>>;
// endregion: 	--- types
