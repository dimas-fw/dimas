// Copyright © 2024 Stephan Kunz

//! Core traits of `DiMAS`
//!

// region:		--- modules
use crate::{
	error::Result,
	message_types::{Message, Response},
	task_signal::TaskSignal,
};
use bitcode::{Decode, Encode};
use std::{
	fmt::{Debug, Display},
	sync::{mpsc::Sender, Arc},
};
use zenoh::Session;
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

/// Trait for hooks into management of [`OperationState`]
pub trait OperationStateHooks {
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
}
// endregion:	--- OperationState

// region:		--- Context
/// Typedef for simplified usage
pub type Context<P> = Arc<dyn ContextAbstraction<P>>;

/// Commonalities for the context
pub trait ContextAbstraction<P>: Send + Sync {
	/// Get the name
	#[must_use]
	fn name(&self) -> &Option<String>;

	/// Get the fully qualified name
	#[must_use]
	fn fq_name(&self) -> Option<String>;

	/// Get the [`Context`]s state
	/// # Panics
	#[must_use]
	fn state(&self) -> OperationState;

	/// Set the [`OperationState`].<br>
	/// Setting new state is done step by step
	/// # Errors
	fn set_state(&self, state: OperationState) -> Result<()>;

	/// Get the uuid
	#[must_use]
	fn uuid(&self) -> String;

	/// Get prefix
	#[must_use]
	fn prefix(&self) -> &Option<String>;

	/// Get session mode
	#[must_use]
	fn mode(&self) -> &String;

	/// Get zenoh session reference
	#[must_use]
	fn session(&self) -> Arc<Session>;

	/// Get sender reference
	#[must_use]
	fn sender(&self) -> &Sender<TaskSignal>;

	/// Gives read access to the properties
	///
	/// # Errors
	fn read(&self) -> Result<std::sync::RwLockReadGuard<'_, P>>;

	/// Gives write access to the properties
	///
	/// # Errors
	fn write(&self) -> Result<std::sync::RwLockWriteGuard<'_, P>>;

	/// Method to do an ad hoc publishing for a `topic`
	///
	/// # Errors
	fn put(&self, topic: &str, message: Message) -> Result<()>;

	/// Method to publish data with a stored Publisher
	///
	/// # Errors
	///
	fn put_with(&self, topic: &str, message: Message) -> Result<()>;

	/// Method to do an ad hoc deletion for the `topic`
	///
	/// # Errors
	fn delete(&self, topic: &str) -> Result<()>;

	/// Method to delete data with a stored Publisher
	///
	/// # Errors
	///
	fn delete_with(&self, topic: &str) -> Result<()>;

	/// Send an ad hoc query using the given `topic`.
	/// The `topic` will be enhanced with the group prefix.
	/// # Errors
	fn get(&self, topic: &str, callback: Box<dyn FnMut(Response)>) -> Result<()>;

	/// Method to query data with a stored Query
	///
	/// # Errors
	///
	fn get_with(&self, topic: &str) -> Result<()>;
}
// endregion:	--- Context

// region:		--- Capability
/// Commonalities for capability components
pub trait Capability {
	/// Checks whether state of capability component is appropriate for the given [`OperationState`].
	/// If not, implementation has to adjusts components state to needs.
	/// # Errors
	fn manage_operation_state(&mut self, state: &OperationState) -> Result<()>;
}

/// Commonalities for communication capability components
pub trait CommunicationCapability: Capability {}
// endregion:	--- Capability
