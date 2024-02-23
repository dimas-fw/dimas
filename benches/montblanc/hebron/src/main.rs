// Copyright © 2024 Stephan Kunz

//! The node 'hebron' publishes a Quaternion value every 100 ms on the topic /chenab
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use std::time::Duration;
use tracing::info;

#[derive(Debug)]
struct AgentProps {}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt().init();

	let properties = AgentProps {};
	let mut agent = Agent::new(Config::local(), properties);

	agent
		.timer()
		.name("timer")
		.interval(Duration::from_millis(100))
		.callback(|ctx, _props| {
			let message = messages::Quaternion::random();
			let _ = ctx.put("chenab", &message);
			// just to see what value has been sent
			info!("sent: '{message}'");
		})
		.add()?;

	agent.start().await;
	Ok(())
}
