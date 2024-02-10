// Copyright © 2024 Stephan Kunz

//! The node 'lyon' subscribes to a Float32 on the topic /amazon and publishes the received value on topic /tigris
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use std::sync::{Arc, RwLock};

use dimas::prelude::*;

#[derive(Debug)]
struct AgentProps {}

fn amazon_callback(ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: f32 = bitcode::decode(message).expect("should not happen");
	let _ = ctx.publish("tigris", value);
	println!("lyon propagates: {value:>14.6}");
}

#[tokio::main]
async fn main() -> Result<()> {
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
