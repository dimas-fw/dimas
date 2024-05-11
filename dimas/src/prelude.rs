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
pub use crate::com::liveliness::{LivelinessSubscriber, LivelinessSubscriberBuilder};
pub use crate::com::publisher::{Publisher, PublisherBuilder};
pub use crate::com::query::{Query, QueryBuilder};
pub use crate::com::queryable::{Queryable, QueryableBuilder};
pub use crate::com::subscriber::{Subscriber, SubscriberBuilder};
pub use crate::context::ArcContext;
pub use crate::timer::{Timer, TimerBuilder};

pub use dimas_com::{Message, Request, Response};
pub use dimas_config::Config;
pub use dimas_core::error::{DimasError, Result};
pub use dimas_core::utils::init_tracing;
// endregeion:  --- modules

// region:    --- types
// Generic wrapper tuple struct for newtype pattern
//#[repr(transparent)]
//pub(crate) struct Wrap<T>(pub T);
// endregion: --- types
