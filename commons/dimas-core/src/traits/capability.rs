// Copyright © 2024 Stephan Kunz
#![allow(unused_imports)]
//! Core traits of `DiMAS`
//!

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use crate::{
	enums::{OperationState, TaskSignal},
	error::Result,
	message_types::{Message, QueryableMsg},
	utils::selector_from,
};
use alloc::{string::String, sync::Arc};
use core::fmt::Debug;
#[cfg(feature = "std")]
use tokio::sync::mpsc::Sender;
use zenoh::Session;
// endregion:	--- modules

// region:		--- Capability
/// Commonalities for capability components
pub trait Capability: Debug {
	/// Checks whether state of capability component is appropriate for the given [`OperationState`].
	/// If not, implementation has to adjusts components state to needs.
	/// # Errors
	fn manage_operation_state(&self, state: &OperationState) -> Result<()>;
}
// endregion:	--- Capability
