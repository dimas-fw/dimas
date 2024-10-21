// Copyright © 2023 Stephan Kunz

//! Most commonly used interface of dimas.
//! 
//! Typically it is sufficient to include the prelude with
//! 
//! ```use dimas::prelude::*;```

// to avoid adding these crates to dependencies
pub extern crate bitcode;
pub extern crate tokio;

// bitcode encoding/decoding
pub use bitcode::{Decode, Encode};

// Duration from tokio
pub use tokio::time::Duration;

// zenoh stuff
pub use zenoh::qos::CongestionControl;
pub use zenoh::qos::Priority;
#[cfg(feature = "unstable")]
pub use zenoh::qos::Reliability;
pub use zenoh::query::ConsolidationMode;
pub use zenoh::query::QueryTarget;
#[cfg(feature = "unstable")]
pub use zenoh::sample::Locality;

// dimas stuff
pub use crate::agent::Agent;
pub use dimas_config::Config;
pub use dimas_core::message_types::{
	ControlResponse, Message, ObservableResponse, QueryMsg, QueryableMsg,
};
pub use dimas_core::traits::Context;
pub use dimas_core::utils::init_tracing;
pub use dimas_core::Result;
pub use dimas_macros::main;
pub use dimas_time::Timer;
