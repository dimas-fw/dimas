// Copyright © 2024 Stephan Kunz

//! Core enums of `DiMAS`
//!

// region:		--- modules
use crate::error::{DimasError, Result};
use bitcode::{Decode, Encode};
use std::fmt::{Debug, Display};
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
		let v = value.to_lowercase();
		match v.as_str() {
			"created" => Ok(Self::Created),
			"configured" => Ok(Self::Configured),
			"inactive" => Ok(Self::Inactive),
			"standby" => Ok(Self::Standby),
			"active" => Ok(Self::Active),
			_ => Err(DimasError::OperationState(value.to_string()).into()),
		}
	}
}

impl Display for OperationState {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Error => write!(f, "Error"),
			Self::Created => write!(f, "Created"),
			Self::Configured => write!(f, "Configured"),
			Self::Inactive => write!(f, "Inactive"),
			Self::Standby => write!(f, "Standby"),
			Self::Active => write!(f, "Active"),
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
	/// Shutdown application
	Shutdown,
	/// State
	State {
		/// Optional OperationState to set
		state: Option<OperationState>,
	},
}
// endregion:	--- Signal
