// Copyright © 2024 Stephan Kunz

//! The node 'kingston' publishes a Vector3 value every 100 ms on the topic /yamuna
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use std::time::Duration;
use tracing::info;

#[derive(Debug)]
struct AgentProps {}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt().init();

	let properties = AgentProps {};
	let mut agent = Agent::new(Config::local(), properties);

	agent.publisher().msg_type("yamuna").add()?;

	agent
		.timer()
		.name("timer")
		.interval(Duration::from_millis(100))
		.callback(|ctx, _props| {
			let message = messages::Vector3::random();
			let _ = ctx.put_with("yamuna", &message);
			// just to see what value has been sent
			info!("sent: '{message}'");
		})
		.add()?;

	agent.start().await;
	Ok(())
}
