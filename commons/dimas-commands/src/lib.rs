// Copyright © 2024 Stephan Kunz

//! Commands for `DiMAS`

// region:		--- modules
use dimas_com::{
	messages::{AboutEntity, ScoutingEntity},
	Communicator,
};
use dimas_config::Config;
use itertools::Itertools;
use std::collections::HashMap;
use std::time::Duration;
use zenoh::config::WhatAmI;
use zenoh::prelude::sync::*;
// endregion:	--- modules

// region:		--- command functions
/// Fetch a list of about messages from all reachable `DiMAS` entities
/// # Panics
#[must_use]
pub fn about_list(com: &Communicator) -> Vec<AboutEntity> {
	let mut map: HashMap<String, AboutEntity> = HashMap::new();

	let selector = String::from("**/about");

	// fetch about from all entities
	com.get(&selector, |response| {
		let response: AboutEntity = response.decode().expect("decode failed");
		map.entry(response.zid().to_string())
			.or_insert(response);
	})
	.expect("query '**/about failed");

	let result: Vec<AboutEntity> = map.values().sorted().cloned().collect();

	result
}

/// Scout for `DiMAS` entities, sorted by zid of entity
/// # Panics
/// if something goes wrong
#[must_use]
pub fn scouting_list(config: &Config) -> Vec<ScoutingEntity> {
	let mut map: HashMap<String, ScoutingEntity> = HashMap::new();
	let what = WhatAmI::Router | WhatAmI::Peer | WhatAmI::Client;
	let receiver = zenoh::scout(what, config.zenoh_config())
		.res()
		.expect("scouting failed");

	while let Ok(hello) = receiver.recv_timeout(Duration::from_millis(250)) {
		let zid = hello.zid.to_string();
		let entry = ScoutingEntity::new(zid.clone(), hello.whatami.to_string(), hello.locators);
		map.entry(zid).or_insert(entry);
	}
	let result: Vec<ScoutingEntity> = map.values().sorted().cloned().collect();

	result
}

// endregion:	--- command functions
