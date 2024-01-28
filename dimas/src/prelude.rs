// Copyright © 2023 Stephan Kunz

// region:    --- modules
pub use crate::agent::Agent;
#[cfg(feature = "queryable")]
pub use crate::com::queryable::Request;
pub use crate::config::Config;
pub use crate::context::Context;
pub use crate::error::{Error, Result};
// endregeion:  --- modules

// region:    --- types
// Generic wrapper tuple struct for newtype pattern
//#[repr(transparent)]
//pub(crate) struct Wrap<T>(pub T);
// endregion: --- types
