// Copyright © 2024 Stephan Kunz

//! Enums for communication capabilities
//!

/// a multi session communicator
mod multi_communicator;
/// a single session communicator
mod single_communicator;

// flatten
#[allow(clippy::module_name_repetitions)]
pub use multi_communicator::MultiCommunicator;
#[allow(clippy::module_name_repetitions)]
pub use single_communicator::SingleCommunicator;

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:      --- factory method
use crate::traits::Communicator;
use alloc::sync::Arc;
use dimas_config::Config;
use dimas_core::Result;

/// Create a [`Communicator`] from a [`Config`]
/// # Errors
pub fn from(config: &Config) -> Result<Arc<dyn Communicator>> {
	if config.sessions().is_none() {
		Ok(Arc::new(SingleCommunicator::new(config)?))
	} else {
		Ok(Arc::new(MultiCommunicator::new(config)?))
	}
}
// endregion:   --- factory method
