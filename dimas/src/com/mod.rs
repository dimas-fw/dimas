// Copyright © 2023 Stephan Kunz

//! Module handles communication with other Agents.
//!

// region:    	--- modules
use dimas_core::{
	error::Result,
	message_types::{Message, ObservableResponse, QueryMsg, QueryableMsg},
	traits::Context,
};
use std::sync::{Arc, Mutex};
// endregion: 	--- modules

// region:		--- types
/// Type definition for a liveliness atomic reference counted callback
pub type ArcLivelinessCallback<P> =
	Arc<Mutex<dyn FnMut(&Context<P>, &str) -> Result<()> + Send + Sync + Unpin + 'static>>;
/// Type definition for a subscribers atomic reference counted `put` callback
pub type ArcPutCallback<P> =
	Arc<Mutex<dyn FnMut(&Context<P>, Message) -> Result<()> + Send + Sync + Unpin + 'static>>;
/// Type definition for a subscribers atomic reference counted `delete` callback
pub type ArcDeleteCallback<P> =
	Arc<Mutex<dyn FnMut(&Context<P>) -> Result<()> + Send + Sync + Unpin + 'static>>;
/// type definition for a queries atomic reference counted `response` callback
pub type ArcQueryCallback<P> =
	Arc<Mutex<dyn FnMut(&Context<P>, QueryableMsg) -> Result<()> + Send + Sync + Unpin + 'static>>;
/// type defnition for a queryables atomic reference counted `request` callback
pub type ArcQueryableCallback<P> =
	Arc<Mutex<dyn FnMut(&Context<P>, QueryMsg) -> Result<()> + Send + Sync + Unpin + 'static>>;
/// Type definition for an observables atomic reference counted `feedback` callback
pub type ArcFeedbackCallback<P> = Arc<
	Mutex<dyn FnMut(&Context<P>) -> Result<ObservableResponse> + Send + Sync + Unpin + 'static>,
>;
/// Type definition for an observables atomic reference counted `control` callback
pub type ArcControlCallback<P> = Arc<
	Mutex<
		dyn FnMut(&Context<P>, Message) -> Result<ObservableResponse>
			+ Send
			+ Sync
			+ Unpin
			+ 'static,
	>,
>;
// endregion:	--- types

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

#[cfg(test)]
mod tests {}
