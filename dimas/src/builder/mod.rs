// Copyright © 2024 Stephan Kunz
#![allow(clippy::module_name_repetitions)]

//! Module handles communication with other Agents.
//!

// region:    	--- modules
pub mod builder_states;
mod error;
#[cfg(feature = "unstable")]
mod liveliness_subscriber_builder;
mod observable_builder;
mod observer_builder;
mod publisher_builder;
mod querier_builder;
mod queryable_builder;
mod subscriber_builder;
mod timer_builder;

// flatten
#[cfg(feature = "unstable")]
pub use liveliness_subscriber_builder::LivelinessSubscriberBuilder;
pub use observable_builder::ObservableBuilder;
pub use observer_builder::ObserverBuilder;
pub use publisher_builder::PublisherBuilder;
pub use querier_builder::QuerierBuilder;
pub use queryable_builder::QueryableBuilder;
pub use subscriber_builder::SubscriberBuilder;
pub use timer_builder::TimerBuilder;
// endregion: 	--- modules

#[cfg(test)]
mod tests {}