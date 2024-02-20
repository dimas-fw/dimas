// Copyright © 2024 Stephan Kunz

//! The node 'geneva' subscribes to
//!   - a `StringMsg` on the topic /parana
//!   - a `StringMsg` on the topic /danube
//!   - a `Pose` on the topic /tagus
//!   - a `Twist` on the topic /congo
//! and publishes on receive of topic /parana
//!   - a `StringMsg` on the topic /arkansas
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use std::sync::{Arc, RwLock};
use tracing::info;

#[derive(Debug, Default)]
struct AgentProps {}

fn parana_callback(ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::StringMsg = bitcode::decode(message).expect("should not happen");
	info!("geneva received: {}", &value.data);
	let _ = ctx.publish("arkansas", &value);
	info!("geneva sent: {}", value.data);
}

fn danube_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::StringMsg = bitcode::decode(message).expect("should not happen");
	info!("geneva received: {}", value.data);
}

fn tagus_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::Pose = bitcode::decode(message).expect("should not happen");
	info!("geneva received Pose");
}

fn congo_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::Twist = bitcode::decode(message).expect("should not happen");
	info!("geneva received Twist");
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
		.put_callback(parana_callback)
		.msg_type("parana")
		.add()?;

	agent
		.subscriber()
		.put_callback(danube_callback)
		.msg_type("danube")
		.add()?;

	agent
		.subscriber()
		.put_callback(tagus_callback)
		.msg_type("tagus")
		.add()?;

	agent
		.subscriber()
		.put_callback(congo_callback)
		.msg_type("congo")
		.add()?;

	agent.start().await;
	Ok(())
}
