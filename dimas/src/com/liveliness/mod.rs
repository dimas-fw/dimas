// Copyright © 2024 Stephan Kunz

//! Module handles liveliness.
//!

// region:    	--- modules
// endregion: 	--- modules

// region:    	--- types
// endregion: 	--- types

/// `LivelinessSubscriber`
mod subscriber;
/// `LivelinessSubscriberBuilder`
mod subscriber_builder;

// flatten
pub use subscriber::*;
pub use subscriber_builder::*;
