//! Copyright © 2023 Stephan Kunz

// region:    --- modules
pub use crate::agent::Agent;
pub use crate::config::Config;
pub use crate::context::Context;
pub use crate::error::{Error, Result};
pub use crate::message::Message;
// endregeion:  --- modules

// region:    --- types
// Generic wrapper tuple struct for newtype pattern
#[repr(transparent)]
pub struct Wrap<T>(pub T);
// endregion: --- types
