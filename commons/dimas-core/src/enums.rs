// Copyright © 2024 Stephan Kunz

//! Core enums of `DiMAS`
//!

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use crate::error::{DimasError, Result};
use bitcode::{Decode, Encode};
use core::fmt::{Debug, Display};
#[cfg(feature = "std")]
use std::prelude::rust_2021::*;
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
	type Error = Box<dyn core::error::Error + Send + Sync + 'static>;

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

// region:		--- Signal
/// All defined commands of `DiMAS`
#[derive(Debug, Decode, Encode)]
pub enum Signal {
	/// About
	About,
	/// respond to Ping
	Ping {
		/// the utc time coordinate when the request was sent
		sent: i64,
	},
	/// Shutdown application
	Shutdown,
	/// State
	State {
		/// Optional `OperationState` to set
		state: Option<OperationState>,
	},
}
// endregion:	--- Signal

// region:		--- TaskSignal
/// Internal signals, used by panic hooks to inform that someting has happened.
#[derive(Debug, Clone)]
pub enum TaskSignal {
	/// Restart a certain liveliness subscriber, identified by its key expression
	#[cfg(feature = "unstable")]
	RestartLiveliness(String),
	/// Restart a certain observable, identified by its key expression
	RestartObservable(String),
	/// Restart a certain queryable, identified by its key expression
	RestartQueryable(String),
	/// Restart a certain lsubscriber, identified by its key expression
	RestartSubscriber(String),
	/// Restart a certain timer, identified by its key expression
	RestartTimer(String),
	/// Shutdown whole process
	Shutdown,
}
// endregion:	--- TaskSignal
