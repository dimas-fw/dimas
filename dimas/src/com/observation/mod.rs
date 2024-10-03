// Copyright © 2024 Stephan Kunz

//! Module handles observation.
//!

// region:    	--- modules
mod observable;
mod observable_builder;
mod observer;
mod observer_builder;

// flatten
pub use observable::*;
pub use observable_builder::*;
pub use observer::*;
pub use observer_builder::*;
// endregion: 	--- modules
