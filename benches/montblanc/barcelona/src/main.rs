// Copyright © 2024 Stephan Kunz

//! The node 'barcelona' subscribes to
//!   - a `TwistWithCovarianceStamed` on the topic /mekong
//! and publishes on receive
//!   - a `WrenchStamped` on the topic /lena
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use std::sync::{Arc, RwLock};

use dimas::prelude::*;

#[derive(Debug, Default)]
struct AgentProps {}

fn mekong_callback(ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::TwistWithCovarianceStamped =
		bitcode::decode(message).expect("should not happen");
	println!("barcelona received Twist");
	let msg = messages::WrenchStamped::random();
	let _ = ctx.publish("lena", msg);
	println!("barcelona sent WrenchStamped");
}

#[tokio::main]
async fn main() -> Result<()> {
	let properties = AgentProps::default();
	let mut agent = Agent::new(Config::default(), properties);

	agent
		.subscriber()
		.put_callback(mekong_callback)
		.msg_type("mekong")
		.add()?;

	agent.start().await;
	Ok(())
}
