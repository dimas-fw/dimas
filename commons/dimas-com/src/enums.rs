// Copyright © 2024 Stephan Kunz

#[doc(hidden)]
extern crate alloc;

use alloc::sync::Arc;
use zenoh::Session;

/// the known implementations of communicators
#[derive(Debug)]
pub enum CommunicatorImplementation {
	/// zenoh
	Zenoh(crate::zenoh::Communicator),
	/// undefined
	Undefined,
}

impl CommunicatorImplementation {
	/// extract session
	#[must_use]
	#[allow(clippy::match_wildcard_for_single_variants)]
	pub fn session(&self) -> Option<Arc<Session>> {
		match self {
			Self::Zenoh(communicator) => Some(communicator.session()),
			_ => None,
		}
	}
}
