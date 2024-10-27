// Copyright © 2023 Stephan Kunz
#![no_std]

//! dimas-com implements the communication capabilities.
//!

// /// Builders
#[cfg(feature = "std")]
pub mod builder;
/// Enums
pub mod enums;
/// Modules errors
pub mod error;
/// the core messages
pub mod messages;
/// a multi session communicator
pub mod multi_communicator;
/// a single session communicator
pub mod single_communicator;
/// `Communicator` trait
pub mod traits;
/// zenoh implementation
pub mod zenoh;

// flatten
pub use multi_communicator::MultiCommunicator;
pub use single_communicator::SingleCommunicator;
