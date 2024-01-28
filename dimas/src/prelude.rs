// Copyright © 2023 Stephan Kunz

// region:    --- modules
pub use crate::agent::Agent;
pub use crate::config::Config;
pub use crate::context::Context;
pub use crate::error::{Error, Result};
#[cfg(feature = "queryable")]
pub use crate::com::queryable::Request;
// endregeion:  --- modules

// region:    --- types
// Generic wrapper tuple struct for newtype pattern
//#[repr(transparent)]
//pub(crate) struct Wrap<T>(pub T);
// endregion: --- types
