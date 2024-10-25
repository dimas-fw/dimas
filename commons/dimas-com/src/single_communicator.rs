// Copyright © 2024 Stephan Kunz

//! Implementation of a multi session/protocol communicator
//!

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
// endregion:	--- modules

// region:		--- SingleCommunicator
/// a multi session communicator
#[derive(Debug)]
pub struct SingleCommunicator {}
// endregion:	--- SingleCommunicator

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<SingleCommunicator>();
	}
}
