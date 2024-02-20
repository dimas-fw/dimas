// Copyright © 2024 Stephan Kunz

//! The node 'lyon' subscribes to a Float32 on the topic /amazon and publishes the received value on topic /tigris
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use std::sync::{Arc, RwLock};
use tracing::info;

#[derive(Debug)]
struct AgentProps {}

fn amazon_callback(ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Float32 = bitcode::decode(message).expect("should not happen");
	let msg = value.data;
	let _ = ctx.publish("tigris", value);
	info!("lyon propagates: {msg:>14.6}");
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

	let properties = AgentProps {};
	let mut agent = Agent::new(Config::default(), properties);

	agent
		.subscriber()
		.put_callback(amazon_callback)
		.msg_type("amazon")
		.add()?;

	agent.start().await;
	Ok(())
}
