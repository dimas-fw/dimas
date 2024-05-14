// Copyright © 2024 Stephan Kunz

//! Core traits of `DiMAS`
//!

// region:		--- modules
use crate::error::Result;
use bitcode::{Decode, Encode};
use std::fmt::Display;
// endregion:	--- modules

// region:		--- OperationState
/// The possible states a `DiMAS` entity can take
#[derive(Debug, Decode, Encode, Clone, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum OperationState {
	/// Entity is in an erronous state
	Error,
	/// Entity is in initial state
	#[default]
	Created,
	/// Entity is setup properly
	Configured,
	/// Entity is listening to important messages only
	Inactive,
	/// Entity has full situational awareness but does
	Standby,
	/// Entity is fully operational
	Active,
}

impl Display for OperationState {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			Self::Error => Display::fmt("error", f),
			Self::Created => Display::fmt("created", f),
			Self::Configured => Display::fmt("configured", f),
			Self::Inactive => Display::fmt("inactive", f),
			Self::Standby => Display::fmt("standby", f),
			Self::Active => Display::fmt("active", f),
		}
	}
}
// endregion:	--- OperationState

// region:		--- ManageState
/// Trait for state management of components
pub trait ManageState {
	/// Checks whether state of component is appropriate for the given [`OperationState`].
	/// If not, adjusts components state to needs.
	/// # Errors
	fn manage_state(&mut self, state: &OperationState) -> Result<()>;
}
// endregion:	--- ManageState
