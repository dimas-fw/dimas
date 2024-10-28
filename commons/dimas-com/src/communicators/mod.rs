// Copyright © 2024 Stephan Kunz

//! Enums for communication capabilities
//!

/// a multi session communicator
mod multi_communicator;
/// a single session communicator
mod single_communicator;

// flatten
pub use multi_communicator::MultiCommunicator;
pub use single_communicator::SingleCommunicator;
