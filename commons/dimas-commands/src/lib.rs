// Copyright © 2024 Stephan Kunz

//! Commands for `DiMAS` control & monitoring programs

// region:		--- modules
use dimas_com::{
	messages::{AboutEntity, ScoutingEntity},
	Communicator,
};
use dimas_config::Config;
use dimas_core::{enums::{OperationState, Signal}, message_types::Message};
use itertools::Itertools;
use std::{collections::HashMap, time::Duration};
use zenoh::{config::WhatAmI, prelude::sync::*};
// endregion:	--- modules

// region:		--- command functions
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

/// Fetch a list of about messages from all reachable `DiMAS` entities
/// # Panics
#[must_use]
pub fn about_list(com: &Communicator, base_selector: &String) -> Vec<AboutEntity> {
	let mut map: HashMap<String, AboutEntity> = HashMap::new();

	let selector = format!("{base_selector}/signal");
	let message = Message::encode(&Signal::About);
	// set state for entities matching the selector
	com.get_with_value(&selector, &message, |response| {
		let response: AboutEntity = response.decode().expect("decode failed");
		map.entry(response.zid().to_string())
			.or_insert(response);
	})
	.expect("querying 'about' failed");

	let result: Vec<AboutEntity> = map.values().sorted().cloned().collect();

	result
}

/// Set the [`OperationState`] of a `DiMAS` entities
/// # Panics
/// if something goes wrong
#[must_use]
pub fn set_state(
	com: &Communicator,
	base_selector: &String,
	state: Option<OperationState>,
) -> Vec<AboutEntity> {
	let mut map: HashMap<String, AboutEntity> = HashMap::new();

	let selector = format!("{base_selector}/signal");
	let message = Message::encode(&Signal::State { state });
	// set state for entities matching the selector
	com.get_with_value(&selector, &message, |response| {
		let response: AboutEntity = response.decode().expect("decode failed");
		map.entry(response.zid().to_string())
			.or_insert(response);
	})
	.expect("querying 'state' failed");

	let result: Vec<AboutEntity> = map.values().sorted().cloned().collect();

	result
}
// endregion:	--- command functions
