// Copyright © 2024 Stephan Kunz
#![allow(clippy::non_canonical_partial_ord_impl)]

//! The `Ping` information of an agent.

#[doc(hidden)]
extern crate alloc;

// region:		--- modules
use alloc::string::String;
use bitcode::{Decode, Encode};
use core::fmt::Display;
// endregion:	--- modules

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
