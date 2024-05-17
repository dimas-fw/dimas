// Copyright © 2023 Stephan Kunz

//! dimas-com implements the communication capabilities.
//!

/// `Communicator`
pub mod communicator;
/// `Message`, `Request`, `Response`, `Feedback`
pub mod message_types;
/// the core messages
pub mod messages;
/// Task signals
pub mod task_signal;

// re-exports
pub use communicator::Communicator;
pub use message_types::*;
