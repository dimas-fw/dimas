// Copyright © 2024 Stephan Kunz

//! The node 'rotterdam' subscribes to
//!   - a `TwistWithCovarianceStamped` on the topic /mekong
//! and publishes on receive
//!   - a `Vector3Stamped` on the topic /murray
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use std::sync::{Arc, RwLock};

use dimas::prelude::*;

#[derive(Debug, Default)]
struct AgentProps {}

fn mekong_callback(ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::TwistWithCovarianceStamped =
		bitcode::decode(message).expect("should not happen");
	println!("rotterdam received TwistWithCovarianceStamped");
	let msg = messages::Vector3Stamped::random();
	let _ = ctx.publish("murray", msg);
	println!("rotterdam sent Vector3Stamped");
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
