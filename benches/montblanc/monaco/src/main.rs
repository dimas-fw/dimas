// Copyright © 2024 Stephan Kunz

//! The node 'monaco' subscribes to
//!   - a `Twist` on the topic /congo
//! and publishes on receive
//!   - a `Float32` on the topic /ohio
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use std::sync::{Arc, RwLock};

use dimas::prelude::*;

#[derive(Debug, Default)]
struct AgentProps {}

fn congo_callback(ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::Twist = bitcode::decode(message).expect("should not happen");
	println!("monaco received Twist");
	let msg = messages::Float32::random();
	let _ = ctx.publish("ohio", &msg);
	println!("monaco sent: {:>14.6}", msg.data);
}

#[tokio::main]
async fn main() -> Result<()> {
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
