// Copyright © 2023 Stephan Kunz


//! Public interface of dimas. Typically it is sufficient to include the prelude with
//! ```use dimas::prelude::*;```

// region:    --- modules
// re-exports
//pub use anyhow::Result;
// anyhows Result
// used std synchronisation primitives
pub use std::sync::Arc;
pub use std::sync::RwLock;
// bitcode encoding/decoding
pub(crate) use bitcode::{decode, encode};
pub use bitcode::{Decode, Encode};
pub extern crate bitcode;

pub use crate::agent::Agent;
pub use crate::com::liveliness_subscriber::{LivelinessSubscriber, LivelinessSubscriberBuilder};
pub use crate::com::message::{Message, Request, Response};
pub use crate::com::publisher::{Publisher, PublisherBuilder};
pub use crate::com::query::{Query, QueryBuilder};
pub use crate::com::queryable::{Queryable, QueryableBuilder};
pub use crate::com::subscriber::{Subscriber, SubscriberBuilder};
pub use crate::config::Config;
pub use crate::context::ArcContext;
pub use crate::error::{DimasError, Result};
pub use crate::timer::{Timer, TimerBuilder};
// endregeion:  --- modules

// region:    --- types
// Generic wrapper tuple struct for newtype pattern
//#[repr(transparent)]
//pub(crate) struct Wrap<T>(pub T);
// endregion: --- types
