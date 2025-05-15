// Copyright © 2024 Stephan Kunz
#![allow(clippy::non_canonical_partial_ord_impl)]

//! The `About` information of an agent.

#[doc(hidden)]
extern crate alloc;

// region:		--- modules
use alloc::string::String;
use bitcode::{Decode, Encode};
use core::fmt::Display;
use dimas_core::enums::OperationState;
// endregion:	--- modules

// region:		--- AboutEntity
/// A `DiMAS` entity
#[repr(C)]
#[derive(Encode, Clone, Decode)]
pub struct AboutEntity {
	name: String,
	kind: String,
	zid: String,
	state: OperationState,
}

impl Display for AboutEntity {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(
			f,
			"name: {} kind: {} state: {} zid: {}",
			&self.name, &self.kind, &self.state, &self.zid
		)
	}
}

impl AboutEntity {
	/// Constructor
	#[must_use]
	pub const fn new(name: String, kind: String, zid: String, state: OperationState) -> Self {
		Self {
			name,
			kind,
			zid,
			state,
		}
	}

	/// Get the Name
	#[must_use]
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Get the Kind
	#[must_use]
	pub fn kind(&self) -> &str {
		&self.kind
	}

	/// Get the Zenoh ID
	#[must_use]
	pub fn zid(&self) -> &str {
		&self.zid
	}

	/// Get the state
	#[must_use]
	pub const fn state(&self) -> &OperationState {
		&self.state
	}
}
// endregion:	--- AboutEntity
