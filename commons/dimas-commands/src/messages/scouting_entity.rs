// Copyright © 2024 Stephan Kunz
#![allow(clippy::non_canonical_partial_ord_impl)]

//! The `Scouting` information of an agent.

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use alloc::{string::String, vec::Vec};
use bitcode::{Decode, Encode};
use core::fmt::Display;
// endregion:	--- modules

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
