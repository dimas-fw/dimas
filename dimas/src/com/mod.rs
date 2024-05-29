// Copyright © 2023 Stephan Kunz

//! Module handles communication with other Agents.
//!

// region:    --- modules
/// `Liveliness`
pub mod liveliness;
/// `LivelinessBulider`
pub mod liveliness_builder;
/// `Publisher`
pub mod publisher;
/// `PublisherBuilder`
pub mod publisher_builder;
/// `Query`
pub mod query;
/// `QueryBuilder`
pub mod query_builder;
/// `Queryable`
pub mod queryable;
/// `QueryableBuilder`
pub mod queryable_builder;
/// `Subscriber`
pub mod subscriber;
/// `SubscriberBuilder`
pub mod subscriber_builder;
// endregion: --- modules

#[cfg(test)]
mod tests {}
