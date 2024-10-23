// Copyright © 2023 Stephan Kunz
#![no_std]

//! dimas-com implements the communication capabilities.
//!

/// Builders
#[cfg(feature = "std")]
pub mod builder;
/// Modules errors
pub mod error;
/// the core messages
pub mod messages;
/// `Communicator` trait
pub mod traits;
/// zenoh implementation
pub mod zenoh;
