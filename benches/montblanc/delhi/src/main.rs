// Copyright © 2024 Stephan Kunz

//! The node 'delhi' publishes an Image value every 1 s on the topic /columbia
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use std::time::Duration;

#[derive(Debug)]
struct AgentProps {}

#[tokio::main]
async fn main() -> Result<()> {
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
			println!("delhi sent: {height:>4} x {width:>4} -> {id}");
		})
		.add()?;

	agent.start().await;
	Ok(())
}
