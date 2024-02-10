// Copyright © 2024 Stephan Kunz

//! The node 'cordoba' publishes a float32 value every 100 ms on the topic /amazon
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use std::time::Duration;
use rand::random;

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
			let value: f32 = random::<f32>() * 1_000_000.0;
			let _ = ctx.publish("amazon", value);
			// just to see what value has been sent
			println!("cordoba sent: {value:>14.6}");
		})
		.add()?;

	agent.start().await;
	Ok(())
}
