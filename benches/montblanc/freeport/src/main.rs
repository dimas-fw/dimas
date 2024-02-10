// Copyright © 2024 Stephan Kunz

//! The node 'freeport' publishes an int64 value every 50 ms on the topic /ganges
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
			let value = rng.gen_range(-999_999_999_999_999_999i64..999_999_999_999_999_999i64);
			let _ = ctx.publish("ganges", value);
			// just to see what value has been sent
			println!("freeport sent: {value:>20}");
		})
		.add()?;

	agent.start().await;
	Ok(())
}
