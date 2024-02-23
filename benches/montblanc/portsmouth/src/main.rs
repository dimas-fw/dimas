// Copyright © 2024 Stephan Kunz

//! The node 'portsmouth' publishes a String value every 200 ms on the topic /danube
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
	let mut agent = Agent::new(Config::default(), properties);

	agent
		.timer()
		.interval(Duration::from_millis(200))
		.callback(move |ctx, _props| {
			let value = "portsmouth/danube: ".to_string() + &messages::random_string(55);
			let message = messages::StringMsg { data: value };
			let _ = ctx.put("danube", &message);
			// just to see what value has been sent
			info!("sent: '{message}'");
		})
		.add()?;

	agent.start().await;
	Ok(())
}
