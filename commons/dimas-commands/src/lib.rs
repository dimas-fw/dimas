// Copyright © 2024 Stephan Kunz
#![allow(clippy::non_canonical_partial_ord_impl)]

//! Commands for `DiMAS`

use std::collections::HashMap;
// region:		--- modules
use std::fmt::Display;
use std::time::Duration;
use derivative::Derivative;
use zenoh::config::{Config, WhatAmI};
use zenoh::prelude::sync::*;
// endregion:	--- modules

// region:		--- DimasEntity
/// List of reachable entities
#[derive(Derivative)]
#[derivative(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct DimasEntity {
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
	/// Fetch the list of reachable entities
	/// # Panics
	///
	#[must_use]
	pub fn fetch(config: &Config) -> Vec<Self> {
		let mut map: HashMap<String, Self> = HashMap::new();
		let what = WhatAmI::Router | WhatAmI::Peer | WhatAmI::Client;
		let receiver = zenoh::scout(what, config.clone())
			.res()
			.expect("scouting failed");

		while let Ok(hello) = receiver.recv_timeout(Duration::from_millis(250)) {
			let zid = hello.zid.to_string();
			map.insert(zid.clone(), Self {
				zid,
				kind: hello.whatami.to_string(),
				locators: hello.locators,
			});
		}
		//dbg!(&map);
		let mut result: Vec<Self> = map.values().cloned().collect(); 
		result.sort();
		//dbg!(&result);
		
		result
	}

	/// Get the Zenoh ID
	#[must_use]
	pub fn zid(&self) -> &str {
		&self.zid
	}

	/// Get the kind 
	#[must_use]
	pub fn kind(&self) -> &str {
		&self.kind
	}
}
// endregion:	--- DimasEntity
