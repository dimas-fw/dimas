// Copyright © 2024 Stephan Kunz

//! The node 'taipei' subscribes to an Image on the topic /columbia and publishes the received value on topic /colorado
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use tracing::info;

#[derive(Debug)]
struct AgentProps {}

fn columbia_callback(
	ctx: &Arc<Context<AgentProps>>,
	_props: &Arc<RwLock<AgentProps>>,
	message: &[u8],
) {
	let mut value: messages::Image = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	value.header.frame_id = value.header.frame_id.replace("Test", "Modified");
	let _ = ctx.put_with("colorado", &value);
	info!("received: '{}'", value);
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt().init();

	let properties = AgentProps {};
	let mut agent = Agent::new(Config::local(), properties);

	agent.publisher().msg_type("colorado").add()?;

	agent
		.subscriber()
		.put_callback(columbia_callback)
		.msg_type("columbia")
		.add()?;

	agent.start().await;
	Ok(())
}
