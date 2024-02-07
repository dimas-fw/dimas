// Copyright © 2023 Stephan Kunz

// region:    --- modules
pub use crate::agent::Agent;
pub use crate::com::publisher::PublisherBuilder;
pub use crate::com::query::QueryBuilder;
pub use crate::com::queryable::{QueryableBuilder, Request};
pub use crate::com::subscriber::SubscriberBuilder;
pub use crate::config::Config;
pub use crate::context::Context;
pub use crate::error::{Error, Result};
pub use crate::timer::TimerBuilder;
// endregeion:  --- modules

// region:    --- types
// Generic wrapper tuple struct for newtype pattern
//#[repr(transparent)]
//pub(crate) struct Wrap<T>(pub T);
// endregion: --- types
