// Copyright © 2024 Stephan Kunz

//! dimas-com implements the communication capabilities.
//!

// region:		--- modules
/// zenoh communicator implementation
pub mod communicator;
/// the liveliness subscriber
#[cfg(feature = "unstable")]
pub mod liveliness;
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
#[allow(clippy::module_name_repetitions)]
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
