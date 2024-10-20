// Copyright © 2023 Stephan Kunz
#![no_std]

//! dimas-com implements the communication capabilities.
//!

// region:		--- modules
/// Modules errors
pub mod error;

/// `Communicator`
mod communicator;
/// the liveliness subscriber
#[cfg(feature = "unstable")]
pub mod liveliness;
/// the core messages
pub mod messages;
/// the observable
pub mod observable;
/// the observer
pub mod observer;
/// the publisher
pub mod publisher;
/// the querier
pub mod querier;
/// the queryable
pub mod queryable;
/// the subscriber
pub mod subscriber;

// flatten
pub use communicator::Communicator;
#[cfg(feature = "unstable")]
pub use liveliness::LivelinessSubscriber;
pub use observable::Observable;
pub use observer::Observer;
pub use publisher::Publisher;
pub use querier::Querier;
pub use queryable::Queryable;
pub use subscriber::Subscriber;
// endregion:	--- modules
