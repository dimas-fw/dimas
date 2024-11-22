// Copyright © 2024 Stephan Kunz

//! Core enums of `DiMAS`
//!

#[doc(hidden)]
extern crate alloc;

// region:		--- modules
use crate::error::Error;
#[cfg(doc)]
use crate::traits::Operational;
use alloc::vec::Vec;
use alloc::{boxed::Box, string::ToString};
use bitcode::{Decode, Encode};
use core::fmt::{Debug, Display};
// endregion:	--- modules

// region:		--- OperationState
/// The possible states a [`Operational`] entity can take
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
	/// Entity has full situational awareness but does not react
	Standby,
	/// Entity is fully operational
	Active,
}

impl From<&Self> for OperationState {
	fn from(value: &Self) -> Self {
		value.clone()
	}
}

impl TryFrom<&str> for OperationState {
	type Error = Box<dyn core::error::Error + Send + Sync + 'static>;

	fn try_from(
		value: &str,
	) -> core::result::Result<Self, Box<dyn core::error::Error + Send + Sync + 'static>> {
		let v = value.to_lowercase();
		match v.as_str() {
			"created" => Ok(Self::Created),
			"configured" => Ok(Self::Configured),
			"inactive" => Ok(Self::Inactive),
			"standby" => Ok(Self::Standby),
			"active" => Ok(Self::Active),
			_ => Err(Error::UnknownOperationState {
				state: value.to_string(),
			}
			.into()),
		}
	}
}

impl Display for OperationState {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
