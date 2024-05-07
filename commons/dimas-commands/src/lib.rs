// Copyright © 2024 Stephan Kunz

//! Commands for `DiMAS`

// region:		--- modules
use zenoh::config::{Config, WhatAmI};
use zenoh::prelude::sync::*;
// endregion:	--- modules

/// Fetch the list of reachable entities
/// # Panics
/// 
#[must_use]
pub fn list(config: Config) -> String {
	let what = WhatAmI::Router | WhatAmI::Peer | WhatAmI::Client;
	let receiver = zenoh::scout(what, config)
		.res()
    	.expect("scouting failed");

	while let Ok(agent) = receiver.recv() {
		println!("{agent}");
	}

	String::new()
}