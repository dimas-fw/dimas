//! Copyright © 2023 Stephan Kunz

// export crates Error type
pub use crate::error::{Error, Result};
// Generic wrapper tuple struct for newtype pattern
pub struct Wrap<T>(pub T);

// public interface of library
pub use crate::agent::Agent;
pub use crate::timer::Repetition;
pub use crate::timer::Timer;
