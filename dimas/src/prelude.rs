// Copyright © 2023 Stephan Kunz

//! Public interface of dimas. Typically it is sufficient to include the prelude with
//! ```use dimas::prelude::*;```

pub extern crate bitcode;

// region:    --- modules
// re-exports
// Duration for Timers
pub use core::time::Duration;
// used std synchronisation primitives
pub use std::sync::{Arc, RwLock};

// bitcode encoding/decoding
pub use bitcode::{Decode, Encode};
// zenoh stuff
pub use zenoh::qos::CongestionControl;
pub use zenoh::qos::Priority;
pub use zenoh::query::ConsolidationMode;
pub use zenoh::query::QueryTarget;
#[cfg(feature = "unstable")]
pub use zenoh::sample::Locality;

// dimas stuff
pub use crate::agent::Agent;
#[cfg(feature = "unstable")]
pub use crate::builder::liveliness::LivelinessSubscriberBuilder;
pub use crate::builder::observable::ObservableBuilder;
pub use crate::builder::observer::ObserverBuilder;
pub use crate::builder::publisher::PublisherBuilder;
pub use crate::builder::querier::QuerierBuilder;
pub use crate::builder::queryable::QueryableBuilder;
pub use crate::builder::subscriber::SubscriberBuilder;
pub use crate::builder::timer::TimerBuilder;
#[cfg(feature = "unstable")]
pub use crate::com::liveliness::LivelinessSubscriber;
pub use crate::com::observable::Observable;
pub use crate::com::observer::Observer;
pub use crate::com::publisher::Publisher;
pub use crate::com::querier::Querier;
pub use crate::com::queryable::Queryable;
pub use crate::com::subscriber::Subscriber;
pub use crate::timer::Timer;

pub use dimas_config::Config;
pub use dimas_core::error::{DimasError, Result};
pub use dimas_core::message_types::{
	ControlResponse, Message, ObservableResponse, QueryMsg, QueryableMsg,
};
pub use dimas_core::traits::Context;
pub use dimas_core::utils::init_tracing;
// endregeion:  --- modules

// region:    --- types
// Generic wrapper tuple struct for newtype pattern
//#[repr(transparent)]
//pub struct Wrap<T>(pub T);
// endregion: --- types
