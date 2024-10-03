// Copyright © 2023 Stephan Kunz
#![no_std]

//! dimas-com implements the communication capabilities.
//!

/// `Communicator`
pub mod communicator;
/// the core messages
pub mod messages;

// re-exports
pub use communicator::Communicator;
