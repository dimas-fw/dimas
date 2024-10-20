// Copyright © 2023 Stephan Kunz
#![no_std]

//! dimas-com implements the communication capabilities.
//!

/// Modules errors
pub mod error;

/// `Communicator` trait
pub mod traits;
/// the core messages
pub mod messages;
/// zenoh implementation
pub mod zenoh;
