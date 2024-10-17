// Copyright © 2024 Stephan Kunz
#![no_std]

//! Core of `DiMAS`

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
