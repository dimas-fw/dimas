// Copyright © 2024 Stephan Kunz

//! The node 'kingston' publishes a Vector3 value every 100 ms on the topic /yamuna
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
			let message = messages::Vector3::random();
			let _ = ctx.publish("yamuna", message);
			// just to see what value has been sent
			println!("kingston sent vector3");
		})
		.add()?;

	agent.start().await;
	Ok(())
}
