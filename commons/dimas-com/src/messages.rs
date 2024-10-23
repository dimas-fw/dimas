// Copyright © 2024 Stephan Kunz
#![allow(clippy::non_canonical_partial_ord_impl)]

//! Module `messages` provides the different Message`s used with DiMAS.

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
#[doc(hidden)]
use alloc::{string::String, vec::Vec};
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

// region:		--- PingEntity
/// A `DiMAS` entity
#[repr(C)]
#[derive(Encode, Clone, Decode)]
pub struct PingEntity {
	name: String,
	zid: String,
	oneway: i64,
}

impl Display for PingEntity {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(
			f,
			"name: {} zid: {} oneway: {}",
			&self.name, &self.zid, &self.oneway
		)
	}
}

impl PingEntity {
	/// Constructor
	#[must_use]
	pub const fn new(name: String, zid: String, oneway: i64) -> Self {
		Self { name, zid, oneway }
	}

	/// Get the Name
	#[must_use]
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Get the Zenoh ID
	#[must_use]
	pub fn zid(&self) -> &str {
		&self.zid
	}

	/// Get the oneway time
	#[must_use]
	pub const fn oneway(&self) -> i64 {
		self.oneway
	}
}
// endregion:	--- PingEntity

// region:		--- ScoutingEntity
/// A `Zenoh` entity
#[repr(C)]
#[derive(Encode, Clone, Decode)]
pub struct ScoutingEntity {
	zid: String,
	kind: String,
	locators: Vec<String>,
}

impl Display for ScoutingEntity {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("ScoutingEntity")
			.field("zid", &self.zid)
			.field("kind", &self.kind)
			.field("locators", &self.locators)
			.finish()
	}
}

impl ScoutingEntity {
	/// Constructor
	#[must_use]
	pub const fn new(zid: String, kind: String, locators: Vec<String>) -> Self {
		Self {
			zid,
			kind,
			locators,
		}
	}

	/// Get the Zenoh ID
	#[must_use]
	pub fn zid(&self) -> &str {
		&self.zid
	}

	/// Get the Kind
	#[must_use]
	pub fn kind(&self) -> &str {
		&self.kind
	}

	/// Get the Locators
	#[must_use]
	pub const fn locators(&self) -> &Vec<String> {
		&self.locators
	}
}
// endregion:	--- ScoutingEntity

// region:		--- ???
// endregion:	--- ???
