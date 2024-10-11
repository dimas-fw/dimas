// Copyright © 2024 Stephan Kunz

//! Module handles liveliness.
//!

// region:    	--- modules
/// `LivelinessSubscriber`
mod liveliness_subscriber;
/// `LivelinessSubscriberBuilder`
mod liveliness_subscriber_builder;

// flatten
pub use liveliness_subscriber::*;
pub use liveliness_subscriber_builder::*;

use dimas_core::{error::Result, traits::Context};
use futures::future::BoxFuture;
use std::sync::Arc;
use tokio::sync::Mutex;
// endregion: 	--- modules

// region:    	--- types
/// Type definition for a boxed liveliness subscribers callback
type LivelinessCallback<P> =
	Box<dyn FnMut(Context<P>, String) -> BoxFuture<'static, Result<()>> + Send + Sync>;
/// Type definition for a liveliness subscribers atomic reference counted callback
type ArcLivelinessCallback<P> = Arc<Mutex<LivelinessCallback<P>>>;
// endregion: 	--- types
