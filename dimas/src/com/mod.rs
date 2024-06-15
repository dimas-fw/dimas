// Copyright © 2023 Stephan Kunz

//! Module handles communication with other Agents.
//!

use bitcode::{Decode, Encode};
// region:    --- modules
use dimas_core::{
	error::Result,
	message_types::{FeedbackMsg, Message, RequestMsg, ResponseMsg, ResultMsg},
	traits::Context,
};
use std::sync::{Arc, Mutex};
// endregion: --- modules

// region:		--- enums
/// Acknowledgement to requests
#[derive(Encode, Decode)]
pub enum Acknowledge {
	/// The request was accepted
	Accepted,
	/// The request was declined
	Declined,
}

/// Status dependant feedback of observable
pub enum Feedback {
	Ongoing(FeedbackMsg),
	Canceled(ResultMsg),
	Failed(ResultMsg),
	Finished(ResultMsg),
}
// endregion:	--- enums

// region:		--- types
/// Type definition for a liveliness atomic reference counted callback
pub type ArcLivelinessCallback<P> =
	Arc<Mutex<dyn Fn(&Context<P>, &str) -> Result<()> + Send + Sync + Unpin + 'static>>;
/// type definition for a queries/observers atomic reference counted `response` callback
pub type ArcResponseCallback<P> =
	Arc<Mutex<dyn Fn(&Context<P>, ResponseMsg) -> Result<()> + Send + Sync + Unpin + 'static>>;
/// type defnition for a queryables/observables atomic reference counted `request` callback
pub type ArcRequestCallback<P> =
	Arc<Mutex<dyn Fn(&Context<P>, RequestMsg) -> Result<()> + Send + Sync + Unpin + 'static>>;
/// Type definition for a subscribers atomic reference counted `put` callback
pub type ArcMessageCallback<P> =
	Arc<Mutex<dyn Fn(&Context<P>, Message) -> Result<()> + Send + Sync + Unpin + 'static>>;
/// Type definition for a subscribers atomic reference counted `delete` callback
pub type ArcDeleteCallback<P> =
	Arc<Mutex<dyn Fn(&Context<P>) -> Result<()> + Send + Sync + Unpin + 'static>>;
/// Type definition for an observers atomic reference counted `feedback` callback
pub type ArcFeedbackCallback<P> =
	Arc<Mutex<dyn Fn(&Context<P>, FeedbackMsg) -> Result<()> + Send + Sync + Unpin + 'static>>;
/// Type definition for an observers atomic reference counted `result` callback
pub type ArcResultCallback<P> =
	Arc<Mutex<dyn Fn(&Context<P>, ResultMsg) -> Result<()> + Send + Sync + Unpin + 'static>>;
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
