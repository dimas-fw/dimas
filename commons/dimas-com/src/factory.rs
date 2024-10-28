// Copyright © 2024 Stephan Kunz

//! Implementation of the communicator factory
//!

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use crate::{
	enums::CommunicatorImplementation, traits::Communicator, MultiCommunicator, SingleCommunicator,
};
use alloc::{sync::Arc, vec::Vec};
use dimas_config::Config;
use dimas_core::Result;
// endregion:   --- modules

// region:      --- Factory
/// Factory for creation of [`Communicator`]
pub struct Factory {}

impl Factory {
	/// Create a [`Communicator`] from a [`Config`]
	/// # Errors
	pub fn from(config: &Config) -> Result<Arc<dyn Communicator>> {
		let _implementations: Vec<CommunicatorImplementation> = Vec::new();
		let num_implementations = 1u8;

		if num_implementations == 1 {
			Ok(Arc::new(SingleCommunicator::new(config)?))
		} else {
			Ok(Arc::new(MultiCommunicator::new(config)?))
		}
	}
}
// endregion:   --- Factory
