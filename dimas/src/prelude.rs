// Copyright © 2023 Stephan Kunz

//! Public interface of dimas. Typically it is sufficient to include the prelude with
//! ```use dimas::prelude::*;```

pub extern crate bitcode;

// region:    --- modules
// re-exports
// used std synchronisation primitives
pub use std::sync::{Arc, RwLock};
// bitcode encoding/decoding
pub use bitcode::{Decode, Encode};
// zenoh stuff
pub use zenoh::publication::CongestionControl;
pub use zenoh::publication::Priority;
pub use zenoh::query::ConsolidationMode;
pub use zenoh::query::QueryTarget;
pub use zenoh::sample::Locality;
pub use zenoh::subscriber::Reliability;

// dimas stuff
pub use crate::agent::Agent;
pub use crate::com::liveliness::LivelinessSubscriber;
pub use crate::com::liveliness_builder::LivelinessSubscriberBuilder;
pub use crate::com::publisher::Publisher;
pub use crate::com::publisher_builder::PublisherBuilder;
pub use crate::com::query::Query;
pub use crate::com::query_builder::QueryBuilder;
pub use crate::com::queryable::Queryable;
pub use crate::com::queryable_builder::QueryableBuilder;
pub use crate::com::subscriber::Subscriber;
pub use crate::com::subscriber_builder::SubscriberBuilder;
pub use crate::timer::Timer;
pub use crate::timer::TimerBuilder;

pub use dimas_config::Config;
pub use dimas_core::error::{DimasError, Result};
pub use dimas_core::message_types::{Message, Request, Response};
pub use dimas_core::traits::Context;
pub use dimas_core::utils::init_tracing;
// endregeion:  --- modules

// region:    --- types
// Generic wrapper tuple struct for newtype pattern
//#[repr(transparent)]
//pub struct Wrap<T>(pub T);
// endregion: --- types
