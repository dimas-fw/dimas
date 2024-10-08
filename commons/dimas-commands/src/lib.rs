// Copyright © 2024 Stephan Kunz

//! Commands for `DiMAS` control & monitoring programs

use chrono::Local;
// region:		--- modules
use core::time::Duration;
use dimas_com::{
	messages::{AboutEntity, PingEntity, ScoutingEntity},
	Communicator,
};
use dimas_config::Config;
use dimas_core::{
	enums::{OperationState, Signal},
	error::Result,
	message_types::Message,
};
use std::collections::HashMap;
use zenoh::{config::WhatAmI, Wait};
// endregion:	--- modules

// region:		--- command functions
/// Fetch a list of about messages from all reachable `DiMAS` entities
/// # Panics
#[must_use]
pub fn about_list(com: &Communicator, base_selector: &String) -> Vec<AboutEntity> {
	let mut map: HashMap<String, AboutEntity> = HashMap::new();

	let selector = format!("{base_selector}/signal");
	let message = Message::encode(&Signal::About);
	// set state for entities matching the selector
	com.get(&selector, Some(message), |response| -> Result<()> {
		let response: AboutEntity = response.decode().expect("decode failed");
		map.entry(response.zid().to_string())
			.or_insert(response);
		Ok(())
	})
	.expect("querying 'about' failed");

	let result: Vec<AboutEntity> = map.values().cloned().collect();

	result
}

/// Fetch a list of about messages from all reachable `DiMAS` entities
/// # Panics
#[must_use]
pub fn ping_list(com: &Communicator, base_selector: &String) -> Vec<(PingEntity, i64)> {
	let mut map: HashMap<String, (PingEntity, i64)> = HashMap::new();

	let selector = format!("{base_selector}/signal");
	let sent = Local::now()
		.naive_utc()
		.and_utc()
		.timestamp_nanos_opt()
		.unwrap_or(0);
	let message = Message::encode(&Signal::Ping { sent });
	// set state for entities matching the selector
	com.get(&selector, Some(message), |response| -> Result<()> {
		let received = Local::now()
			.naive_utc()
			.and_utc()
			.timestamp_nanos_opt()
			.unwrap_or(0);

		let response: PingEntity = response.decode().expect("decode failed");
		let roundtrip = received - sent;
		map.entry(response.zid().to_string())
			.or_insert((response, roundtrip));
		Ok(())
	})
	.expect("querying 'about' failed");

	let result: Vec<(PingEntity, i64)> = map.values().cloned().collect();

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
		.wait()
		.expect("scouting failed");

	while let Ok(hello) = receiver.recv_timeout(Duration::from_millis(250)) {
		let zid = hello.zid().to_string();
		let locators: Vec<String> = hello.locators().iter().map(|locator| {
			locator.to_string()
		}).collect();
		
		let entry = ScoutingEntity::new(
			zid.clone(),
			hello.whatami().to_string(),
			locators,
		);
		map.entry(zid).or_insert(entry);
	}
	let result: Vec<ScoutingEntity> = map.values().cloned().collect();

	result
}

/// Set the [`OperationState`] of `DiMAS` entities
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
	com.get(&selector, Some(message), |response| -> Result<()> {
		let response: AboutEntity = response.decode().expect("decode failed");
		map.entry(response.zid().to_string())
			.or_insert(response);
		Ok(())
	})
	.expect("querying 'state' failed");

	let result: Vec<AboutEntity> = map.values().cloned().collect();

	result
}

/// Shutdown of `DiMAS` entities
/// # Panics
/// if something goes wrong
#[must_use]
pub fn shutdown(com: &Communicator, base_selector: &String) -> Vec<AboutEntity> {
	let mut map: HashMap<String, AboutEntity> = HashMap::new();

	let selector = format!("{base_selector}/signal");
	let message = Message::encode(&Signal::Shutdown);
	// set state for entities matching the selector
	com.get(&selector, Some(message), |response| -> Result<()> {
		let response: AboutEntity = response.decode().expect("decode failed");
		map.entry(response.zid().to_string())
			.or_insert(response);
		Ok(())
	})
	.expect("querying 'state' failed");

	let result: Vec<AboutEntity> = map.values().cloned().collect();

	result
}
// endregion:	--- command functions
