//! `DiMAS` observation example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas::prelude::*;
// endregion:	--- modules

/// request structure for observer and observable
#[derive(Debug, Encode, Decode)]
pub struct FibonacciRequest {
	/// limit
	pub limit: u128,
}
