// Copyright © 2024 Stephan Kunz

//! The node 'medellin' publishes an Int32 value every 10 ms on the topic /nilnilee
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
		.interval(Duration::from_millis(50))
		.callback(|ctx, _props| {
			let value: i32 = random::<i32>();
			let _ = ctx.publish("nile", value);
			// just to see what value has been sent
			println!("medellin sent: {value:>12}");
		})
		.add()?;

	agent.start().await;
	Ok(())
}
