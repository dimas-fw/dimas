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
pub use zenoh::core::Priority;
pub use zenoh::publisher::CongestionControl;
pub use zenoh::query::ConsolidationMode;
pub use zenoh::query::QueryTarget;
pub use zenoh::sample::Locality;
pub use zenoh::subscriber::Reliability;

// dimas stuff
pub use crate::agent::Agent;
pub use crate::builder::liveliness::LivelinessSubscriberBuilder;
pub use crate::builder::observable::ObservableBuilder;
pub use crate::builder::observer::ObserverBuilder;
pub use crate::builder::publisher::PublisherBuilder;
pub use crate::builder::query::QueryBuilder;
pub use crate::builder::queryable::QueryableBuilder;
pub use crate::builder::subscriber::SubscriberBuilder;
pub use crate::builder::timer::TimerBuilder;
pub use crate::com::liveliness::LivelinessSubscriber;
pub use crate::com::observable::Observable;
pub use crate::com::observer::Observer;
pub use crate::com::publisher::Publisher;
pub use crate::com::query::Query;
pub use crate::com::queryable::Queryable;
pub use crate::com::subscriber::Subscriber;
pub use crate::timer::Timer;

pub use dimas_config::Config;
pub use dimas_core::error::{DimasError, Result};
pub use dimas_core::message_types::{
	Message, ObservableMsg, ObservableResponse, QueryMsg, QueryableMsg,
};
pub use dimas_core::traits::Context;
pub use dimas_core::utils::init_tracing;
// endregeion:  --- modules

// region:    --- types
// Generic wrapper tuple struct for newtype pattern
//#[repr(transparent)]
//pub struct Wrap<T>(pub T);
// endregion: --- types
