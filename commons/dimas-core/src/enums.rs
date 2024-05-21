// Copyright © 2024 Stephan Kunz

//! Core enums of `DiMAS`
//!

// region:		--- modules
use bitcode::{Decode, Encode};
use std::fmt::{Debug, Display};
use crate::error::{DimasError, Result};
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

impl TryFrom<&str> for OperationState {
	type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

	fn try_from(value: &str) -> Result<Self> {
		match value {
			"Created" | "created" => Ok(Self::Created),
			"Configured" | "configured" => Ok(Self::Configured),
			"Inactive" | "inactive" => Ok(Self::Inactive),
			"Standby" | "standby" => Ok(Self::Standby),
			"Active" | "active" => Ok(Self::Active),
			_ => Err(DimasError::OperationState(value.to_string()).into()),
		}
	}
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

// region:		--- Signal
/// All defined commands of `DiMAS`
#[derive(Debug, Decode, Encode)]
pub enum Signal {
	/// About
	About,
	/// State
	State {
		/// Optional OperationState to set
		state: Option<OperationState>,
	},
	/// Allows better implementation of new signals, must be last!
	Unknown
}
// endregion:	--- Signal

