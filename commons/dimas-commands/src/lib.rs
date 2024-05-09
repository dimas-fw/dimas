// Copyright © 2024 Stephan Kunz
#![allow(clippy::non_canonical_partial_ord_impl)]

//! Commands for `DiMAS`

use std::collections::HashMap;
// region:		--- modules
use derivative::Derivative;
use dimas_com::Communicator;
use dimas_config::Config;
use itertools::Itertools;
use std::fmt::Display;
use std::time::Duration;
use zenoh::config::{Locator, WhatAmI};
use zenoh::prelude::sync::*;
// endregion:	--- modules

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
	/// Sout for `DiMAS` entities, sorted by zid of entity
	/// # Panics
	/// if something goes wrong
	#[must_use]
	pub fn scout(config: &Config) -> Vec<Self> {
		let mut map: HashMap<String, Self> = HashMap::new();
		let what = WhatAmI::Router | WhatAmI::Peer | WhatAmI::Client;
		let receiver = zenoh::scout(what, config.zenoh_config())
			.res()
			.expect("scouting failed");

		while let Ok(hello) = receiver.recv_timeout(Duration::from_millis(250)) {
			let zid = hello.zid.to_string();
			map.entry(zid.clone()).or_insert(Self {
				zid,
				kind: hello.whatami.to_string(),
				locators: hello.locators,
			});
		}
		let result: Vec<Self> = map.values().sorted().cloned().collect();

		result
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

// region:		--- about_list
/// Fetch a list of about messages from all reachable `DiMAS` entities
/// # Panics
#[must_use]
pub fn about_list(com: &Communicator) -> Vec<String> {
	let map: HashMap<String, String> = HashMap::new();

	let selector = String::from("**/about");

	// fetch about from all entities
	let replies = com
		.session
		.get(&selector)
		.consolidation(ConsolidationMode::None)
		.target(QueryTarget::All)
		.allowed_destination(Locality::Any)
		.timeout(Duration::from_millis(1000))
		.res()
		.expect("failed to create 'Receiver'");

	while let Ok(reply) = replies.recv() {
		match reply.sample {
			Ok(sample) => println!(
				">> Received ('{}': '{}')",
				sample.key_expr.as_str(),
				sample.value,
			),
			Err(err) => println!(
				">> Received (ERROR: '{}')",
				String::try_from(&err).expect("snh")
			),
		}
	}

	let result: Vec<String> = map.values().sorted().cloned().collect();

	result
}
// endregion:	--- about_list
