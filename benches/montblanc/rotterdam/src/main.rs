// Copyright © 2024 Stephan Kunz

//! The node 'rotterdam' subscribes to
//!   - a `TwistWithCovarianceStamped` on the topic /mekong
//! and publishes on receive
//!   - a `Vector3Stamped` on the topic /murray
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use std::sync::{Arc, RwLock};
use tracing::info;

#[derive(Debug, Default)]
struct AgentProps {}

fn mekong_callback(ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::TwistWithCovarianceStamped =
		bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	let msg = messages::Vector3Stamped {
		header: value.header,
		vector: value.twist.twist.linear,
	};
	let _ = ctx.put("murray", &msg);
	info!("sent: '{}'", msg);
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::INFO)
		.init();

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
