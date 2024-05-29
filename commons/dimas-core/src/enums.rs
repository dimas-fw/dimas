// Copyright © 2024 Stephan Kunz

//! Core enums of `DiMAS`
//!

// region:		--- modules
use crate::error::{DimasError, Result};
use bitcode::{Decode, Encode};
use std::{
	fmt::{Debug, Display},
	sync::{mpsc::Receiver, Mutex},
	time::Duration,
};
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
	/// respond to Ping
	Ping {
		/// the utc time coordinate when the request was sent
		sent: i64,
	},
	/// Shutdown application
	Shutdown,
	/// State
	State {
		/// Optional OperationState to set
		state: Option<OperationState>,
	},
}
// endregion:	--- Signal

// region:		--- TaskSignal
/// Internal signals, used by panic hooks to inform that someting has happened.
#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum TaskSignal {
	/// Restart a certain liveliness subscriber, identified by its key expression
	RestartLiveliness(String),
	/// Restart a certain queryable, identified by its key expression
	RestartQueryable(String),
	/// Restart a certain lsubscriber, identified by its key expression
	RestartSubscriber(String),
	/// Restart a certain timer, identified by its key expression
	RestartTimer(String),
	/// Shutdown whole process
	Shutdown,
}

/// Wait non-blocking for [`TaskSignal`]s.<br>
/// # Panics
pub async fn wait_for_task_signals(rx: &Mutex<Receiver<TaskSignal>>) -> Box<TaskSignal> {
	loop {
		if let Ok(signal) = rx.lock().expect("snh").try_recv() {
			return Box::new(signal);
		};
		// TODO: maybe there is a better solution than sleep
		tokio::time::sleep(Duration::from_millis(1)).await;
	}
}
// endregion:	--- TaskSignal
