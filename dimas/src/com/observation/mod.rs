// Copyright © 2024 Stephan Kunz

//! Module handles observation.
//!

// region:    	--- modules
mod observable;
mod observable_builder;
mod observer;
mod observer_builder;

// flatten
pub use observable::*;
pub use observable_builder::*;
pub use observer::*;
pub use observer_builder::*;

use dimas_core::{
	error::Result,
	message_types::{ControlResponse, Message, ObservableResponse},
	traits::Context,
};
use std::sync::{Arc, Mutex};
// endregion: 	--- modules

// region:    	--- types
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
pub type ArcObservableExecutionFunction<P> =
	Arc<tokio::sync::Mutex<dyn FnMut(Context<P>) -> Result<Message> + Send + Sync + 'static>>;
// endregion: 	--- types
