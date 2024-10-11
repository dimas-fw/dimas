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
use futures::future::BoxFuture;
use std::sync::Arc;
use tokio::sync::Mutex;
// endregion: 	--- modules

// region:    	--- types
/// Type definition for a subscribers `put` callback
type SubscriberPutCallback<P> =
	Box<dyn FnMut(Context<P>, Message) -> BoxFuture<'static, Result<()>> + Send + Sync>;
/// Type definition for a subscribers atomic reference counted `put` callback
type ArcSubscriberPutCallback<P> = Arc<Mutex<SubscriberPutCallback<P>>>;
/// Type definition for a subscribers `delete` callback
type SubscriberDeleteCallback<P> =
	Box<dyn FnMut(Context<P>) -> BoxFuture<'static, Result<()>> + Send + Sync>;
/// Type definition for a subscribers atomic reference counted `delete` callback
type ArcSubscriberDeleteCallback<P> = Arc<Mutex<SubscriberDeleteCallback<P>>>;
// endregion: 	--- types
