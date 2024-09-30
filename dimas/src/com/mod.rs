// Copyright © 2023 Stephan Kunz

//! Module handles communication with other Agents.
//!

// region:    	--- modules
#[cfg(feature = "unstable")]
pub mod liveliness;
pub mod observation;
pub mod pubsub;
pub mod queries;

use dimas_core::{
	error::Result,
	message_types::{ControlResponse, Message, ObservableResponse, QueryMsg, QueryableMsg},
	traits::Context,
};
use std::sync::{Arc, Mutex};
// endregion: 	--- modules

// region:		--- types
/// Type definition for a liveliness atomic reference counted callback
#[cfg(feature = "unstable")]
pub type ArcLivelinessCallback<P> =
	Arc<Mutex<dyn FnMut(Context<P>, &str) -> Result<()> + Send + Sync + 'static>>;
// ------ Subscriber
/// Type definition for a subscribers atomic reference counted `put` callback
pub type ArcPutCallback<P> =
	Arc<Mutex<dyn FnMut(Context<P>, Message) -> Result<()> + Send + Sync + 'static>>;
/// Type definition for a subscribers atomic reference counted `delete` callback
pub type ArcDeleteCallback<P> =
	Arc<Mutex<dyn FnMut(Context<P>) -> Result<()> + Send + Sync + 'static>>;
// ------ Querier
/// type definition for a queriers atomic reference counted `response` callback
pub type ArcQuerierCallback<P> =
	Arc<Mutex<dyn FnMut(Context<P>, QueryableMsg) -> Result<()> + Send + Sync + 'static>>;
// ------ Queryable
/// type defnition for a queryables atomic reference counted `request` callback
pub type ArcQueryableCallback<P> =
	Arc<Mutex<dyn FnMut(Context<P>, QueryMsg) -> Result<()> + Send + Sync + 'static>>;
// ------ Observer
/// Type definition for an observer atomic reference counted `control` callback
pub type ArcObserverControlCallback<P> =
	Arc<Mutex<dyn FnMut(Context<P>, ControlResponse) -> Result<()> + Send + Sync + 'static>>;
/// Type definition for an observables atomic reference counted `result` callback
pub type ArcObserverResponseCallback<P> =
	Arc<Mutex<dyn FnMut(Context<P>, ObservableResponse) -> Result<()> + Send + Sync + 'static>>;
// ------ Observable
/// Type definition for an observables atomic reference counted `control` callback
pub type ArcObservableControlCallback<P> =
	Arc<Mutex<dyn FnMut(Context<P>, Message) -> Result<ControlResponse> + Send + Sync + 'static>>;
/// Type definition for an observables atomic reference counted `feedback` callback
pub type ArcObservableFeedbackCallback<P> =
	Arc<Mutex<dyn FnMut(Context<P>) -> Result<Message> + Send + Sync + 'static>>;
/// Type definition for an observables atomic reference counted `execution` function
pub type ArcObservableExecutionCallback<P> =
	Arc<tokio::sync::Mutex<dyn FnMut(Context<P>) -> Result<Message> + Send + Sync + 'static>>;
// endregion:	--- types

#[cfg(test)]
mod tests {}
