// Copyright © 2023 Stephan Kunz
#![no_std]

//! dimas-com implements the communication capabilities.
//!

// /// Builders
#[cfg(feature = "std")]
pub mod builder;
/// the different communicators
pub mod communicator;
/// Enums
pub mod enums;
/// Modules errors
pub mod error;
/// `Communicator` trait
pub mod traits;
/// zenoh implementation
pub mod zenoh;

// flatten
pub use communicator::*;
