// Copyright © 2024 Stephan Kunz

//! The node 'monaco' subscribes to
//!   - a `Twist` on the topic /congo
//! and publishes on receive
//!   - a `Float32` on the topic /ohio
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use std::sync::{Arc, RwLock};
use tracing::info;

#[derive(Debug, Default)]
struct AgentProps {}

fn congo_callback(ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Twist = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	let msg = messages::Float32::random();
	let _ = ctx.put("ohio", &msg);
	info!("sent: '{msg}'");
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt().init();

	let properties = AgentProps::default();
	let mut agent = Agent::new(Config::default(), properties);

	agent
		.subscriber()
		.put_callback(congo_callback)
		.msg_type("congo")
		.add()?;

	agent.start().await;
	Ok(())
}
