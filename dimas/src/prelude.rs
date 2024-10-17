// Copyright © 2023 Stephan Kunz

//! simplified interface of dimas.
//! Typically it is sufficient to include the prelude with
//! ```use dimas::prelude::*;```

pub extern crate bitcode;

// region:    --- modules
// re-exports
// Duration for Timers
pub use tokio::time::Duration;
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
pub use crate::builder::LivelinessSubscriberBuilder;
pub use crate::builder::ObservableBuilder;
pub use crate::builder::ObserverBuilder;
pub use crate::builder::PublisherBuilder;
pub use crate::builder::QuerierBuilder;
pub use crate::builder::QueryableBuilder;
pub use crate::builder::SubscriberBuilder;
pub use crate::builder::TimerBuilder;
pub use dimas_time::Timer;

pub use dimas_config::Config;
pub use dimas_core::message_types::{
	ControlResponse, Message, ObservableResponse, QueryMsg, QueryableMsg,
};
pub use dimas_core::traits::Context;
pub use dimas_core::utils::init_tracing;
pub use dimas_core::Result;
// endregeion:  --- modules

// region:    --- types
// Generic wrapper tuple struct for newtype pattern
//#[repr(transparent)]
//pub struct Wrap<T>(pub T);
// endregion: --- types
