// Copyright © 2024 Stephan Kunz

//! The node 'freeport' publishes an Int64 value every 50 ms on the topic /ganges
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use rand::random;
use std::time::Duration;

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
			let value: i64 = random::<i64>();
			let message = messages::Int64 { data: value };
			let _ = ctx.publish("ganges", message);
			// just to see what value has been sent
			println!("freeport sent: {value:>20}");
		})
		.add()?;

	agent.start().await;
	Ok(())
}
