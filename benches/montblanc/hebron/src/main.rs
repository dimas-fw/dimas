// Copyright © 2024 Stephan Kunz

//! The node 'hebron' publishes a Quaternion value every 100 ms on the topic /chenab
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use std::time::Duration;

#[derive(Debug)]
struct AgentProps {}

#[tokio::main]
async fn main() -> Result<()> {
	let properties = AgentProps {};
	let mut agent = Agent::new(Config::default(), properties);

	agent
		.timer()
		.interval(Duration::from_millis(100))
		.callback(|ctx, _props| {
			let message = messages::Quaternion::random();
			let _ = ctx.publish("chenab", message);
			// just to see what value has been sent
			println!("hebron sent quaternion");
		})
		.add()?;

	agent.start().await;
	Ok(())
}
