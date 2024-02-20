// Copyright © 2024 Stephan Kunz

//! The node 'freeport' publishes an Int64 value every 50 ms on the topic /ganges
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use std::time::Duration;
use tracing::info;

#[derive(Debug)]
struct AgentProps {}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

	let properties = AgentProps {};
	let mut agent = Agent::new(Config::default(), properties);

	agent
		.timer()
		.interval(Duration::from_millis(50))
		.callback(|ctx, _props| {
			let message = messages::Int64::random();
			let value = message.data;
			let _ = ctx.publish("ganges", message);
			// just to see what value has been sent
			info!("freeport sent: {value:>20}");
		})
		.add()?;

	agent.start().await;
	Ok(())
}
