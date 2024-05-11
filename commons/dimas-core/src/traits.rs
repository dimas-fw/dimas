// Copyright © 2024 Stephan Kunz

//! Core traits of `DiMAS`
//!

// region:		--- modules
use crate::error::Result;
use bitcode::{Decode, Encode};
use std::fmt::Display;
// endregion:	--- modules

// region:		--- StateTransistion
/// Trait for transitions between [`OperationState`]s
pub trait StateTransistion {
	/// Transition for unrecovable Error
	/// # Panics
	fn perish(&mut self) -> ! {
		panic!("recovery not defined/possible, process exits")
	}
	/// Transition from Error to Created
	/// Default implementation dies
	/// # Errors
	fn recover(&mut self, wanted: &OperationState) -> Result<()> {
		let _ = wanted;
		self.perish()
	}
	/// Transition from any state to Error
	/// Default implementation tries to recover
	/// # Errors
	fn error(&mut self, state: &OperationState) -> Result<()> {
		self.recover(state)
	}

	/// Transition from Created to Configured
	/// Default implementation does nothing
	/// # Errors
	fn configure(&mut self) -> Result<()> {
		Ok(())
	}
	/// Transition from Configured to Created
	/// Default implementation does nothing
	/// # Errors
	fn deconfigure(&mut self) -> Result<()> {
		Ok(())
	}

	/// Transition from Configured to Inactive
	/// Default implementation does nothing
	/// # Errors
	fn attend(&mut self) -> Result<()> {
		Ok(())
	}
	/// Transtion from Inactive to Configured
	/// Default implementation does nothing
	/// # Errors
	fn deattend(&mut self) -> Result<()> {
		Ok(())
	}

	/// Transition from Inactive to Standby
	/// # Errors
	/// Default implementation does nothing
	fn standby(&mut self) -> Result<()> {
		Ok(())
	}
	/// Transition from Standby to Inactive
	/// # Errors
	/// Default implementation does nothing
	fn destandby(&mut self) -> Result<()> {
		Ok(())
	}

	/// Transition from Standby to Active
	/// Default implementation does nothing
	/// # Errors
	fn activate(&mut self) -> Result<()> {
		Ok(())
	}
	/// Transition from Active to Standby
	/// Default implementation does nothing
	/// # Errors
	fn deactivate(&mut self) -> Result<()> {
		Ok(())
	}
}
// endregion:	--- StateTransistion

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
