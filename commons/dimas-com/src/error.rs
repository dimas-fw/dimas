// Copyright © 2024 Stephan Kunz

//! `dimas-com` errors

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
#[cfg(doc)]
use crate::zenoh::{Communicator, Observable, Observer, Publisher, Querier, Queryable, Subscriber};
use alloc::{boxed::Box, string::String};
#[cfg(doc)]
use dimas_core::message_types::Message;
use thiserror::Error;
#[cfg(doc)]
use zenoh::query::Query;
// endregion:	--- modules

// region:		--- Error
/// `dimas-com` error type.
#[derive(Error, Debug)]
pub enum Error {
	/// Not available/implemented
	#[error("no implementation available")]
	NotImplemented,
	/// no communicator for that id
	#[error("no communicator with id: {0}")]
	NoCommunicator(String),
	/// No zenoh available/implemented
	#[error("no zenoh session available")]
	NoZenohSession,
	/// Invalid selector
	#[error("invalid selector for '{0}'")]
	InvalidSelector(String),
	/// Creation of the [`Communicator`] was not possible
	#[error("creation of zenoh session failed with reason: {source}")]
	CreateCommunicator {
		/// the original zenoh error
		source: Box<dyn core::error::Error + Send + Sync>,
	},
	/// Accessing a [`Publisher`] failed
	#[error("getting the publisher failed")]
	AccessPublisher,
	/// A Mutex is poisoned.
	#[error("a Mutex poison error happened in {0}")]
	MutexPoison(String),
	/// Publishing a [`Message`] via `put` failed
	#[error("publishing a put message failed with reason: {source}")]
	PublishingPut {
		/// the original zenoh error
		source: Box<dyn core::error::Error + Send + Sync>,
	},
	/// Publishing a [`Message`] via `delete` failed
	#[error("publishing a delete message failed with reason: {source}")]
	PublishingDelete {
		/// the original zenoh error
		source: Box<dyn core::error::Error + Send + Sync>,
	},
	/// Creation of a [`Query`] failed
	#[error("creation of a query failed with reason: {source}")]
	QueryCreation {
		/// the original zenoh error
		source: Box<dyn core::error::Error + Send + Sync>,
	},
	/// Callback of a [`Query`] failed
	#[error("callback of query failed with reason: {source}")]
	QueryCallback {
		/// the original callback error
		source: Box<dyn core::error::Error + Send + Sync>,
	},
	/// read access failed
	#[error("read storage for {0} failed")]
	ReadAccess(String),
	/// write access failed
	#[error("write storage for {0} failed")]
	ModifyStruct(String),
	/// Creation of a [`Subscriber`] failed
	#[error("creation of a subscriber failed with reason: {source}")]
	SubscriberCreation {
		/// the original zenoh error
		source: Box<dyn core::error::Error + Send + Sync>,
	},
	/// Callback of a [`Subscriber`] failed
	#[error("callback of subscriber failed with reason: {source}")]
	SubscriberCallback {
		/// the original callback error
		source: Box<dyn core::error::Error + Send + Sync>,
	},
	/// Accessing the [`Querier`] failed.
	#[error("accessing querier '{selector}' failed")]
	AccessingQuerier {
		/// query selector
		selector: String,
	},
	/// Accessing the [`Queryable`] failed.
	#[error("accessing queryable '{selector}' failed")]
	AccessingQueryable {
		/// query selector
		selector: String,
	},
	/// Accessing the [`Observable`] for a [`Observer`] failed.
	#[error("accessing observable '{selector}' failed")]
	AccessingObservable {
		/// observables selector
		selector: String,
	},
}
// region:		--- Error
