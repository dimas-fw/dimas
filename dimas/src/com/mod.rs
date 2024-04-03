// Copyright © 2023 Stephan Kunz

//! Module handles communication with other Agents.
//!

// region:    --- modules
// to avoid doc warnings
#[allow(unused_imports)]
use super::agent::Agent;
#[allow(unused_imports)]
use communicator::Communicator;
/// Communicator
pub mod communicator;
/// Liveliness
pub mod liveliness_subscriber;
/// Message
pub mod message;
/// Publisher
pub mod publisher;
/// Query
pub mod query;
/// Queryable
pub mod queryable;
/// Subscriber
pub mod subscriber;
// endregion: --- modules

#[cfg(test)]
mod tests {}
