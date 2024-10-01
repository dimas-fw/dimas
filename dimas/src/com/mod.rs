// Copyright © 2023 Stephan Kunz

//! Module handles communication with other Agents.
//!

// region:    	--- modules
#[cfg(feature = "unstable")]
pub mod liveliness;
pub mod observation;
pub mod pubsub;
pub mod queries;
// endregion: 	--- modules

// region:		--- types
// endregion:	--- types

#[cfg(test)]
mod tests {}
