// Copyright © 2024 Stephan Kunz

//! The node 'delhi' publishes an Image value every 1 s on the topic /columbia
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
		.interval(Duration::from_secs(1))
		.callback(|ctx, _props| {
			let message = messages::Image::random();
			let height = message.height;
			let width = message.width;
			let id = message.header.frame_id.clone();
			let _ = ctx.publish("columbia", message);
			// just to see what has been sent
			info!("delhi sent: {height:>4} x {width:>4} -> {id}");
		})
		.add()?;

	agent.start().await;
	Ok(())
}
