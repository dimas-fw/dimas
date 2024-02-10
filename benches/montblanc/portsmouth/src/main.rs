// Copyright © 2024 Stephan Kunz

//! The node 'portsmouth' publishes a String value every 200 ms on the topic /danube
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

	let values = [
		"Another one bites the dust",
		"The quick brown fox jumps over the fence",
		"To be or not to be"
	];
	let mut index = 0;

	agent
		.timer()
		.interval(Duration::from_millis(200))
		.callback(move |ctx, _props| {
			let value = values[index].to_string();
			let _ = ctx.publish("danube", &value);
			// just to see what value has been sent
			println!("freeport sent: {value}");
			index = (index + 1) % 3;
		})
		.add()?;

	agent.start().await;
	Ok(())
}
