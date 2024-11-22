// Copyright © 2024 Stephan Kunz

//! Capability interface for `DiMAS`
//!

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use alloc::boxed::Box;
use anyhow::Result;
use core::fmt::Debug;

use crate::error::Error;

use super::CapabilityDescription;
// endregion:	--- modules

// region:		--- Capability
/// contract for a `Capability`
pub trait Capability: Debug {
    /// get description
    /// # Errors
    /// if function is not implemented
    /// if no description is set
    fn description(&self) -> Result<Box<dyn CapabilityDescription>> {
        let err = Error::NotImplemented.into();
        Err(err)
    }

}
// endregion:   --- Capability