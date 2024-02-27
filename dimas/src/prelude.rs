// Copyright © 2023 Stephan Kunz

// region:    --- modules
// re-exports
// used std synchronisation primitives
pub use std::sync::Arc;
pub use std::sync::RwLock;
// bitcode encoding/decoding
pub use bitcode::{Encode, encode, Decode, decode};
pub extern crate bitcode;

pub use crate::agent::Agent;
pub use crate::com::message::{Message, Request};
pub use crate::com::publisher::{Publisher, PublisherBuilder};
pub use crate::com::query::{Query, QueryBuilder};
pub use crate::com::queryable::{Queryable, QueryableBuilder};
pub use crate::com::subscriber::{Subscriber, SubscriberBuilder};
pub use crate::config::Config;
pub use crate::context::Context;
pub use crate::error::{Error, Result};
pub use crate::timer::{Timer, TimerBuilder};
// endregeion:  --- modules

// region:    --- types
// Generic wrapper tuple struct for newtype pattern
//#[repr(transparent)]
//pub(crate) struct Wrap<T>(pub T);
// endregion: --- types
