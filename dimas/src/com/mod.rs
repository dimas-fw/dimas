// Copyright © 2023 Stephan Kunz

// region:    --- modules
/// Communicator
pub mod communicator;
/// Liveliness
#[cfg(feature = "liveliness")]
pub mod liveliness_subscriber;
/// Publisher
#[cfg(feature = "publisher")]
pub mod publisher;
/// Query
#[cfg(feature = "query")]
pub mod query;
/// Queryable
#[cfg(feature = "queryable")]
pub mod queryable;
/// Subscriber
#[cfg(feature = "subscriber")]
pub mod subscriber;
// endregion: --- modules

#[cfg(test)]
mod tests {}
