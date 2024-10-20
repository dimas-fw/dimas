// Copyright © 2024 Stephan Kunz

//! Errors from com

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
#[cfg(feature = "std")]
use std::prelude::rust_2021::*;

#[cfg(doc)]
use super::{Communicator, Observable, Observer, Publisher, Querier, Queryable, Subscriber};
#[cfg(doc)]
use dimas_core::message_types::Message;
#[cfg(doc)]
use zenoh::query::Query;
// endregion:	--- modules

// region:		--- Error
/// Com error type.
pub enum Error {
	/// Not available/implemented
	NotImplemented,
	/// Invalid selector
	InvalidSelector(String),
	/// Creation of the [`Communicator`] was not possible
	CreateCommunicator {
		/// the original zenoh error
		source: Box<dyn core::error::Error + Send + Sync>,
	},
	/// Accessing a [`Publisher`] failed
	AccessPublisher,
	/// Publishing a [`Message`] via `put` failed
	PublishingPut {
		/// the original zenoh error
		source: Box<dyn core::error::Error + Send + Sync>,
	},
	/// Publishing a [`Message`] via `delete` failed
	PublishingDelete {
		/// the original zenoh error
		source: Box<dyn core::error::Error + Send + Sync>,
	},
	/// Creation of a [`Query`] failed
	QueryCreation {
		/// the original zenoh error
		source: Box<dyn core::error::Error + Send + Sync>,
	},
	/// Callback of a [`Query`] failed
	QueryCallback {
		/// the original callback error
		source: Box<dyn core::error::Error + Send + Sync>,
	},
	/// Creation of a [`Subscriber`] failed
	SubscriberCreation {
		/// the original zenoh error
		source: Box<dyn core::error::Error + Send + Sync>,
	},
	/// Callback of a [`Subscriber`] failed
	SubscriberCallback {
		/// the original callback error
		source: Box<dyn core::error::Error + Send + Sync>,
	},
	/// Accessing the [`Querier`] failed.
	AccessingQuerier {
		/// query selector
		selector: String,
	},
	/// Accessing the [`Queryable`] failed.
	AccessingQueryable {
		/// query selector
		selector: String,
	},
	/// Accessing the [`Observable`] for a [`Observer`] failed.
	AccessingObservable {
		/// observables selector
		selector: String,
	},
}
// region:		--- Error

// region:      --- boilerplate
impl core::fmt::Display for Error {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl core::fmt::Debug for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::NotImplemented => {
				write!(f, "no implementation available")
			}
			Self::InvalidSelector(location) => {
				write!(f, "invalid selector for '{location}'")
			}
			Self::CreateCommunicator { source } => {
				write!(f, "creation of zenoh session failed: reason {source}")
			}
			Self::AccessPublisher => {
				write!(f, "getting the publisher failed")
			}
			Self::PublishingPut { source } => {
				write!(f, "publishing a put message failed: reason {source}")
			}
			Self::PublishingDelete { source } => {
				write!(f, "publishing a delete message failed: reason {source}")
			}
			Self::QueryCreation { source } => {
				write!(f, "creation of a query failed: reason {source}")
			}
			Self::QueryCallback { source } => {
				write!(f, "callback of query failed: reason {source}")
			}
			Self::SubscriberCreation { source } => {
				write!(f, "creation of a subscriber failed: reason {source}")
			}
			Self::SubscriberCallback { source } => {
				write!(f, "callback of subscriber failed: reason {source}")
			}
			Self::AccessingQuerier { selector } => {
				write!(f, "accessing querier '{selector}' failed")
			}
			Self::AccessingQueryable { selector } => {
				write!(f, "accessing queryable '{selector}' failed")
			}
			Self::AccessingObservable { selector } => {
				write!(f, "accessing observable '{selector}' failed")
			}
		}
	}
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
		match *self {
			Self::CreateCommunicator { ref source }
			| Self::PublishingPut { ref source }
			| Self::PublishingDelete { ref source }
			| Self::QueryCreation { ref source }
			| Self::QueryCallback { ref source }
			| Self::SubscriberCreation { ref source }
			| Self::SubscriberCallback { ref source } => Some(source.as_ref()),
			Self::NotImplemented
			| Self::AccessPublisher
			| Self::AccessingQuerier { .. }
			| Self::AccessingQueryable { .. }
			| Self::AccessingObservable { .. }
			| Self::InvalidSelector(_) => None,
		}
	}
}
// endregion:   --- boilerplate
