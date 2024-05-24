// Copyright © 2024 Stephan Kunz

//! Core traits of `DiMAS`
//!

// region:		--- modules
use crate::{
	enums::OperationState,
	error::Result,
	message_types::{Message, Response},
	task_signal::TaskSignal,
};
use std::{
	fmt::Debug,
	sync::{mpsc::Sender, Arc},
};
use zenoh::Session;
// endregion:	--- modules

// region:		--- Context
/// Typedef for simplified usage
pub type Context<P> = Arc<dyn ContextAbstraction<P>>;

/// Commonalities for the context
pub trait ContextAbstraction<P>: Debug + Send + Sync {
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

	/// Method to do an ad hoc deletion for the `topic`
	///
	/// # Errors
	fn delete(&self, topic: &str) -> Result<()>;

	/// Send an ad hoc query using the given `topic` with an optional [`Message`].
	/// The `topic` will be enhanced with the group prefix.
	/// # Errors
	fn get(
		&self,
		topic: &str,
		message: Option<&Message>,
		callback: Box<dyn FnMut(Response)>,
	) -> Result<()>;

	/// Method to query data with a stored Query and an optional [`Message`]
	///
	/// # Errors
	///
	fn get_with(&self, topic: &str, message: Option<&Message>) -> Result<()>;
}
// endregion:	--- Context

// region:		--- Capability
/// Commonalities for capability components
pub trait Capability: Debug {
	/// Checks whether state of capability component is appropriate for the given [`OperationState`].
	/// If not, implementation has to adjusts components state to needs.
	/// # Errors
	fn manage_operation_state(&mut self, state: &OperationState) -> Result<()>;
}
// endregion:	--- Capability
