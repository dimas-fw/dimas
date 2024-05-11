// Copyright © 2024 Stephan Kunz
#![allow(clippy::non_canonical_partial_ord_impl)]

//! Module `messages` provides the different Message`s used with DiMAS.

use bitcode::{Decode, Encode};
// region:		--- modules
use derivative::Derivative;
use std::fmt::Display;
use zenoh::config::Locator;
// endregion:	--- modules

// region:		--- AboutEntity
/// A `DiMAS` entity
#[derive(Encode, Decode, Derivative)]
#[derivative(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct AboutEntity {
	name: String,
	kind: String,
	zid: String,
}

impl Display for AboutEntity {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ScoutingEntity")
			.field("name", &self.name)
			.field("kind", &self.kind)
			.field("zid", &self.zid)
			.finish()
	}
}

impl AboutEntity {
	/// Constructor
	#[must_use]
	pub const fn new(name: String, kind: String, zid: String) -> Self {
		Self { name, kind, zid }
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
}
// endregion:	--- AboutEntity

// region:		--- ScoutingEntity
/// A `Zenoh` entity
#[derive(Derivative)]
#[derivative(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct ScoutingEntity {
	zid: String,
	kind: String,
	#[derivative(PartialOrd = "ignore", Ord = "ignore")]
	locators: Vec<Locator>,
}

impl Display for ScoutingEntity {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
	pub fn new(zid: String, kind: String, locators: Vec<Locator>) -> Self {
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
	pub const fn locators(&self) -> &Vec<Locator> {
		&self.locators
	}
}
// endregion:	--- ScoutingEntity