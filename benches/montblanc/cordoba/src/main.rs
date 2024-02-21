// Copyright © 2024 Stephan Kunz

//! The node 'cordoba' publishes a Float32 value every 100 ms on the topic /amazon
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
	let mut agent = Agent::new(Config::local(), properties);

	agent
		.timer()
		.interval(Duration::from_millis(100))
		.callback(|ctx, _props| {
			let message = messages::Float32::random();
			let _ = ctx.publish("amazon", &message);
			// just to see what value has been sent
			info!("sent: '{message}'");
		})
		.add()?;

	agent.start().await;
	Ok(())
}
