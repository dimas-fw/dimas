// Copyright © 2023 Stephan Kunz

//! Module handles communication with other Agents.
//!

// region:    --- modules
use std::sync::{Arc, Mutex};

use dimas_core::{
	error::Result,
	message_types::{Message, Request, Response},
	traits::Context,
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

// region:		--- types
/// Type definition for liveliness atomic reference counted callback function
pub type ArcLivelinessCallback<P> =
	Arc<Mutex<dyn Fn(&Context<P>, &str) -> Result<()> + Send + Sync + Unpin + 'static>>;
/// type definition for the queries callback function
pub type ArcQueryCallback<P> =
	Arc<Mutex<dyn Fn(&Context<P>, Response) -> Result<()> + Send + Sync + Unpin + 'static>>;
/// type defnition for the queryables atomic reference counted callback function.
pub type ArcQueryableCallback<P> =
	Arc<Mutex<dyn Fn(&Context<P>, Request) -> Result<()> + Send + Sync + Unpin + 'static>>;
/// Type definition for a subscribers atomic reference counted `put` callback function
pub type ArcSubscriberPutCallback<P> =
	Arc<Mutex<dyn Fn(&Context<P>, Message) -> Result<()> + Send + Sync + Unpin + 'static>>;

/// Type definition for a subscribers atomic reference counted `delete` callback function
pub type ArcSubscriberDeleteCallback<P> =
	Arc<Mutex<dyn Fn(&Context<P>) -> Result<()> + Send + Sync + Unpin + 'static>>;
// endregion:	--- types

#[cfg(test)]
mod tests {}
