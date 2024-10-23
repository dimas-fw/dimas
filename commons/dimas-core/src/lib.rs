// Copyright © 2024 Stephan Kunz
#![no_std]

//! Core of `DiMAS`

/// States for usage in builders
#[cfg(feature = "std")]
pub mod builder_states;
/// Enums
pub mod enums;
/// Error handling
pub mod error;
/// `Message`, `Request`, `Response`, `Feedback`
pub mod message_types;
/// Traits
pub mod traits;
/// Utilities
pub mod utils;

// flatten
pub use error::Result;
