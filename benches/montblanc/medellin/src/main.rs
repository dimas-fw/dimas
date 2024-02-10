// Copyright © 2024 Stephan Kunz

//! The node 'medellin' publishes an int32 value every 10 ms on the topic /nilnilee
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use std::time::Duration;
use rand::Rng;

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
			let mut rng = rand::thread_rng();
			let value = rng.gen_range(-999_999_999i32..999_999_999i32);
			let _ = ctx.publish("ganges", value);
			// just to see what value has been sent
			println!("medellin sent: {value:>11}");
		})
		.add()?;

	agent.start().await;
	Ok(())
}
