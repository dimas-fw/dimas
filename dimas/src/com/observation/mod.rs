// Copyright © 2024 Stephan Kunz

//! Module handles observation.
//!

// region:    	--- modules
mod observable;
mod observable_builder;
mod observer;
mod observer_builder;

use std::sync::Arc;

use dimas_core::{
	error::Result,
	message_types::{ControlResponse, Message, ObservableResponse},
	traits::Context,
};
use futures::future::BoxFuture;
// flatten
pub use observable::*;
pub use observable_builder::*;
pub use observer::*;
pub use observer_builder::*;
use tokio::sync::Mutex;
// endregion: 	--- modules

// region:    	--- types
/// Type definition for an observers `control` callback
type ObserverControlCallback<P> =
	Box<dyn FnMut(Context<P>, ControlResponse) -> BoxFuture<'static, Result<()>> + Send + Sync>;
/// Type definition for an observers atomic reference counted `control` callback
type ArcObserverControlCallback<P> = Arc<Mutex<ObserverControlCallback<P>>>;
/// Type definition for an observers `response` callback
type ObserverResponseCallback<P> =
	Box<dyn FnMut(Context<P>, ObservableResponse) -> BoxFuture<'static, Result<()>> + Send + Sync>;
/// Type definition for an observers atomic reference counted `response` callback
type ArcObserverResponseCallback<P> = Arc<Mutex<ObserverResponseCallback<P>>>;
/// Type definition for an observables `control` callback
type ObservableControlCallback<P> = Box<
	dyn FnMut(Context<P>, Message) -> BoxFuture<'static, Result<ControlResponse>> + Send + Sync,
>;
/// Type definition for an observables atomic reference counted `control` callback
type ArcObservableControlCallback<P> = Arc<Mutex<ObservableControlCallback<P>>>;
/// Type definition for an observables `feedback` callback
type ObservableFeedbackCallback<P> =
	Box<dyn FnMut(Context<P>) -> BoxFuture<'static, Result<Message>> + Send + Sync>;
/// Type definition for an observables atomic reference counted `feedback` callback
type ArcObservableFeedbackCallback<P> = Arc<Mutex<ObservableFeedbackCallback<P>>>;
/// Type definition for an observables atomic reference counted `execution` callback
type ObservableExecutionCallback<P> =
	Box<dyn FnMut(Context<P>) -> BoxFuture<'static, Result<Message>> + Send + Sync>;
/// Type definition for an observables atomic reference counted `execution` callback
type ArcObservableExecutionCallback<P> = Arc<Mutex<ObservableExecutionCallback<P>>>;
// endregion: 	--- types
