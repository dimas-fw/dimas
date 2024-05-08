// Copyright © 2024 Stephan Kunz

//! Commands for `DiMAS`

// region:		--- modules
use std::fmt::Display;
use zenoh::config::{Config, WhatAmI};
use zenoh::prelude::sync::*;
// endregion:	--- modules

// region:		--- DimasEntity
/// List of reachable entities
#[derive(Debug)]
pub struct DimasEntity {
	zid: String,
	kind: String,
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
	pub fn fetch(config: Config) -> String {
		let what = WhatAmI::Router | WhatAmI::Peer | WhatAmI::Client;
		let receiver = zenoh::scout(what, config)
			.res()
			.expect("scouting failed");

		while let Ok(hello) = receiver.recv() {
			//dbg!(&hello);
			let entity = Self {
				zid: hello.zid.to_string(),
				kind: hello.whatami.to_string(),
				locators: hello.locators,
			};
			println!("{entity}");
		}

		String::new()
	}
}
// endregion:	--- DimasEntity
