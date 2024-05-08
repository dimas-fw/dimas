// Copyright © 2024 Stephan Kunz
#![allow(clippy::non_canonical_partial_ord_impl)]

//! Commands for `DiMAS`

use std::collections::HashMap;
// region:		--- modules
use derivative::Derivative;
use itertools::Itertools;
use std::fmt::Display;
use std::time::Duration;
use zenoh::config::{Config, WhatAmI};
use zenoh::prelude::sync::*;
// endregion:	--- modules

// region:		--- DimasEntity
/// A `DiMAS` entity
#[derive(Derivative)]
#[derivative(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct DimasEntity {
	name: String,
	zid: String,
	#[derivative(PartialOrd="ignore", Ord="ignore")]
	kind: String,
	#[derivative(PartialOrd="ignore", Ord="ignore")]
	locators: Vec<Locator>,
}

impl Display for DimasEntity {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("DimasEntity")
			.field("zid", &self.zid)
			.field("kind", &self.kind)
			.field("locators", &self.locators)
			.finish()
	}
}

impl DimasEntity {
	/// Fetch the list of reachable entities sorted by name of entity
	/// # Panics
	/// if something goes wrong
	#[must_use]
	pub fn fetch(config: &Config) -> Vec<Self> {
		let mut map: HashMap<String, Self> = HashMap::new();
		let what = WhatAmI::Router | WhatAmI::Peer | WhatAmI::Client;
		let receiver = zenoh::scout(what, config.clone())
			.res()
			.expect("scouting failed");

		while let Ok(hello) = receiver.recv_timeout(Duration::from_millis(250)) {
			let zid = hello.zid.to_string();
			map.entry(zid.clone()).or_insert(Self {
				name: zid.to_string(),
				zid,
				kind: hello.whatami.to_string(),
				locators: hello.locators,
			});
		}
		let result: Vec<Self> = map.values().sorted().cloned().collect();
		
		result
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

	/// Get the Kind 
	#[must_use]
	pub fn kind(&self) -> &str {
		&self.kind
	}
}
// endregion:	--- DimasEntity
